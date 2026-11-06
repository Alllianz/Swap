use crate::engine::swap::model::{SwapDecision, TriAssetModel};
use crate::engine::swap::types::{AssetId, SynchronizedBar};
use super::absolute::AbsoluteMomentumFilter;
use super::relative::{RelativeMomentumScorer, RiskAdjustmentMode};

/// Estrategia Cuantitativa de Dual Momentum Multiactivo 50/50 (Ruta B - Dual Allocation).
/// Asigna 50% de la cartera al mejor activo y 50% al segundo mejor activo.
/// Regla Dual Momentum Clásica: Si el 1° o el 2° activo tienen momentum absoluto negativo (tendencia bajista),
/// transfiere esa porción (50%) a CASH para recortar el Drawdown de forma institucional.
/// Posiciones Cortas: Si un activo colapsa con momentum negativo extremo, asigna el 50% a posición corta 1x.
pub struct DualMomentumStrategy {
    pub name: String,
    pub relative_scorer: RelativeMomentumScorer,
    pub absolute_filter: AbsoluteMomentumFilter,
    pub allow_shorts: bool,
}

impl Default for DualMomentumStrategy {
    fn default() -> Self {
        Self {
            name: "Ruta B: Dual Momentum Sistemático 50/50 (Cross-Asset Dual Allocation)".to_string(),
            relative_scorer: RelativeMomentumScorer::default(),
            absolute_filter: AbsoluteMomentumFilter::default(),
            allow_shorts: true,
        }
    }
}

impl DualMomentumStrategy {
    pub fn new(short_window: usize, medium_window: usize, absolute_window: usize) -> Self {
        Self {
            name: format!(
                "Ruta B: Dual Momentum 50/50 (S:{}/M:{}/Abs:{})",
                short_window, medium_window, absolute_window
            ),
            relative_scorer: RelativeMomentumScorer::new(short_window, medium_window),
            absolute_filter: AbsoluteMomentumFilter::new(absolute_window, 120),
            allow_shorts: true,
        }
    }

    /// Permite personalizar el modo de ajuste por riesgo (Sortino vs Sharpe).
    pub fn with_risk_mode(mut self, risk_mode: RiskAdjustmentMode) -> Self {
        self.relative_scorer = self.relative_scorer.with_risk_mode(risk_mode);
        self
    }

    /// Habilita o deshabilita la toma de posiciones cortas tácticas.
    pub fn with_shorts(mut self, allow_shorts: bool) -> Self {
        self.allow_shorts = allow_shorts;
        self
    }
}

impl TriAssetModel for DualMomentumStrategy {
    fn name(&self) -> &str {
        &self.name
    }

    fn evaluate(&mut self, history: &[SynchronizedBar], current_step: usize) -> SwapDecision {
        let n = current_step + 1;
        if n < self.relative_scorer.medium_window + 5 {
            return SwapDecision::dual(
                AssetId::Cash,
                AssetId::Cash,
                0.0,
                "Inicializando histórico causal para cálculo de momentum".to_string(),
            );
        }

        // Extracción causal de series de precios cerrados
        let btc_prices: Vec<f64> = history[0..n].iter().map(|b| b.btc_close).collect();
        let gold_prices: Vec<f64> = history[0..n].iter().map(|b| b.gold_close).collect();
        let sp500_prices: Vec<f64> = history[0..n].iter().map(|b| b.sp500_close).collect();

        // 1. Puntuación de Momentum Relativo Asimétrico (Sortino)
        let score_btc = self.relative_scorer.compute_score(&btc_prices);
        let score_gold = self.relative_scorer.compute_score(&gold_prices);
        let score_sp500 = self.relative_scorer.compute_score(&sp500_prices);

        // 2. Filtro de Momentum Absoluto
        let abs_btc = self.absolute_filter.is_bullish(&btc_prices);
        let abs_gold = self.absolute_filter.is_bullish(&gold_prices);
        let abs_sp500 = self.absolute_filter.is_bullish(&sp500_prices);

        // Ordenamiento por puntuación relativa descendente
        let mut candidates = [
            (AssetId::Btc, score_btc, abs_btc),
            (AssetId::Gold, score_gold, abs_gold),
            (AssetId::Sp500, score_sp500, abs_sp500),
        ];
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let (best_asset, best_score, best_is_bullish) = candidates[0];
        let (second_asset, second_score, second_is_bullish) = candidates[1];
        let (worst_asset, worst_score, worst_is_bullish) = candidates[2];
        let score_margin = (best_score - second_score).abs();

        let risk_label = match self.relative_scorer.risk_mode {
            RiskAdjustmentMode::SortinoDownside => "Sortino",
            RiskAdjustmentMode::SharpeTotalVolatility => "Sharpe",
        };

        // --- ASIGNACIÓN SLOT 1 (50% MEJOR ACTIVO) ---
        // Regla Dual Momentum: Si tiene tendencia absoluta positiva y score positivo -> Long
        // Si presenta colapso extremo bajista y se permiten shorts -> Short
        // En caso contrario -> CASH (preservación de capital para recortar Drawdown)
        let slot_1 = if best_is_bullish && best_score > 0.0 {
            best_asset
        } else if self.allow_shorts && !best_is_bullish && best_score < -0.25 {
            best_asset.to_short().unwrap_or(AssetId::Cash)
        } else {
            AssetId::Cash
        };

        // --- ASIGNACIÓN SLOT 2 (50% SEGUNDO MEJOR ACTIVO) ---
        // Regla Dual Momentum: Si tiene tendencia absoluta positiva y score positivo -> Long
        // Si no: Si el peor activo del mercado colapsa con momentum extremo negativo (< -0.20)
        // y Slot 1 no está ya en short -> Posición táctica Short 1x sobre el peor activo (50%)
        // En caso contrario -> CASH (preservación de capital)
        let slot_2 = if second_is_bullish && second_score > 0.0 {
            second_asset
        } else if self.allow_shorts && !slot_1.is_short() && !worst_is_bullish && worst_score < -0.20 {
            worst_asset.to_short().unwrap_or(AssetId::Cash)
        } else {
            AssetId::Cash
        };

        let explanation = format!(
            "Dual 50/50 [{}]: Slot 1={} (Score: {:.2}) | Slot 2={} (Score: {:.2}) | Peor: {} ({:.2})",
            risk_label,
            slot_1.symbol(),
            best_score,
            slot_2.symbol(),
            second_score,
            worst_asset.symbol(),
            worst_score
        );

        SwapDecision::dual(slot_1, slot_2, score_margin, explanation)
    }
}
