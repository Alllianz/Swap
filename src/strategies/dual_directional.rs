use crate::data::kline::Kline;
use crate::engine::model::EquationModel;
use crate::engine::types::SignalAction;

/// Ensamble Dual Desacoplado: Combina un modelo especialista Long-Only
/// y un modelo especialista Short-Only operando con resolución causal de señales.
pub struct DualDirectionalModel<L: EquationModel, S: EquationModel> {
    pub long_model: L,
    pub short_model: S,
    pub name_str: String,
}

impl<L: EquationModel, S: EquationModel> DualDirectionalModel<L, S> {
    pub fn new(long_model: L, short_model: S) -> Self {
        Self {
            long_model,
            short_model,
            name_str: "SWAP Dual Ensemble (Long + Short Desacoplado)".to_string(),
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name_str = name.into();
        self
    }
}

impl<L: EquationModel, S: EquationModel> EquationModel for DualDirectionalModel<L, S> {
    fn name(&self) -> &str {
        &self.name_str
    }

    fn evaluate(&mut self, history: &[Kline]) -> SignalAction {
        let sig_long = self.long_model.evaluate(history);
        let sig_short = self.short_model.evaluate(history);

        match (sig_long, sig_short) {
            (SignalAction::Buy, SignalAction::Sell) => {
                // Conflicto causal: si ambos disparan a la vez, neutralidad Flat estricta
                SignalAction::Flat
            }
            (SignalAction::Buy, _) => SignalAction::Buy,
            (_, SignalAction::Sell) => SignalAction::Sell,
            _ => SignalAction::Flat,
        }
    }

    fn reset(&mut self) {
        self.long_model.reset();
        self.short_model.reset();
    }
}
