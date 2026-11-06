use crate::engine::swap::report::SwapReport;
use crate::engine::swap::simulator::SwapSimulator;
use crate::engine::swap::types::{MarginMode, SwapConfig, SynchronizedBar};
use crate::metrics::ranking::{compute_multicriteria_rankings, CandidateRankScore};
use crate::metrics::swap_fitness::{evaluate_swap_fitness, SwapFitnessResult};
use crate::strategies::dual_momentum::{DualMomentumStrategy, RiskAdjustmentMode};
use crate::strategies::regime_hybrid::RegimeHybridStrategy;
use crate::strategies::swap_gbdt::SwapGbdtStrategy;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CandidateOptimizationResult {
    pub candidate_id: String,
    pub architecture: String,
    pub params_summary: String,
    pub report: SwapReport,
    pub swap_fitness: SwapFitnessResult,
    pub rank_score: Option<CandidateRankScore>,
}

/// Ejecuta la auto-optimización multicriterio con la métrica estricta de SWAP EVO
/// operando EXCLUSIVAMENTE sobre la ventana de intersección real de los 3 activos.
pub fn run_swap_intersection_optimization(
    bars: &[SynchronizedBar],
    initial_capital: f64,
    leverage: f64,
) -> Vec<CandidateOptimizationResult> {
    let total_bars = bars.len();
    if total_bars < 100 {
        return Vec::new();
    }

    let lev = leverage.max(1.0);

    // Partición causal: 50% In-Sample / 50% Out-Of-Sample
    let is_bars = ((total_bars as f64) * 0.50).round() as usize;

    // Espacio de búsqueda de candidatos
    let mut candidates = Vec::new();

    // 1. Grid de Ruta B: Dual Momentum (ventanas cortas, medias, absolutas y retención)
    let dm_configs = [
        (14, 45, 40, 3, 0.010),
        (14, 63, 60, 5, 0.015),
        (21, 63, 60, 5, 0.015),
        (21, 63, 90, 7, 0.020),
        (21, 90, 90, 5, 0.015),
        (30, 90, 120, 10, 0.025),
        (10, 30, 40, 4, 0.010),
        (28, 84, 70, 6, 0.015),
    ];

    for &(short_w, med_w, abs_w, hold_bars, hys) in &dm_configs {
        candidates.push((
            format!("DM_S{}_M{}_A{}_H{}_HY{:.3}", short_w, med_w, abs_w, hold_bars, hys),
            "Dual Momentum".to_string(),
            format!("Corto:{}d, Med:{}d, Abs:{}d, MinHold:{}b, Hyst:{:.3}", short_w, med_w, abs_w, hold_bars, hys),
            SwapConfig {
                initial_capital,
                swap_fee_rate: 0.0010,
                slippage_rate: 0.0005,
                min_holding_bars: hold_bars,
                hysteresis_threshold: hys,
                margin_mode: MarginMode::Cross,
                leverage: lev,
            },
            1, // Tipo DM
            short_w,
            med_w,
            abs_w,
            0.0,
        ));
    }

    // 2. Grid de Ruta A: SWAP GBDT (umbral de salida a cash y retención)
    let gbdt_configs = [
        (-0.25, 3, 0.010),
        (-0.15, 5, 0.015),
        (-0.05, 5, 0.015),
        (0.00, 7, 0.020),
        (-0.10, 4, 0.012),
        (-0.20, 6, 0.018),
    ];

    for &(cash_th, hold_bars, hys) in &gbdt_configs {
        candidates.push((
            format!("GBDT_C{:.2}_H{}_HY{:.3}", cash_th, hold_bars, hys),
            "SWAP GBDT ML".to_string(),
            format!("CashThresh:{:.2}, MinHold:{}b, Hyst:{:.3}", cash_th, hold_bars, hys),
            SwapConfig {
                initial_capital,
                swap_fee_rate: 0.0010,
                slippage_rate: 0.0005,
                min_holding_bars: hold_bars,
                hysteresis_threshold: hys,
                margin_mode: MarginMode::Cross,
                leverage: lev,
            },
            2, // Tipo GBDT
            0,
            0,
            0,
            cash_th,
        ));
    }

    // 3. Grid de Ruta C: Híbrido de Regímenes
    let regime_configs = [
        (3, 0.010),
        (5, 0.015),
        (7, 0.020),
        (10, 0.025),
    ];

    for &(hold_bars, hys) in &regime_configs {
        candidates.push((
            format!("REGIME_H{}_HY{:.3}", hold_bars, hys),
            "Híbrido Regímenes".to_string(),
            format!("MinHold:{}b, Hyst:{:.3}", hold_bars, hys),
            SwapConfig {
                initial_capital,
                swap_fee_rate: 0.0010,
                slippage_rate: 0.0005,
                min_holding_bars: hold_bars,
                hysteresis_threshold: hys,
                margin_mode: MarginMode::Cross,
                leverage: lev,
            },
            3, // Tipo Regímenes
            0,
            0,
            0,
            0.0,
        ));
    }

    // Punto de inicio de la simulación adaptado: en datasets reducidos (ej. 720 velas de 1H),
    // arranca estrictamente en 0 para cubrir todo el histórico disponible.
    let start_idx = if total_bars < 500 {
        0
    } else {
        65.min(total_bars / 5)
    };

    // Evaluación paralela con Rayon de cada candidato en la ventana de intersección real
    let mut results: Vec<CandidateOptimizationResult> = candidates
        .into_par_iter()
        .map(|(cid, arch, p_summary, cfg, typ, s_w, m_w, a_w, c_th)| {
            let sim = SwapSimulator::new(cfg);
            let report = match typ {
                1 => {
                    let mut model = DualMomentumStrategy::new(s_w, m_w, a_w);
                    sim.run(&mut model, bars, start_idx, total_bars)
                }
                2 => {
                    let mut model = SwapGbdtStrategy::new(c_th as f32);
                    sim.run(&mut model, bars, start_idx, total_bars)
                }
                _ => {
                    let mut model = RegimeHybridStrategy::new();
                    sim.run(&mut model, bars, start_idx, total_bars)
                }
            };

            let swap_fit = evaluate_swap_fitness(&report, is_bars);

            CandidateOptimizationResult {
                candidate_id: cid,
                architecture: arch,
                params_summary: p_summary,
                report,
                swap_fitness: swap_fit,
                rank_score: None,
            }
        })
        .collect();

    // Computar el ranking multicriterio institucional de SWAP (por Puestos)
    let rankings_map = compute_multicriteria_rankings(
        &results,
        |c| c.candidate_id.clone(),
        |c| c.swap_fitness.slope_ratio,
        |c| c.swap_fitness.capital_dd_ratio,
        |c| c.swap_fitness.composite_linearity,
        |c| c.report.sortino_ratio,
    );

    for res in &mut results {
        if let Some(r_score) = rankings_map.get(&res.candidate_id) {
            res.rank_score = Some(r_score.clone());
        }
    }

    // Ordenar de mejor a peor según SWAP Rank Fitness
    results.sort_by(|a, b| {
        let f_a = a.rank_score.as_ref().map(|r| r.swap_rank_fitness).unwrap_or(0.0);
        let f_b = b.rank_score.as_ref().map(|r| r.swap_rank_fitness).unwrap_or(0.0);
        f_b.partial_cmp(&f_a).unwrap_or(std::cmp::Ordering::Equal)
    });

    results
}

/// Ejecuta la evaluación y optimización exhaustiva de la población completa de individuos
/// EXCLUSIVAMENTE de la RUTA B (Dual Momentum 50/50 Cross-Asset).
pub fn run_ruta_b_population_optimization(
    bars: &[SynchronizedBar],
    initial_capital: f64,
    leverage: f64,
) -> Vec<CandidateOptimizationResult> {
    let total_bars = bars.len();
    if total_bars < 100 {
        return Vec::new();
    }

    let lev = leverage.max(1.0);
    let is_bars = ((total_bars as f64) * 0.50).round() as usize;

    // Grid de 18 individuos cuantitativos de Ruta B con nomenclatura técnica de parámetros
    let dm_population = [
        ("DM_S7_M21_A30_H2_HY008_SORT_SH", 7, 21, 30, 2, 0.008, RiskAdjustmentMode::SortinoDownside, true, "S:7d, M:21d, Abs:30d, Hold:2b, Hys:0.008, Sortino, Shorts"),
        ("DM_S10_M30_A40_H3_HY010_SORT_SH", 10, 30, 40, 3, 0.010, RiskAdjustmentMode::SortinoDownside, true, "S:10d, M:30d, Abs:40d, Hold:3b, Hys:0.010, Sortino, Shorts"),
        ("DM_S10_M30_A40_H3_HY010_SHRP_SH", 10, 30, 40, 3, 0.010, RiskAdjustmentMode::SharpeTotalVolatility, true, "S:10d, M:30d, Abs:40d, Hold:3b, Hys:0.010, Sharpe, Shorts"),
        ("DM_S14_M45_A40_H3_HY010_SORT_SH", 14, 45, 40, 3, 0.010, RiskAdjustmentMode::SortinoDownside, true, "S:14d, M:45d, Abs:40d, Hold:3b, Hys:0.010, Sortino, Shorts"),
        ("DM_S14_M45_A40_H3_HY010_SHRP_SH", 14, 45, 40, 3, 0.010, RiskAdjustmentMode::SharpeTotalVolatility, true, "S:14d, M:45d, Abs:40d, Hold:3b, Hys:0.010, Sharpe, Shorts"),
        ("DM_S14_M63_A60_H5_HY015_SORT_SH", 14, 63, 60, 5, 0.015, RiskAdjustmentMode::SortinoDownside, true, "S:14d, M:63d, Abs:60d, Hold:5b, Hys:0.015, Sortino, Shorts"),
        ("DM_S21_M63_A60_H5_HY015_SORT_SH", 21, 63, 60, 5, 0.015, RiskAdjustmentMode::SortinoDownside, true, "S:21d, M:63d, Abs:60d, Hold:5b, Hys:0.015, Sortino, Shorts"),
        ("DM_S21_M63_A90_H7_HY020_SORT_SH", 21, 63, 90, 7, 0.020, RiskAdjustmentMode::SortinoDownside, true, "S:21d, M:63d, Abs:90d, Hold:7b, Hys:0.020, Sortino, Shorts"),
        ("DM_S21_M90_A90_H5_HY015_SORT_SH", 21, 90, 90, 5, 0.015, RiskAdjustmentMode::SortinoDownside, true, "S:21d, M:90d, Abs:90d, Hold:5b, Hys:0.015, Sortino, Shorts"),
        ("DM_S30_M90_A120_H10_HY025_SORT_SH", 30, 90, 120, 10, 0.025, RiskAdjustmentMode::SortinoDownside, true, "S:30d, M:90d, Abs:120d, Hold:10b, Hys:0.025, Sortino, Shorts"),
        ("DM_S28_M84_A70_H6_HY015_SORT_SH", 28, 84, 70, 6, 0.015, RiskAdjustmentMode::SortinoDownside, true, "S:28d, M:84d, Abs:70d, Hold:6b, Hys:0.015, Sortino, Shorts"),
        ("DM_S35_M105_A120_H8_HY020_SORT_SH", 35, 105, 120, 8, 0.020, RiskAdjustmentMode::SortinoDownside, true, "S:35d, M:105d, Abs:120d, Hold:8b, Hys:0.020, Sortino, Shorts"),
        ("DM_S10_M30_A40_H3_HY010_SORT_LO", 10, 30, 40, 3, 0.010, RiskAdjustmentMode::SortinoDownside, false, "S:10d, M:30d, Abs:40d, Hold:3b, Hys:0.010, Sortino, LongOnly"),
        ("DM_S14_M63_A60_H5_HY015_SORT_LO", 14, 63, 60, 5, 0.015, RiskAdjustmentMode::SortinoDownside, false, "S:14d, M:63d, Abs:60d, Hold:5b, Hys:0.015, Sortino, LongOnly"),
        ("DM_S21_M90_A90_H7_HY020_SORT_LO", 21, 90, 90, 7, 0.020, RiskAdjustmentMode::SortinoDownside, false, "S:21d, M:90d, Abs:90d, Hold:7b, Hys:0.020, Sortino, LongOnly"),
        ("DM_S21_M63_A60_H7_HY030_SORT_SH", 21, 63, 60, 7, 0.030, RiskAdjustmentMode::SortinoDownside, true, "S:21d, M:63d, Abs:60d, Hold:7b, Hys:0.030, Sortino, Shorts"),
        ("DM_S14_M45_A40_H3_HY005_SORT_SH", 14, 45, 40, 3, 0.005, RiskAdjustmentMode::SortinoDownside, true, "S:14d, M:45d, Abs:40d, Hold:3b, Hys:0.005, Sortino, Shorts"),
        ("DM_S21_M63_A90_H5_HY015_SHRP_SH", 21, 63, 90, 5, 0.015, RiskAdjustmentMode::SharpeTotalVolatility, true, "S:21d, M:63d, Abs:90d, Hold:5b, Hys:0.015, Sharpe, Shorts"),
    ];

    let start_idx = if total_bars < 500 {
        0
    } else {
        65.min(total_bars / 5)
    };

    let mut results: Vec<CandidateOptimizationResult> = dm_population
        .into_par_iter()
        .map(|(cid, s_w, m_w, a_w, hold_bars, hys, risk_mode, allow_shorts, label)| {
            let cfg = SwapConfig {
                initial_capital,
                swap_fee_rate: 0.0010,
                slippage_rate: 0.0005,
                min_holding_bars: hold_bars,
                hysteresis_threshold: hys,
                margin_mode: MarginMode::Cross,
                leverage: lev,
            };
            let sim = SwapSimulator::new(cfg);
            let mut model = DualMomentumStrategy::new(s_w, m_w, a_w)
                .with_risk_mode(risk_mode)
                .with_shorts(allow_shorts);

            let report = sim.run(&mut model, bars, start_idx, total_bars);
            let swap_fit = evaluate_swap_fitness(&report, is_bars);

            CandidateOptimizationResult {
                candidate_id: cid.to_string(),
                architecture: "Ruta B: Dual Momentum 50/50".to_string(),
                params_summary: label.to_string(),
                report,
                swap_fitness: swap_fit,
                rank_score: None,
            }
        })
        .collect();

    let rankings_map = compute_multicriteria_rankings(
        &results,
        |c| c.candidate_id.clone(),
        |c| c.swap_fitness.slope_ratio,
        |c| c.swap_fitness.capital_dd_ratio,
        |c| c.swap_fitness.composite_linearity,
        |c| c.report.sortino_ratio,
    );

    for res in &mut results {
        if let Some(r_score) = rankings_map.get(&res.candidate_id) {
            res.rank_score = Some(r_score.clone());
        }
    }

    results.sort_by(|a, b| {
        let f_a = a.rank_score.as_ref().map(|r| r.swap_rank_fitness).unwrap_or(0.0);
        let f_b = b.rank_score.as_ref().map(|r| r.swap_rank_fitness).unwrap_or(0.0);
        f_b.partial_cmp(&f_a).unwrap_or(std::cmp::Ordering::Equal)
    });

    results
}
