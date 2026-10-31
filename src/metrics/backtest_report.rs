use crate::engine::trade_logger::TradeLog;
use crate::engine::types::ExitReason;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestReport {
    pub initial_capital: f64,
    pub final_capital: f64,
    pub net_profit: f64,
    pub total_return_pct: f64,
    pub total_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub win_rate_pct: f64,
    pub profit_factor: f64,
    pub gross_profit: f64,
    pub gross_loss: f64,
    pub total_fees: f64,
    pub max_drawdown_amount: f64,
    pub max_drawdown_pct_peak: f64,       // Drawdown tradicional vs pico
    pub max_drawdown_pct_capital: f64,    // Drawdown vs capital inicial (nuevo estándar Allianz)
    pub max_stagnation_bars: usize,
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub total_longs: usize,
    pub winning_longs: usize,
    pub losing_longs: usize,
    pub win_rate_long_pct: f64,
    pub total_shorts: usize,
    pub winning_shorts: usize,
    pub losing_shorts: usize,
    pub win_rate_short_pct: f64,
    pub liquidations: usize,
    pub score: f64,                       // Score oficial: PnL / MaxDD ($)
    pub trades: Vec<TradeLog>,
    pub equity_curve: Vec<(i64, f64)>,    // (timestamp, equity)
}

impl BacktestReport {
    pub fn max_drawdown_pct_capital(&self) -> f64 {
        self.max_drawdown_pct_capital
    }

    pub fn max_drawdown_pct_peak(&self) -> f64 {
        self.max_drawdown_pct_peak
    }

    pub fn score(&self) -> f64 {
        self.score
    }
}

pub fn calculate_metrics(
    initial_capital: f64,
    final_capital: f64,
    trades: Vec<TradeLog>,
    equity_curve: Vec<(i64, f64)>,
    max_stagnation_bars: usize,
) -> BacktestReport {
    let total_trades = trades.len();
    let mut winning_trades = 0;
    let mut losing_trades = 0;
    let mut winning_longs = 0;
    let mut losing_longs = 0;
    let mut winning_shorts = 0;
    let mut losing_shorts = 0;
    let mut gross_profit = 0.0;
    let mut gross_loss = 0.0;
    let mut total_fees = 0.0;
    let mut total_longs = 0;
    let mut total_shorts = 0;
    let mut liquidations = 0;

    let mut returns = Vec::with_capacity(total_trades);

    for t in &trades {
        total_fees += t.total_fees;
        if t.is_long {
            total_longs += 1;
            if t.net_pnl > 0.0 {
                winning_longs += 1;
            } else {
                losing_longs += 1;
            }
        } else {
            total_shorts += 1;
            if t.net_pnl > 0.0 {
                winning_shorts += 1;
            } else {
                losing_shorts += 1;
            }
        }

        if t.exit_reason == ExitReason::Liquidation {
            liquidations += 1;
        }

        if t.net_pnl > 0.0 {
            winning_trades += 1;
            gross_profit += t.net_pnl;
        } else {
            losing_trades += 1;
            gross_loss += t.net_pnl.abs();
        }

        returns.push(t.return_pct);
    }

    let net_profit = final_capital - initial_capital;
    let total_return_pct = if initial_capital > 0.0 {
        (net_profit / initial_capital) * 100.0
    } else {
        0.0
    };

    let win_rate_pct = if total_trades > 0 {
        (winning_trades as f64 / total_trades as f64) * 100.0
    } else {
        0.0
    };

    let win_rate_long_pct = if total_longs > 0 {
        (winning_longs as f64 / total_longs as f64) * 100.0
    } else {
        0.0
    };

    let win_rate_short_pct = if total_shorts > 0 {
        (winning_shorts as f64 / total_shorts as f64) * 100.0
    } else {
        0.0
    };

    let profit_factor = if gross_loss > 0.0 {
        gross_profit / gross_loss
    } else if gross_profit > 0.0 {
        f64::INFINITY
    } else {
        0.0
    };

    // Calcular Max Drawdown sobre la curva de balance
    let mut peak = initial_capital;
    let mut max_dd_amount = 0.0;
    let mut max_dd_pct_peak = 0.0;

    for &(_, eq) in &equity_curve {
        if eq > peak {
            peak = eq;
        }
        let dd = peak - eq;
        if dd > max_dd_amount {
            max_dd_amount = dd;
        }
        if peak > 0.0 {
            let dd_pct = (dd / peak) * 100.0;
            if dd_pct > max_dd_pct_peak {
                max_dd_pct_peak = dd_pct;
            }
        }
    }

    let max_dd_pct_capital = if initial_capital > 0.0 {
        (max_dd_amount / initial_capital) * 100.0
    } else {
        0.0
    };

    // Score oficial Allianz: PnL / MaxDD ($), con MaxDD acotado en >= 1.0
    let score = net_profit / max_dd_amount.max(1.0);

    // Sharpe y Sortino
    let mean_ret = if !returns.is_empty() {
        returns.iter().sum::<f64>() / (returns.len() as f64)
    } else {
        0.0
    };

    let variance = if returns.len() > 1 {
        returns
            .iter()
            .map(|r| (r - mean_ret).powi(2))
            .sum::<f64>()
            / ((returns.len() - 1) as f64)
    } else {
        0.0
    };
    let std_dev = variance.sqrt();

    let sharpe_ratio = if std_dev > 1e-8 {
        (mean_ret / std_dev) * (returns.len() as f64).sqrt()
    } else {
        0.0
    };

    let downside_variance = if returns.len() > 1 {
        returns
            .iter()
            .filter(|&&r| r < 0.0)
            .map(|r| r.powi(2))
            .sum::<f64>()
            / ((returns.len() - 1) as f64)
    } else {
        0.0
    };
    let downside_std_dev = downside_variance.sqrt();

    let sortino_ratio = if downside_std_dev > 1e-8 {
        (mean_ret / downside_std_dev) * (returns.len() as f64).sqrt()
    } else {
        0.0
    };

    BacktestReport {
        initial_capital,
        final_capital,
        net_profit,
        total_return_pct,
        total_trades,
        winning_trades,
        losing_trades,
        win_rate_pct,
        profit_factor,
        gross_profit,
        gross_loss,
        total_fees,
        max_drawdown_amount: max_dd_amount,
        max_drawdown_pct_peak: max_dd_pct_peak,
        max_drawdown_pct_capital: max_dd_pct_capital,
        max_stagnation_bars,
        sharpe_ratio,
        sortino_ratio,
        total_longs,
        winning_longs,
        losing_longs,
        win_rate_long_pct,
        total_shorts,
        winning_shorts,
        losing_shorts,
        win_rate_short_pct,
        liquidations,
        score,
        trades,
        equity_curve,
    }
}
