use crate::data::kline::Kline;
use chrono::{Datelike, TimeZone, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YearlyPerformance {
    pub year: i32,
    pub is_partial: bool,
    pub start_timestamp: i64,
    pub end_timestamp: i64,
    pub strategy_return_pct: f64,
    pub bh_return_pct: f64,
    pub diff_pp: f64,
    pub beat_bh: bool,
}

/// Calcula el desglose anualizado de rendimientos comparando la estrategia contra Buy & Hold
pub fn calculate_yearly_returns(
    curve: &[(i64, f64)],
    klines: &[Kline],
) -> Vec<YearlyPerformance> {
    if curve.is_empty() || klines.is_empty() {
        return Vec::new();
    }

    let mut years_map: std::collections::BTreeMap<i32, (usize, usize)> = std::collections::BTreeMap::new();

    for (i, &(ts, _)) in curve.iter().enumerate() {
        if let Some(dt) = Utc.timestamp_millis_opt(ts).single() {
            let y = dt.year();
            years_map
                .entry(y)
                .and_modify(|range| range.1 = i)
                .or_insert((i, i));
        }
    }

    let mut results = Vec::new();
    let total_years = years_map.len();

    for (idx, (&year, &(start_i, end_i))) in years_map.iter().enumerate() {
        let is_partial = idx == 0 || idx == total_years - 1;

        let base_eq = if start_i > 0 { curve[start_i - 1].1 } else { curve[start_i].1 };
        let end_eq = curve[end_i].1;
        let strat_ret = if base_eq > 0.0 {
            ((end_eq / base_eq) - 1.0) * 100.0
        } else {
            0.0
        };

        // Buy & Hold con el precio de cierre de klines
        let base_price = if start_i > 0 { klines[start_i - 1].close } else { klines[start_i].close };
        let end_price = klines[end_i].close;
        let bh_ret = if base_price > 0.0 {
            ((end_price / base_price) - 1.0) * 100.0
        } else {
            0.0
        };

        let diff_pp = strat_ret - bh_ret;
        let beat_bh = diff_pp >= 0.0;

        results.push(YearlyPerformance {
            year,
            is_partial,
            start_timestamp: curve[start_i].0,
            end_timestamp: curve[end_i].0,
            strategy_return_pct: strat_ret,
            bh_return_pct: bh_ret,
            diff_pp,
            beat_bh,
        });
    }

    results
}
