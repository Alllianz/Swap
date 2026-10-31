use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SmoothnessMetrics {
    pub sortino_ratio: f64,
    pub downside_deviation: f64,
    pub smoothness_score: f64,
    pub ulcer_index: f64,
    pub r2_score: f64,
    pub slope_per_bar: f64,
}

/// Calcula el Ulcer Index, R² lineal y suavidad institucional de la curva de equidad
pub fn calculate_equity_smoothness(
    curve: &[(i64, f64)],
    initial_capital: f64,
) -> SmoothnessMetrics {
    let n = curve.len();
    if n < 2 {
        return SmoothnessMetrics::default();
    }

    // 1. Cálculo de Ulcer Index sobre la curva de balance
    let mut peak = curve[0].1;
    let mut sum_sq_dd = 0.0f64;

    for &(_, eq) in curve {
        if eq > peak {
            peak = eq;
        }
        let dd_pct = if peak > 0.0 {
            ((peak - eq) / peak) * 100.0
        } else {
            0.0
        };
        sum_sq_dd += dd_pct * dd_pct;
    }
    let ulcer_index = (sum_sq_dd / (n as f64)).sqrt();

    // 2. Cálculo de Sortino Ratio sobre los retornos
    let mut returns = Vec::with_capacity(n - 1);
    for i in 1..n {
        let prev = curve[i - 1].1;
        let curr = curve[i].1;
        if prev > 0.0 {
            returns.push((curr - prev) / prev);
        }
    }

    let (sortino_ratio, downside_dev) = if returns.is_empty() {
        (0.0, 0.0)
    } else {
        let mean_ret = returns.iter().sum::<f64>() / (returns.len() as f64);
        let mut sum_downside_sq = 0.0f64;
        let mut downside_count = 0;

        for &r in &returns {
            if r < 0.0 {
                sum_downside_sq += r * r;
                downside_count += 1;
            }
        }

        let downside_dev = if downside_count > 0 {
            (sum_downside_sq / (returns.len() as f64)).sqrt()
        } else {
            0.0
        };

        let sortino = if downside_dev > 1e-8 {
            (mean_ret / downside_dev) * (returns.len() as f64).sqrt()
        } else {
            0.0
        };

        (sortino, downside_dev)
    };

    // 3. Regresión Lineal: Pendiente ($/vela) y R²
    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    let mut sum_xy = 0.0;
    let mut sum_xx = 0.0;
    let n_f = n as f64;

    for (i, &(_, eq)) in curve.iter().enumerate() {
        let x = i as f64;
        let y = eq;
        sum_x += x;
        sum_y += y;
        sum_xy += x * y;
        sum_xx += x * x;
    }

    let slope = (n_f * sum_xy - sum_x * sum_y) / (n_f * sum_xx - sum_x * sum_x).max(1e-9);
    let intercept = (sum_y - slope * sum_x) / n_f;

    let y_mean = sum_y / n_f;
    let mut ss_tot = 0.0;
    let mut ss_res = 0.0;
    for (i, &(_, eq)) in curve.iter().enumerate() {
        let y_pred = slope * (i as f64) + intercept;
        ss_res += (eq - y_pred).powi(2);
        ss_tot += (eq - y_mean).powi(2);
    }

    let r2 = if ss_tot > 1e-9 {
        (1.0 - (ss_res / ss_tot)).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let pnl = curve.last().map(|&(_, eq)| eq - initial_capital).unwrap_or(0.0);
    let smoothness_score = if ulcer_index > 0.0 {
        (pnl / ulcer_index.max(1.0)) * r2
    } else {
        pnl * r2
    };

    SmoothnessMetrics {
        sortino_ratio,
        downside_deviation: downside_dev,
        smoothness_score,
        ulcer_index,
        r2_score: r2,
        slope_per_bar: slope,
    }
}
