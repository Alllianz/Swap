use crate::engine::swap::model::{SwapDecision, TriAssetModel};
use crate::engine::swap::types::SynchronizedBar;
use super::macro_tree::MacroRegimeClassifier;
use super::regime::MarketRegime;

/// Estrategia Híbrida de Regímenes Macroeconómicos SWAP (Ruta C).
/// Combina el árbol clasificador de estados macro (Risk-On / Growth / Defensive / Panic)
/// con estricta gobernanza de capital y rotación tri-activo sin Look-Ahead Bias.
pub struct RegimeHybridStrategy {
    pub name: String,
    pub last_regime: Option<MarketRegime>,
}

impl Default for RegimeHybridStrategy {
    fn default() -> Self {
        Self {
            name: "Ruta C: Híbrido de Regímenes SWAP (Risk-On / Growth / Safe / Cash)".to_string(),
            last_regime: None,
        }
    }
}

impl RegimeHybridStrategy {
    pub fn new() -> Self {
        Self::default()
    }
}

impl TriAssetModel for RegimeHybridStrategy {
    fn name(&self) -> &str {
        &self.name
    }

    fn evaluate(&mut self, history: &[SynchronizedBar], current_step: usize) -> SwapDecision {
        let (regime, confidence, explanation) = MacroRegimeClassifier::classify(history, current_step);
        self.last_regime = Some(regime);

        let target_asset = regime.target_asset();

        SwapDecision {
            target_asset,
            confidence_score: confidence,
            explanation: format!("[{}] {}", regime.description(), explanation),
        }
    }

    fn reset(&mut self) {
        self.last_regime = None;
    }
}
