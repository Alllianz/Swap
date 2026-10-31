use crate::engine::swap::types::AssetId;
use serde::{Deserialize, Serialize};

/// Regímenes de mercado macroeconómico y de liquidez global
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarketRegime {
    /// Expansión de liquidez y alto apetito por riesgo -> Alocación en BTC
    RiskOnLiquidity,
    /// Crecimiento económico institucional y baja volatilidad -> Alocación en S&P 500
    SteadyGrowthEquities,
    /// Estagflación, aversión al riesgo o refugio de valor -> Alocación en ORO
    DefensiveGoldFlight,
    /// Pánico sistémico, shock de volatilidad o contracción -> Alocación en CASH
    SystemicPanicCash,
}

impl MarketRegime {
    pub fn target_asset(&self) -> AssetId {
        match self {
            MarketRegime::RiskOnLiquidity => AssetId::Btc,
            MarketRegime::SteadyGrowthEquities => AssetId::Sp500,
            MarketRegime::DefensiveGoldFlight => AssetId::Gold,
            MarketRegime::SystemicPanicCash => AssetId::Cash,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            MarketRegime::RiskOnLiquidity => "Régimen RISK-ON / Expansión de Liquidez (BTC)",
            MarketRegime::SteadyGrowthEquities => "Régimen CRECIMIENTO ESTABLE / Renta Variable (S&P 500)",
            MarketRegime::DefensiveGoldFlight => "Régimen DEFENSIVO / Refugio Inflacionario (ORO)",
            MarketRegime::SystemicPanicCash => "Régimen PÁNICO SISTÉMICO / Preservación de Capital (CASH)",
        }
    }
}
