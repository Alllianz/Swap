use crate::engine::swap::model::{SwapDecision, TriAssetModel};
use crate::engine::swap::types::{AssetId, SynchronizedBar};
use super::features::extract_cross_asset_features;
use super::gbdt::SwapGbdtEnsemble;

/// Estrategia de Ensamble SWAP GBDT Tri-Activo (Ruta A - Long/Short/Cash).
/// Ejecuta inferencia multimodelo mediante árboles de boosting de decisión
/// sobre 16 variables cuantitativas cross-asset sin Look-Ahead Bias.
pub struct SwapGbdtStrategy {
    pub name: String,
    pub btc_model: SwapGbdtEnsemble,
    pub gold_model: SwapGbdtEnsemble,
    pub sp500_model: SwapGbdtEnsemble,
    pub cash_threshold: f32,
    pub allow_shorts: bool,
}

impl Default for SwapGbdtStrategy {
    fn default() -> Self {
        Self {
            name: "Ruta A: SWAP GBDT Tri-Activo (Scoring & Ranking ML Long/Short)".to_string(),
            btc_model: SwapGbdtEnsemble::build_btc_model(),
            gold_model: SwapGbdtEnsemble::build_gold_model(),
            sp500_model: SwapGbdtEnsemble::build_sp500_model(),
            cash_threshold: -0.15,
            allow_shorts: true,
        }
    }
}

impl SwapGbdtStrategy {
    pub fn new(cash_threshold: f32) -> Self {
        Self {
            name: format!("Ruta A: SWAP GBDT (CashThresh: {:.2})", cash_threshold),
            btc_model: SwapGbdtEnsemble::build_btc_model(),
            gold_model: SwapGbdtEnsemble::build_gold_model(),
            sp500_model: SwapGbdtEnsemble::build_sp500_model(),
            cash_threshold,
            allow_shorts: true,
        }
    }

    pub fn with_shorts(mut self, allow_shorts: bool) -> Self {
        self.allow_shorts = allow_shorts;
        self
    }
}

impl TriAssetModel for SwapGbdtStrategy {
    fn name(&self) -> &str {
        &self.name
    }

    fn evaluate(&mut self, history: &[SynchronizedBar], current_step: usize) -> SwapDecision {
        if current_step < 35 {
            return SwapDecision::single(
                AssetId::Cash,
                0.0,
                "Inicializando histórico causal para variables SWAP".to_string(),
            );
        }

        let feats = extract_cross_asset_features(history, current_step);

        let score_btc = self.btc_model.predict(&feats);
        let score_gold = self.gold_model.predict(&feats);
        let score_sp500 = self.sp500_model.predict(&feats);

        let mut scores = [
            (AssetId::Btc, score_btc),
            (AssetId::Gold, score_gold),
            (AssetId::Sp500, score_sp500),
        ];

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let (winner_asset, winner_score) = scores[0];
        let runner_up_score = scores[1].1;
        let margin = (winner_score - runner_up_score).abs() as f64;

        if winner_score < self.cash_threshold {
            // Mercado general por debajo del umbral de Cash.
            // Si las posiciones cortas están habilitadas y el peor activo presenta fuerte aceleración bajista:
            let (loser_asset, loser_score) = scores[2];

            if self.allow_shorts && loser_score < -0.22
                && let Some(short_target) = loser_asset.to_short()
            {
                return SwapDecision::single(
                    short_target,
                    (loser_score.abs() * 0.6) as f64,
                    format!(
                        "SWAP GBDT detecta alta probabilidad bajista en {} (Score: {:.3}). Entrada Táctica en {}",
                        loser_asset.symbol(), loser_score, short_target.name()
                    ),
                );
            }

            SwapDecision::single(
                AssetId::Cash,
                0.2,
                format!(
                    "SWAP GBDT detecta régimen bajista general (BTC: {:.2}, ORO: {:.2}, SP: {:.2}). Alocación defensiva en CASH",
                    score_btc, score_gold, score_sp500
                ),
            )
        } else {
            SwapDecision::single(
                winner_asset,
                margin,
                format!(
                    "SWAP GBDT selecciona {} (Score: {:.3}, Ventaja: +{:.3}) | [BTC: {:.2}, ORO: {:.2}, SP500: {:.2}]",
                    winner_asset.symbol(),
                    winner_score,
                    margin,
                    score_btc,
                    score_gold,
                    score_sp500
                ),
            )
        }
    }

    fn reset(&mut self) {}
}
