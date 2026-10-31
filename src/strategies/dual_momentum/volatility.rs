/// Cálculo causal de volatilidad realizada histórica (desviación estándar de rendimientos logarítmicos).
/// 0% Look-Ahead Bias: opera estrictamente sobre precios pasados en la ventana [end - window ..= end].
pub fn calculate_realized_volatility(prices: &[f64], window: usize) -> f64 {
    let n = prices.len();
    if n < window + 1 || window < 2 {
        return 0.15; // Estimación base no nula por defecto
    }

    let start = n - window - 1;
    let mut log_returns = Vec::with_capacity(window);

    for i in start..(n - 1) {
        let p0 = prices[i];
        let p1 = prices[i + 1];
        if p0 > 1e-6 && p1 > 1e-6 {
            log_returns.push((p1 / p0).ln());
        }
    }

    if log_returns.len() < 2 {
        return 0.15;
    }

    let m_count = log_returns.len() as f64;
    let mean = log_returns.iter().sum::<f64>() / m_count;
    let variance = log_returns
        .iter()
        .map(|r| (r - mean).powi(2))
        .sum::<f64>()
        / (m_count - 1.0);

    // Anualizado (raíz de 365 días)
    (variance.sqrt() * (365.25f64).sqrt()).max(0.01)
}
