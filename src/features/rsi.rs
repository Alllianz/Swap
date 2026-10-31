/// Relative Strength Index (RSI) de Wilder estrictamente causal
pub fn calculate_rsi(prices: &[f64], period: usize) -> Vec<f64> {
    let n = prices.len();
    if n == 0 || period == 0 {
        return Vec::new();
    }
    let mut rsi = vec![50.0; n];
    if n <= period {
        return rsi;
    }

    let mut gains = 0.0;
    let mut losses = 0.0;

    for i in 1..=period {
        let diff = prices[i] - prices[i - 1];
        if diff >= 0.0 {
            gains += diff;
        } else {
            losses += diff.abs();
        }
    }

    let mut avg_gain = gains / (period as f64);
    let mut avg_loss = losses / (period as f64);

    if avg_loss == 0.0 {
        rsi[period] = 100.0;
    } else {
        let rs = avg_gain / avg_loss;
        rsi[period] = 100.0 - (100.0 / (1.0 + rs));
    }

    for i in (period + 1)..n {
        let diff = prices[i] - prices[i - 1];
        let current_gain = if diff > 0.0 { diff } else { 0.0 };
        let current_loss = if diff < 0.0 { diff.abs() } else { 0.0 };

        avg_gain = (avg_gain * (period as f64 - 1.0) + current_gain) / (period as f64);
        avg_loss = (avg_loss * (period as f64 - 1.0) + current_loss) / (period as f64);

        if avg_loss == 0.0 {
            rsi[i] = 100.0;
        } else {
            let rs = avg_gain / avg_loss;
            rsi[i] = 100.0 - (100.0 / (1.0 + rs));
        }
    }

    rsi
}
