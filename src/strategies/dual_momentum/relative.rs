use super::volatility::{calculate_downside_deviation, calculate_realized_volatility};

/// Modalidad de ajuste por riesgo para la normalización del Momentum Relativo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskAdjustmentMode {
    /// Ratio Sortino: Normaliza por la desviación a la baja (Downside Deviation).
    /// NO penaliza rallies alcistas ni rendimientos positivos explosivos; castiga únicamente
    /// la volatilidad de caídas (semivarianza bajista). Ideal para activos asimétricos como BTC.
    SortinoDownside,
    /// Ratio Sharpe: Normaliza por la volatilidad simétrica total clásica.
    SharpeTotalVolatility,
}

/// Puntuador de Momentum Relativo Cross-Asset.
/// Normaliza los rendimientos por el riesgo específico de cada activo
/// para comparar equitativamente instrumentos de muy distinta beta (ej. BTC vs ORO vs SP500).
#[derive(Debug, Clone)]
pub struct RelativeMomentumScorer {
    pub short_window: usize,
    pub medium_window: usize,
    pub risk_mode: RiskAdjustmentMode,
}

impl Default for RelativeMomentumScorer {
    fn default() -> Self {
        Self {
            short_window: 21,  // ~3 semanas
            medium_window: 63, // ~3 meses
            risk_mode: RiskAdjustmentMode::SortinoDownside,
        }
    }
}

impl RelativeMomentumScorer {
    pub fn new(short_window: usize, medium_window: usize) -> Self {
        Self {
            short_window,
            medium_window,
            risk_mode: RiskAdjustmentMode::SortinoDownside,
        }
    }

    /// Permite configurar explícitamente el modo de riesgo (Sortino vs Sharpe).
    pub fn with_risk_mode(mut self, risk_mode: RiskAdjustmentMode) -> Self {
        self.risk_mode = risk_mode;
        self
    }

    /// Calcula la puntuación de fuerza relativa ajustada por riesgo (Causal, 0% look-ahead).
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

        // Ponderación causal: 40% corto plazo + 60% mediano plazo
        let combined_return = 0.40 * ret_short + 0.60 * ret_med;

        let risk_measure = match self.risk_mode {
            RiskAdjustmentMode::SortinoDownside => {
                calculate_downside_deviation(prices, self.medium_window)
            }
            RiskAdjustmentMode::SharpeTotalVolatility => {
                calculate_realized_volatility(prices, self.medium_window).max(0.05)
            }
        };

        combined_return / risk_measure
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relative_scorer_sortino_favors_rally() {
        // En una serie fuertemente alcista con poca caída, Sortino debe dar un score superior a Sharpe
        let mut prices = vec![100.0];
        for i in 1..70 {
            prices.push(100.0 * (1.0 + (i as f64) * 0.015));
        }

        let scorer_sortino = RelativeMomentumScorer::new(21, 63);
        let scorer_sharpe = RelativeMomentumScorer::new(21, 63)
            .with_risk_mode(RiskAdjustmentMode::SharpeTotalVolatility);

        let score_sortino = scorer_sortino.compute_score(&prices);
        let score_sharpe = scorer_sharpe.compute_score(&prices);

        assert!(
            score_sortino > score_sharpe,
            "Sortino ({}) no debe penalizar el rally, superando a Sharpe ({})",
            score_sortino,
            score_sharpe
        );
    }
}
