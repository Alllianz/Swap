use crate::data::kline::Kline;
use crate::data::timeframe::tf_to_duration_millis;
use crate::engine::swap::types::SynchronizedBar;
use std::collections::BTreeMap;

/// Sincronizador multiactivo causal (0% Look-Ahead Bias).
/// Alinea velas temporales de mercados heterogéneos (Crypto 24/7 vs Renta Variable y Materias Primas)
/// en cualquier temporalidad ("1D", "1H", etc.), aplicando forward-fill en fines de semana,
/// feriados o fuera de horario bursátil para activos no continuos (SP500 y Oro).
pub struct MultiAssetSynchronizer;

impl MultiAssetSynchronizer {
    /// Sincroniza barras multiactivo para un timeframe determinado ("1D", "1H", etc.).
    /// Causal y con 0% Look-Ahead Bias: alinea contra la serie temporal continua de Bitcoin.
    /// Para activos con horarios de negociación restringidos o días no hábiles (fines de semana,
    /// feriados o noches en SP500 y Oro), aplica forward-fill estricto del último precio de cierre liquidado.
    pub fn synchronize(
        timeframe: &str,
        btc_klines: &[Kline],
        gold_klines: &[Kline],
        sp500_klines: &[Kline],
    ) -> Vec<SynchronizedBar> {
        let step_ms = tf_to_duration_millis(timeframe).max(60_000);

        // Mapear Oro y SP500 ordenados por timestamp
        let mut gold_map: BTreeMap<i64, &Kline> = BTreeMap::new();
        for k in gold_klines {
            gold_map.insert(k.timestamp, k);
        }

        let mut sp500_map: BTreeMap<i64, &Kline> = BTreeMap::new();
        for k in sp500_klines {
            sp500_map.insert(k.timestamp, k);
        }

        let mut synchronized = Vec::with_capacity(btc_klines.len());
        let mut last_gold_close = gold_klines.first().map(|k| k.close).unwrap_or(1800.0);
        let mut last_sp500_close = sp500_klines.first().map(|k| k.close).unwrap_or(240.0);

        for btc in btc_klines {
            let bar_start = btc.timestamp;
            let bar_end = bar_start + step_ms;

            // 1. Obtener vela de Oro (real en ventana o forward-fill del último cierre)
            let (gold_open, gold_high, gold_low, gold_close, gold_vol) =
                match gold_map.range(bar_start..bar_end).next() {
                    Some((_, gold)) => {
                        last_gold_close = gold.close;
                        (gold.open, gold.high, gold.low, gold.close, gold.volume)
                    }
                    None => {
                        if let Some((_, prev_gold)) = gold_map.range(..=bar_start).next_back() {
                            last_gold_close = prev_gold.close;
                        }
                        (last_gold_close, last_gold_close, last_gold_close, last_gold_close, 0.0)
                    }
                };

            // 2. Obtener vela de SP500 (real en ventana bursátil o forward-fill del último cierre)
            let (sp_open, sp_high, sp_low, sp_close, sp_vol) =
                match sp500_map.range(bar_start..bar_end).next() {
                    Some((_, sp500)) => {
                        last_sp500_close = sp500.close;
                        (sp500.open, sp500.high, sp500.low, sp500.close, sp500.volume)
                    }
                    None => {
                        if let Some((_, prev_sp)) = sp500_map.range(..=bar_start).next_back() {
                            last_sp500_close = prev_sp.close;
                        }
                        (last_sp500_close, last_sp500_close, last_sp500_close, last_sp500_close, 0.0)
                    }
                };

            synchronized.push(SynchronizedBar {
                timestamp: btc.timestamp,
                btc_open: btc.open,
                btc_high: btc.high,
                btc_low: btc.low,
                btc_close: btc.close,
                btc_volume: btc.volume,
                gold_open,
                gold_high,
                gold_low,
                gold_close,
                gold_volume: gold_vol,
                sp500_open: sp_open,
                sp500_high: sp_high,
                sp500_low: sp_low,
                sp500_close: sp_close,
                sp500_volume: sp_vol,
            });
        }

        synchronized
    }

    /// Conserva total retrocompatibilidad con código preexistente para temporalidad diaria.
    pub fn synchronize_daily(
        btc_klines: &[Kline],
        gold_klines: &[Kline],
        sp500_klines: &[Kline],
    ) -> Vec<SynchronizedBar> {
        Self::synchronize("1D", btc_klines, gold_klines, sp500_klines)
    }
}
