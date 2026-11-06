use super::components::{
    get_client_javascript, get_dashboard_css, render_compare_charts_section,
    render_detail_kpis_section, render_detail_trades_section, render_head_to_head_section,
    render_header, render_leaderboard_section,
};
use super::exporter::{build_swap_dashboard_payload_with_meta, StrategyInput};
use super::models::SwapDashboardPayload;
use crate::engine::swap::optimizer::CandidateOptimizationResult;
use crate::engine::swap::types::SynchronizedBar;
use std::error::Error;
use std::fs::{self, File};
use std::io::Write;

pub fn generate_swap_dashboard_to_file(
    payload: &SwapDashboardPayload,
    output_dir: &str,
    file_name: &str,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    fs::create_dir_all(output_dir)?;

    let json_data = serde_json::to_string(payload)?;

    let css = get_dashboard_css();
    let header_html = render_header(
        &payload.title,
        &payload.timeframe,
        &payload.common_start_date,
        &payload.common_end_date,
        payload.total_bars,
    );
    let leaderboard_html = render_leaderboard_section();
    let head_to_head_html = render_head_to_head_section();
    let compare_html = render_compare_charts_section();
    let detail_kpis_html = render_detail_kpis_section();
    let detail_trades_html = render_detail_trades_section();
    let js = get_client_javascript();

    let full_html = format!(
        r#"<!DOCTYPE html>
<html lang="es">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">
  <title>{title} · SWAP Quant Engine</title>
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Montserrat:wght@400;500;600;700;800;900&family=JetBrains+Mono:wght@400;600;700&display=swap" rel="stylesheet">
  <script src="https://cdn.jsdelivr.net/npm/chart.js@4.4.1/dist/chart.umd.min.js"></script>
  <style>
{css}
  </style>
</head>
<body>
{header_html}

<main class="wrap">
{leaderboard_html}
{head_to_head_html}
{compare_html}
{detail_kpis_html}
{detail_trades_html}
</main>

<script id="datos" type="application/json">
{json_data}
</script>

<script>
{js}
</script>
</body>
</html>"#,
        title = payload.title,
        css = css,
        header_html = header_html,
        leaderboard_html = leaderboard_html,
        compare_html = compare_html,
        detail_kpis_html = detail_kpis_html,
        detail_trades_html = detail_trades_html,
        json_data = json_data,
        js = js
    );

    let full_path = format!("{}/{}", output_dir, file_name);

    let mut file = File::create(&full_path)?;
    file.write_all(full_html.as_bytes())?;

    println!("  🌐 Dashboard HTML SWAP generado con éxito en: {}", full_path);
    Ok(full_path)
}

pub fn generate_swap_dashboard(
    payload: &SwapDashboardPayload,
    output_dir: &str,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let filename = format!("dashboard_swap_{}.html", payload.timeframe.to_lowercase());
    generate_swap_dashboard_to_file(payload, output_dir, &filename)
}

/// Genera los dashboards cuantitativos tras una sesión de optimización:
/// 1. `dashboard_A.html` / `dashboard_gbdt.html`: Todos los candidatos evaluados para GBDT con el ganador en 1° lugar.
/// 2. `dashboard_B.html` / `dashboard_dual_momentum.html`: Todos los candidatos evaluados para Dual Momentum con el ganador en 1° lugar.
/// 3. `dashboard_comparativo.html`: Comparativa institucional mano a mano (Ganador GBDT vs Ganador Dual Momentum) vs Benchmarks.
pub fn generate_optimization_dashboards(
    bars: &[SynchronizedBar],
    is_split_idx: usize,
    timeframe: &str,
    results: &[CandidateOptimizationResult],
    output_dir: &str,
) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
    let mut generated_paths = Vec::new();

    // Paletas institucionales
    let palette_a = [
        "#FF9100", "#FF6D00", "#FFA726", "#FFB74D", "#FF7043", "#F4511E", "#FF8A65", "#D84315",
    ];
    let palette_b = [
        "#00E676", "#00C853", "#69F0AE", "#1DE9B6", "#00BFA5", "#76FF03", "#64DD17", "#B9F6CA",
    ];

    // Separar candidatos por arquitectura preservando el orden de ranking de SWAP Fitness
    let candidates_a: Vec<&CandidateOptimizationResult> = results
        .iter()
        .filter(|c| c.candidate_id.starts_with("GBDT") || c.architecture.contains("GBDT"))
        .collect();

    let candidates_b: Vec<&CandidateOptimizationResult> = results
        .iter()
        .filter(|c| c.candidate_id.starts_with("DM") || c.architecture.contains("Dual Momentum"))
        .collect();

    // 1. Dashboard GBDT (ML)
    if !candidates_a.is_empty() {
        let inputs_a: Vec<StrategyInput> = candidates_a
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let rank_tag = match i {
                    0 => "🥇 ",
                    1 => "🥈 ",
                    2 => "🥉 ",
                    _ => "",
                };
                StrategyInput {
                    id: format!("opt_a_{}", c.candidate_id),
                    name: format!("{}GBDT · {} ({})", rank_tag, c.candidate_id, c.params_summary),
                    architecture: "GBDT".to_string(),
                    color: palette_a[i % palette_a.len()].to_string(),
                    report: &c.report,
                    fitness: &c.swap_fitness,
                }
            })
            .collect();

        let payload_a = build_swap_dashboard_payload_with_meta(
            bars,
            is_split_idx,
            &inputs_a,
            "SWAP Optimización · GBDT Tri-Activo (ML)",
            timeframe,
        );
        let path_a = generate_swap_dashboard_to_file(&payload_a, output_dir, "dashboard_A.html")?;
        let _ = generate_swap_dashboard_to_file(&payload_a, output_dir, "dashboard_gbdt.html");
        generated_paths.push(path_a);
    }

    // 2. Dashboard Dual Momentum
    if !candidates_b.is_empty() {
        let inputs_b: Vec<StrategyInput> = candidates_b
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let rank_tag = match i {
                    0 => "🥇 ",
                    1 => "🥈 ",
                    2 => "🥉 ",
                    _ => "",
                };
                StrategyInput {
                    id: format!("opt_b_{}", c.candidate_id),
                    name: format!("{}Dual Momentum · {} ({})", rank_tag, c.candidate_id, c.params_summary),
                    architecture: "Dual Momentum".to_string(),
                    color: palette_b[i % palette_b.len()].to_string(),
                    report: &c.report,
                    fitness: &c.swap_fitness,
                }
            })
            .collect();

        let payload_b = build_swap_dashboard_payload_with_meta(
            bars,
            is_split_idx,
            &inputs_b,
            "SWAP Optimización · Dual Momentum 50/50",
            timeframe,
        );
        let path_b = generate_swap_dashboard_to_file(&payload_b, output_dir, "dashboard_B.html")?;
        let _ = generate_swap_dashboard_to_file(&payload_b, output_dir, "dashboard_dual_momentum.html");
        generated_paths.push(path_b);
    }

    // 3. Dashboard Comparativo (Ganador GBDT vs Ganador Dual Momentum)
    let mut best_candidates = Vec::new();
    if let Some(w_a) = candidates_a.first() {
        best_candidates.push((*w_a, "GBDT", "#FF9100"));
    }
    if let Some(w_b) = candidates_b.first() {
        best_candidates.push((*w_b, "Dual Momentum", "#00E676"));
    }

    // Ordenar por SWAP Fitness Score descendente
    best_candidates.sort_by(|a, b| {
        let fit_a = a.0.swap_fitness.swap_fitness_score;
        let fit_b = b.0.swap_fitness.swap_fitness_score;
        fit_b.partial_cmp(&fit_a).unwrap_or(std::cmp::Ordering::Equal)
    });

    if !best_candidates.is_empty() {
        let inputs_comp: Vec<StrategyInput> = best_candidates
            .iter()
            .enumerate()
            .map(|(i, (c, arch_label, color))| {
                let rank_tag = match i {
                    0 => "🥇 ",
                    1 => "🥈 ",
                    _ => "",
                };
                StrategyInput {
                    id: format!("winner_{}", c.candidate_id),
                    name: format!("{}{} · {} ({})", rank_tag, arch_label, c.candidate_id, c.params_summary),
                    architecture: arch_label.to_string(),
                    color: (*color).to_string(),
                    report: &c.report,
                    fitness: &c.swap_fitness,
                }
            })
            .collect();

        let payload_comp = build_swap_dashboard_payload_with_meta(
            bars,
            is_split_idx,
            &inputs_comp,
            "SWAP Comparativa Institucional · GBDT vs Dual Momentum",
            timeframe,
        );
        let path_comp = generate_swap_dashboard_to_file(&payload_comp, output_dir, "dashboard_comparativo.html")?;
        generated_paths.push(path_comp);
    }

    Ok(generated_paths)
}
