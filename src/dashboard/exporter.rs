use super::models::{
    AssetBenchmarkDto, StrategyAllocationPointDto, SwapDashboardPayload, SwapDashboardStrategyDto,
    SwapMetricsDto, SwapTradeDto, YearlyPerformanceDto,
};
use crate::engine::swap::report::SwapReport;
use crate::engine::swap::types::SynchronizedBar;
use crate::metrics::swap_fitness::SwapFitnessResult;
use chrono::{Datelike, TimeZone, Utc};
use std::collections::BTreeMap;

pub struct StrategyInput<'a> {
    pub id: String,
    pub name: String,
    pub architecture: String,
    pub color: String,
    pub report: &'a SwapReport,
    pub fitness: &'a SwapFitnessResult,
}

pub fn build_swap_dashboard_payload(
    bars: &[SynchronizedBar],
    is_split_idx: usize,
    strategies_input: &[StrategyInput],
) -> SwapDashboardPayload {
    let total_bars = bars.len();
    assert!(total_bars > 0, "No se pueden exportar métricas con barras vacías");

    let first_bar = &bars[0];
    let last_bar = &bars[total_bars - 1];
    let initial_capital = strategies_input
        .first()
        .map(|s| s.report.initial_capital)
        .unwrap_or(10_000.0);

    let timestamps: Vec<i64> = bars.iter().map(|b| b.timestamp).collect();
    let dates: Vec<String> = bars.iter().map(|b| b.date_string()).collect();

    // 1. Calcular Benchmarks Individuales
    let benchmarks = build_benchmarks(bars, initial_capital);

    // 2. Extraer retornos anuales de Benchmarks
    let btc_yearly = calculate_asset_yearly_returns(bars, |b| b.btc_close);
    let gold_yearly = calculate_asset_yearly_returns(bars, |b| b.gold_close);
    let sp500_yearly = calculate_asset_yearly_returns(bars, |b| b.sp500_close);

    // 3. Procesar cada Estrategia
    let mut strategies = Vec::new();
    for input in strategies_input {
        let rep = input.report;
        let fit = input.fitness;

        let equity_curve: Vec<f64> = rep.equity_curve.iter().map(|(_, eq)| *eq).collect();
        let drawdown_curve = calculate_drawdown_curve(&equity_curve);

        // Reconstruir timeline de asignación de activos
        let allocations = build_allocations_timeline(bars, &rep.trades, &equity_curve, &drawdown_curve);

        // Trades format
        let mut trades = Vec::new();
        for (idx, t) in rep.trades.iter().enumerate() {
            let pnl_pct = if t.capital_before > 0.0 {
                ((t.capital_after / t.capital_before) - 1.0) * 100.0
            } else {
                0.0
            };

            let holding_bars = if idx + 1 < rep.trades.len() {
                rep.trades[idx + 1].step.saturating_sub(t.step)
            } else {
                total_bars.saturating_sub(t.step)
            };

            trades.push(SwapTradeDto {
                swap_index: t.swap_index,
                timestamp: t.timestamp,
                date: t.date_str.clone(),
                from_asset: t.from_asset.symbol().to_string(),
                to_asset: t.to_asset.symbol().to_string(),
                exit_price: t.exit_price,
                entry_price: t.entry_price,
                capital_before: t.capital_before,
                capital_after: t.capital_after,
                pnl_pct,
                fee_paid: t.fee_paid,
                holding_bars,
                reason: t.reason.clone(),
            });
        }

        // Rendimientos Anuales
        let strat_yearly = calculate_curve_yearly_returns(bars, &equity_curve);
        let mut yearly_returns = Vec::new();
        for (&year, &strat_ret) in &strat_yearly {
            let btc_ret = *btc_yearly.get(&year).unwrap_or(&0.0);
            let gold_ret = *gold_yearly.get(&year).unwrap_or(&0.0);
            let sp500_ret = *sp500_yearly.get(&year).unwrap_or(&0.0);

            yearly_returns.push(YearlyPerformanceDto {
                year,
                strategy_return_pct: strat_ret,
                btc_return_pct: btc_ret,
                gold_return_pct: gold_ret,
                sp500_return_pct: sp500_ret,
                diff_vs_btc: strat_ret - btc_ret,
                diff_vs_sp500: strat_ret - sp500_ret,
                beat_btc: strat_ret > btc_ret,
                beat_sp500: strat_ret > sp500_ret,
            });
        }

        let metrics = SwapMetricsDto {
            total_return_pct: rep.total_return_pct,
            cagr_pct: rep.cagr_pct,
            sharpe_ratio: rep.sharpe_ratio,
            sortino_ratio: rep.sortino_ratio,
            max_drawdown_pct: rep.max_drawdown_pct,
            max_drawdown_bars: rep.max_drawdown_duration_bars,
            calmar_ratio: rep.calmar_ratio,
            total_swaps: rep.total_swaps,
            total_fees_paid: rep.total_fees_paid,
            win_rate_swaps_pct: rep.win_rate_swaps_pct,
            exposure_btc_pct: rep.exposure_btc_pct,
            exposure_gold_pct: rep.exposure_gold_pct,
            exposure_sp500_pct: rep.exposure_sp500_pct,
            exposure_cash_pct: rep.exposure_cash_pct,
            swap_fitness_score: fit.swap_fitness_score,
            capital_dd_ratio: fit.capital_dd_ratio,
            composite_r2: fit.composite_linearity,
            slope_ratio: fit.slope_ratio,
        };

        strategies.push(SwapDashboardStrategyDto {
            id: input.id.clone(),
            name: input.name.clone(),
            architecture: input.architecture.clone(),
            color: input.color.clone(),
            initial_capital: rep.initial_capital,
            final_capital: rep.final_capital,
            net_profit: rep.net_profit,
            metrics,
            equity_curve,
            drawdown_curve,
            allocations,
            trades,
            yearly_returns,
        });
    }

    SwapDashboardPayload {
        title: "SWAP Multi-Asset Rotation Dashboard".to_string(),
        timeframe: "1D".to_string(),
        generated_at_utc: Utc::now().to_rfc3339(),
        common_start_date: first_bar.date_string(),
        common_end_date: last_bar.date_string(),
        total_bars,
        is_split_idx,
        timestamps,
        dates,
        benchmarks,
        strategies,
    }
}

fn build_benchmarks(bars: &[SynchronizedBar], initial_capital: f64) -> Vec<AssetBenchmarkDto> {
    let mut btc_curve = Vec::with_capacity(bars.len());
    let mut gold_curve = Vec::with_capacity(bars.len());
    let mut sp500_curve = Vec::with_capacity(bars.len());
    let mut eq_curve = Vec::with_capacity(bars.len());

    let btc_0 = bars[0].btc_close.max(1e-6);
    let gold_0 = bars[0].gold_close.max(1e-6);
    let sp500_0 = bars[0].sp500_close.max(1e-6);

    for b in bars {
        let btc_val = initial_capital * (b.btc_close / btc_0);
        let gold_val = initial_capital * (b.gold_close / gold_0);
        let sp500_val = initial_capital * (b.sp500_close / sp500_0);
        let eq_val = (btc_val + gold_val + sp500_val) / 3.0;

        btc_curve.push(btc_val);
        gold_curve.push(gold_val);
        sp500_curve.push(sp500_val);
        eq_curve.push(eq_val);
    }

    vec![
        create_benchmark_dto("btc", "Bitcoin (Buy & Hold)", "BTC", "#F7931A", bars[0].btc_close, bars.last().unwrap().btc_close, initial_capital, btc_curve, bars.len()),
        create_benchmark_dto("gold", "Oro / PAXG (Buy & Hold)", "GOLD", "#FFD700", bars[0].gold_close, bars.last().unwrap().gold_close, initial_capital, gold_curve, bars.len()),
        create_benchmark_dto("sp500", "S&P 500 (Buy & Hold)", "SP500", "#00B0FF", bars[0].sp500_close, bars.last().unwrap().sp500_close, initial_capital, sp500_curve, bars.len()),
        create_benchmark_dto("equal_weight", "Cartera Equitativa (1/3 c/u)", "EQ3", "#E040FB", 1.0, 1.0, initial_capital, eq_curve, bars.len()),
    ]
}

fn create_benchmark_dto(
    id: &str,
    name: &str,
    symbol: &str,
    color: &str,
    p0: f64,
    p1: f64,
    initial_cap: f64,
    equity_curve: Vec<f64>,
    total_bars: usize,
) -> AssetBenchmarkDto {
    let final_cap = *equity_curve.last().unwrap_or(&initial_cap);
    let total_return_pct = if initial_cap > 0.0 {
        ((final_cap / initial_cap) - 1.0) * 100.0
    } else {
        0.0
    };

    let years = (total_bars as f64) / 365.25;
    let cagr_pct = if years > 0.0 && final_cap > 0.0 && initial_cap > 0.0 {
        ((final_cap / initial_cap).powf(1.0 / years) - 1.0) * 100.0
    } else {
        0.0
    };

    let drawdown_curve = calculate_drawdown_curve(&equity_curve);
    let max_drawdown_pct = drawdown_curve.iter().cloned().fold(0.0f64, f64::max);

    // Sharpe ratio diario anualizado
    let mut returns = Vec::with_capacity(equity_curve.len().saturating_sub(1));
    for i in 1..equity_curve.len() {
        let r = (equity_curve[i] / equity_curve[i - 1].max(1e-6)) - 1.0;
        returns.push(r);
    }
    let n = returns.len() as f64;
    let sharpe_ratio = if n > 1.0 {
        let mean = returns.iter().sum::<f64>() / n;
        let var = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (n - 1.0);
        let std = var.sqrt();
        if std > 1e-9 {
            (mean / std) * (365.25f64).sqrt()
        } else {
            0.0
        }
    } else {
        0.0
    };

    AssetBenchmarkDto {
        id: id.to_string(),
        name: name.to_string(),
        symbol: symbol.to_string(),
        color: color.to_string(),
        initial_price: p0,
        final_price: p1,
        total_return_pct,
        cagr_pct,
        sharpe_ratio,
        max_drawdown_pct,
        equity_curve,
        drawdown_curve,
    }
}

fn calculate_drawdown_curve(curve: &[f64]) -> Vec<f64> {
    let mut dd_curve = Vec::with_capacity(curve.len());
    let mut peak = if !curve.is_empty() { curve[0] } else { 1.0 };

    for &val in curve {
        if val > peak {
            peak = val;
        }
        let dd = if peak > 0.0 {
            ((peak - val) / peak) * 100.0
        } else {
            0.0
        };
        dd_curve.push(dd.max(0.0));
    }
    dd_curve
}

fn build_allocations_timeline(
    bars: &[SynchronizedBar],
    trades: &[crate::engine::swap::types::SwapTradeRecord],
    equity_curve: &[f64],
    drawdown_curve: &[f64],
) -> Vec<StrategyAllocationPointDto> {
    let mut timeline = Vec::with_capacity(bars.len());
    let mut current_asset = "CASH".to_string();
    let mut trade_idx = 0;

    for (step, bar) in bars.iter().enumerate() {
        while trade_idx < trades.len() && trades[trade_idx].step <= step {
            current_asset = trades[trade_idx].to_asset.symbol().to_string();
            trade_idx += 1;
        }

        let eq = *equity_curve.get(step).unwrap_or(&10_000.0);
        let dd = *drawdown_curve.get(step).unwrap_or(&0.0);

        timeline.push(StrategyAllocationPointDto {
            timestamp: bar.timestamp,
            date: bar.date_string(),
            asset: current_asset.clone(),
            equity: eq,
            drawdown_pct: dd,
        });
    }

    timeline
}

fn calculate_asset_yearly_returns(
    bars: &[SynchronizedBar],
    get_price: impl Fn(&SynchronizedBar) -> f64,
) -> BTreeMap<i32, f64> {
    let mut years_map: BTreeMap<i32, (f64, f64)> = BTreeMap::new(); // year -> (open, close)

    for bar in bars {
        if let Some(dt) = Utc.timestamp_millis_opt(bar.timestamp).single() {
            let year = dt.year();
            let p = get_price(bar);
            years_map
                .entry(year)
                .and_modify(|(_, last)| *last = p)
                .or_insert((p, p));
        }
    }

    let mut result = BTreeMap::new();
    for (year, (first, last)) in years_map {
        let ret = if first > 0.0 {
            ((last / first) - 1.0) * 100.0
        } else {
            0.0
        };
        result.insert(year, ret);
    }
    result
}

fn calculate_curve_yearly_returns(
    bars: &[SynchronizedBar],
    curve: &[f64],
) -> BTreeMap<i32, f64> {
    let mut years_map: BTreeMap<i32, (f64, f64)> = BTreeMap::new();

    for (i, bar) in bars.iter().enumerate() {
        if let Some(dt) = Utc.timestamp_millis_opt(bar.timestamp).single() {
            let year = dt.year();
            let val = *curve.get(i).unwrap_or(&10_000.0);
            years_map
                .entry(year)
                .and_modify(|(_, last)| *last = val)
                .or_insert((val, val));
        }
    }

    let mut result = BTreeMap::new();
    for (year, (first, last)) in years_map {
        let ret = if first > 0.0 {
            ((last / first) - 1.0) * 100.0
        } else {
            0.0
        };
        result.insert(year, ret);
    }
    result
}
