/// Filtro causal de Momentum Absoluto (Tendencia intrínseca vs Preservación).
/// Verifica si un activo tiene tendencia alcista real o si debe descartarse para preservar capital.
pub struct AbsoluteMomentumFilter {
    pub lookback_bars: usize,
    pub sma_filter_bars: usize,
}

impl Default for AbsoluteMomentumFilter {
    fn default() -> Self {
        Self {
            lookback_bars: 60,
            sma_filter_bars: 100,
        }
    }
}

impl AbsoluteMomentumFilter {
    pub fn new(lookback_bars: usize, sma_filter_bars: usize) -> Self {
        Self {
            lookback_bars,
            sma_filter_bars,
        }
    }

    /// Evalúa si el activo pasa el filtro de Momentum Absoluto (Causal).
    /// Requiere: Retorno en la ventana > 0 O precio actual > Media Móvil Simple.
    pub fn is_bullish(&self, prices: &[f64]) -> bool {
        let n = prices.len();
        if n < self.lookback_bars {
            return true; // Período de inicio sin historial suficiente
        }

        let current_price = prices[n - 1];
        let past_price = prices[n - self.lookback_bars];
        let raw_return = (current_price / past_price.max(1e-6)) - 1.0;

        // Comprobación de SMA si hay suficiente historia
        let passes_sma = if n >= self.sma_filter_bars {
            let sum_sma: f64 = prices[(n - self.sma_filter_bars)..n].iter().sum();
            let sma = sum_sma / (self.sma_filter_bars as f64);
            current_price >= sma
        } else {
            raw_return > -0.02
        };

        raw_return > 0.0 || passes_sma
    }
}
