use crate::engine::swap::types::SynchronizedBar;

pub const TOTAL_SWAP_FEATURES: usize = 16;

/// Extractor causal de 16 variables tabulares para el ensamble SWAP GBDT Tri-Activo.
/// Garantiza 0% Look-Ahead Bias: solo consulta hasta la barra `current_idx`.
pub fn extract_cross_asset_features(
    history: &[SynchronizedBar],
    current_idx: usize,
) -> [f32; TOTAL_SWAP_FEATURES] {
    let mut feats = [0.0f32; TOTAL_SWAP_FEATURES];
    let n = current_idx + 1;
    if n < 35 {
        return feats;
    }

    let b_curr = history[current_idx].btc_close;
    let g_curr = history[current_idx].gold_close;
    let s_curr = history[current_idx].sp500_close;

    // Retornos a 14 y 30 barras
    let b_14 = history[current_idx - 14].btc_close;
    let b_30 = history[current_idx - 30].btc_close;
    let g_14 = history[current_idx - 14].gold_close;
    let g_30 = history[current_idx - 30].gold_close;
    let s_14 = history[current_idx - 14].sp500_close;
    let s_30 = history[current_idx - 30].sp500_close;

    feats[0] = ((b_curr / b_14.max(1e-6) - 1.0) as f32).clamp(-1.0, 1.0);
    feats[1] = ((b_curr / b_30.max(1e-6) - 1.0) as f32).clamp(-1.0, 1.0);
    feats[4] = ((g_curr / g_14.max(1e-6) - 1.0) as f32).clamp(-1.0, 1.0);
    feats[5] = ((g_curr / g_30.max(1e-6) - 1.0) as f32).clamp(-1.0, 1.0);
    feats[8] = ((s_curr / s_14.max(1e-6) - 1.0) as f32).clamp(-1.0, 1.0);
    feats[9] = ((s_curr / s_30.max(1e-6) - 1.0) as f32).clamp(-1.0, 1.0);

    // Volatilidades realizadas a 30 barras
    feats[2] = calc_vol_f32(history, current_idx, 30, |b| b.btc_close);
    feats[6] = calc_vol_f32(history, current_idx, 30, |b| b.gold_close);
    feats[10] = calc_vol_f32(history, current_idx, 30, |b| b.sp500_close);

    // Distancias a la SMA 30
    let btc_sma30 = calc_sma(history, current_idx, 30, |b| b.btc_close);
    let gold_sma30 = calc_sma(history, current_idx, 30, |b| b.gold_close);
    let sp500_sma30 = calc_sma(history, current_idx, 30, |b| b.sp500_close);

    feats[3] = ((b_curr / btc_sma30.max(1e-6) - 1.0) as f32).clamp(-1.0, 1.0);
    feats[7] = ((g_curr / gold_sma30.max(1e-6) - 1.0) as f32).clamp(-1.0, 1.0);
    feats[11] = ((s_curr / sp500_sma30.max(1e-6) - 1.0) as f32).clamp(-1.0, 1.0);

    // Ratios cruzados (Fuerza Relativa Causal)
    let btc_sp_ratio_curr = b_curr / s_curr.max(1e-6);
    let btc_sp_ratio_prev = b_30 / s_30.max(1e-6);
    feats[12] = ((btc_sp_ratio_curr / btc_sp_ratio_prev.max(1e-6) - 1.0) as f32).clamp(-1.0, 1.0);

    let btc_gold_ratio_curr = b_curr / g_curr.max(1e-6);
    let btc_gold_ratio_prev = b_30 / g_30.max(1e-6);
    feats[13] = ((btc_gold_ratio_curr / btc_gold_ratio_prev.max(1e-6) - 1.0) as f32).clamp(-1.0, 1.0);

    let gold_sp_ratio_curr = g_curr / s_curr.max(1e-6);
    let gold_sp_ratio_prev = g_30 / s_30.max(1e-6);
    feats[14] = ((gold_sp_ratio_curr / gold_sp_ratio_prev.max(1e-6) - 1.0) as f32).clamp(-1.0, 1.0);

    // Spread de volatilidad BTC vs SP500
    feats[15] = (feats[2] / feats[10].max(0.01)).clamp(0.1, 10.0);

    feats
}

fn calc_sma<F: Fn(&SynchronizedBar) -> f64>(
    history: &[SynchronizedBar],
    curr: usize,
    window: usize,
    extractor: F,
) -> f64 {
    let start = curr.saturating_sub(window - 1);
    let sum: f64 = history[start..=curr].iter().map(extractor).sum();
    sum / (window as f64)
}

fn calc_vol_f32<F: Fn(&SynchronizedBar) -> f64>(
    history: &[SynchronizedBar],
    curr: usize,
    window: usize,
    extractor: F,
) -> f32 {
    let start = curr.saturating_sub(window);
    let mut log_rets = Vec::with_capacity(window);
    for i in start..curr {
        let p0 = extractor(&history[i]);
        let p1 = extractor(&history[i + 1]);
        if p0 > 1e-6 && p1 > 1e-6 {
            log_rets.push((p1 / p0).ln());
        }
    }
    if log_rets.len() < 2 {
        return 0.15;
    }
    let m = log_rets.iter().sum::<f64>() / (log_rets.len() as f64);
    let var = log_rets.iter().map(|r| (r - m).powi(2)).sum::<f64>() / (log_rets.len() - 1) as f64;
    ((var.sqrt() * (365.25f64).sqrt()) as f32).clamp(0.01, 2.5)
}
