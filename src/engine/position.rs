use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub is_long: bool,
    pub entry_price: f64,
    pub amount: f64,
    pub entry_step: usize,
    pub entry_timestamp: i64,
    pub stop_loss: f64,
    pub take_profit: Option<f64>,
    pub liquidation_price: f64,
    pub initial_margin: f64,
}

impl Position {
    pub fn new(
        is_long: bool,
        entry_price: f64,
        amount: f64,
        entry_step: usize,
        entry_timestamp: i64,
        stop_loss: f64,
        take_profit: Option<f64>,
        liquidation_price: f64,
        initial_margin: f64,
    ) -> Self {
        Self {
            is_long,
            entry_price,
            amount,
            entry_step,
            entry_timestamp,
            stop_loss,
            take_profit,
            liquidation_price,
            initial_margin,
        }
    }
}
