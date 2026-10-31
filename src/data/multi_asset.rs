use crate::engine::swap::types::{AssetId, SynchronizedBar};
use serde::{Deserialize, Serialize};

/// Vela individual multiactivo etiquetada por símbolo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiAssetKline {
    pub symbol: String,
    pub timeframe: String,
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

/// Contenedor de series temporales multiactivo sincronizadas
#[derive(Debug, Clone)]
pub struct MultiAssetDataset {
    pub timeframe: String,
    pub bars: Vec<SynchronizedBar>,
    pub start_ts: i64,
    pub end_ts: i64,
}

impl MultiAssetDataset {
    pub fn new(timeframe: String, bars: Vec<SynchronizedBar>) -> Self {
        let start_ts = bars.first().map(|b| b.timestamp).unwrap_or(0);
        let end_ts = bars.last().map(|b| b.timestamp).unwrap_or(0);
        Self {
            timeframe,
            bars,
            start_ts,
            end_ts,
        }
    }

    pub fn len(&self) -> usize {
        self.bars.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bars.is_empty()
    }

    /// Retorna los retornos acumulados Buy & Hold para cada activo
    pub fn calculate_buy_and_hold_returns(&self) -> (f64, f64, f64) {
        if self.bars.len() < 2 {
            return (0.0, 0.0, 0.0);
        }
        let first = self.bars.first().unwrap();
        let last = self.bars.last().unwrap();

        let btc_ret = (last.get_close(AssetId::Btc) / first.get_close(AssetId::Btc).max(1e-6) - 1.0) * 100.0;
        let gold_ret = (last.get_close(AssetId::Gold) / first.get_close(AssetId::Gold).max(1e-6) - 1.0) * 100.0;
        let sp_ret = (last.get_close(AssetId::Sp500) / first.get_close(AssetId::Sp500).max(1e-6) - 1.0) * 100.0;

        (btc_ret, gold_ret, sp_ret)
    }
}
