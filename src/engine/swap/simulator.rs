use super::model::TriAssetModel;
use super::report::SwapReport;
use super::types::{AssetId, SwapConfig, SwapTradeRecord, SynchronizedBar};

/// Simulador de Intercambio y Rotación de Cartera Cuantitativa Tri-Activo.
/// Garantiza 0% Look-Ahead Bias: la señal de intercambio se genera con la vela cerrada $t$,
/// y la actualización de capital sigue la evolución de precios barra a barra.
pub struct SwapSimulator {
    pub config: SwapConfig,
}

impl SwapSimulator {
    pub fn new(config: SwapConfig) -> Self {
        Self { config }
    }

    /// Ejecuta la simulación causal de rotación de cartera en el rango [start_idx..end_idx]
    pub fn run<M: TriAssetModel>(
        &self,
        model: &mut M,
        bars: &[SynchronizedBar],
        start_idx: usize,
        end_idx: usize,
    ) -> SwapReport {
        let total_n = bars.len();
        if total_n == 0 || start_idx >= end_idx || start_idx >= total_n {
            return self.empty_report(model.name());
        }

        let target_end = end_idx.min(total_n);
        let mut capital = self.config.initial_capital;
        let mut peak_capital = capital;
        let mut max_drawdown_pct = 0.0;
        let mut max_dd_duration = 0;
        let mut current_dd_duration = 0;

        let mut current_asset;
        let mut bars_held = 0;
        let mut trades: Vec<SwapTradeRecord> = Vec::new();
        let mut equity_curve: Vec<(i64, f64)> = Vec::with_capacity(target_end - start_idx);
        let mut daily_returns: Vec<f64> = Vec::with_capacity(target_end - start_idx);

        let mut btc_bars = 0;
        let mut gold_bars = 0;
        let mut sp500_bars = 0;
        let mut cash_bars = 0;
        let mut profitable_swaps = 0;
        let mut total_fees_paid = 0.0;
        let mut swap_entry_capital = capital;

        // Punto de referencia para benchmarks
        let initial_bar = &bars[start_idx];
        let btc_initial = initial_bar.btc_close;
        let gold_initial = initial_bar.gold_close;
        let sp500_initial = initial_bar.sp500_close;

        // Primer decisión en start_idx
        let initial_decision = model.evaluate(&bars[0..=start_idx], start_idx);
        current_asset = initial_decision.target_asset;
        if current_asset != AssetId::Cash {
            let fee = capital * (self.config.swap_fee_rate + self.config.slippage_rate);
            capital -= fee;
            total_fees_paid += fee;
            trades.push(SwapTradeRecord {
                swap_index: 1,
                step: start_idx,
                timestamp: initial_bar.timestamp,
                date_str: initial_bar.date_string(),
                from_asset: AssetId::Cash,
                to_asset: current_asset,
                exit_price: 1.0,
                entry_price: initial_bar.get_close(current_asset),
                capital_before: self.config.initial_capital,
                capital_after: capital,
                fee_paid: fee,
                reason: initial_decision.explanation,
            });
        }
        equity_curve.push((initial_bar.timestamp, capital));

        for step in (start_idx + 1)..target_end {
            let prev_bar = &bars[step - 1];
            let bar = &bars[step];

            // 1. --- Actualizar el valor del capital según el rendimiento del activo en tenencia ---
            let price_return = match current_asset {
                AssetId::Btc => bar.btc_close / prev_bar.btc_close.max(1e-6) - 1.0,
                AssetId::Gold => bar.gold_close / prev_bar.gold_close.max(1e-6) - 1.0,
                AssetId::Sp500 => bar.sp500_close / prev_bar.sp500_close.max(1e-6) - 1.0,
                AssetId::Cash => 0.0,
            };

            let prior_capital = capital;
            capital *= 1.0 + price_return;
            daily_returns.push((capital - prior_capital) / prior_capital.max(1e-6));

            // Acumular tiempo de exposición
            match current_asset {
                AssetId::Btc => btc_bars += 1,
                AssetId::Gold => gold_bars += 1,
                AssetId::Sp500 => sp500_bars += 1,
                AssetId::Cash => cash_bars += 1,
            }

            // Tracking de Peak y Drawdown
            if capital > peak_capital {
                peak_capital = capital;
                current_dd_duration = 0;
            } else {
                current_dd_duration += 1;
                if current_dd_duration > max_dd_duration {
                    max_dd_duration = current_dd_duration;
                }
                let dd = (peak_capital - capital) / peak_capital * 100.0;
                if dd > max_drawdown_pct {
                    max_drawdown_pct = dd;
                }
            }

            // 2. --- Consulta causal al modelo en base a la historia hasta este step ---
            let decision = model.evaluate(&bars[0..=step], step);
            bars_held += 1;

            // 3. --- Evaluación de Swap / Rotación ---
            let should_swap = decision.target_asset != current_asset
                && (bars_held >= self.config.min_holding_bars
                    || decision.target_asset == AssetId::Cash
                    || decision.confidence_score >= self.config.hysteresis_threshold);

            if should_swap {
                let fee = capital * (self.config.swap_fee_rate + self.config.slippage_rate);
                let cap_before = capital;
                capital -= fee;
                total_fees_paid += fee;

                if cap_before > swap_entry_capital {
                    profitable_swaps += 1;
                }
                swap_entry_capital = capital;

                let swap_idx = trades.len() + 1;
                trades.push(SwapTradeRecord {
                    swap_index: swap_idx,
                    step,
                    timestamp: bar.timestamp,
                    date_str: bar.date_string(),
                    from_asset: current_asset,
                    to_asset: decision.target_asset,
                    exit_price: bar.get_close(current_asset),
                    entry_price: bar.get_close(decision.target_asset),
                    capital_before: cap_before,
                    capital_after: capital,
                    fee_paid: fee,
                    reason: decision.explanation,
                });

                current_asset = decision.target_asset;
                bars_held = 0;
            }

            equity_curve.push((bar.timestamp, capital));
        }

        // Verificación del último periodo
        if capital > swap_entry_capital {
            profitable_swaps += 1;
        }

        let final_bar = &bars[target_end - 1];
        let total_bars = (target_end - start_idx).max(1);
        let total_days = total_bars as f64; // asumiendo 1D
        let years = (total_days / 365.25).max(0.01);

        let net_profit = capital - self.config.initial_capital;
        let total_return_pct = (net_profit / self.config.initial_capital) * 100.0;
        let cagr_pct = ((capital / self.config.initial_capital).powf(1.0 / years) - 1.0) * 100.0;

        // Cálculo de Sharpe y Sortino
        let (sharpe_ratio, sortino_ratio) = calculate_risk_ratios(&daily_returns);
        let calmar_ratio = if max_drawdown_pct > 0.001 {
            cagr_pct / max_drawdown_pct
        } else {
            0.0
        };

        let total_swaps = trades.len();
        let win_rate_swaps_pct = if total_swaps > 0 {
            (profitable_swaps as f64 / (total_swaps + 1) as f64) * 100.0
        } else {
            0.0
        };

        // Métricas de Benchmarks Buy & Hold
        let btc_return = (final_bar.btc_close / btc_initial - 1.0) * 100.0;
        let gold_return = (final_bar.gold_close / gold_initial - 1.0) * 100.0;
        let sp500_return = (final_bar.sp500_close / sp500_initial - 1.0) * 100.0;
        let eq_weight_return = (btc_return + gold_return + sp500_return) / 3.0;

        SwapReport {
            strategy_name: model.name().to_string(),
            initial_capital: self.config.initial_capital,
            final_capital: capital,
            net_profit,
            total_return_pct,
            cagr_pct,
            sharpe_ratio,
            sortino_ratio,
            max_drawdown_pct,
            max_drawdown_duration_bars: max_dd_duration,
            calmar_ratio,
            total_swaps,
            total_fees_paid,
            profitable_swaps,
            win_rate_swaps_pct,
            exposure_btc_pct: (btc_bars as f64 / total_bars as f64) * 100.0,
            exposure_gold_pct: (gold_bars as f64 / total_bars as f64) * 100.0,
            exposure_sp500_pct: (sp500_bars as f64 / total_bars as f64) * 100.0,
            exposure_cash_pct: (cash_bars as f64 / total_bars as f64) * 100.0,
            benchmark_btc_return_pct: btc_return,
            benchmark_gold_return_pct: gold_return,
            benchmark_sp500_return_pct: sp500_return,
            benchmark_equal_weight_return_pct: eq_weight_return,
            alpha_vs_sp500: total_return_pct - sp500_return,
            alpha_vs_btc: total_return_pct - btc_return,
            equity_curve,
            trades,
        }
    }

    fn empty_report(&self, name: &str) -> SwapReport {
        SwapReport {
            strategy_name: name.to_string(),
            initial_capital: self.config.initial_capital,
            final_capital: self.config.initial_capital,
            net_profit: 0.0,
            total_return_pct: 0.0,
            cagr_pct: 0.0,
            sharpe_ratio: 0.0,
            sortino_ratio: 0.0,
            max_drawdown_pct: 0.0,
            max_drawdown_duration_bars: 0,
            calmar_ratio: 0.0,
            total_swaps: 0,
            total_fees_paid: 0.0,
            profitable_swaps: 0,
            win_rate_swaps_pct: 0.0,
            exposure_btc_pct: 0.0,
            exposure_gold_pct: 0.0,
            exposure_sp500_pct: 0.0,
            exposure_cash_pct: 100.0,
            benchmark_btc_return_pct: 0.0,
            benchmark_gold_return_pct: 0.0,
            benchmark_sp500_return_pct: 0.0,
            benchmark_equal_weight_return_pct: 0.0,
            alpha_vs_sp500: 0.0,
            alpha_vs_btc: 0.0,
            equity_curve: Vec::new(),
            trades: Vec::new(),
        }
    }
}

fn calculate_risk_ratios(returns: &[f64]) -> (f64, f64) {
    if returns.len() < 2 {
        return (0.0, 0.0);
    }
    let n = returns.len() as f64;
    let mean = returns.iter().sum::<f64>() / n;
    let variance = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (n - 1.0);
    let std_dev = variance.sqrt();

    let downside_variance = returns
        .iter()
        .filter(|&&r| r < 0.0)
        .map(|r| r.powi(2))
        .sum::<f64>()
        / (n - 1.0).max(1.0);
    let downside_dev = downside_variance.sqrt();

    let annual_factor = (365.25f64).sqrt();
    let sharpe = if std_dev > 1e-7 {
        (mean / std_dev) * annual_factor
    } else {
        0.0
    };

    let sortino = if downside_dev > 1e-7 {
        (mean / downside_dev) * annual_factor
    } else {
        0.0
    };

    (sharpe, sortino)
}
