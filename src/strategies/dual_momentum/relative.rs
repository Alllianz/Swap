use super::volatility::calculate_realized_volatility;

/// Puntuador de Momentum Relativo Cross-Asset.
/// Normaliza los rendimientos por la volatilidad realizada de cada activo (Sharpe momentum)
/// para comparar equitativamente activos de muy distinta beta (ej. BTC vs ORO vs SP500).
pub struct RelativeMomentumScorer {
    pub short_window: usize,
    pub medium_window: usize,
}

impl Default for RelativeMomentumScorer {
    fn default() -> Self {
        Self {
            short_window: 21,  // ~3 semanas
            medium_window: 63, // ~3 meses
        }
    }
}

impl RelativeMomentumScorer {
    pub fn new(short_window: usize, medium_window: usize) -> Self {
        Self {
            short_window,
            medium_window,
        }
    }

    /// Calcula la puntuación de fuerza relativa ajustada por riesgo (Causal).
    pub fn compute_score(&self, prices: &[f64]) -> f64 {
        let n = prices.len();
        if n < self.medium_window + 1 {
            return 0.0;
        }

        let curr = prices[n - 1];
        let p_short = prices[n - self.short_window];
        let p_med = prices[n - self.medium_window];

        let ret_short = (curr / p_short.max(1e-6)) - 1.0;
        let ret_med = (curr / p_med.max(1e-6)) - 1.0;

        let vol = calculate_realized_volatility(prices, self.medium_window);

        // Ponderación 40% corto plazo + 60% mediano plazo, normalizado por volatilidad
        let combined_return = 0.40 * ret_short + 0.60 * ret_med;
        combined_return / vol.max(0.05)
    }
}
