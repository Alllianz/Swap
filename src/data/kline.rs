use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

/// Representación canónica de una vela de precio (Kline / Candlestick).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Kline {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

impl Kline {
    pub fn new(timestamp: i64, open: f64, high: f64, low: f64, close: f64, volume: f64) -> Self {
        Self {
            timestamp,
            open,
            high,
            low,
            close,
            volume,
        }
    }

    /// Devuelve la fecha UTC formateada
    pub fn date_utc_string(&self) -> String {
        Utc.timestamp_millis_opt(self.timestamp)
            .single()
            .map(|dt: DateTime<Utc>| dt.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| self.timestamp.to_string())
    }

    /// Devuelve solo el día en formato YYYY-MM-DD
    pub fn day_utc_string(&self) -> String {
        Utc.timestamp_millis_opt(self.timestamp)
            .single()
            .map(|dt: DateTime<Utc>| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| self.timestamp.to_string())
    }
}
