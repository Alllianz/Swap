use crate::data::kline::Kline;

/// Timestamp obligatorio para frontera In-Sample (IS) y Out-Of-Sample (OOS)
/// 01-01-2023 00:00:00 UTC = 1672531200000 ms
pub const MANDATORY_OOS_START_TS: i64 = 1_672_531_200_000;

/// Obtiene los índices de corte causal para In-Sample y Out-Of-Sample
pub fn get_dataset_partition_indices(klines: &[Kline], min_warmup_bars: usize) -> (usize, usize) {
    let total_len = klines.len();
    if total_len == 0 {
        return (0, 0);
    }

    let is_start_idx = min_warmup_bars.min(total_len.saturating_sub(1));

    let oos_start_idx = klines
        .iter()
        .position(|k| k.timestamp >= MANDATORY_OOS_START_TS)
        .unwrap_or_else(|| {
            // Fallback al 80% si la fecha límite no está en el dataset
            ((total_len as f64) * 0.80).round() as usize
        });

    let safe_oos_idx = oos_start_idx.max(is_start_idx).min(total_len);
    (is_start_idx, safe_oos_idx)
}
