use crate::engine::swap::report::SwapReport;
use crate::engine::swap::simulator::SwapSimulator;
use crate::engine::swap::types::{SwapConfig, SynchronizedBar};
use crate::metrics::ranking::{compute_multicriteria_rankings, CandidateRankScore};
use crate::metrics::swap_fitness::{evaluate_swap_fitness, SwapFitnessResult};
use crate::strategies::dual_momentum::DualMomentumStrategy;
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
) -> Vec<CandidateOptimizationResult> {
    let total_bars = bars.len();
    if total_bars < 100 {
        return Vec::new();
    }

    // Partición causal estricta: 70% In-Sample (Entrenamiento) / 30% Out-Of-Sample (Validación no vista)
    let is_bars = ((total_bars as f64) * 0.70).round() as usize;

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
            },
            3, // Tipo Regímenes
            0,
            0,
            0,
            0.0,
        ));
    }

    // Evaluación paralela con Rayon de cada candidato en la ventana de intersección real
    let mut results: Vec<CandidateOptimizationResult> = candidates
        .into_par_iter()
        .map(|(cid, arch, p_summary, cfg, typ, s_w, m_w, a_w, c_th)| {
            let sim = SwapSimulator::new(cfg);
            let report = match typ {
                1 => {
                    let mut model = DualMomentumStrategy::new(s_w, m_w, a_w);
                    sim.run(&mut model, bars, 65, total_bars)
                }
                2 => {
                    let mut model = SwapGbdtStrategy::new(c_th as f32);
                    sim.run(&mut model, bars, 65, total_bars)
                }
                _ => {
                    let mut model = RegimeHybridStrategy::new();
                    sim.run(&mut model, bars, 65, total_bars)
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
