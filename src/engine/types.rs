use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalAction {
    Flat,
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExitReason {
    SignalReversal,
    TakeProfit,
    StopLoss,
    Liquidation,
    MaxHoldingTime,
    EndOfBacktest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestConfig {
    pub initial_capital: f64,
    pub leverage: f64,
    pub position_size_pct: f64, // ej. 0.10 para 10% del capital
    pub fee_rate: f64,          // ej. 0.0005 (0.05% por operación)
    pub max_holding_bars: usize,
    pub use_compound: bool,     // Interés compuesto vs balance nominal fijo
    pub atr_sl_multiplier: f64, // Multiplicador ATR para Stop Loss (0.0 = desactivado)
    #[serde(default)]
    pub max_holding_bars_long: Option<usize>,
    #[serde(default)]
    pub max_holding_bars_short: Option<usize>,
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            initial_capital: 10_000.0,
            leverage: 10.0,
            position_size_pct: 0.10,
            fee_rate: 0.0005,
            max_holding_bars: 24,
            use_compound: false,
            atr_sl_multiplier: 0.0,
            max_holding_bars_long: None,
            max_holding_bars_short: None,
        }
    }
}
