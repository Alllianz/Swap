use super::types::{AssetId, SynchronizedBar};

/// Decisión emitida por una estrategia o modelo de intercambio tri-activo
#[derive(Debug, Clone)]
pub struct SwapDecision {
    /// Activo seleccionado para alocar el capital (o Slot 1 / 50% en dual)
    pub target_asset: AssetId,
    /// Activo secundario opcional (Slot 2 / 50% en carteras duales)
    pub secondary_asset: Option<AssetId>,
    /// Puntuación de convicción o ventaja relativa (para auditoría e histéresis)
    pub confidence_score: f64,
    /// Razón explicativa textual para logging y auditoría cuantitativa
    pub explanation: String,
}

impl SwapDecision {
    /// Construye una decisión de alocación al 100% en un solo activo
    pub fn single(target_asset: AssetId, confidence_score: f64, explanation: String) -> Self {
        Self {
            target_asset,
            secondary_asset: None,
            confidence_score,
            explanation,
        }
    }

    /// Construye una decisión de alocación dual dividida (50% activo 1, 50% activo 2)
    pub fn dual(
        primary_asset: AssetId,
        secondary_asset: AssetId,
        confidence_score: f64,
        explanation: String,
    ) -> Self {
        Self {
            target_asset: primary_asset,
            secondary_asset: Some(secondary_asset),
            confidence_score,
            explanation,
        }
    }
}

/// Trait unificado para cualquier modelo de rotación / intercambio tri-activo.
/// Garantiza estricta causalidad (0% Look-Ahead Bias): el método `evaluate`
/// recibe exclusivamente el slice histórico hasta la barra actual `history[0..=current_step]`.
pub trait TriAssetModel: Send + Sync {
    /// Nombre identificador del modelo o estrategia
    fn name(&self) -> &str;

    /// Evalúa la serie temporal causal y determina en qué activo posicionarse
    fn evaluate(&mut self, history: &[SynchronizedBar], current_step: usize) -> SwapDecision;

    /// Reinicia cualquier estado interno entre ejecuciones de backtesting
    fn reset(&mut self) {}
}
