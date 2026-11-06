use crate::engine::swap::types::AssetId;
use serde::{Deserialize, Serialize};

/// Regímenes de mercado macroeconómico y de liquidez global
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarketRegime {
    /// Expansión de liquidez y alto apetito por riesgo -> Alocación Long en BTC
    RiskOnLiquidity,
    /// Crecimiento económico institucional y baja volatilidad -> Alocación Long en S&P 500
    SteadyGrowthEquities,
    /// Estagflación, aversión al riesgo o refugio de valor -> Alocación Long en ORO
    DefensiveGoldFlight,
    /// Mercado bajista estructural en Renta Variable USA -> Posición SHORT en S&P 500 (SPY)
    BearMarketEquities,
    /// Criptoinvierno / Contracción severa de liquidez en activos digitales -> Posición SHORT en BTC
    CryptoWinter,
    /// Corrección cíclica en metales preciosos -> Posición SHORT en ORO
    GoldCorrection,
    /// Pánico sistémico generalizado, shock de volatilidad o iliquidez -> Alocación defensiva en CASH
    SystemicPanicCash,
}

impl MarketRegime {
    pub fn target_asset(&self) -> AssetId {
        match self {
            MarketRegime::RiskOnLiquidity => AssetId::Btc,
            MarketRegime::SteadyGrowthEquities => AssetId::Sp500,
            MarketRegime::DefensiveGoldFlight => AssetId::Gold,
            MarketRegime::BearMarketEquities => AssetId::ShortSp500,
            MarketRegime::CryptoWinter => AssetId::ShortBtc,
            MarketRegime::GoldCorrection => AssetId::ShortGold,
            MarketRegime::SystemicPanicCash => AssetId::Cash,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            MarketRegime::RiskOnLiquidity => "Régimen RISK-ON / Expansión de Liquidez (BTC Long)",
            MarketRegime::SteadyGrowthEquities => "Régimen CRECIMIENTO ESTABLE / Renta Variable (S&P 500 Long)",
            MarketRegime::DefensiveGoldFlight => "Régimen DEFENSIVO / Refugio Inflacionario (ORO Long)",
            MarketRegime::BearMarketEquities => "Régimen MERCADO BAJISTA / Cobertura Inversa (SHORT SPY)",
            MarketRegime::CryptoWinter => "Régimen CRIPTOINVIERNO / Mercado Bajista Digital (SHORT BTC)",
            MarketRegime::GoldCorrection => "Régimen CORRECCIÓN METALES / Desinflación (SHORT ORO)",
            MarketRegime::SystemicPanicCash => "Régimen PÁNICO SISTÉMICO / Preservación de Capital (CASH)",
        }
    }
}
