use super::models::{
    AssetBenchmarkDto, RobustSectionMetricsDto, SectionKpiDto, StrategyAllocationPointDto,
    SwapDashboardPayload, SwapDashboardStrategyDto, SwapMetricsDto, SwapTradeDto,
    YearlyPerformanceDto,
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
    build_swap_dashboard_payload_with_meta(
        bars,
        is_split_idx,
        strategies_input,
        "SWAP Multi-Asset Rotation Dashboard",
        "1D",
    )
}

pub fn build_swap_dashboard_payload_with_meta(
    bars: &[SynchronizedBar],
    is_split_idx: usize,
    strategies_input: &[StrategyInput],
    title: &str,
    timeframe: &str,
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

        // Alinear la curva de equidad con el dataset completo (rellenar período warmup inicial con capital inicial)
        let start_offset = total_bars.saturating_sub(rep.equity_curve.len());
        let mut equity_curve = Vec::with_capacity(total_bars);
        for _ in 0..start_offset {
            equity_curve.push(rep.initial_capital);
        }
        equity_curve.extend(rep.equity_curve.iter().map(|(_, eq)| *eq));

        let drawdown_curve = calculate_drawdown_curve(&equity_curve);

        // Reconstruir timeline de asignación de activos
        let allocations = build_allocations_timeline(bars, rep, &equity_curve, &drawdown_curve);

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

        // Métricas robustas multisección (IS / OOS / COMPLETO)
        let is_section = compute_section_kpi(bars, &equity_curve, &rep.trades, 0, is_split_idx, rep.initial_capital);
        let oos_section = compute_section_kpi(bars, &equity_curve, &rep.trades, is_split_idx, total_bars, rep.initial_capital);
        let completo_section = compute_section_kpi(bars, &equity_curve, &rep.trades, 0, total_bars, rep.initial_capital);

        let score_is = is_section.score;
        let score_oos = oos_section.score;
        let score_completo = completo_section.score;

        let retention_pct = if score_is.abs() > 1e-6 {
            (score_oos / score_is) * 100.0
        } else {
            0.0
        };

        let vel_is = is_split_idx as f64;
        let vel_oos = (total_bars.saturating_sub(is_split_idx)) as f64;
        let retention_velas_pct = if vel_oos > 0.0 {
            retention_pct * (vel_is / vel_oos)
        } else {
            0.0
        };

        let section_metrics = RobustSectionMetricsDto {
            score_is,
            score_oos,
            score_completo,
            retention_pct,
            retention_velas_pct,
            is_section,
            oos_section,
            completo_section,
        };

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
            exposure_short_btc_pct: rep.exposure_short_btc_pct,
            exposure_short_gold_pct: rep.exposure_short_gold_pct,
            exposure_short_sp500_pct: rep.exposure_short_sp500_pct,
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
            margin_mode: rep.margin_mode.name().to_string(),
            leverage: rep.leverage,
            color: input.color.clone(),
            initial_capital: rep.initial_capital,
            final_capital: rep.final_capital,
            net_profit: rep.net_profit,
            metrics,
            section_metrics,
            equity_curve,
            drawdown_curve,
            allocations,
            trades,
            yearly_returns,
        });
    }

    SwapDashboardPayload {
        title: title.to_string(),
        timeframe: timeframe.to_string(),
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

fn compute_section_kpi(
    bars: &[SynchronizedBar],
    equity_curve: &[f64],
    trades: &[crate::engine::swap::types::SwapTradeRecord],
    start_step: usize,
    end_step: usize,
    _initial_cap_ref: f64,
) -> SectionKpiDto {
    if start_step >= end_step || bars.is_empty() || equity_curve.is_empty() {
        return SectionKpiDto {
            score: 0.0,
            return_pct: 0.0,
            pnl_usd: 0.0,
            diff_vs_bh_pct: 0.0,
            bh_return_pct: 0.0,
            trades: 0,
            total_bars: 0,
            max_dd_pct: 0.0,
            max_dd_usd: 0.0,
            max_dd_pico_pct: 0.0,
            max_stagnation_bars: 0,
            bh_max_dd_pico_pct: 0.0,
            start_date: "".to_string(),
            end_date: "".to_string(),
        };
    }

    let actual_end = end_step.min(bars.len()).min(equity_curve.len());
    let sub_bars = &bars[start_step..actual_end];
    let sub_eq = &equity_curve[start_step..actual_end];

    let start_date = sub_bars[0].date_string();
    let end_date = sub_bars.last().unwrap().date_string();
    let total_bars = sub_bars.len();

    let eq_start = sub_eq[0];
    let eq_end = *sub_eq.last().unwrap();
    let pnl_usd = eq_end - eq_start;
    let return_pct = if eq_start > 0.0 {
        ((eq_end / eq_start) - 1.0) * 100.0
    } else {
        0.0
    };

    // Drawdowns
    let mut peak = eq_start;
    let mut max_dd_usd = 0.0;
    let mut max_dd_pico_pct = 0.0;
    for &val in sub_eq {
        if val > peak {
            peak = val;
        }
        let drop = peak - val;
        if drop > max_dd_usd {
            max_dd_usd = drop;
        }
        let p = if peak > 0.0 {
            (drop / peak) * 100.0
        } else {
            0.0
        };
        if p > max_dd_pico_pct {
            max_dd_pico_pct = p;
        }
    }
    // El Drawdown se calcula estrictamente contra el capital total en ese momento (peak previo)
    let max_dd_pct = max_dd_pico_pct;

    // Max stagnation
    let mut max_stagnation_bars = 0;
    let mut cur_peak = eq_start;
    let mut cur_peak_idx = 0;
    for (i, &val) in sub_eq.iter().enumerate() {
        if val > cur_peak {
            let stag = i.saturating_sub(cur_peak_idx);
            if stag > max_stagnation_bars {
                max_stagnation_bars = stag;
            }
            cur_peak = val;
            cur_peak_idx = i;
        }
    }
    let last_stag = sub_eq.len().saturating_sub(cur_peak_idx);
    if last_stag > max_stagnation_bars {
        max_stagnation_bars = last_stag;
    }

    // Trades
    let trades_count = trades
        .iter()
        .filter(|t| t.step >= start_step && t.step < actual_end)
        .count();

    // Score de Rendimiento vs Riesgo (PnL / MaxDD USD, acotado en 1.0)
    let score = pnl_usd / (if max_dd_usd > 1.0 { max_dd_usd } else { 1.0 });

    // Benchmark BTC en el tramo
    let btc_start = sub_bars[0].btc_close.max(1e-6);
    let btc_end = sub_bars.last().unwrap().btc_close;
    let bh_return_pct = ((btc_end / btc_start) - 1.0) * 100.0;
    let diff_vs_bh_pct = return_pct - bh_return_pct;

    let mut btc_peak = btc_start;
    let mut bh_max_dd_pico_pct = 0.0;
    for b in sub_bars {
        if b.btc_close > btc_peak {
            btc_peak = b.btc_close;
        }
        let drop_pct = if btc_peak > 0.0 {
            ((btc_peak - b.btc_close) / btc_peak) * 100.0
        } else {
            0.0
        };
        if drop_pct > bh_max_dd_pico_pct {
            bh_max_dd_pico_pct = drop_pct;
        }
    }

    SectionKpiDto {
        score,
        return_pct,
        pnl_usd,
        diff_vs_bh_pct,
        bh_return_pct,
        trades: trades_count,
        total_bars,
        max_dd_pct,
        max_dd_usd,
        max_dd_pico_pct,
        max_stagnation_bars,
        bh_max_dd_pico_pct,
        start_date,
        end_date,
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
        create_benchmark_dto(
            BenchmarkMeta {
                id: "btc",
                name: "Bitcoin (Buy & Hold)",
                symbol: "BTC",
                color: "#F7931A",
                p0: bars[0].btc_close,
                p1: bars.last().unwrap().btc_close,
            },
            initial_capital,
            btc_curve,
            bars.len(),
        ),
        create_benchmark_dto(
            BenchmarkMeta {
                id: "gold",
                name: "Oro / PAXG (Buy & Hold)",
                symbol: "GOLD",
                color: "#FFD700",
                p0: bars[0].gold_close,
                p1: bars.last().unwrap().gold_close,
            },
            initial_capital,
            gold_curve,
            bars.len(),
        ),
        create_benchmark_dto(
            BenchmarkMeta {
                id: "sp500",
                name: "S&P 500 (Buy & Hold)",
                symbol: "SP500",
                color: "#00B0FF",
                p0: bars[0].sp500_close,
                p1: bars.last().unwrap().sp500_close,
            },
            initial_capital,
            sp500_curve,
            bars.len(),
        ),
        create_benchmark_dto(
            BenchmarkMeta {
                id: "equal_weight",
                name: "Cartera Equitativa (1/3 c/u)",
                symbol: "EQ3",
                color: "#E040FB",
                p0: 1.0,
                p1: 1.0,
            },
            initial_capital,
            eq_curve,
            bars.len(),
        ),
    ]
}

struct BenchmarkMeta<'a> {
    id: &'a str,
    name: &'a str,
    symbol: &'a str,
    color: &'a str,
    p0: f64,
    p1: f64,
}

fn create_benchmark_dto(
    meta: BenchmarkMeta<'_>,
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
        id: meta.id.to_string(),
        name: meta.name.to_string(),
        symbol: meta.symbol.to_string(),
        color: meta.color.to_string(),
        initial_price: meta.p0,
        final_price: meta.p1,
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
    report: &crate::engine::swap::report::SwapReport,
    equity_curve: &[f64],
    drawdown_curve: &[f64],
) -> Vec<StrategyAllocationPointDto> {
    let mut timeline = Vec::with_capacity(bars.len());
    let total_bars = bars.len();
    let alloc_len = report.daily_allocations.len();
    let start_offset = total_bars.saturating_sub(alloc_len);

    for (step, bar) in bars.iter().enumerate() {
        let eq = *equity_curve.get(step).unwrap_or(&10_000.0);
        let dd = *drawdown_curve.get(step).unwrap_or(&0.0);

        if step >= start_offset && (step - start_offset) < alloc_len {
            let (a1, w1, a2_opt, w2) = report.daily_allocations[step - start_offset];
            let mut w_btc = 0.0;
            let mut w_gold = 0.0;
            let mut w_sp500 = 0.0;
            let mut w_sh_btc = 0.0;
            let mut w_sh_gold = 0.0;
            let mut w_sh_sp500 = 0.0;
            let mut w_cash = 0.0;

            let mut add_w = |asset: crate::engine::swap::types::AssetId, weight: f64| {
                match asset {
                    crate::engine::swap::types::AssetId::Btc => w_btc += weight,
                    crate::engine::swap::types::AssetId::Gold => w_gold += weight,
                    crate::engine::swap::types::AssetId::Sp500 => w_sp500 += weight,
                    crate::engine::swap::types::AssetId::ShortBtc => w_sh_btc += weight,
                    crate::engine::swap::types::AssetId::ShortGold => w_sh_gold += weight,
                    crate::engine::swap::types::AssetId::ShortSp500 => w_sh_sp500 += weight,
                    crate::engine::swap::types::AssetId::Cash => w_cash += weight,
                }
            };

            add_w(a1, w1);
            if let Some(a2) = a2_opt {
                add_w(a2, w2);
            }

            let asset_label = if let Some(a2) = a2_opt {
                if a1 == a2 {
                    a1.symbol().to_string()
                } else {
                    format!("{:.0}% {} + {:.0}% {}", w1 * 100.0, a1.symbol(), w2 * 100.0, a2.symbol())
                }
            } else {
                a1.symbol().to_string()
            };

            timeline.push(StrategyAllocationPointDto {
                timestamp: bar.timestamp,
                date: bar.date_string(),
                asset: asset_label,
                equity: eq,
                drawdown_pct: dd,
                weight_btc: w_btc,
                weight_gold: w_gold,
                weight_sp500: w_sp500,
                weight_sh_btc: w_sh_btc,
                weight_sh_gold: w_sh_gold,
                weight_sh_sp500: w_sh_sp500,
                weight_cash: w_cash,
            });
        } else {
            timeline.push(StrategyAllocationPointDto {
                timestamp: bar.timestamp,
                date: bar.date_string(),
                asset: "CASH".to_string(),
                equity: eq,
                drawdown_pct: dd,
                weight_btc: 0.0,
                weight_gold: 0.0,
                weight_sp500: 0.0,
                weight_sh_btc: 0.0,
                weight_sh_gold: 0.0,
                weight_sh_sp500: 0.0,
                weight_cash: 1.0,
            });
        }
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
