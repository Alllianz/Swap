use crate::data::kline::Kline;
use crate::engine::model::EquationModel;
use crate::engine::types::SignalAction;
use crate::features::rsi::calculate_rsi;

/// Estrategia cuantitativa de reversión a la media con RSI de Wilder
pub struct RsiReversionStrategy {
    pub name_str: String,
    pub period: usize,
    pub oversold_threshold: f64,
    pub overbought_threshold: f64,
}

impl RsiReversionStrategy {
    pub fn new(period: usize, oversold: f64, overbought: f64) -> Self {
        Self {
            name_str: format!("RSI Reversion ({}, {:.0}/{:.0})", period, oversold, overbought),
            period,
            oversold_threshold: oversold,
            overbought_threshold: overbought,
        }
    }
}

impl EquationModel for RsiReversionStrategy {
    fn name(&self) -> &str {
        &self.name_str
    }

    fn evaluate(&mut self, history: &[Kline]) -> SignalAction {
        let n = history.len();
        if n < self.period + 3 {
            return SignalAction::Flat;
        }

        let closes: Vec<f64> = history.iter().map(|k| k.close).collect();
        let rsi_series = calculate_rsi(&closes, self.period);

        let curr_rsi = rsi_series[n - 1];
        let prev_rsi = rsi_series[n - 2];

        // Rebote desde sobreventa: prev_rsi < oversold y ahora cruza hacia arriba
        if prev_rsi <= self.oversold_threshold && curr_rsi > self.oversold_threshold {
            SignalAction::Buy
        }
        // Agotamiento en sobrecompra: prev_rsi > overbought y ahora cruza hacia abajo
        else if prev_rsi >= self.overbought_threshold && curr_rsi < self.overbought_threshold {
            SignalAction::Sell
        } else {
            SignalAction::Flat
        }
    }
}
