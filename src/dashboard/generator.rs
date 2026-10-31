use super::components::{
    get_client_javascript, get_dashboard_css, render_compare_charts_section,
    render_detail_kpis_section, render_detail_trades_section, render_header,
    render_leaderboard_section,
};
use super::models::SwapDashboardPayload;
use std::error::Error;
use std::fs::{self, File};
use std::io::Write;

pub fn generate_swap_dashboard(
    payload: &SwapDashboardPayload,
    output_dir: &str,
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

    let filename = format!("dashboard_swap_{}.html", payload.timeframe.to_lowercase());
    let full_path = format!("{}/{}", output_dir, filename);

    let mut file = File::create(&full_path)?;
    file.write_all(full_html.as_bytes())?;

    println!("  🌐 Dashboard HTML SWAP generado con éxito en: {}", full_path);
    Ok(full_path)
}
