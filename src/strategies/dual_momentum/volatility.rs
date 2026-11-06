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

/// Cálculo causal de Desviación a la Baja (Downside Deviation / Semivarianza anualizada).
/// 0% Look-Ahead Bias: opera estrictamente sobre precios pasados en la ventana [end - window ..= end].
///
/// A diferencia de la volatilidad simétrica total, penaliza ÚNICAMENTE los rendimientos negativos (días bajistas)
/// con respecto al umbral MAR = 0.0 (Minimum Acceptable Return).
/// Los retornos positivos extraordinarios (rallies explosivos de BTC o quiebres alcistas de acciones)
/// no incrementan el denominador, evitando penalizar al activo líder durante fases alcistas.
pub fn calculate_downside_deviation(prices: &[f64], window: usize) -> f64 {
    let n = prices.len();
    if n < window + 1 || window < 2 {
        return 0.10; // Estimación base razonable
    }

    let start = n - window - 1;
    let mut downside_sq_sum = 0.0;
    let mut count = 0;

    for i in start..(n - 1) {
        let p0 = prices[i];
        let p1 = prices[i + 1];
        if p0 > 1e-6 && p1 > 1e-6 {
            let r = (p1 / p0).ln();
            if r < 0.0 {
                downside_sq_sum += r * r;
            }
            count += 1;
        }
    }

    if count < 2 {
        return 0.10;
    }

    // Semivarianza muestral a la baja con N - 1 grados de libertad
    let downside_variance = downside_sq_sum / ((count - 1) as f64);

    // Anualizado (raíz de 365.25 días). Suelo prudente de 0.02 (2% anual) para evitar división por cero
    // o distorsiones numéricas si todos los rendimientos de la ventana fueron positivos.
    (downside_variance.sqrt() * (365.25f64).sqrt()).max(0.02)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_downside_deviation_flat_prices() {
        let prices = vec![100.0; 50];
        let dd = calculate_downside_deviation(&prices, 20);
        // Sin caídas, debe aplicar el floor prudente de 0.02
        assert_eq!(dd, 0.02);
    }

    #[test]
    fn test_downside_deviation_only_rallies() {
        // Precios que suben todos los días: sin caídas
        let prices: Vec<f64> = (1..=50).map(|x| 100.0 * (1.0 + x as f64 * 0.01)).collect();
        let dd = calculate_downside_deviation(&prices, 20);
        assert_eq!(dd, 0.02);
    }

    #[test]
    fn test_downside_deviation_detects_drops() {
        // Precios que alternan subidas y caídas fuertes
        let mut prices = vec![100.0];
        for i in 1..50 {
            let last = prices[i - 1];
            let next = if i % 2 == 0 { last * 1.05 } else { last * 0.90 };
            prices.push(next);
        }
        let dd = calculate_downside_deviation(&prices, 20);
        assert!(dd > 0.05, "Downside deviation debe registrar las caídas del 10%");
    }
}
