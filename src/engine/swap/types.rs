use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

/// Identificador único de cada activo o posición en el universo de intercambio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetId {
    /// Bitcoin Long (Alta beta, liquidez global, expansión monetaria)
    Btc,
    /// Oro físico Long / PAX Gold / XAU (Refugio de valor, cobertura inflacionaria / estagflación)
    Gold,
    /// S&P 500 Long / SPY (Renta variable USA, crecimiento económico institucional)
    Sp500,
    /// Posición corta en Bitcoin (1x Inverso, beneficio en caídas de BTC)
    ShortBtc,
    /// Posición corta en Oro físico (1x Inverso, beneficio en correcciones de metales)
    ShortGold,
    /// Posición corta en S&P 500 (1x Inverso, cobertura o beneficio en mercados bajistas de USA)
    ShortSp500,
    /// Dólar / USDT / Liquidez (Preservación estricta de capital)
    Cash,
}

impl AssetId {
    pub fn name(&self) -> &'static str {
        match self {
            AssetId::Btc => "BTC (Bitcoin Long)",
            AssetId::Gold => "ORO (Gold / PAXG Long)",
            AssetId::Sp500 => "S&P 500 (SPY Long)",
            AssetId::ShortBtc => "SHORT BTC (Bitcoin Inverso)",
            AssetId::ShortGold => "SHORT ORO (Oro Inverso)",
            AssetId::ShortSp500 => "SHORT S&P 500 (SPY Inverso)",
            AssetId::Cash => "CASH (USDT / Dólar)",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            AssetId::Btc => "BTC",
            AssetId::Gold => "GOLD",
            AssetId::Sp500 => "SP500",
            AssetId::ShortBtc => "SH_BTC",
            AssetId::ShortGold => "SH_GOLD",
            AssetId::ShortSp500 => "SH_SP500",
            AssetId::Cash => "CASH",
        }
    }

    pub fn is_short(&self) -> bool {
        matches!(
            self,
            AssetId::ShortBtc | AssetId::ShortGold | AssetId::ShortSp500
        )
    }

    pub fn is_long(&self) -> bool {
        matches!(self, AssetId::Btc | AssetId::Gold | AssetId::Sp500)
    }

    pub fn to_short(&self) -> Option<AssetId> {
        match self {
            AssetId::Btc => Some(AssetId::ShortBtc),
            AssetId::Gold => Some(AssetId::ShortGold),
            AssetId::Sp500 => Some(AssetId::ShortSp500),
            _ => None,
        }
    }

    pub fn to_long(&self) -> Option<AssetId> {
        match self {
            AssetId::Btc | AssetId::ShortBtc => Some(AssetId::Btc),
            AssetId::Gold | AssetId::ShortGold => Some(AssetId::Gold),
            AssetId::Sp500 | AssetId::ShortSp500 => Some(AssetId::Sp500),
            AssetId::Cash => Some(AssetId::Cash),
        }
    }
}

/// Barra de precios multiactivo sincronizada en el tiempo.
/// Garantiza estricta causalidad temporal (0% Look-Ahead Bias).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynchronizedBar {
    /// Timestamp Unix en milisegundos correspondiente al cierre de la barra
    pub timestamp: i64,
    /// Precios de BTC
    pub btc_open: f64,
    pub btc_high: f64,
    pub btc_low: f64,
    pub btc_close: f64,
    pub btc_volume: f64,
    /// Precios de ORO
    pub gold_open: f64,
    pub gold_high: f64,
    pub gold_low: f64,
    pub gold_close: f64,
    pub gold_volume: f64,
    /// Precios de S&P 500
    pub sp500_open: f64,
    pub sp500_high: f64,
    pub sp500_low: f64,
    pub sp500_close: f64,
    pub sp500_volume: f64,
}

impl SynchronizedBar {
    pub fn get_close(&self, asset: AssetId) -> f64 {
        match asset {
            AssetId::Btc | AssetId::ShortBtc => self.btc_close,
            AssetId::Gold | AssetId::ShortGold => self.gold_close,
            AssetId::Sp500 | AssetId::ShortSp500 => self.sp500_close,
            AssetId::Cash => 1.0,
        }
    }

    pub fn get_open(&self, asset: AssetId) -> f64 {
        match asset {
            AssetId::Btc | AssetId::ShortBtc => self.btc_open,
            AssetId::Gold | AssetId::ShortGold => self.gold_open,
            AssetId::Sp500 | AssetId::ShortSp500 => self.sp500_open,
            AssetId::Cash => 1.0,
        }
    }

    pub fn date_string(&self) -> String {
        Utc.timestamp_millis_opt(self.timestamp)
            .single()
            .map(|dt: DateTime<Utc>| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| self.timestamp.to_string())
    }
}

/// Registro detallado de una rotación o swap de activos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapTradeRecord {
    pub swap_index: usize,
    pub step: usize,
    pub timestamp: i64,
    pub date_str: String,
    pub from_asset: AssetId,
    pub to_asset: AssetId,
    pub exit_price: f64,
    pub entry_price: f64,
    pub capital_before: f64,
    pub capital_after: f64,
    pub fee_paid: f64,
    pub reason: String,
}

/// Modo de gestión de capital y riesgo de margen
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarginMode {
    /// Capital Cruzado (Cross Margin): El balance global de la cuenta actúa como colateral unificado.
    /// Las pérdidas de cualquier posición van contra todo el capital de la cuenta.
    Cross,
    /// Margen Aislado (Isolated Margin): La pérdida de cada posición está estrictamente acotada a su porción asignada.
    Isolated,
}

impl MarginMode {
    pub fn name(&self) -> &'static str {
        match self {
            MarginMode::Cross => "Capital Cruzado (Cross Margin)",
            MarginMode::Isolated => "Margen Aislado (Isolated Margin)",
        }
    }
}

/// Configuración operativa del simulador de Swap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapConfig {
    pub initial_capital: f64,
    pub swap_fee_rate: f64,
    pub slippage_rate: f64,
    pub min_holding_bars: usize,
    pub hysteresis_threshold: f64,
    pub margin_mode: MarginMode,
    pub leverage: f64,
}

impl Default for SwapConfig {
    fn default() -> Self {
        Self {
            initial_capital: 10_000.0,
            swap_fee_rate: 0.0010,     // 0.10%
            slippage_rate: 0.0005,     // 0.05%
            min_holding_bars: 5,       // Mínimo 5 barras antes de permitir otro swap
            hysteresis_threshold: 0.015, // Ventaja mínima del nuevo activo para justificar el costo de swap
            margin_mode: MarginMode::Cross, // Capital Cruzado por defecto
            leverage: 5.0,             // Apalancamiento 5X por defecto
        }
    }
}
