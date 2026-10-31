use crate::engine::swap::model::{SwapDecision, TriAssetModel};
use crate::engine::swap::types::{AssetId, SynchronizedBar};
use super::absolute::AbsoluteMomentumFilter;
use super::relative::RelativeMomentumScorer;

/// Estrategia Cuantitativa de Dual Momentum Multiactivo (Ruta B).
/// Combina Momentum Relativo (Risk-Adjusted Cross-Asset Ranking)
/// con Momentum Absoluto (Filtro de régimen bajista / Preservación en Cash u Oro).
pub struct DualMomentumStrategy {
    pub name: String,
    pub relative_scorer: RelativeMomentumScorer,
    pub absolute_filter: AbsoluteMomentumFilter,
}

impl Default for DualMomentumStrategy {
    fn default() -> Self {
        Self {
            name: "Ruta B: Dual Momentum Sistemático (Cross-Asset)".to_string(),
            relative_scorer: RelativeMomentumScorer::default(),
            absolute_filter: AbsoluteMomentumFilter::default(),
        }
    }
}

impl DualMomentumStrategy {
    pub fn new(short_window: usize, medium_window: usize, absolute_window: usize) -> Self {
        Self {
            name: format!(
                "Ruta B: Dual Momentum (S:{}/M:{}/Abs:{})",
                short_window, medium_window, absolute_window
            ),
            relative_scorer: RelativeMomentumScorer::new(short_window, medium_window),
            absolute_filter: AbsoluteMomentumFilter::new(absolute_window, 120),
        }
    }
}

impl TriAssetModel for DualMomentumStrategy {
    fn name(&self) -> &str {
        &self.name
    }

    fn evaluate(&mut self, history: &[SynchronizedBar], current_step: usize) -> SwapDecision {
        let n = current_step + 1;
        if n < self.relative_scorer.medium_window + 5 {
            return SwapDecision {
                target_asset: AssetId::Cash,
                confidence_score: 0.0,
                explanation: "Inicializando histórico para cálculo de momentum".to_string(),
            };
        }

        // Extracción causal de series de precios cerrados
        let btc_prices: Vec<f64> = history[0..n].iter().map(|b| b.btc_close).collect();
        let gold_prices: Vec<f64> = history[0..n].iter().map(|b| b.gold_close).collect();
        let sp500_prices: Vec<f64> = history[0..n].iter().map(|b| b.sp500_close).collect();

        // 1. Puntuación de Momentum Relativo
        let score_btc = self.relative_scorer.compute_score(&btc_prices);
        let score_gold = self.relative_scorer.compute_score(&gold_prices);
        let score_sp500 = self.relative_scorer.compute_score(&sp500_prices);

        // 2. Filtro de Momentum Absoluto
        let abs_btc = self.absolute_filter.is_bullish(&btc_prices);
        let abs_gold = self.absolute_filter.is_bullish(&gold_prices);
        let abs_sp500 = self.absolute_filter.is_bullish(&sp500_prices);

        // Ordenamiento por puntuación relativa
        let mut candidates = [
            (AssetId::Btc, score_btc, abs_btc),
            (AssetId::Gold, score_gold, abs_gold),
            (AssetId::Sp500, score_sp500, abs_sp500),
        ];
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let (best_asset, best_score, best_is_bullish) = candidates[0];
        let second_score = candidates[1].1;
        let score_margin = best_score - second_score;

        if best_is_bullish && best_score > 0.0 {
            SwapDecision {
                target_asset: best_asset,
                confidence_score: score_margin.abs(),
                explanation: format!(
                    "Líder: {} (Score: {:.2}, Margen: +{:.2}) | Tendencia Absoluta Positiva",
                    best_asset.symbol(),
                    best_score,
                    score_margin
                ),
            }
        } else if abs_gold && score_gold > -0.5 {
            // Refugio defensivo en Oro si el líder falló pero el Oro está sano
            SwapDecision {
                target_asset: AssetId::Gold,
                confidence_score: 0.1,
                explanation: format!(
                    "Régimen adverso para líder ({}); Refugio táctico en ORO (Score ORO: {:.2})",
                    best_asset.symbol(),
                    score_gold
                ),
            }
        } else {
            // Salida a Cash si todo el mercado está cayendo
            SwapDecision {
                target_asset: AssetId::Cash,
                confidence_score: 0.2,
                explanation: format!(
                    "Pánico / Tendencia Absoluta Negativa en los 3 activos. Preservación en CASH (BTC: {:.2}, ORO: {:.2}, SP500: {:.2})",
                    score_btc, score_gold, score_sp500
                ),
            }
        }
    }
}
