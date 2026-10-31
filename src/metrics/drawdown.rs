use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawdownPoint {
    pub timestamp: i64,
    pub dd_vs_peak_pct: f64,
    pub dd_vs_capital_pct: f64,
    pub dd_amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawdownSummary {
    pub max_dd_amount: f64,
    pub max_dd_pct_peak: f64,
    pub max_dd_pct_capital: f64,
    pub series: Vec<DrawdownPoint>,
}

/// Computa la serie temporal punto a punto de drawdown y estadísticas
pub fn compute_drawdown_series(curve: &[(i64, f64)], initial_capital: f64) -> DrawdownSummary {
    if curve.is_empty() {
        return DrawdownSummary {
            max_dd_amount: 0.0,
            max_dd_pct_peak: 0.0,
            max_dd_pct_capital: 0.0,
            series: Vec::new(),
        };
    }

    let mut peak = curve[0].1;
    let mut max_dd_amount = 0.0;
    let mut max_dd_pct_peak = 0.0;
    let mut max_dd_pct_capital = 0.0;
    let mut series = Vec::with_capacity(curve.len());

    for &(ts, eq) in curve {
        if eq > peak {
            peak = eq;
        }
        let dd = (peak - eq).max(0.0);
        let dd_peak_pct = if peak > 0.0 { (dd / peak) * 100.0 } else { 0.0 };
        let dd_cap_pct = if initial_capital > 0.0 { (dd / initial_capital) * 100.0 } else { 0.0 };

        if dd > max_dd_amount {
            max_dd_amount = dd;
        }
        if dd_peak_pct > max_dd_pct_peak {
            max_dd_pct_peak = dd_peak_pct;
        }
        if dd_cap_pct > max_dd_pct_capital {
            max_dd_pct_capital = dd_cap_pct;
        }

        series.push(DrawdownPoint {
            timestamp: ts,
            dd_vs_peak_pct: -dd_peak_pct,
            dd_vs_capital_pct: -dd_cap_pct,
            dd_amount: dd,
        });
    }

    DrawdownSummary {
        max_dd_amount,
        max_dd_pct_peak,
        max_dd_pct_capital,
        series,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drawdown_calculation() {
        let curve = vec![
            (1000, 1000.0), // peak
            (2000, 800.0),  // dd = 200 (20% vs peak, 20% vs capital)
            (3000, 1200.0), // new peak
            (4000, 900.0),  // dd = 300 (25% vs peak, 30% vs capital)
        ];

        let summary = compute_drawdown_series(&curve, 1000.0);
        assert_eq!(summary.max_dd_amount, 300.0);
        assert!((summary.max_dd_pct_peak - 25.0).abs() < 1e-4);
        assert!((summary.max_dd_pct_capital - 30.0).abs() < 1e-4);
    }
}
