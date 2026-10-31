use crate::data::kline::Kline;
use crate::engine::model::EquationModel;
use crate::engine::types::SignalAction;
use crate::features::ema::calculate_ema;

/// Estrategia cuantitativa clásica de cruce de medias móviles exponenciales (EMA Cross).
/// Diseñada con estricta causalidad temporal (0% Look-Ahead Bias).
pub struct EmaCrossStrategy {
    pub name_str: String,
    pub fast_period: usize,
    pub slow_period: usize,
}

impl EmaCrossStrategy {
    pub fn new(fast_period: usize, slow_period: usize) -> Self {
        Self {
            name_str: format!("EMA Cross ({}/{})", fast_period, slow_period),
            fast_period,
            slow_period,
        }
    }
}

impl EquationModel for EmaCrossStrategy {
    fn name(&self) -> &str {
        &self.name_str
    }

    fn evaluate(&mut self, history: &[Kline]) -> SignalAction {
        let n = history.len();
        if n < self.slow_period + 2 {
            return SignalAction::Flat;
        }

        // Extraer los precios de cierre disponibles hasta este instante causal
        let closes: Vec<f64> = history.iter().map(|k| k.close).collect();

        let fast_ema = calculate_ema(&closes, self.fast_period);
        let slow_ema = calculate_ema(&closes, self.slow_period);

        let curr_fast = fast_ema[n - 1];
        let curr_slow = slow_ema[n - 1];
        let prev_fast = fast_ema[n - 2];
        let prev_slow = slow_ema[n - 2];

        // Cruce alcista (Golden Cross)
        if prev_fast <= prev_slow && curr_fast > curr_slow {
            SignalAction::Buy
        }
        // Cruce bajista (Death Cross)
        else if prev_fast >= prev_slow && curr_fast < curr_slow {
            SignalAction::Sell
        } else {
            SignalAction::Flat
        }
    }
}
