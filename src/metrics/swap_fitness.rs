use crate::engine::swap::report::SwapReport;
use serde::{Deserialize, Serialize};

/// Métricas de linealidad R² para la evaluación de curvas de equidad del motor SWAP
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LinearityMetrics {
    pub r2_global: f64,
    pub r2_is: f64,
    pub r2_oos: f64,
    pub slope_global: f64,
    pub composite_linearity_score: f64,
}

/// Resultado de evaluación del Fitness Institucional SWAP
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SwapFitnessResult {
    /// Puntuación final SWAP Fitness (0 a 100, mayor es mejor)
    pub swap_fitness_score: f64,
    /// Ratio Ganancia Neta / Máximo Drawdown Nominal (50% del peso)
    pub capital_dd_ratio: f64,
    /// Puntuación de Linealidad R² compuesta (25% del peso)
    pub composite_linearity: f64,
    /// Simetría de Pendiente IS/OOS (25% del peso, penaliza overfitting)
    pub slope_ratio: f64,
    pub r2_global: f64,
    pub r2_is: f64,
    pub r2_oos: f64,
    pub slope_is: f64,
    pub slope_oos: f64,
}


/// Calcula el coeficiente de determinación R² y la pendiente mediante regresión lineal
pub fn calculate_series_r2(curve: &[(i64, f64)]) -> (f64, f64) {
    let n = curve.len();
    if n < 2 {
        return (0.0, 0.0);
    }

    let mut sum_x = 0.0f64;
    let mut sum_y = 0.0f64;
    for (i, &(_, eq)) in curve.iter().enumerate() {
        sum_x += i as f64;
        sum_y += eq;
    }
    let mean_x = sum_x / (n as f64);
    let mean_y = sum_y / (n as f64);

    let mut s_xx = 0.0f64;
    let mut s_yy = 0.0f64;
    let mut s_xy = 0.0f64;

    for (i, &(_, eq)) in curve.iter().enumerate() {
        let dx = (i as f64) - mean_x;
        let dy = eq - mean_y;
        s_xx += dx * dx;
        s_yy += dy * dy;
        s_xy += dx * dy;
    }

    if s_xx <= 1e-12 || s_yy <= 1e-12 {
        return (0.0, 0.0);
    }

    let slope = s_xy / s_xx;
    if slope <= 0.0 {
        return (0.0, slope);
    }

    let r2 = ((s_xy * s_xy) / (s_xx * s_yy)).clamp(0.0, 1.0);
    (r2, slope)
}

/// Evalúa la linealidad R² compuesta de SWAP (Global, In-Sample y Out-Of-Sample)
pub fn evaluate_equity_linearity(curve: &[(i64, f64)], is_bars: usize) -> LinearityMetrics {
    if curve.len() < 4 {
        return LinearityMetrics::default();
    }

    let (r2_global, slope_global) = calculate_series_r2(curve);

    let (is_slice, oos_slice) = if curve.len() <= is_bars {
        let train_len = ((curve.len() as f64) * 0.70).round() as usize;
        (&curve[0..train_len], &curve[train_len..])
    } else {
        let is_len = is_bars.min(curve.len());
        (&curve[0..is_len], &curve[is_len..])
    };

    let (r2_is, _) = calculate_series_r2(is_slice);
    let (r2_oos, _) = calculate_series_r2(oos_slice);

    let min_seg_r2 = r2_is.min(r2_oos);
    let avg_seg_r2 = (r2_is + r2_oos) / 2.0;

    let composite = 0.40 * r2_global + 0.35 * avg_seg_r2 + 0.25 * min_seg_r2;

    LinearityMetrics {
        r2_global,
        r2_is,
        r2_oos,
        slope_global,
        composite_linearity_score: composite.clamp(0.0, 1.0),
    }
}

/// Calcula la simetría de pendiente IS/OOS de SWAP (evita modelos que ganan solo en una ventana)
pub fn calculate_slope_ratio(slope_is: f64, slope_oos: f64) -> f64 {
    if slope_is > 0.0 && slope_oos > 0.0 {
        let ratio = slope_is / slope_oos;
        if ratio < 1.0 {
            ratio
        } else {
            1.0 / ratio
        }
    } else {
        0.0
    }
}

/// Ratio institucional Capital / Max Drawdown Nominal
pub fn calculate_capital_max_dd_ratio(final_cap: f64, max_dd_nom: f64, initial_cap: f64) -> f64 {
    let profit = (final_cap - initial_cap).max(0.0);
    if max_dd_nom > 0.0 {
        (profit / max_dd_nom).clamp(0.0, 50.0)
    } else if profit > 0.0 {
        50.0
    } else {
        0.0
    }
}

/// Score compuesto ponderado institucional de SWAP EVO:
/// 1. Ratio Capital / Max Drawdown: 50.0%
/// 2. Linealidad R² compuesta: 25.0%
/// 3. Simetría de Pendiente IS/OOS: 25.0%
pub fn calculate_autotuning_score(
    slope_ratio: f64,
    capital_dd_ratio: f64,
    composite_linearity: f64,
) -> f64 {
    let cap_dd_norm = (capital_dd_ratio / 5.0).clamp(0.0, 1.0);
    (0.25 * composite_linearity + 0.25 * slope_ratio + 0.50 * cap_dd_norm) * 100.0
}

/// Evalúa el Fitness exacto de SWAP sobre un reporte de rotación de cartera
pub fn evaluate_swap_fitness(report: &SwapReport, is_bars: usize) -> SwapFitnessResult {
    let curve = &report.equity_curve;
    if curve.len() < 10 || report.net_profit <= 0.0 {
        return SwapFitnessResult::default();
    }

    let lin = evaluate_equity_linearity(curve, is_bars);

    let is_len = is_bars.min(curve.len());
    let oos_len = curve.len().saturating_sub(is_len);

    let (slope_is, slope_oos) = if is_len > 0 && oos_len > 0 {
        let profit_is = curve[is_len - 1].1 - report.initial_capital;
        let profit_oos = report.final_capital - curve[is_len - 1].1;
        let s_is = profit_is / (is_len as f64);
        let s_oos = profit_oos / (oos_len as f64);
        (s_is, s_oos)
    } else {
        (0.0, 0.0)
    };

    let slope_ratio = calculate_slope_ratio(slope_is, slope_oos);

    let mut peak = curve[0].1;
    let mut max_dd_amount = 0.0f64;
    for &(_, eq) in curve {
        if eq > peak {
            peak = eq;
        }
        let dd = peak - eq;
        if dd > max_dd_amount {
            max_dd_amount = dd;
        }
    }

    let cap_dd_ratio = calculate_capital_max_dd_ratio(
        report.final_capital,
        max_dd_amount,
        report.initial_capital,
    );

    let score = calculate_autotuning_score(
        slope_ratio,
        cap_dd_ratio,
        lin.composite_linearity_score,
    );

    SwapFitnessResult {
        swap_fitness_score: score,
        capital_dd_ratio: cap_dd_ratio,
        composite_linearity: lin.composite_linearity_score,
        slope_ratio,
        r2_global: lin.r2_global,
        r2_is: lin.r2_is,
        r2_oos: lin.r2_oos,
        slope_is,
        slope_oos,
    }
}
