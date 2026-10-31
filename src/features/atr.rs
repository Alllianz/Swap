use crate::data::kline::Kline;

/// Calcula el Average True Range (ATR) de Wilder de forma 100% causal sin Look-Ahead Bias.
pub fn calculate_atr(klines: &[Kline], period: usize) -> Vec<f64> {
    let n = klines.len();
    if n == 0 {
        return Vec::new();
    }
    let mut atr = vec![0.0; n];
    if n == 1 {
        atr[0] = klines[0].high - klines[0].low;
        return atr;
    }

    let mut tr = vec![0.0; n];
    tr[0] = klines[0].high - klines[0].low;
    for i in 1..n {
        let hl = klines[i].high - klines[i].low;
        let hc = (klines[i].high - klines[i - 1].close).abs();
        let lc = (klines[i].low - klines[i - 1].close).abs();
        tr[i] = hl.max(hc).max(lc);
    }

    let mut current_sum = 0.0;
    for i in 0..n {
        current_sum += tr[i];
        if i >= period {
            current_sum -= tr[i - period];
            atr[i] = current_sum / (period as f64);
        } else {
            atr[i] = current_sum / ((i + 1) as f64);
        }
    }

    atr
}
