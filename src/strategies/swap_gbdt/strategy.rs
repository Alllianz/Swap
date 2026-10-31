use crate::engine::swap::model::{SwapDecision, TriAssetModel};
use crate::engine::swap::types::{AssetId, SynchronizedBar};
use super::features::extract_cross_asset_features;
use super::gbdt::SwapGbdtEnsemble;

/// Estrategia de Ensamble SWAP GBDT Tri-Activo (Ruta A).
/// Ejecuta inferencia multimodelo mediante árboles de boosting de decisión
/// sobre 16 variables cuantitativas cross-asset sin Look-Ahead Bias.
pub struct SwapGbdtStrategy {
    pub name: String,
    pub btc_model: SwapGbdtEnsemble,
    pub gold_model: SwapGbdtEnsemble,
    pub sp500_model: SwapGbdtEnsemble,
    pub cash_threshold: f32,
}

impl Default for SwapGbdtStrategy {
    fn default() -> Self {
        Self {
            name: "Ruta A: SWAP GBDT Tri-Activo (Scoring & Ranking ML)".to_string(),
            btc_model: SwapGbdtEnsemble::build_btc_model(),
            gold_model: SwapGbdtEnsemble::build_gold_model(),
            sp500_model: SwapGbdtEnsemble::build_sp500_model(),
            cash_threshold: -0.15,
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
        }
    }
}

impl TriAssetModel for SwapGbdtStrategy {
    fn name(&self) -> &str {
        &self.name
    }

    fn evaluate(&mut self, history: &[SynchronizedBar], current_step: usize) -> SwapDecision {
        if current_step < 35 {
            return SwapDecision {
                target_asset: AssetId::Cash,
                confidence_score: 0.0,
                explanation: "Inicializando histórico causal para variables SWAP".to_string(),
            };
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
            SwapDecision {
                target_asset: AssetId::Cash,
                confidence_score: 0.2,
                explanation: format!(
                    "SWAP GBDT detecta régimen bajista general (BTC: {:.2}, ORO: {:.2}, SP: {:.2}). Alocación en CASH",
                    score_btc, score_gold, score_sp500
                ),
            }
        } else {
            SwapDecision {
                target_asset: winner_asset,
                confidence_score: margin,
                explanation: format!(
                    "SWAP GBDT selecciona {} (Score: {:.3}, Ventaja: +{:.3}) | [BTC: {:.2}, ORO: {:.2}, SP500: {:.2}]",
                    winner_asset.symbol(),
                    winner_score,
                    margin,
                    score_btc,
                    score_gold,
                    score_sp500
                ),
            }
        }
    }
}
