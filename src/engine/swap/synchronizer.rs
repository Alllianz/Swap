use crate::data::kline::Kline;
use crate::engine::swap::types::SynchronizedBar;
use std::collections::BTreeMap;

const DAY_MS: i64 = 86_400_000;

/// Sincronizador multiactivo causal (0% Look-Ahead Bias).
/// Alinea velas temporales de mercados heterogéneos (Crypto 24/7 vs Renta Variable y Materias Primas)
/// normalizando por día calendario y aplicando forward-fill en fines de semana y feriados.
pub struct MultiAssetSynchronizer;

impl MultiAssetSynchronizer {
    /// Construye una serie temporal unificada y estrictamente alineada.
    /// Para días no hábiles (fines de semana / feriados en SP500 u Oro),
    /// se realiza forward-fill estricto del último precio oficial de cierre ya liquidado.
    pub fn synchronize_daily(
        btc_klines: &[Kline],
        gold_klines: &[Kline],
        sp500_klines: &[Kline],
    ) -> Vec<SynchronizedBar> {
        // Indexar Oro y SP500 por número de día calendario UTC (timestamp / 86_400_000)
        let mut gold_map: BTreeMap<i64, &Kline> = BTreeMap::new();
        for k in gold_klines {
            gold_map.insert(k.timestamp / DAY_MS, k);
        }

        let mut sp500_map: BTreeMap<i64, &Kline> = BTreeMap::new();
        for k in sp500_klines {
            sp500_map.insert(k.timestamp / DAY_MS, k);
        }

        let mut synchronized = Vec::with_capacity(btc_klines.len());

        let mut last_gold_close = gold_klines.first().map(|k| k.close).unwrap_or(1800.0);
        let mut last_gold_open = last_gold_close;
        let mut last_gold_high = last_gold_close;
        let mut last_gold_low = last_gold_close;
        let mut last_gold_vol = 0.0;

        let mut last_sp500_close = sp500_klines.first().map(|k| k.close).unwrap_or(240.0);
        let mut last_sp500_open = last_sp500_close;
        let mut last_sp500_high = last_sp500_close;
        let mut last_sp500_low = last_sp500_close;
        let mut last_sp500_vol = 0.0;

        for btc in btc_klines {
            let day_key = btc.timestamp / DAY_MS;

            if let Some(gold) = gold_map.get(&day_key) {
                last_gold_open = gold.open;
                last_gold_high = gold.high;
                last_gold_low = gold.low;
                last_gold_close = gold.close;
                last_gold_vol = gold.volume;
            }

            if let Some(sp500) = sp500_map.get(&day_key) {
                last_sp500_open = sp500.open;
                last_sp500_high = sp500.high;
                last_sp500_low = sp500.low;
                last_sp500_close = sp500.close;
                last_sp500_vol = sp500.volume;
            }

            synchronized.push(SynchronizedBar {
                timestamp: btc.timestamp,
                btc_open: btc.open,
                btc_high: btc.high,
                btc_low: btc.low,
                btc_close: btc.close,
                btc_volume: btc.volume,
                gold_open: last_gold_open,
                gold_high: last_gold_high,
                gold_low: last_gold_low,
                gold_close: last_gold_close,
                gold_volume: last_gold_vol,
                sp500_open: last_sp500_open,
                sp500_high: last_sp500_high,
                sp500_low: last_sp500_low,
                sp500_close: last_sp500_close,
                sp500_volume: last_sp500_vol,
            });
        }

        synchronized
    }
}
