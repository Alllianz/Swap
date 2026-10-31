/// Exponential Moving Average (EMA) estrictamente causal
pub fn calculate_ema(values: &[f64], period: usize) -> Vec<f64> {
    let n = values.len();
    let mut ema = vec![0.0; n];
    if n == 0 || period == 0 {
        return ema;
    }

    let alpha = 2.0 / (period as f64 + 1.0);
    ema[0] = values[0];
    for i in 1..n {
        ema[i] = alpha * values[i] + (1.0 - alpha) * ema[i - 1];
    }
    ema
}
