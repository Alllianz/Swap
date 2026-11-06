use crate::dashboard::{
    build_swap_dashboard_payload_with_meta, generate_optimization_dashboards,
    generate_swap_dashboard_to_file, StrategyInput,
};
use crate::data::downloader::{
    download_all_historical_binance_klines, download_yahoo_finance_klines,
    insert_multi_asset_klines,
};
use crate::data::loader::MultiAssetLoader;
use crate::data::timeframe::normalize_timeframe;
use crate::engine::swap::optimizer::{
    run_ruta_b_population_optimization, run_swap_intersection_optimization,
};
use crate::engine::swap::report::SwapReport;
use crate::engine::swap::simulator::SwapSimulator;
use crate::engine::swap::types::{MarginMode, SwapConfig};
use crate::metrics::swap_fitness::{evaluate_swap_fitness, SwapFitnessResult};
use crate::strategies::dual_momentum::DualMomentumStrategy;
use crate::strategies::regime_hybrid::RegimeHybridStrategy;
use crate::strategies::swap_gbdt::SwapGbdtStrategy;
use crate::ui::prompts::{
    pause, prompt_leverage_multi, prompt_leverage_value, prompt_margin_mode, prompt_swap_config,
    prompt_timeframe,
};
use reqwest::Client;
use rusqlite::Connection;
use std::error::Error;
use std::io::{self, Write};

/// Ejecuta la simulación individual de una de las tres rutas de swap
pub fn run_single_swap_strategy(
    conn: &Connection,
    route: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if route == "B" {
        println!("\n  🔍 Modalidad de Simulación para RUTA B (Dual Momentum 50/50):");
        println!("  [1] ⚡ Simulación Individual con Apalancamiento Personalizado");
        println!("  [2] 🧬 Evaluar Población Completa de 18 Individuos (Dashboard Exclusivo con Comparador Mano a Mano)");
        println!("  [3] ⚡ Barrido Multi-Apalancamiento (Sweep 1X a 10X en Consola y Dashboard)");
        print!("\n  👉 Selecciona opción (1-3) [Por defecto '2']: ");
        let _ = io::stdout().flush();
        let mut choice = String::new();
        let _ = io::stdin().read_line(&mut choice);
        let trimmed = choice.trim();
        if trimmed == "2" || trimmed.is_empty() {
            return run_ruta_b_population_session(conn);
        } else if trimmed == "3" {
            return run_leverage_sweep_session(conn, Some("B"));
        }
    } else if route == "A" {
        println!("\n  🔍 Modalidad de Simulación para RUTA A (GBDT Tri-Activo):");
        println!("  [1] ⚡ Simulación Individual con Apalancamiento Personalizado");
        println!("  [2] ⚡ Barrido Multi-Apalancamiento (Sweep 1X a 10X en Consola y Dashboard)");
        print!("\n  👉 Selecciona opción (1-2) [Por defecto '1']: ");
        let _ = io::stdout().flush();
        let mut choice = String::new();
        let _ = io::stdin().read_line(&mut choice);
        if choice.trim() == "2" {
            return run_leverage_sweep_session(conn, Some("A"));
        }
    }

    let tf = prompt_timeframe();
    let tf_clean = normalize_timeframe(&tf);
    println!("\n  🔄 Cargando y sincronizando dataset multiactivo (BTC, ORO, SP500) en [{}]...", tf_clean);
    let dataset = MultiAssetLoader::load_dataset(conn, &tf_clean)?;

    println!(
        "  ✅ Dataset cargado: {} barras sincronizadas (0% Look-Ahead Bias)",
        dataset.len()
    );
    if let (Some(first), Some(last)) = (dataset.bars.first(), dataset.bars.last()) {
        println!(
            "     Rango temporal: {} → {}",
            first.date_string(),
            last.date_string()
        );
    }

    println!("\n  ⚙️ Configuración de Parámetros de Simulación:");
    let config = prompt_swap_config();
    let sim = SwapSimulator::new(config);

    // En temporalidades intradía (1H, 4H), el backtest arranca estrictamente desde la vela 0
    // de las velas disponibles hasta la última (ej. 720 velas -> paso 0 a 720).
    let start_step = if tf_clean == "1H" || tf_clean == "4H" || tf_clean.ends_with('M') {
        0
    } else {
        65.min(dataset.len().saturating_sub(1) / 4)
    };

    println!("\n  🚀 Ejecutando simulación causal desde la vela {} hasta la {}...", start_step, dataset.len());
    let report: SwapReport = match route {
        "A" => {
            let mut strategy = SwapGbdtStrategy::default();
            sim.run(&mut strategy, &dataset.bars, start_step, dataset.len())
        }
        "B" => {
            let mut strategy = DualMomentumStrategy::default();
            sim.run(&mut strategy, &dataset.bars, start_step, dataset.len())
        }
        "C" => {
            let mut strategy = RegimeHybridStrategy::default();
            sim.run(&mut strategy, &dataset.bars, start_step, dataset.len())
        }
        _ => return Err("Ruta no reconocida".into()),
    };

    println!("\n  📊 RESULTADOS CUANTITATIVOS:");
    report.print_summary_card();

    // Generar dashboard HTML interactivo para esta estrategia
    let total_bars = dataset.len();
    let is_split_idx = ((total_bars as f64) * 0.50).round() as usize;
    let fit = evaluate_swap_fitness(&report, is_split_idx);

    let (strat_id, strat_name, arch_name, color, file_name) = match route {
        "A" => (
            "gbdt".to_string(),
            "GBDT".to_string(),
            "GBDT".to_string(),
            "#FF9100".to_string(),
            "dashboard_gbdt.html",
        ),
        "B" => (
            "dual_momentum".to_string(),
            "Dual Momentum".to_string(),
            "Dual Momentum".to_string(),
            "#00E676".to_string(),
            "dashboard_dual_momentum.html",
        ),
        _ => (
            "swap_strategy".to_string(),
            format!("Estrategia {}", route),
            "SWAP Custom".to_string(),
            "#FFD600".to_string(),
            "dashboard_single.html",
        ),
    };

    let strat_input = StrategyInput {
        id: strat_id,
        name: strat_name.clone(),
        architecture: arch_name,
        color,
        report: &report,
        fitness: &fit,
    };

    let payload = build_swap_dashboard_payload_with_meta(
        &dataset.bars,
        is_split_idx,
        &[strat_input],
        &format!("SWAP Reporte Individual — {}", strat_name),
        &tf_clean,
    );

    let output_dir = "dashboard";
    println!("\n  🌐 Generando dashboard interactivo HTML...");
    match generate_swap_dashboard_to_file(&payload, output_dir, file_name) {
        Ok(path) => {
            let abs_path = std::fs::canonicalize(&path)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| path.clone());
            println!("  \x1b[1;32m✅ Dashboard HTML generado con éxito:\x1b[0m");
            println!("     • Archivo:   \x1b[1;36m{}\x1b[0m", path);
            println!("     • Enlace:    \x1b[1;36mfile:///{}\x1b[0m", abs_path.replace('\\', "/"));
            // También actualizamos dashboard_swap.html como acceso rápido genérico
            let _ = generate_swap_dashboard_to_file(&payload, output_dir, "dashboard_swap.html");
        }
        Err(e) => {
            println!("  ⚠️ Error al generar dashboard HTML: {}", e);
        }
    }

    // Opción para ver los últimos trades de swap
    print!("\n  📋 ¿Deseas ver el desglose de rotaciones de cartera? (s/N): ");
    let _ = io::stdout().flush();
    let mut show_trades = String::new();
    let _ = io::stdin().read_line(&mut show_trades);
    if show_trades.trim().eq_ignore_ascii_case("s") {
        print_trades_table(&report);
    }

    pause();
    Ok(())
}

/// Ejecuta la comparativa simultánea de 2 o 3 estrategias vs Benchmarks
pub fn run_comparative_leaderboard(conn: &Connection) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("\n  ⚔️ MODALIDAD DE COMPARATIVA MANO A MANO:");
    println!("  [1] ⚔️ 2 Estrategias: GBDT Tri-Activo vs Dual Momentum 50/50");
    println!("  [2] 🏆 3 Estrategias: GBDT Tri-Activo vs Dual Momentum vs Regime Hybrid (Rutas A, B y C)");
    println!("  [3] 🎯 Selección Personalizada (Elige 2 o 3 estrategias del catálogo)");
    print!("\n  👉 Selecciona opción (1-3) [Por defecto '2']: ");
    let _ = io::stdout().flush();
    let mut choice = String::new();
    let _ = io::stdin().read_line(&mut choice);
    let trimmed = choice.trim();

    struct StratConfig {
        id: String,
        name: String,
        arch: String,
        color: String,
        kind: usize,
    }

    let selected_strats: Vec<StratConfig> = match trimmed {
        "1" => vec![
            StratConfig {
                id: "gbdt".to_string(),
                name: "GBDT Tri-Activo".to_string(),
                arch: "Ruta A: GBDT Tri-Activo".to_string(),
                color: "#FF9100".to_string(),
                kind: 1,
            },
            StratConfig {
                id: "dual_momentum".to_string(),
                name: "Dual Momentum 50/50".to_string(),
                arch: "Ruta B: Dual Momentum".to_string(),
                color: "#00E676".to_string(),
                kind: 2,
            },
        ],
        "3" => {
            println!("\n  📋 Catálogo de Estrategias Cuantitativas:");
            println!("     [1] 🌲 GBDT Tri-Activo (Scoring & Ranking ML)");
            println!("     [2] 📈 Dual Momentum 50/50 Estándar (Lookback 14 / Filtro 45)");
            println!("     [3] 🛡️ Regime Hybrid (Clasificador Macro + Filtro Volatilidad)");
            println!("     [4] ⚡ Dual Momentum Ágil (Lookback 7 / Filtro 20)");
            println!("     [5] 🐢 Dual Momentum Conservador (Lookback 30 / Filtro 60)");
            println!("     [6] 📉 Dual Momentum con Shorts Tácticos");
            print!("\n  👉 Ingresa 2 o 3 números separados por coma o espacio (ej. '1, 2' o '1, 2, 3') [Por defecto '1, 2, 3']: ");
            let _ = io::stdout().flush();
            let mut custom_input = String::new();
            let _ = io::stdin().read_line(&mut custom_input);
            let nums: Vec<usize> = custom_input
                .split([',', ' ', ';'])
                .filter_map(|s| s.trim().parse::<usize>().ok())
                .filter(|&n| (1..=6).contains(&n))
                .collect();

            let target_nums = if nums.len() >= 2 {
                nums.into_iter().take(3).collect()
            } else {
                vec![1, 2, 3]
            };

            let get_strat = |k: usize| match k {
                1 => StratConfig {
                    id: "gbdt".to_string(),
                    name: "GBDT Tri-Activo".to_string(),
                    arch: "Ruta A: GBDT Tri-Activo".to_string(),
                    color: "#FF9100".to_string(),
                    kind: 1,
                },
                2 => StratConfig {
                    id: "dual_momentum".to_string(),
                    name: "Dual Momentum 50/50".to_string(),
                    arch: "Ruta B: Dual Momentum".to_string(),
                    color: "#00E676".to_string(),
                    kind: 2,
                },
                3 => StratConfig {
                    id: "regime_hybrid".to_string(),
                    name: "Regime Hybrid".to_string(),
                    arch: "Ruta C: Regime Hybrid".to_string(),
                    color: "#00B0FF".to_string(),
                    kind: 3,
                },
                4 => StratConfig {
                    id: "dm_agile".to_string(),
                    name: "DM Ágil (7/20)".to_string(),
                    arch: "Dual Momentum Rápido".to_string(),
                    color: "#00E5FF".to_string(),
                    kind: 4,
                },
                5 => StratConfig {
                    id: "dm_conservative".to_string(),
                    name: "DM Conservador (30/60)".to_string(),
                    arch: "Dual Momentum Tendencial".to_string(),
                    color: "#7C4DFF".to_string(),
                    kind: 5,
                },
                _ => StratConfig {
                    id: "dm_shorts".to_string(),
                    name: "DM + Shorts".to_string(),
                    arch: "Dual Momentum con Shorts".to_string(),
                    color: "#FF5252".to_string(),
                    kind: 6,
                },
            };

            target_nums.into_iter().map(get_strat).collect()
        }
        _ => vec![
            StratConfig {
                id: "gbdt".to_string(),
                name: "GBDT Tri-Activo".to_string(),
                arch: "Ruta A: GBDT Tri-Activo".to_string(),
                color: "#FF9100".to_string(),
                kind: 1,
            },
            StratConfig {
                id: "dual_momentum".to_string(),
                name: "Dual Momentum 50/50".to_string(),
                arch: "Ruta B: Dual Momentum".to_string(),
                color: "#00E676".to_string(),
                kind: 2,
            },
            StratConfig {
                id: "regime_hybrid".to_string(),
                name: "Regime Hybrid".to_string(),
                arch: "Ruta C: Regime Hybrid".to_string(),
                color: "#00B0FF".to_string(),
                kind: 3,
            },
        ],
    };

    let tf = prompt_timeframe();
    let tf_clean = normalize_timeframe(&tf);
    println!("\n  🔄 Cargando y sincronizando dataset multiactivo en [{}]...", tf_clean);
    let dataset = MultiAssetLoader::load_dataset(conn, &tf_clean)?;

    let config = prompt_swap_config();
    let sim = SwapSimulator::new(config);

    let start_step = if tf_clean == "1H" || tf_clean == "4H" || tf_clean.ends_with('M') {
        0
    } else {
        65.min(dataset.len().saturating_sub(1) / 4)
    };
    let total_bars = dataset.len();
    let is_split_idx = ((total_bars as f64) * 0.50).round() as usize;

    println!("\n  ⚡ Evaluando {} estrategias simultáneamente (velas {} → {})...", selected_strats.len(), start_step, dataset.len());

    struct EvalItem {
        id: String,
        name: String,
        arch: String,
        color: String,
        report: SwapReport,
        fitness: SwapFitnessResult,
    }

    let mut evaluated: Vec<EvalItem> = Vec::new();

    for sc in &selected_strats {
        let rep = match sc.kind {
            1 => {
                let mut strat = SwapGbdtStrategy::default();
                sim.run(&mut strat, &dataset.bars, start_step, dataset.len())
            }
            2 => {
                let mut strat = DualMomentumStrategy::default();
                sim.run(&mut strat, &dataset.bars, start_step, dataset.len())
            }
            3 => {
                let mut strat = RegimeHybridStrategy::default();
                sim.run(&mut strat, &dataset.bars, start_step, dataset.len())
            }
            4 => {
                let mut strat = DualMomentumStrategy::new(7, 20, 20);
                sim.run(&mut strat, &dataset.bars, start_step, dataset.len())
            }
            5 => {
                let mut strat = DualMomentumStrategy::new(30, 60, 60);
                sim.run(&mut strat, &dataset.bars, start_step, dataset.len())
            }
            _ => {
                let mut strat = DualMomentumStrategy::new(14, 45, 40).with_shorts(true);
                sim.run(&mut strat, &dataset.bars, start_step, dataset.len())
            }
        };

        let fit = evaluate_swap_fitness(&rep, is_split_idx);
        evaluated.push(EvalItem {
            id: sc.id.clone(),
            name: sc.name.clone(),
            arch: sc.arch.clone(),
            color: sc.color.clone(),
            report: rep,
            fitness: fit,
        });
    }

    let mut leaderboard_indices: Vec<usize> = (0..evaluated.len()).collect();
    leaderboard_indices.sort_by(|&i, &j| {
        evaluated[j]
            .report
            .sharpe_ratio
            .partial_cmp(&evaluated[i].report.sharpe_ratio)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    println!();
    println!("  \x1b[1;36m╔══════════════════════════════════════════════════════════════════════════════════════════════════════════╗\x1b[0m");
    println!("  \x1b[1;36m║\x1b[0m                  \x1b[1;33m🏆 LEADERBOARD COMPARATIVO — {} ESTRATEGIAS CUANTITATIVAS 🏆\x1b[0m                  \x1b[1;36m║\x1b[0m", evaluated.len());
    println!("  \x1b[1;36m╠══════════════════════════════════════════════════════════════════════════════════════════════════════════╣\x1b[0m");
    println!("  \x1b[1;36m║\x1b[0m  Puesto  Estrategia                        Retorno %    CAGR %   Sharpe   Max DD %   Swaps   WinRate %  \x1b[1;36m║\x1b[0m");
    println!("  \x1b[1;36m╠══════════════════════════════════════════════════════════════════════════════════════════════════════════╣\x1b[0m");

    for (rank, &idx) in leaderboard_indices.iter().enumerate() {
        let e = &evaluated[idx];
        let medal = match rank {
            0 => "🥇 1°",
            1 => "🥈 2°",
            2 => "🥉 3°",
            _ => "    ",
        };
        println!(
            "  \x1b[1;36m║\x1b[0m  {:<4}  {:<32} {:>+10.2}% {:>8.2}%  {:>6.2}   {:>7.2}%   {:>5}   {:>7.1}%  \x1b[1;36m║\x1b[0m",
            medal,
            e.name,
            e.report.total_return_pct,
            e.report.cagr_pct,
            e.report.sharpe_ratio,
            e.report.max_drawdown_pct,
            e.report.total_swaps,
            e.report.win_rate_swaps_pct
        );
    }

    println!("  \x1b[1;36m╠══════════════════════════════════════════════════════════════════════════════════════════════════════════╣\x1b[0m");
    println!("  \x1b[1;36m║\x1b[0m  BENCHMARKS PASIVOS DE REFERENCIA (BUY & HOLD):                                                          \x1b[1;36m║\x1b[0m");
    if let Some(first_e) = evaluated.first() {
        println!(
            "  \x1b[1;36m║\x1b[0m   • B&H Bitcoin: {:>+8.2}%  |  • B&H Oro: {:>+8.2}%  |  • B&H S&P 500: {:>+8.2}%                 \x1b[1;36m║\x1b[0m",
            first_e.report.benchmark_btc_return_pct,
            first_e.report.benchmark_gold_return_pct,
            first_e.report.benchmark_sp500_return_pct
        );
        println!(
            "  \x1b[1;36m║\x1b[0m   • Cartera Equitativa (1/3 c/u): {:>+8.2}%                                                       \x1b[1;36m║\x1b[0m",
            first_e.report.benchmark_equal_weight_return_pct
        );
    }
    println!("  \x1b[1;36m╚══════════════════════════════════════════════════════════════════════════════════════════════════════════╝\x1b[0m");
    println!();

    // Matriz Comparativa Mano a Mano (2 o 3 estrategias)
    let h2h_entries: Vec<HeadToHeadEntry> = evaluated
        .iter()
        .map(|e| HeadToHeadEntry {
            name: &e.name,
            report: &e.report,
            fitness: &e.fitness,
        })
        .collect();

    print_head_to_head_matrix(&h2h_entries);

    // Dashboard HTML
    let strat_inputs: Vec<StrategyInput> = evaluated
        .iter()
        .map(|e| StrategyInput {
            id: e.id.clone(),
            name: e.name.clone(),
            architecture: e.arch.clone(),
            color: e.color.clone(),
            report: &e.report,
            fitness: &e.fitness,
        })
        .collect();

    let title = format!("SWAP Leaderboard: Comparativa Mano a Mano ({} Estrategias)", evaluated.len());
    let payload = build_swap_dashboard_payload_with_meta(
        &dataset.bars,
        is_split_idx,
        &strat_inputs,
        &title,
        &tf_clean,
    );

    let output_dir = "dashboard";
    println!("  🌐 Generando dashboard comparativo interactivo HTML...");
    match generate_swap_dashboard_to_file(&payload, output_dir, "dashboard_comparativo.html") {
        Ok(path) => {
            let abs_path = std::fs::canonicalize(&path)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| path.clone());
            println!("  \x1b[1;32m✅ Dashboard Comparativo HTML generado con éxito:\x1b[0m");
            println!("     • Archivo:   \x1b[1;36m{}\x1b[0m", path);
            println!("     • Enlace:    \x1b[1;36mfile:///{}\x1b[0m", abs_path.replace('\\', "/"));
        }
        Err(e) => {
            println!("  ⚠️ Error al generar dashboard comparativo HTML: {}", e);
        }
    }
    println!();

    pause();
    Ok(())
}

/// Menú interactivo de descarga de velas desde Binance API y Yahoo Finance
pub async fn run_download_menu(
    conn: &mut Connection,
    client: &Client,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("\n  📥 DESCARGADOR DE VELAS MULTIACTIVO (Binance & Yahoo Finance API):");
    println!("  [1] 🥇 Descargar Historial Completo de PAXGUSDT (Oro Físico / Binance)");
    println!("  [2] ₿  Descargar Historial Completo de BTCUSDT (Bitcoin / Binance)");
    println!("  [3] 📈 Descargar Historial Real de SPY (S&P 500 ETF / Yahoo Finance)");
    println!("  [0] Volver al Menú Principal");
    print!("\n  👉 Selecciona un activo (0-3) [Por defecto '3']: ");
    let _ = io::stdout().flush();

    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);

    match input.trim() {
        "1" => {
            let tf = prompt_timeframe();
            let tf_clean = normalize_timeframe(&tf);
            println!("\n  ⚙️ Modo de Descarga para PAXGUSDT [{}]:", tf_clean);
            println!("  [1] 🌟 Completa desde el Génesis (Descarga TODOS los lotes históricos)");
            println!("  [2] 🔄 Incremental (Desde la última vela registrada)");
            print!("\n  👉 Selecciona modo (1-2) [Por defecto '1']: ");
            let _ = io::stdout().flush();
            input.clear();
            let _ = io::stdin().read_line(&mut input);
            let from_scratch = !input.trim().eq("2");

            let total = download_all_historical_binance_klines(conn, client, "PAXGUSDT", &tf_clean, from_scratch).await?;
            println!("\n  ✅ ¡Proceso finalizado! Total de velas de Oro guardadas: {}", total);
        }
        "2" => {
            let tf = prompt_timeframe();
            let tf_clean = normalize_timeframe(&tf);
            println!("\n  ⚙️ Modo de Descarga para BTCUSDT [{}]:", tf_clean);
            println!("  [1] 🌟 Completa desde el Génesis");
            println!("  [2] 🔄 Incremental");
            print!("\n  👉 Selecciona modo (1-2) [Por defecto '1']: ");
            let _ = io::stdout().flush();
            input.clear();
            let _ = io::stdin().read_line(&mut input);
            let from_scratch = !input.trim().eq("2");

            let total = download_all_historical_binance_klines(conn, client, "BTCUSDT", &tf_clean, from_scratch).await?;
            println!("\n  ✅ ¡Proceso finalizado! Total de velas de BTC guardadas: {}", total);
        }
        "3" | "" => {
            println!("\n  ⚙️ Selecciona temporalidad para descargar SPY (S&P 500 ETF de Yahoo Finance):");
            println!("  [1] 📅 Diario (1D) — Historial completo de 10 Años");
            println!("  [2] ⏱️ 1 Hora (1H) — Historial de hasta 730 Días (~2 Años / velas intradía de 1H)");
            println!("  [3] ⏱️ 4 Horas (4H) — Historial de hasta 730 Días (~2 Años / velas intradía de 4H)");
            print!("\n  👉 Selecciona temporalidad (1-3) [Por defecto '3']: ");
            let _ = io::stdout().flush();
            let mut tf_choice = String::new();
            let _ = io::stdin().read_line(&mut tf_choice);
            let (range, interval, tf_tag) = match tf_choice.trim() {
                "1" => ("10y", "1d", "1D"),
                "2" => ("730d", "1h", "1H"),
                _ => ("730d", "4h", "4H"),
            };

            println!("\n  🌐 Conectando a Yahoo Finance API para SPY [{}] (range: {})...", tf_tag, range);
            let klines = download_yahoo_finance_klines(client, "SPY", range, interval).await?;
            println!("  📥 Recibidas {} velas reales [{}] de SPY (S&P 500).", klines.len(), tf_tag);
            if let (Some(first), Some(last)) = (klines.first(), klines.last()) {
                println!("     Rango temporal SPY: {} → {}", first.date_utc_string(), last.date_utc_string());
            }

            let inserted = insert_multi_asset_klines(conn, "SP500", tf_tag, &klines)?;
            println!("  💾 Almacenadas {} velas de SP500 [{}] en candles.db.", inserted, tf_tag);
        }
        "0" => return Ok(()),
        _ => {
            println!("  ⚠️ Opción no válida.");
        }
    }

    pause();
    Ok(())
}

fn print_trades_table(report: &SwapReport) {
    println!("\n  ── HISTORIAL DE ROTACIONES / SWAPS ({}) ──", report.trades.len());
    println!("  {:<4} | {:<10} | {:<18} | {:<12} | {:<12} | {:<9} | Razón",
        "N°", "Fecha", "Rotación", "Cap Antes", "Cap Después", "Comisión"
    );
    println!("  {}", "-".repeat(105));

    for trade in report.trades.iter().take(20) {
        println!(
            "  {:<4} | {:<10} | {:>6} → {:<6} | ${:<10.2} | ${:<10.2} | ${:<7.2} | {}",
            trade.swap_index,
            trade.date_str,
            trade.from_asset.symbol(),
            trade.to_asset.symbol(),
            trade.capital_before,
            trade.capital_after,
            trade.fee_paid,
            trade.reason
        );
    }
    if report.trades.len() > 20 {
        println!("  ... y {} rotaciones más.", report.trades.len() - 20);
    }
}

/// Ejecuta la sesión de auto-optimización multicriterio con la métrica estricta de SWAP
/// operando EXCLUSIVAMENTE sobre la ventana de intersección real de los 3 activos.
pub fn run_optimization_session(conn: &Connection) -> Result<(), Box<dyn Error + Send + Sync>> {
    let tf = prompt_timeframe();
    let tf_clean = normalize_timeframe(&tf);
    println!("\n  🔄 Cargando dataset de INTERSECCIÓN REAL (BTC, PAXG, SP500) en [{}]...", tf_clean);
    let dataset = MultiAssetLoader::load_real_intersection_dataset(conn, &tf_clean)?;

    println!(
        "  ✅ Dataset de Intersección cargado: {} barras reales sincronizadas.",
        dataset.len()
    );
    if let (Some(first), Some(last)) = (dataset.bars.first(), dataset.bars.last()) {
        println!(
            "     Rango Real Común: {} → {} (0% datos sintéticos)",
            first.date_string(),
            last.date_string()
        );
    }

    print!("\n  💵 Capital Inicial para optimización en USD [Por defecto $10,000]: ");
    let _ = io::stdout().flush();
    let mut input = String::new();
    let mut initial_capital = 10_000.0;
    if io::stdin().read_line(&mut input).is_ok()
        && let Ok(val) = input.trim().parse::<f64>()
        && val > 0.0
    {
        initial_capital = val;
    }

    let leverage = prompt_leverage_value(5.0);

    println!("\n  🧬 Iniciando Auto-Optimización Institucional SWAP EVO...");
    println!("     Apalancamiento de Cartera: {:.1}X (Modo Capital Cruzado)", leverage);
    println!("     Métrica: SWAP Fitness (50% Cap/DD + 25% R² Compuesto + 25% Slope IS/OOS)");
    println!("     Ventana de Optimización: 70% In-Sample / 30% Out-Of-Sample Causal\n");

    let results = run_swap_intersection_optimization(&dataset.bars, initial_capital, leverage);

    if results.is_empty() {
        println!("  ⚠️ No se obtuvieron resultados de optimización.");
        pause();
        return Ok(());
    }

    println!("  \x1b[1;36m╔══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════╗\x1b[0m");
    println!("  \x1b[1;36m║\x1b[0m                        \x1b[1;33m🏆 RESULTADOS DE OPTIMIZACIÓN — MÉTRICA INSTITUCIONAL SWAP EVO 🏆\x1b[0m                      \x1b[1;36m║\x1b[0m");
    println!("  \x1b[1;36m╠══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════╣\x1b[0m");
    println!("  \x1b[1;36m║\x1b[0m Puesto  Candidato ID           Arquitectura      SWAP Fit    Score Fit  Cap/DD   R² Comp  SlopeRatio  Sharpe  Max DD%  \x1b[1;36m║\x1b[0m");
    println!("  \x1b[1;36m╠══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════╣\x1b[0m");

    for (idx, res) in results.iter().enumerate() {
        let rank_display = match idx {
            0 => "🥇 1°".to_string(),
            1 => "🥈 2°".to_string(),
            2 => "🥉 3°".to_string(),
            _ => format!("{:>3}°", idx + 1),
        };
        let swap_rank = res.rank_score.as_ref().map(|r| r.swap_rank_fitness).unwrap_or(0.0);
        println!(
            "  \x1b[1;36m║\x1b[0m  {:<4}  {:<21}  {:<16}  {:>7.2}%   {:>8.2}  {:>6.2}   {:>6.3}    {:>6.3}    {:>6.2}   {:>6.2}% \x1b[1;36m║\x1b[0m",
            rank_display,
            res.candidate_id,
            res.architecture,
            swap_rank,
            res.swap_fitness.swap_fitness_score,
            res.swap_fitness.capital_dd_ratio,
            res.swap_fitness.composite_linearity,
            res.swap_fitness.slope_ratio,
            res.report.sharpe_ratio,
            res.report.max_drawdown_pct
        );
    }
    println!("  \x1b[1;36m╚══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════╝\x1b[0m");

    if let Some(winner) = results.first() {
        println!("\n  👑 MEJOR CONFIGURACIÓN ENCONTRADA POR SWAP EVO:");
        println!("     • Candidato:      {}", winner.candidate_id);
        println!("     • Arquitectura:   {}", winner.architecture);
        println!("     • Parámetros:     {}", winner.params_summary);
        println!("     • SWAP Fitness:   {:.2} puntos (Rank: {:.2}%)", winner.swap_fitness.swap_fitness_score, winner.rank_score.as_ref().map(|r| r.swap_rank_fitness).unwrap_or(0.0));
        println!("     • Retorno Total:  +{:+.2}% (Final: ${:.2})", winner.report.total_return_pct, winner.report.final_capital);
        println!("     • Sharpe / Sortino:{:.2} / {:.2}", winner.report.sharpe_ratio, winner.report.sortino_ratio);
        println!("     • Max Drawdown:   {:.2}%", winner.report.max_drawdown_pct);
    }

    let total_bars = dataset.len();
    let is_split_idx = ((total_bars as f64) * 0.50).round() as usize;
    let output_dir = "dashboard";

    println!("\n  🌐 Generando dashboards interactivos de optimización...");
    match generate_optimization_dashboards(
        &dataset.bars,
        is_split_idx,
        &tf_clean,
        &results,
        output_dir,
    ) {
        Ok(files) => {
            println!("  \x1b[1;32m✅ 4 Dashboards HTML generados con éxito en la carpeta '{}':\x1b[0m", output_dir);
            for file in &files {
                let abs = std::fs::canonicalize(file)
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| file.clone());
                println!("     • Enlace: \x1b[1;36mfile:///{}\x1b[0m", abs.replace('\\', "/"));
            }
        }
        Err(e) => {
            println!("  ⚠️ Error al generar dashboards de optimización: {}", e);
        }
    }

    pause();
    Ok(())
}

/// Ejecuta la evaluación y optimización exhaustiva de la población completa de individuos
/// EXCLUSIVAMENTE de RUTA B (Dual Momentum 50/50 Cross-Asset).
pub fn run_ruta_b_population_session(conn: &Connection) -> Result<(), Box<dyn Error + Send + Sync>> {
    let tf = prompt_timeframe();
    let tf_clean = normalize_timeframe(&tf);
    println!("\n  🔄 Cargando dataset multiactivo para RUTA B (BTC, PAXG, SP500) en [{}]...", tf_clean);
    let dataset = MultiAssetLoader::load_real_intersection_dataset(conn, &tf_clean)?;

    println!(
        "  ✅ Dataset de Intersección cargado: {} barras reales sincronizadas.",
        dataset.len()
    );
    if let (Some(first), Some(last)) = (dataset.bars.first(), dataset.bars.last()) {
        println!(
            "     Rango Real Común: {} → {} (0% datos sintéticos, 0% look-ahead)",
            first.date_string(),
            last.date_string()
        );
    }

    print!("\n  💵 Capital Inicial para simulación en USD [Por defecto $10,000]: ");
    let _ = io::stdout().flush();
    let mut input = String::new();
    let mut initial_capital = 10_000.0;
    if io::stdin().read_line(&mut input).is_ok()
        && let Ok(val) = input.trim().parse::<f64>()
        && val > 0.0
    {
        initial_capital = val;
    }

    let leverage = prompt_leverage_value(5.0);

    println!("\n  🧬 Iniciando Simulación de Población de Individuos — Dual Momentum 50/50...");
    println!("     Estrategia: 50% Mejor Activo + 50% Segundo Mejor Activo / CASH");
    println!("     Modo Capital: Capital Cruzado con Apalancamiento {:.1}X (Cross Margin — colateral unificado)", leverage);
    println!("     Población: 18 Individuos Cuantitativos (Ventanas Cortas, Medias, Filtros Absolutos, Sharpe vs Sortino y Shorts)");
    println!("     Métrica: SWAP Fitness (50% Cap/DD + 25% R² Compuesto + 25% Slope IS/OOS)\n");

    let results = run_ruta_b_population_optimization(&dataset.bars, initial_capital, leverage);

    if results.is_empty() {
        println!("  ⚠️ No se obtuvieron resultados de la población de Ruta B.");
        pause();
        return Ok(());
    }

    println!("  \x1b[1;36m╔═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════╗\x1b[0m");
    println!("  \x1b[1;36m║\x1b[0m                                   \x1b[1;32m🏆 LEADERBOARD EXCLUSIVO — POBLACIÓN DE INDIVIDUOS RUTA B 🏆\x1b[0m                                    \x1b[1;36m║\x1b[0m");
    println!("  \x1b[1;36m╠═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════╣\x1b[0m");
    println!("  \x1b[1;36m║\x1b[0m Puesto  Individuo ID (Parámetros)            Retorno %    CAGR %   Sharpe  Sortino  Max DD %   Swaps  WinRate%  SWAP Fit   Linealidad R²  \x1b[1;36m║\x1b[0m");
    println!("  \x1b[1;36m╠═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════╣\x1b[0m");

    for (idx, res) in results.iter().enumerate() {
        let rank_display = match idx {
            0 => "🥇 1°".to_string(),
            1 => "🥈 2°".to_string(),
            2 => "🥉 3°".to_string(),
            _ => format!("{:>3}°", idx + 1),
        };
        let swap_rank = res.rank_score.as_ref().map(|r| r.swap_rank_fitness).unwrap_or(0.0);
        println!(
            "  \x1b[1;36m║\x1b[0m  {:<4}  {:<34} {:>+9.2}%  {:>7.2}%   {:>5.2}   {:>6.2}   {:>7.2}%   {:>5}   {:>6.1}%  {:>7.2}%     {:>6.3}      \x1b[1;36m║\x1b[0m",
            rank_display,
            res.candidate_id,
            res.report.total_return_pct,
            res.report.cagr_pct,
            res.report.sharpe_ratio,
            res.report.sortino_ratio,
            res.report.max_drawdown_pct,
            res.report.total_swaps,
            res.report.win_rate_swaps_pct,
            swap_rank,
            res.swap_fitness.composite_linearity
        );
    }
    println!("  \x1b[1;36m╚═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════╝\x1b[0m");

    if let Some(winner) = results.first() {
        println!("\n  👑 MEJOR INDIVIDUO DE RUTA B ENCONTRADO:");
        println!("     • Individuo:      {}", winner.candidate_id);
        println!("     • Configuración:  {}", winner.params_summary);
        println!("     • SWAP Fitness:   {:.2} pts (Rank: {:.2}%)", winner.swap_fitness.swap_fitness_score, winner.rank_score.as_ref().map(|r| r.swap_rank_fitness).unwrap_or(0.0));
        println!("     • Retorno Total:  +{:+.2}% (Final: ${:.2})", winner.report.total_return_pct, winner.report.final_capital);
        println!("     • Sharpe / Sortino:{:.2} / {:.2}", winner.report.sharpe_ratio, winner.report.sortino_ratio);
        println!("     • Max Drawdown:   {:.2}%", winner.report.max_drawdown_pct);
    }

    let total_bars = dataset.len();
    let is_split_idx = ((total_bars as f64) * 0.50).round() as usize;

    let palette_b = [
        "#00E676", "#00C853", "#69F0AE", "#1DE9B6", "#00BFA5", "#76FF03", "#64DD17",
        "#B9F6CA", "#00E5FF", "#26A69A", "#4CAF50", "#81C784", "#A5D6A7", "#00897B",
        "#004D40", "#2E7D32", "#1B5E20", "#388E3C",
    ];

    let strat_inputs: Vec<StrategyInput> = results
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
                id: format!("b_{}", c.candidate_id),
                name: format!("{}Dual Momentum · {} ({})", rank_tag, c.candidate_id, c.params_summary),
                architecture: "Dual Momentum 50/50".to_string(),
                color: palette_b[i % palette_b.len()].to_string(),
                report: &c.report,
                fitness: &c.swap_fitness,
            }
        })
        .collect();

    let payload = build_swap_dashboard_payload_with_meta(
        &dataset.bars,
        is_split_idx,
        &strat_inputs,
        "SWAP Población Exclusiva · Dual Momentum 50/50 (18 Individuos)",
        &tf_clean,
    );

    let output_dir = "dashboard";
    println!("\n  🌐 Generando dashboard interactivo exclusivo para Dual Momentum (con Comparador Mano a Mano)...");
    match generate_swap_dashboard_to_file(&payload, output_dir, "dashboard_ruta_b_individuos.html") {
        Ok(path) => {
            let abs_path = std::fs::canonicalize(&path)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| path.clone());
            println!("  \x1b[1;32m✅ Dashboard Exclusivo de Dual Momentum generado con éxito:\x1b[0m");
            println!("     • Archivo:   \x1b[1;36m{}\x1b[0m", path);
            println!("     • Enlace:    \x1b[1;36mfile:///{}\x1b[0m", abs_path.replace('\\', "/"));

            // También actualizamos dashboard_B.html y dashboard_dual_momentum.html para acceso directo
            let _ = generate_swap_dashboard_to_file(&payload, output_dir, "dashboard_B.html");
            let _ = generate_swap_dashboard_to_file(&payload, output_dir, "dashboard_dual_momentum.html");
        }
        Err(e) => {
            println!("  ⚠️ Error al generar dashboard de Dual Momentum: {}", e);
        }
    }

    pause();
    Ok(())
}

/// Entrada de datos para la comparativa Mano a Mano
pub struct HeadToHeadEntry<'a> {
    pub name: &'a str,
    pub report: &'a SwapReport,
    pub fitness: &'a SwapFitnessResult,
}

/// Imprime la matriz comparativa Mano a Mano para 2 o 3 estrategias vs Benchmark
pub fn print_head_to_head_matrix(entries: &[HeadToHeadEntry]) {
    if entries.len() == 2 {
        print_head_to_head_2(&entries[0], &entries[1]);
    } else if entries.len() >= 3 {
        print_head_to_head_3(&entries[0], &entries[1], &entries[2]);
    }
}

fn print_head_to_head_2(e1: &HeadToHeadEntry, e2: &HeadToHeadEntry) {
    let rep_a = e1.report;
    let fit_a = e1.fitness;
    let rep_b = e2.report;
    let fit_b = e2.fitness;

    println!();
    println!("  \x1b[1;36m╔══════════════════════════════════╤════════════════════════╤════════════════════════╤════════════════════════╗\x1b[0m");
    println!("  \x1b[1;36m║\x1b[0m                         \x1b[1;33m⚔️ COMPARATIVA MANO A MANO: {} vs {} ⚔️\x1b[0m                         \x1b[1;36m║\x1b[0m", e1.name, e2.name);
    println!("  \x1b[1;36m╠══════════════════════════════════╪════════════════════════╪════════════════════════╪════════════════════════╣\x1b[0m");
    println!(
        "  \x1b[1;36m║\x1b[0m \x1b[1m{:<32}\x1b[0m │ \x1b[1;33m{:<22}\x1b[0m │ \x1b[1;32m{:<22}\x1b[0m │ \x1b[1;37m{:<22}\x1b[0m │",
        "Métrica Cuantitativa", e1.name, e2.name, "Benchmark (Equitativo)"
    );
    println!("  \x1b[1;36m╠══════════════════════════════════╪════════════════════════╪════════════════════════╪════════════════════════╣\x1b[0m");

    let print_row = |metric: &str, a: &str, b: &str, bench: &str| {
        println!(
            "  \x1b[1;36m║\x1b[0m  {:<32} │ {:>22} │ {:>22} │ {:>22} │",
            metric, a, b, bench
        );
    };

    let print_header_section = |title: &str| {
        println!("  \x1b[1;36m╟──────────────────────────────────┼────────────────────────┼────────────────────────┼────────────────────────╢\x1b[0m");
        println!("  \x1b[1;36m║\x1b[0m  \x1b[1;35m{:<32}\x1b[0m │                        │                        │                        │", title);
        println!("  \x1b[1;36m╟──────────────────────────────────┼────────────────────────┼────────────────────────┼────────────────────────╢\x1b[0m");
    };

    let bench_final = rep_a.initial_capital * (1.0 + rep_a.benchmark_equal_weight_return_pct / 100.0);
    let bench_pnl = bench_final - rep_a.initial_capital;

    print_header_section("── CAPITAL Y RETORNO ──");
    print_row("Apalancamiento Operativo", &format!("{:.1}X Cruzado", rep_a.leverage), &format!("{:.1}X Cruzado", rep_b.leverage), "1.0X (Spot)");
    print_row("Capital Inicial", &format!("${:.2}", rep_a.initial_capital), &format!("${:.2}", rep_b.initial_capital), &format!("${:.2}", rep_a.initial_capital));
    print_row("Capital Final", &format!("${:.2}", rep_a.final_capital), &format!("${:.2}", rep_b.final_capital), &format!("${:.2}", bench_final));
    print_row("Ganancia Neta (PnL)", &format!("{:>+14.2}", rep_a.net_profit), &format!("{:>+14.2}", rep_b.net_profit), &format!("{:>+14.2}", bench_pnl));
    print_row("Retorno Total (%)", &format!("{:>+14.2}%", rep_a.total_return_pct), &format!("{:>+14.2}%", rep_b.total_return_pct), &format!("{:>+14.2}%", rep_a.benchmark_equal_weight_return_pct));
    print_row("CAGR Anualizado (%)", &format!("{:>+14.2}%", rep_a.cagr_pct), &format!("{:>+14.2}%", rep_b.cagr_pct), "-");

    print_header_section("── CONTROL DE RIESGO ──");
    print_row("Máximo Drawdown (%)", &format!("{:>14.2}%", rep_a.max_drawdown_pct), &format!("{:>14.2}%", rep_b.max_drawdown_pct), "-");
    print_row("Duración Máx DD (barras)", &format!("{} barras", rep_a.max_drawdown_duration_bars), &format!("{} barras", rep_b.max_drawdown_duration_bars), "-");
    print_row("Sharpe Ratio", &format!("{:>14.2}", rep_a.sharpe_ratio), &format!("{:>14.2}", rep_b.sharpe_ratio), "-");
    print_row("Sortino Ratio", &format!("{:>14.2}", rep_a.sortino_ratio), &format!("{:>14.2}", rep_b.sortino_ratio), "-");
    print_row("Calmar Ratio", &format!("{:>14.2}", rep_a.calmar_ratio), &format!("{:>14.2}", rep_b.calmar_ratio), "-");

    print_header_section("── SWAP FITNESS Y ROBUSTEZ ──");
    print_row("SWAP Fitness Score", &format!("{:>14.2} pts", fit_a.swap_fitness_score), &format!("{:>14.2} pts", fit_b.swap_fitness_score), "-");
    print_row("Linealidad R² Compuesta", &format!("{:>14.3}", fit_a.composite_linearity), &format!("{:>14.3}", fit_b.composite_linearity), "-");
    print_row("Simetría IS/OOS (SlopeRatio)", &format!("{:>14.3}", fit_a.slope_ratio), &format!("{:>14.3}", fit_b.slope_ratio), "-");
    print_row("Ratio Ganancia / Max DD USD", &format!("{:>14.2}", fit_a.capital_dd_ratio), &format!("{:>14.2}", fit_b.capital_dd_ratio), "-");

    print_header_section("── OPERATIVA Y TRANSACCIÓN ──");
    print_row("Total Swaps / Rotaciones", &format!("{}", rep_a.total_swaps), &format!("{}", rep_b.total_swaps), "0");
    print_row("Swaps Ganadores", &format!("{}", rep_a.profitable_swaps), &format!("{}", rep_b.profitable_swaps), "-");
    print_row("Tasa de Acierto (WinRate)", &format!("{:>14.1}%", rep_a.win_rate_swaps_pct), &format!("{:>14.1}%", rep_b.win_rate_swaps_pct), "-");
    print_row("Comisiones + Slippage ($)", &format!("${:>13.2}", rep_a.total_fees_paid), &format!("${:>13.2}", rep_b.total_fees_paid), "$0.00");

    print_header_section("── DISTRIBUCIÓN DE EXPOSICIÓN ──");
    print_row("Exposición Bitcoin (BTC)", &format!("{:>14.1}%", rep_a.exposure_btc_pct), &format!("{:>14.1}%", rep_b.exposure_btc_pct), "33.3%");
    print_row("Exposición Oro (PAXG)", &format!("{:>14.1}%", rep_a.exposure_gold_pct), &format!("{:>14.1}%", rep_b.exposure_gold_pct), "33.3%");
    print_row("Exposición S&P 500 (SPY)", &format!("{:>14.1}%", rep_a.exposure_sp500_pct), &format!("{:>14.1}%", rep_b.exposure_sp500_pct), "33.3%");
    let short_a = rep_a.exposure_short_btc_pct + rep_a.exposure_short_gold_pct + rep_a.exposure_short_sp500_pct;
    let short_b = rep_b.exposure_short_btc_pct + rep_b.exposure_short_gold_pct + rep_b.exposure_short_sp500_pct;
    print_row("Exposición Posiciones Short", &format!("{:>14.1}%", short_a), &format!("{:>14.1}%", short_b), "0.0%");
    print_row("Preservación en Dólar (CASH)", &format!("{:>14.1}%", rep_a.exposure_cash_pct), &format!("{:>14.1}%", rep_b.exposure_cash_pct), "0.0%");

    print_header_section("── GENERACIÓN DE ALPHA ──");
    print_row("Alpha vs S&P 500 B&H", &format!("{:>+14.2}%", rep_a.alpha_vs_sp500), &format!("{:>+14.2}%", rep_b.alpha_vs_sp500), &format!("{:>+14.2}%", rep_a.benchmark_equal_weight_return_pct - rep_a.benchmark_sp500_return_pct));
    print_row("Alpha vs Bitcoin B&H", &format!("{:>+14.2}%", rep_a.alpha_vs_btc), &format!("{:>+14.2}%", rep_b.alpha_vs_btc), &format!("{:>+14.2}%", rep_a.benchmark_equal_weight_return_pct - rep_a.benchmark_btc_return_pct));

    println!("  \x1b[1;36m╚══════════════════════════════════╧════════════════════════╧════════════════════════╧════════════════════════╝\x1b[0m");
    println!();
}

fn print_head_to_head_3(e1: &HeadToHeadEntry, e2: &HeadToHeadEntry, e3: &HeadToHeadEntry) {
    let rep_a = e1.report;
    let fit_a = e1.fitness;
    let rep_b = e2.report;
    let fit_b = e2.fitness;
    let rep_c = e3.report;
    let fit_c = e3.fitness;

    println!();
    println!("  \x1b[1;36m╔════════════════════════════════╤════════════════════╤════════════════════╤════════════════════╤════════════════════╗\x1b[0m");
    println!("  \x1b[1;36m║\x1b[0m                               \x1b[1;33m⚔️ COMPARATIVA MANO A MANO (3 ESTRATEGIAS) ⚔️\x1b[0m                                 \x1b[1;36m║\x1b[0m");
    println!("  \x1b[1;36m╠════════════════════════════════╪════════════════════╪════════════════════╪════════════════════╪════════════════════╣\x1b[0m");
    println!(
        "  \x1b[1;36m║\x1b[0m \x1b[1m{:<30}\x1b[0m │ \x1b[1;33m{:<18}\x1b[0m │ \x1b[1;32m{:<18}\x1b[0m │ \x1b[1;34m{:<18}\x1b[0m │ \x1b[1;37m{:<18}\x1b[0m │",
        "Métrica Cuantitativa", e1.name, e2.name, e3.name, "Benchmark (Equit.)"
    );
    println!("  \x1b[1;36m╠════════════════════════════════╪════════════════════╪════════════════════╪════════════════════╪════════════════════╣\x1b[0m");

    let print_row = |metric: &str, a: &str, b: &str, c: &str, bench: &str| {
        println!(
            "  \x1b[1;36m║\x1b[0m  {:<30} │ {:>18} │ {:>18} │ {:>18} │ {:>18} │",
            metric, a, b, c, bench
        );
    };

    let print_header_section = |title: &str| {
        println!("  \x1b[1;36m╟────────────────────────────────┼────────────────────┼────────────────────┼────────────────────┼────────────────────╢\x1b[0m");
        println!("  \x1b[1;36m║\x1b[0m  \x1b[1;35m{:<30}\x1b[0m │                    │                    │                    │                    │", title);
        println!("  \x1b[1;36m╟────────────────────────────────┼────────────────────┼────────────────────┼────────────────────┼────────────────────╢\x1b[0m");
    };

    let bench_final = rep_a.initial_capital * (1.0 + rep_a.benchmark_equal_weight_return_pct / 100.0);
    let bench_pnl = bench_final - rep_a.initial_capital;

    print_header_section("── CAPITAL Y RETORNO ──");
    print_row("Apalancamiento Operativo", &format!("{:.1}X Cruzado", rep_a.leverage), &format!("{:.1}X Cruzado", rep_b.leverage), &format!("{:.1}X Cruzado", rep_c.leverage), "1.0X (Spot)");
    print_row("Capital Inicial", &format!("${:.2}", rep_a.initial_capital), &format!("${:.2}", rep_b.initial_capital), &format!("${:.2}", rep_c.initial_capital), &format!("${:.2}", rep_a.initial_capital));
    print_row("Capital Final", &format!("${:.2}", rep_a.final_capital), &format!("${:.2}", rep_b.final_capital), &format!("${:.2}", rep_c.final_capital), &format!("${:.2}", bench_final));
    print_row("Ganancia Neta (PnL)", &format!("{:>+14.2}", rep_a.net_profit), &format!("{:>+14.2}", rep_b.net_profit), &format!("{:>+14.2}", rep_c.net_profit), &format!("{:>+14.2}", bench_pnl));
    print_row("Retorno Total (%)", &format!("{:>+14.2}%", rep_a.total_return_pct), &format!("{:>+14.2}%", rep_b.total_return_pct), &format!("{:>+14.2}%", rep_c.total_return_pct), &format!("{:>+14.2}%", rep_a.benchmark_equal_weight_return_pct));
    print_row("CAGR Anualizado (%)", &format!("{:>+14.2}%", rep_a.cagr_pct), &format!("{:>+14.2}%", rep_b.cagr_pct), &format!("{:>+14.2}%", rep_c.cagr_pct), "-");

    print_header_section("── CONTROL DE RIESGO ──");
    print_row("Máximo Drawdown (%)", &format!("{:>14.2}%", rep_a.max_drawdown_pct), &format!("{:>14.2}%", rep_b.max_drawdown_pct), &format!("{:>14.2}%", rep_c.max_drawdown_pct), "-");
    print_row("Duración Máx DD (barras)", &format!("{} barras", rep_a.max_drawdown_duration_bars), &format!("{} barras", rep_b.max_drawdown_duration_bars), &format!("{} barras", rep_c.max_drawdown_duration_bars), "-");
    print_row("Sharpe Ratio", &format!("{:>14.2}", rep_a.sharpe_ratio), &format!("{:>14.2}", rep_b.sharpe_ratio), &format!("{:>14.2}", rep_c.sharpe_ratio), "-");
    print_row("Sortino Ratio", &format!("{:>14.2}", rep_a.sortino_ratio), &format!("{:>14.2}", rep_b.sortino_ratio), &format!("{:>14.2}", rep_c.sortino_ratio), "-");
    print_row("Calmar Ratio", &format!("{:>14.2}", rep_a.calmar_ratio), &format!("{:>14.2}", rep_b.calmar_ratio), &format!("{:>14.2}", rep_c.calmar_ratio), "-");

    print_header_section("── SWAP FITNESS Y ROBUSTEZ ──");
    print_row("SWAP Fitness Score", &format!("{:>14.2} pts", fit_a.swap_fitness_score), &format!("{:>14.2} pts", fit_b.swap_fitness_score), &format!("{:>14.2} pts", fit_c.swap_fitness_score), "-");
    print_row("Linealidad R² Compuesta", &format!("{:>14.3}", fit_a.composite_linearity), &format!("{:>14.3}", fit_b.composite_linearity), &format!("{:>14.3}", fit_c.composite_linearity), "-");
    print_row("Simetría IS/OOS (SlopeRatio)", &format!("{:>14.3}", fit_a.slope_ratio), &format!("{:>14.3}", fit_b.slope_ratio), &format!("{:>14.3}", fit_c.slope_ratio), "-");
    print_row("Ratio Ganancia / Max DD USD", &format!("{:>14.2}", fit_a.capital_dd_ratio), &format!("{:>14.2}", fit_b.capital_dd_ratio), &format!("{:>14.2}", fit_c.capital_dd_ratio), "-");

    print_header_section("── OPERATIVA Y TRANSACCIÓN ──");
    print_row("Total Swaps / Rotaciones", &format!("{}", rep_a.total_swaps), &format!("{}", rep_b.total_swaps), &format!("{}", rep_c.total_swaps), "0");
    print_row("Swaps Ganadores", &format!("{}", rep_a.profitable_swaps), &format!("{}", rep_b.profitable_swaps), &format!("{}", rep_c.profitable_swaps), "-");
    print_row("Tasa de Acierto (WinRate)", &format!("{:>14.1}%", rep_a.win_rate_swaps_pct), &format!("{:>14.1}%", rep_b.win_rate_swaps_pct), &format!("{:>14.1}%", rep_c.win_rate_swaps_pct), "-");
    print_row("Comisiones + Slippage ($)", &format!("${:>13.2}", rep_a.total_fees_paid), &format!("${:>13.2}", rep_b.total_fees_paid), &format!("${:>13.2}", rep_c.total_fees_paid), "$0.00");

    print_header_section("── DISTRIBUCIÓN DE EXPOSICIÓN ──");
    print_row("Exposición Bitcoin (BTC)", &format!("{:>14.1}%", rep_a.exposure_btc_pct), &format!("{:>14.1}%", rep_b.exposure_btc_pct), &format!("{:>14.1}%", rep_c.exposure_btc_pct), "33.3%");
    print_row("Exposición Oro (PAXG)", &format!("{:>14.1}%", rep_a.exposure_gold_pct), &format!("{:>14.1}%", rep_b.exposure_gold_pct), &format!("{:>14.1}%", rep_c.exposure_gold_pct), "33.3%");
    print_row("Exposición S&P 500 (SPY)", &format!("{:>14.1}%", rep_a.exposure_sp500_pct), &format!("{:>14.1}%", rep_b.exposure_sp500_pct), &format!("{:>14.1}%", rep_c.exposure_sp500_pct), "33.3%");
    let short_a = rep_a.exposure_short_btc_pct + rep_a.exposure_short_gold_pct + rep_a.exposure_short_sp500_pct;
    let short_b = rep_b.exposure_short_btc_pct + rep_b.exposure_short_gold_pct + rep_b.exposure_short_sp500_pct;
    let short_c = rep_c.exposure_short_btc_pct + rep_c.exposure_short_gold_pct + rep_c.exposure_short_sp500_pct;
    print_row("Exposición Posiciones Short", &format!("{:>14.1}%", short_a), &format!("{:>14.1}%", short_b), &format!("{:>14.1}%", short_c), "0.0%");
    print_row("Preservación en Dólar (CASH)", &format!("{:>14.1}%", rep_a.exposure_cash_pct), &format!("{:>14.1}%", rep_b.exposure_cash_pct), &format!("{:>14.1}%", rep_c.exposure_cash_pct), "0.0%");

    print_header_section("── GENERACIÓN DE ALPHA ──");
    print_row("Alpha vs S&P 500 B&H", &format!("{:>+14.2}%", rep_a.alpha_vs_sp500), &format!("{:>+14.2}%", rep_b.alpha_vs_sp500), &format!("{:>+14.2}%", rep_c.alpha_vs_sp500), &format!("{:>+14.2}%", rep_a.benchmark_equal_weight_return_pct - rep_a.benchmark_sp500_return_pct));
    print_row("Alpha vs Bitcoin B&H", &format!("{:>+14.2}%", rep_a.alpha_vs_btc), &format!("{:>+14.2}%", rep_b.alpha_vs_btc), &format!("{:>+14.2}%", rep_c.alpha_vs_btc), &format!("{:>+14.2}%", rep_a.benchmark_equal_weight_return_pct - rep_a.benchmark_btc_return_pct));

    println!("  \x1b[1;36m╚════════════════════════════════╧════════════════════╧════════════════════╧════════════════════╧════════════════════╝\x1b[0m");
    println!();
}

/// Ejecuta un estudio comparativo multi-apalancamiento (Leverage Sweep)
pub fn run_leverage_sweep_session(
    conn: &Connection,
    default_route: Option<&str>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let strat_choice = if let Some(r) = default_route {
        r.to_uppercase()
    } else {
        println!("\n  🎯 SELECCIONA LA ESTRATEGIA PARA EL ESTUDIO MULTI-APALANCAMIENTO:");
        println!("  [1] 🌲 GBDT Tri-Activo (Scoring & Ranking ML)");
        println!("  [2] 📈 Dual Momentum 50/50 (Rotación Dual Cross-Asset)");
        println!("  [3] ⚔️ Ambas Estrategias (Comparativa GBDT vs Dual Momentum en cada nivel)");
        print!("\n  👉 Selecciona opción (1-3) [Por defecto '2']: ");
        let _ = io::stdout().flush();
        let mut input = String::new();
        let _ = io::stdin().read_line(&mut input);
        match input.trim() {
            "1" => "A".to_string(),
            "3" => "BOTH".to_string(),
            _ => "B".to_string(),
        }
    };

    let tf = prompt_timeframe();
    let tf_clean = normalize_timeframe(&tf);
    println!("\n  🔄 Cargando dataset multiactivo sincronizado en [{}]...", tf_clean);
    let dataset = MultiAssetLoader::load_dataset(conn, &tf_clean)?;
    println!(
        "  ✅ Dataset cargado: {} barras sincronizadas (0% Look-Ahead Bias)",
        dataset.len()
    );

    print!("\n  💵 Capital Inicial para simulación en USD [Por defecto $10,000]: ");
    let _ = io::stdout().flush();
    let mut input = String::new();
    let mut initial_capital = 10_000.0;
    if io::stdin().read_line(&mut input).is_ok()
        && let Ok(val) = input.trim().parse::<f64>()
        && val > 0.0
    {
        initial_capital = val;
    }

    let margin_mode = prompt_margin_mode();
    let leverage_levels = prompt_leverage_multi(&[1.0, 2.0, 3.0, 5.0, 10.0]);

    println!("\n  ⚡ Ejecutando barrido causal de apalancamientos: {:?}...", leverage_levels);

    let start_step = if tf_clean == "1H" || tf_clean == "4H" || tf_clean.ends_with('M') {
        0
    } else {
        65.min(dataset.len().saturating_sub(1) / 4)
    };
    let total_bars = dataset.len();
    let is_split_idx = ((total_bars as f64) * 0.50).round() as usize;

    struct SweepItem {
        id: String,
        name: String,
        arch: String,
        leverage: f64,
        margin_mode: MarginMode,
        report: SwapReport,
        fitness: SwapFitnessResult,
        liquidated: bool,
        color: String,
    }

    let mut sweep_results: Vec<SweepItem> = Vec::new();

    let palette = [
        "#00E5FF", // Cyan
        "#00E676", // Verde
        "#FFEA00", // Amarillo
        "#FF9100", // Naranja
        "#FF5252", // Rojo
        "#E040FB", // Magenta
        "#7C4DFF", // Violeta
        "#69F0AE", // Menta
    ];

    let mut color_idx = 0;

    for &lev in &leverage_levels {
        let mut run_strat = |strat_type: &str, strat_label: &str| {
            let cfg = SwapConfig {
                initial_capital,
                margin_mode,
                leverage: lev,
                ..Default::default()
            };

            let sim = SwapSimulator::new(cfg);
            let rep = match strat_type {
                "A" => {
                    let mut s = SwapGbdtStrategy::default();
                    sim.run(&mut s, &dataset.bars, start_step, dataset.len())
                }
                "B" => {
                    let mut s = DualMomentumStrategy::default();
                    sim.run(&mut s, &dataset.bars, start_step, dataset.len())
                }
                _ => return,
            };

            let fit = evaluate_swap_fitness(&rep, is_split_idx);
            let liquidated = rep.trades.iter().any(|t| t.reason.contains("LIQUIDACI"))
                || rep.equity_curve.last().map(|(_, eq)| *eq <= 0.0).unwrap_or(false);

            let color = palette[color_idx % palette.len()].to_string();
            color_idx += 1;

            let id = format!("{}_{:.0}x", strat_label.to_lowercase().replace(' ', "_"), lev);
            let name = format!("{} ({:.1}X)", strat_label, lev);
            let arch = format!("{} {:.1}X", margin_mode.name(), lev);

            sweep_results.push(SweepItem {
                id,
                name,
                arch,
                leverage: lev,
                margin_mode,
                report: rep,
                fitness: fit,
                liquidated,
                color,
            });
        };

        if strat_choice == "A" || strat_choice == "BOTH" {
            run_strat("A", "GBDT");
        }
        if strat_choice == "B" || strat_choice == "BOTH" {
            run_strat("B", "Dual Momentum");
        }
    }

    if sweep_results.is_empty() {
        println!("  ⚠️ No se generaron simulaciones en el barrido.");
        pause();
        return Ok(());
    }

    println!("\n  \x1b[1;36m╔═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════╗\x1b[0m");
    println!("  \x1b[1;36m║\x1b[0m                                       \x1b[1;33m⚡ ESTUDIO COMPARATIVO MULTI-APALANCAMIENTO (LEVERAGE SWEEP) ⚡\x1b[0m                                \x1b[1;36m║\x1b[0m");
    println!("  \x1b[1;36m╠═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════╣\x1b[0m");
    println!("  \x1b[1;36m║\x1b[0m Estrategia        Nivel   Modo Margen     Retorno %     CAGR %   Sharpe  Sortino  Max DD %   Swaps  WinRate%  SWAP Fit  Estado / Alerta     \x1b[1;36m║\x1b[0m");
    println!("  \x1b[1;36m╠═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════╣\x1b[0m");

    for res in &sweep_results {
        let margin_str = match res.margin_mode {
            MarginMode::Cross => "Cruzado",
            MarginMode::Isolated => "Aislado",
        };
        let status_str = if res.liquidated {
            "💥 LIQUIDADA"
        } else if res.report.max_drawdown_pct > 50.0 {
            "⚠️ Riesgo Crítico"
        } else if res.report.max_drawdown_pct > 30.0 {
            "⚠️ Volatilidad Alta"
        } else if res.report.sharpe_ratio >= 1.5 {
            "⭐ Óptimo Sharpe"
        } else {
            "✅ Operativa Sólida"
        };

        println!(
            "  \x1b[1;36m║\x1b[0m {:<17} {:>4.1}X   {:<13} {:>+10.2}%  {:>+8.2}%  {:>6.2}   {:>6.2}  {:>7.2}%   {:>5}   {:>7.1}%  {:>7.1}   {:<19} \x1b[1;36m║\x1b[0m",
            res.name,
            res.leverage,
            margin_str,
            res.report.total_return_pct,
            res.report.cagr_pct,
            res.report.sharpe_ratio,
            res.report.sortino_ratio,
            res.report.max_drawdown_pct,
            res.report.total_swaps,
            res.report.win_rate_swaps_pct,
            res.fitness.swap_fitness_score,
            status_str
        );
    }
    println!("  \x1b[1;36m╚═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════╝\x1b[0m\n");

    if let Some(first_res) = sweep_results.first() {
        println!("  📌 Benchmarks Buy & Hold de Referencia:");
        println!("     • Bitcoin (BTC):      {:>+8.2}%", first_res.report.benchmark_btc_return_pct);
        println!("     • Oro Físico (PAXG):  {:>+8.2}%", first_res.report.benchmark_gold_return_pct);
        println!("     • S&P 500 (SPY):      {:>+8.2}%", first_res.report.benchmark_sp500_return_pct);
        println!("     • Cartera Equitativa: {:>+8.2}%\n", first_res.report.benchmark_equal_weight_return_pct);
    }

    let strat_inputs: Vec<StrategyInput> = sweep_results
        .iter()
        .map(|res| StrategyInput {
            id: res.id.clone(),
            name: res.name.clone(),
            architecture: res.arch.clone(),
            color: res.color.clone(),
            report: &res.report,
            fitness: &res.fitness,
        })
        .collect();

    let title = format!(
        "SWAP Estudio Multi-Apalancamiento (Sweep {:.0}X a {:.0}X)",
        leverage_levels.first().copied().unwrap_or(1.0),
        leverage_levels.last().copied().unwrap_or(10.0)
    );

    let payload = build_swap_dashboard_payload_with_meta(
        &dataset.bars,
        is_split_idx,
        &strat_inputs,
        &title,
        &tf_clean,
    );

    let output_dir = "dashboard";
    let file_name = "dashboard_leverage_sweep.html";
    println!("  🌐 Generando dashboard multi-apalancamiento interactivo HTML...");
    match generate_swap_dashboard_to_file(&payload, output_dir, file_name) {
        Ok(path) => {
            let abs_path = std::fs::canonicalize(&path)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| path.clone());
            println!("  \x1b[1;32m✅ Dashboard Multi-Apalancamiento HTML generado con éxito:\x1b[0m");
            println!("     • Archivo:   \x1b[1;36m{}\x1b[0m", path);
            println!("     • Enlace:    \x1b[1;36mfile:///{}\x1b[0m", abs_path.replace('\\', "/"));
        }
        Err(e) => {
            println!("  ⚠️ Error al generar dashboard de apalancamiento: {}", e);
        }
    }
    println!();

    pause();
    Ok(())
}
