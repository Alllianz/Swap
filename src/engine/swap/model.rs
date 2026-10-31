use super::types::{AssetId, SynchronizedBar};

/// Decisión emitida por una estrategia o modelo de intercambio tri-activo
#[derive(Debug, Clone)]
pub struct SwapDecision {
    /// Activo seleccionado para alocar el capital
    pub target_asset: AssetId,
    /// Puntuación de convicción o ventaja relativa (para auditoría e histéresis)
    pub confidence_score: f64,
    /// Razón explicativa textual para logging y auditoría cuantitativa
    pub explanation: String,
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
