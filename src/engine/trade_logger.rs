use super::types::ExitReason;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeLog {
    pub trade_id: usize,
    pub is_long: bool,
    pub entry_price: f64,
    pub exit_price: f64,
    pub amount: f64,
    pub entry_step: usize,
    pub exit_step: usize,
    pub entry_timestamp: i64,
    pub exit_timestamp: i64,
    pub gross_pnl: f64,
    pub total_fees: f64,
    pub net_pnl: f64,
    pub return_pct: f64,
    pub exit_reason: ExitReason,
}

impl TradeLog {
    pub fn holding_bars(&self) -> usize {
        self.exit_step.saturating_sub(self.entry_step)
    }

    pub fn is_winner(&self) -> bool {
        self.net_pnl > 0.0
    }
}
