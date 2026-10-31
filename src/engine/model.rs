use crate::data::kline::Kline;
use super::types::SignalAction;

/// Trait común para cualquier modelo o estrategia causal de trading cuantitativo.
/// Garantiza estricta causalidad (0% Look-Ahead Bias): la función `evaluate` solo recibe
/// el slice histórico de velas `history = &klines[0..=step]`.
pub trait EquationModel: Send + Sync {
    fn name(&self) -> &str;
    fn evaluate(&mut self, history: &[Kline]) -> SignalAction;
    fn reset(&mut self) {}
}
