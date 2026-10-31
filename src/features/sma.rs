/// Simple Moving Average (SMA) estrictamente causal
pub fn calculate_sma(values: &[f64], period: usize) -> Vec<f64> {
    let n = values.len();
    let mut sma = vec![0.0; n];
    if n == 0 || period == 0 {
        return sma;
    }

    let mut sum = 0.0;
    for i in 0..n {
        sum += values[i];
        if i >= period {
            sum -= values[i - period];
            sma[i] = sum / (period as f64);
        } else {
            sma[i] = sum / ((i + 1) as f64);
        }
    }
    sma
}
