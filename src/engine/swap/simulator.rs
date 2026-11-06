use super::model::TriAssetModel;
use super::report::SwapReport;
use super::types::{AssetId, SwapConfig, SwapTradeRecord, SynchronizedBar};

/// Simulador de Intercambio y Rotación de Cartera Cuantitativa Tri-Activo (Long / Short / Cash).
/// Garantiza 0% Look-Ahead Bias: la señal de intercambio se genera con la vela cerrada $t$,
/// y la actualización de capital sigue la evolución de precios barra a barra.
/// Soporta tanto alocaciones completas (100% mono-activo) como asignaciones duales (50%/50%).
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

        let mut slot_1: AssetId;
        let mut slot_2: Option<AssetId>;
        let mut bars_held_1 = 0;
        let mut bars_held_2 = 0;
        let mut trades: Vec<SwapTradeRecord> = Vec::new();
        let mut equity_curve: Vec<(i64, f64)> = Vec::with_capacity(target_end - start_idx);
        let mut daily_returns: Vec<f64> = Vec::with_capacity(target_end - start_idx);
        let mut daily_allocations: Vec<(AssetId, f64, Option<AssetId>, f64)> =
            Vec::with_capacity(target_end - start_idx);

        // Contadores ponderados de exposición
        let mut btc_weight = 0.0;
        let mut gold_weight = 0.0;
        let mut sp500_weight = 0.0;
        let mut short_btc_weight = 0.0;
        let mut short_gold_weight = 0.0;
        let mut short_sp500_weight = 0.0;
        let mut cash_weight = 0.0;

        let mut profitable_swaps = 0;
        let mut total_fees_paid = 0.0;
        let mut swap_entry_capital = capital;

        // Punto de referencia para benchmarks
        let initial_bar = &bars[start_idx];
        let btc_initial = initial_bar.btc_close;
        let gold_initial = initial_bar.gold_close;
        let sp500_initial = initial_bar.sp500_close;

        let lev = self.config.leverage.max(1.0);

        // Primer decisión en start_idx
        let initial_decision = model.evaluate(&bars[0..=start_idx], start_idx);
        slot_1 = initial_decision.target_asset;
        slot_2 = initial_decision.secondary_asset;

        let is_dual = slot_2.is_some();
        let w1 = if is_dual { 0.5 } else { 1.0 };
        let w2 = if is_dual { 0.5 } else { 0.0 };

        if slot_1 != AssetId::Cash {
            let fee = capital * w1 * lev * (self.config.swap_fee_rate + self.config.slippage_rate);
            capital -= fee;
            total_fees_paid += fee;
            trades.push(SwapTradeRecord {
                swap_index: 1,
                step: start_idx,
                timestamp: initial_bar.timestamp,
                date_str: initial_bar.date_string(),
                from_asset: AssetId::Cash,
                to_asset: slot_1,
                exit_price: 1.0,
                entry_price: initial_bar.get_close(slot_1),
                capital_before: self.config.initial_capital,
                capital_after: capital,
                fee_paid: fee,
                reason: if is_dual {
                    format!("[Slot 1 - 50% ({:.0}X)] {}", lev, initial_decision.explanation)
                } else {
                    format!("[{:.0}X] {}", lev, initial_decision.explanation)
                },
            });
        }

        if let Some(s2) = slot_2
            && s2 != AssetId::Cash
        {
            let fee = capital * w2 * lev * (self.config.swap_fee_rate + self.config.slippage_rate);
            let cap_before = capital;
            capital -= fee;
            total_fees_paid += fee;
            trades.push(SwapTradeRecord {
                swap_index: trades.len() + 1,
                step: start_idx,
                timestamp: initial_bar.timestamp,
                date_str: initial_bar.date_string(),
                from_asset: AssetId::Cash,
                to_asset: s2,
                exit_price: 1.0,
                entry_price: initial_bar.get_close(s2),
                capital_before: cap_before,
                capital_after: capital,
                fee_paid: fee,
                reason: format!("[Slot 2 - 50% ({:.0}X)] {}", lev, initial_decision.explanation),
            });
        }

        equity_curve.push((initial_bar.timestamp, capital));
        daily_allocations.push((slot_1, w1, slot_2, w2));

        for step in (start_idx + 1)..target_end {
            let prev_bar = &bars[step - 1];
            let bar = &bars[step];

            let is_dual = slot_2.is_some();
            let cur_w1 = if is_dual { 0.5 } else { 1.0 };
            let cur_w2 = if is_dual { 0.5 } else { 0.0 };

            // 1. --- Actualizar el valor del capital según el rendimiento de activos en tenencia ---
            let r1 = calc_asset_return(slot_1, prev_bar, bar);
            let r2 = match slot_2 {
                Some(s2) => calc_asset_return(s2, prev_bar, bar),
                None => 0.0,
            };

            let prior_capital = capital;
            match self.config.margin_mode {
                crate::engine::swap::types::MarginMode::Cross => {
                    // Modo Capital Cruzado Apalancado (Cross Margin):
                    // El colateral unificado respalda todas las posiciones y el retorno se amplifica por el apalancamiento.
                    let price_return = cur_w1 * r1 + cur_w2 * r2;
                    let leveraged_return = price_return * lev;
                    capital *= 1.0 + leveraged_return;

                    if capital <= 0.0 {
                        capital = 0.0;
                        trades.push(SwapTradeRecord {
                            swap_index: trades.len() + 1,
                            step,
                            timestamp: bar.timestamp,
                            date_str: bar.date_string(),
                            from_asset: slot_1,
                            to_asset: AssetId::Cash,
                            exit_price: bar.get_close(slot_1),
                            entry_price: 1.0,
                            capital_before: prior_capital,
                            capital_after: 0.0,
                            fee_paid: 0.0,
                            reason: format!(
                                "[LIQUIDACIÓN CRUZADA {:.0}X] El balance total de la cuenta se agotó por pérdidas acumuladas",
                                lev
                            ),
                        });
                        daily_returns.push(-1.0);
                        equity_curve.push((bar.timestamp, 0.0));
                        daily_allocations.push((AssetId::Cash, 0.0, None, 0.0));
                        break;
                    }
                }
                crate::engine::swap::types::MarginMode::Isolated => {
                    // Modo Margen Aislado:
                    // Cada posición tiene su pérdida acotada estrictamente a su asignación inicial.
                    let eff_r1 = (r1 * lev).max(-1.0);
                    let eff_r2 = (r2 * lev).max(-1.0);
                    let price_return = cur_w1 * eff_r1 + cur_w2 * eff_r2;
                    capital *= 1.0 + price_return;
                }
            }

            daily_returns.push((capital - prior_capital) / prior_capital.max(1e-6));

            // Acumular tiempo de exposición ponderado
            accumulate_exposure(
                slot_1,
                cur_w1,
                &mut btc_weight,
                &mut gold_weight,
                &mut sp500_weight,
                &mut short_btc_weight,
                &mut short_gold_weight,
                &mut short_sp500_weight,
                &mut cash_weight,
            );
            if let Some(s2) = slot_2 {
                accumulate_exposure(
                    s2,
                    cur_w2,
                    &mut btc_weight,
                    &mut gold_weight,
                    &mut sp500_weight,
                    &mut short_btc_weight,
                    &mut short_gold_weight,
                    &mut short_sp500_weight,
                    &mut cash_weight,
                );
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
            bars_held_1 += 1;
            bars_held_2 += 1;

            // 3. --- Evaluación de Swap / Rotación ---
            match decision.secondary_asset {
                Some(new_2) => {
                    let new_1 = decision.target_asset;
                    let mut cur_2 = slot_2.unwrap_or(AssetId::Cash);

                    // Si el conjunto de activos en el Top 2 no cambió, preservamos sin pagar comisiones
                    if (slot_1 == new_1 && cur_2 == new_2) || (slot_1 == new_2 && cur_2 == new_1) {
                        // Mismo Top 2 retenido
                    } else if slot_1 == new_1 || cur_2 == new_1 {
                        if cur_2 == new_1 {
                            std::mem::swap(&mut slot_1, &mut cur_2);
                            std::mem::swap(&mut bars_held_1, &mut bars_held_2);
                        }
                        let can_swap = cur_2 != new_2
                            && (bars_held_2 >= self.config.min_holding_bars
                                || new_2 == AssetId::Cash
                                || decision.confidence_score >= self.config.hysteresis_threshold);
                        if can_swap {
                            let fee = capital * 0.5 * lev * (self.config.swap_fee_rate + self.config.slippage_rate);
                            let cap_before = capital;
                            capital -= fee;
                            total_fees_paid += fee;

                            if cap_before > swap_entry_capital {
                                profitable_swaps += 1;
                            }
                            swap_entry_capital = capital;

                            trades.push(SwapTradeRecord {
                                swap_index: trades.len() + 1,
                                step,
                                timestamp: bar.timestamp,
                                date_str: bar.date_string(),
                                from_asset: cur_2,
                                to_asset: new_2,
                                exit_price: bar.get_close(cur_2),
                                entry_price: bar.get_close(new_2),
                                capital_before: cap_before,
                                capital_after: capital,
                                fee_paid: fee,
                                reason: format!("[Slot 2 - 50% ({:.0}X)] {}", lev, decision.explanation),
                            });
                            cur_2 = new_2;
                            bars_held_2 = 0;
                        }
                    } else if slot_1 == new_2 || cur_2 == new_2 {
                        if slot_1 == new_2 {
                            std::mem::swap(&mut slot_1, &mut cur_2);
                            std::mem::swap(&mut bars_held_1, &mut bars_held_2);
                        }
                        let can_swap = slot_1 != new_1
                            && (bars_held_1 >= self.config.min_holding_bars
                                || new_1 == AssetId::Cash
                                || decision.confidence_score >= self.config.hysteresis_threshold);
                        if can_swap {
                            let fee = capital * 0.5 * lev * (self.config.swap_fee_rate + self.config.slippage_rate);
                            let cap_before = capital;
                            capital -= fee;
                            total_fees_paid += fee;

                            if cap_before > swap_entry_capital {
                                profitable_swaps += 1;
                            }
                            swap_entry_capital = capital;

                            trades.push(SwapTradeRecord {
                                swap_index: trades.len() + 1,
                                step,
                                timestamp: bar.timestamp,
                                date_str: bar.date_string(),
                                from_asset: slot_1,
                                to_asset: new_1,
                                exit_price: bar.get_close(slot_1),
                                entry_price: bar.get_close(new_1),
                                capital_before: cap_before,
                                capital_after: capital,
                                fee_paid: fee,
                                reason: format!("[Slot 1 - 50% ({:.0}X)] {}", lev, decision.explanation),
                            });
                            slot_1 = new_1;
                            bars_held_1 = 0;
                        }
                    } else {
                        // Ambos activos rotan
                        let can_swap_1 = slot_1 != new_1
                            && (bars_held_1 >= self.config.min_holding_bars
                                || new_1 == AssetId::Cash
                                || decision.confidence_score >= self.config.hysteresis_threshold);
                        if can_swap_1 {
                            let fee = capital * 0.5 * lev * (self.config.swap_fee_rate + self.config.slippage_rate);
                            let cap_before = capital;
                            capital -= fee;
                            total_fees_paid += fee;

                            if cap_before > swap_entry_capital {
                                profitable_swaps += 1;
                            }
                            swap_entry_capital = capital;

                            trades.push(SwapTradeRecord {
                                swap_index: trades.len() + 1,
                                step,
                                timestamp: bar.timestamp,
                                date_str: bar.date_string(),
                                from_asset: slot_1,
                                to_asset: new_1,
                                exit_price: bar.get_close(slot_1),
                                entry_price: bar.get_close(new_1),
                                capital_before: cap_before,
                                capital_after: capital,
                                fee_paid: fee,
                                reason: format!("[Slot 1 - 50% ({:.0}X)] {}", lev, decision.explanation),
                            });
                            slot_1 = new_1;
                            bars_held_1 = 0;
                        }

                        let can_swap_2 = cur_2 != new_2
                            && (bars_held_2 >= self.config.min_holding_bars
                                || new_2 == AssetId::Cash
                                || decision.confidence_score >= self.config.hysteresis_threshold);
                        if can_swap_2 {
                            let fee = capital * 0.5 * lev * (self.config.swap_fee_rate + self.config.slippage_rate);
                            let cap_before = capital;
                            capital -= fee;
                            total_fees_paid += fee;

                            if cap_before > swap_entry_capital {
                                profitable_swaps += 1;
                            }
                            swap_entry_capital = capital;

                            trades.push(SwapTradeRecord {
                                swap_index: trades.len() + 1,
                                step,
                                timestamp: bar.timestamp,
                                date_str: bar.date_string(),
                                from_asset: cur_2,
                                to_asset: new_2,
                                exit_price: bar.get_close(cur_2),
                                entry_price: bar.get_close(new_2),
                                capital_before: cap_before,
                                capital_after: capital,
                                fee_paid: fee,
                                reason: format!("[Slot 2 - 50% ({:.0}X)] {}", lev, decision.explanation),
                            });
                            cur_2 = new_2;
                            bars_held_2 = 0;
                        }
                    }
                    slot_2 = Some(cur_2);
                }
                None => {
                    // Modo Single-Asset al 100% (Ruta A y Ruta C)
                    if let Some(s2) = slot_2 {
                        if s2 != AssetId::Cash {
                            let fee = capital * 0.5 * lev * (self.config.swap_fee_rate + self.config.slippage_rate);
                            capital -= fee;
                            total_fees_paid += fee;
                            trades.push(SwapTradeRecord {
                                swap_index: trades.len() + 1,
                                step,
                                timestamp: bar.timestamp,
                                date_str: bar.date_string(),
                                from_asset: s2,
                                to_asset: AssetId::Cash,
                                exit_price: bar.get_close(s2),
                                entry_price: 1.0,
                                capital_before: capital + fee,
                                capital_after: capital,
                                fee_paid: fee,
                                reason: format!("Cierre de Slot 2 a Cash ({:.0}X)", lev),
                            });
                        }
                        slot_2 = None;
                    }

                    let should_swap = decision.target_asset != slot_1
                        && (bars_held_1 >= self.config.min_holding_bars
                            || decision.target_asset == AssetId::Cash
                            || decision.confidence_score >= self.config.hysteresis_threshold);

                    if should_swap {
                        let fee = capital * lev * (self.config.swap_fee_rate + self.config.slippage_rate);
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
                            from_asset: slot_1,
                            to_asset: decision.target_asset,
                            exit_price: bar.get_close(slot_1),
                            entry_price: bar.get_close(decision.target_asset),
                            capital_before: cap_before,
                            capital_after: capital,
                            fee_paid: fee,
                            reason: decision.explanation,
                        });

                        slot_1 = decision.target_asset;
                        bars_held_1 = 0;
                    }
                }
            }

            let cur_w1 = if slot_2.is_some() { 0.5 } else { 1.0 };
            let cur_w2 = if slot_2.is_some() { 0.5 } else { 0.0 };
            daily_allocations.push((slot_1, cur_w1, slot_2, cur_w2));
            equity_curve.push((bar.timestamp, capital));
        }

        // Verificación del último periodo
        if capital > swap_entry_capital {
            profitable_swaps += 1;
        }

        let final_bar = &bars[target_end - 1];
        let total_bars = (target_end - start_idx).max(1);
        let total_days = total_bars as f64;
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
            margin_mode: self.config.margin_mode,
            leverage: self.config.leverage,
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
            exposure_btc_pct: (btc_weight / total_bars as f64) * 100.0,
            exposure_gold_pct: (gold_weight / total_bars as f64) * 100.0,
            exposure_sp500_pct: (sp500_weight / total_bars as f64) * 100.0,
            exposure_short_btc_pct: (short_btc_weight / total_bars as f64) * 100.0,
            exposure_short_gold_pct: (short_gold_weight / total_bars as f64) * 100.0,
            exposure_short_sp500_pct: (short_sp500_weight / total_bars as f64) * 100.0,
            exposure_cash_pct: (cash_weight / total_bars as f64) * 100.0,
            benchmark_btc_return_pct: btc_return,
            benchmark_gold_return_pct: gold_return,
            benchmark_sp500_return_pct: sp500_return,
            benchmark_equal_weight_return_pct: eq_weight_return,
            alpha_vs_sp500: total_return_pct - sp500_return,
            alpha_vs_btc: total_return_pct - btc_return,
            equity_curve,
            trades,
            daily_allocations,
        }
    }

    fn empty_report(&self, name: &str) -> SwapReport {
        SwapReport {
            strategy_name: name.to_string(),
            margin_mode: self.config.margin_mode,
            leverage: self.config.leverage,
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
            exposure_short_btc_pct: 0.0,
            exposure_short_gold_pct: 0.0,
            exposure_short_sp500_pct: 0.0,
            exposure_cash_pct: 100.0,
            benchmark_btc_return_pct: 0.0,
            benchmark_gold_return_pct: 0.0,
            benchmark_sp500_return_pct: 0.0,
            benchmark_equal_weight_return_pct: 0.0,
            alpha_vs_sp500: 0.0,
            alpha_vs_btc: 0.0,
            equity_curve: Vec::new(),
            trades: Vec::new(),
            daily_allocations: Vec::new(),
        }
    }
}

fn calc_asset_return(asset: AssetId, prev_bar: &SynchronizedBar, bar: &SynchronizedBar) -> f64 {
    match asset {
        AssetId::Btc => bar.btc_close / prev_bar.btc_close.max(1e-6) - 1.0,
        AssetId::Gold => bar.gold_close / prev_bar.gold_close.max(1e-6) - 1.0,
        AssetId::Sp500 => bar.sp500_close / prev_bar.sp500_close.max(1e-6) - 1.0,
        AssetId::ShortBtc => -(bar.btc_close / prev_bar.btc_close.max(1e-6) - 1.0),
        AssetId::ShortGold => -(bar.gold_close / prev_bar.gold_close.max(1e-6) - 1.0),
        AssetId::ShortSp500 => -(bar.sp500_close / prev_bar.sp500_close.max(1e-6) - 1.0),
        AssetId::Cash => 0.0,
    }
}

#[allow(clippy::too_many_arguments)]
fn accumulate_exposure(
    asset: AssetId,
    weight: f64,
    btc_w: &mut f64,
    gold_w: &mut f64,
    sp500_w: &mut f64,
    sh_btc_w: &mut f64,
    sh_gold_w: &mut f64,
    sh_sp500_w: &mut f64,
    cash_w: &mut f64,
) {
    match asset {
        AssetId::Btc => *btc_w += weight,
        AssetId::Gold => *gold_w += weight,
        AssetId::Sp500 => *sp500_w += weight,
        AssetId::ShortBtc => *sh_btc_w += weight,
        AssetId::ShortGold => *sh_gold_w += weight,
        AssetId::ShortSp500 => *sh_sp500_w += weight,
        AssetId::Cash => *cash_w += weight,
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
