use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

/// Identificador único de cada activo en el universo de intercambio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetId {
    /// Bitcoin (Alta beta, liquidez global, expansión monetaria)
    Btc,
    /// Oro físico / PAX Gold / XAU (Refugio de valor, cobertura inflacionaria / estagflación)
    Gold,
    /// S&P 500 / SPY (Renta variable USA, crecimiento económico institucional)
    Sp500,
    /// Dólar / USDT / Liquidez (Preservación estricta de capital ante pánico de mercado)
    Cash,
}

impl AssetId {
    pub fn name(&self) -> &'static str {
        match self {
            AssetId::Btc => "BTC (Bitcoin)",
            AssetId::Gold => "ORO (Gold / PAXG)",
            AssetId::Sp500 => "S&P 500 (SPY)",
            AssetId::Cash => "CASH (USDT / Dólar)",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            AssetId::Btc => "BTC",
            AssetId::Gold => "GOLD",
            AssetId::Sp500 => "SP500",
            AssetId::Cash => "CASH",
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
            AssetId::Btc => self.btc_close,
            AssetId::Gold => self.gold_close,
            AssetId::Sp500 => self.sp500_close,
            AssetId::Cash => 1.0,
        }
    }

    pub fn get_open(&self, asset: AssetId) -> f64 {
        match asset {
            AssetId::Btc => self.btc_open,
            AssetId::Gold => self.gold_open,
            AssetId::Sp500 => self.sp500_open,
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

/// Configuración de ejecución del motor de Swap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapConfig {
    /// Capital inicial en dólares (ej. 10_000 USD)
    pub initial_capital: f64,
    /// Comisión por swap / intercambio (ej. 0.0010 para 0.10%)
    pub swap_fee_rate: f64,
    /// Estimación de slippage por swap (ej. 0.0005 para 0.05%)
    pub slippage_rate: f64,
    /// Mínimo número de barras antes de permitir otro swap (anti-churning / comisiones)
    pub min_holding_bars: usize,
    /// Umbral mínimo de ventaja relativa para justificar una rotación (histéresis)
    pub hysteresis_threshold: f64,
}

impl Default for SwapConfig {
    fn default() -> Self {
        Self {
            initial_capital: 10_000.0,
            swap_fee_rate: 0.0010,
            slippage_rate: 0.0005,
            min_holding_bars: 5,
            hysteresis_threshold: 0.015,
        }
    }
}
