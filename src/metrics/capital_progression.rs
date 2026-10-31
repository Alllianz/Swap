use crate::engine::trade_logger::TradeLog;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CapitalPeriodCheckpoint {
    pub period_idx: usize,
    pub start_timestamp: i64,
    pub end_timestamp: i64,
    pub initial_equity: f64,
    pub final_equity: f64,
    pub return_pct: f64,
    pub max_dd_pct: f64,
    pub trades_count: usize,
    pub win_rate_pct: f64,
    pub is_positive: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MilestoneHit {
    pub label: String,
    pub target_equity: f64,
    pub reached: bool,
    pub timestamp: i64,
    pub bar_index: usize,
    pub days_elapsed: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CapitalProgressionSummary {
    pub total_days: f64,
    pub total_years: f64,
    pub initial_capital: f64,
    pub final_capital: f64,
    pub total_return_pct: f64,
    pub max_drawdown_pct: f64,
    pub cagr_pct: f64,
    pub calmar_ratio: f64,
    pub max_underwater_bars: usize,
    pub checkpoints: Vec<CapitalPeriodCheckpoint>,
    pub milestones: Vec<MilestoneHit>,
}

pub fn compute_capital_progression(
    curve: &[(i64, f64)],
    trades: &[TradeLog],
    n_periods: usize,
) -> CapitalProgressionSummary {
    if curve.is_empty() {
        return CapitalProgressionSummary {
            total_days: 0.0,
            total_years: 0.0,
            initial_capital: 10000.0,
            final_capital: 10000.0,
            total_return_pct: 0.0,
            max_drawdown_pct: 0.0,
            cagr_pct: 0.0,
            calmar_ratio: 0.0,
            max_underwater_bars: 0,
            checkpoints: Vec::new(),
            milestones: Vec::new(),
        };
    }

    let initial_capital = curve.first().map(|&(_, eq)| eq).unwrap_or(10000.0);
    let final_capital = curve.last().map(|&(_, eq)| eq).unwrap_or(initial_capital);
    let start_ts = curve.first().map(|&(ts, _)| ts).unwrap_or(0);
    let end_ts = curve.last().map(|&(ts, _)| ts).unwrap_or(0);

    let total_days = ((end_ts - start_ts) as f64 / 86_400_000.0).max(0.001);
    let total_years = total_days / 365.25;

    let total_return_pct = if initial_capital > 0.0 {
        ((final_capital - initial_capital) / initial_capital) * 100.0
    } else {
        0.0
    };

    let cagr_pct = if total_years > 0.05 && initial_capital > 0.0 && final_capital > 0.0 {
        ((final_capital / initial_capital).powf(1.0 / total_years) - 1.0) * 100.0
    } else {
        total_return_pct
    };

    let mut peak = initial_capital;
    let mut max_dd_pct = 0.0;
    let mut current_underwater = 0;
    let mut max_underwater_bars = 0;

    for &(_, eq) in curve {
        if eq > peak {
            peak = eq;
            current_underwater = 0;
        } else {
            current_underwater += 1;
            if current_underwater > max_underwater_bars {
                max_underwater_bars = current_underwater;
            }
        }
        let dd = peak - eq;
        let dd_pct = if peak > 0.0 { (dd / peak) * 100.0 } else { 0.0 };
        if dd_pct > max_dd_pct {
            max_dd_pct = dd_pct;
        }
    }

    let calmar_ratio = if max_dd_pct > 0.0 {
        cagr_pct / max_dd_pct
    } else {
        0.0
    };

    // Checkpoints temporales
    let mut checkpoints = Vec::new();
    let num_periods = n_periods.max(1);
    let step_size = (curve.len() as f64 / num_periods as f64).ceil() as usize;

    for p in 0..num_periods {
        let p_start = p * step_size;
        if p_start >= curve.len() {
            break;
        }
        let p_end = ((p + 1) * step_size).min(curve.len()) - 1;

        let init_eq = curve[p_start].1;
        let fin_eq = curve[p_end].1;
        let p_ret = if init_eq > 0.0 {
            ((fin_eq - init_eq) / init_eq) * 100.0
        } else {
            0.0
        };

        let start_time = curve[p_start].0;
        let end_time = curve[p_end].0;

        let period_trades: Vec<&TradeLog> = trades
            .iter()
            .filter(|t| t.exit_timestamp >= start_time && t.exit_timestamp <= end_time)
            .collect();

        let t_count = period_trades.len();
        let win_count = period_trades.iter().filter(|t| t.net_pnl > 0.0).count();
        let win_rate = if t_count > 0 {
            (win_count as f64 / t_count as f64) * 100.0
        } else {
            0.0
        };

        checkpoints.push(CapitalPeriodCheckpoint {
            period_idx: p + 1,
            start_timestamp: start_time,
            end_timestamp: end_time,
            initial_equity: init_eq,
            final_equity: fin_eq,
            return_pct: p_ret,
            max_dd_pct: 0.0,
            trades_count: t_count,
            win_rate_pct: win_rate,
            is_positive: p_ret >= 0.0,
        });
    }

    // Hitos (+10%, +25%, +50%, +100%, +200%, +500%)
    let targets = [
        ("+10%", 1.10),
        ("+25%", 1.25),
        ("+50%", 1.50),
        ("+100% (2X)", 2.00),
        ("+200% (3X)", 3.00),
        ("+500% (6X)", 6.00),
        ("+1000% (11X)", 11.00),
    ];

    let mut milestones = Vec::new();
    for (label, mult) in targets {
        let target_eq = initial_capital * mult;
        let mut hit = false;
        let mut hit_ts = 0;
        let mut hit_bar = 0;
        let mut hit_days = 0.0;

        for (bar_idx, &(ts, eq)) in curve.iter().enumerate() {
            if eq >= target_eq {
                hit = true;
                hit_ts = ts;
                hit_bar = bar_idx;
                hit_days = (ts - start_ts) as f64 / 86_400_000.0;
                break;
            }
        }

        milestones.push(MilestoneHit {
            label: label.to_string(),
            target_equity: target_eq,
            reached: hit,
            timestamp: hit_ts,
            bar_index: hit_bar,
            days_elapsed: hit_days,
        });
    }

    CapitalProgressionSummary {
        total_days,
        total_years,
        initial_capital,
        final_capital,
        total_return_pct,
        max_drawdown_pct: max_dd_pct,
        cagr_pct,
        calmar_ratio,
        max_underwater_bars,
        checkpoints,
        milestones,
    }
}
