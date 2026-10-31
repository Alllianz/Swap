use crate::data::downloader::{
    download_all_historical_binance_klines, download_yahoo_finance_klines,
    insert_multi_asset_klines,
};
use crate::data::loader::MultiAssetLoader;
use crate::engine::swap::optimizer::run_swap_intersection_optimization;
use crate::engine::swap::report::SwapReport;
use crate::engine::swap::simulator::SwapSimulator;
use crate::strategies::dual_momentum::DualMomentumStrategy;
use crate::strategies::regime_hybrid::RegimeHybridStrategy;
use crate::strategies::swap_gbdt::SwapGbdtStrategy;
use crate::ui::prompts::{pause, prompt_swap_config, prompt_timeframe};
use reqwest::Client;
use rusqlite::Connection;
use std::error::Error;
use std::io::{self, Write};

/// Ejecuta la simulación individual de una de las tres rutas de swap
pub fn run_single_swap_strategy(
    conn: &Connection,
    route: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let tf = prompt_timeframe();
    println!("\n  🔄 Cargando y sincronizando dataset multiactivo (BTC, ORO, SP500)...");
    let dataset = MultiAssetLoader::load_dataset(conn, &tf)?;

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

    println!("\n  🚀 Ejecutando simulación causal...");
    let report: SwapReport = match route {
        "A" => {
            let mut strategy = SwapGbdtStrategy::default();
            sim.run(&mut strategy, &dataset.bars, 65, dataset.len())
        }
        "B" => {
            let mut strategy = DualMomentumStrategy::default();
            sim.run(&mut strategy, &dataset.bars, 65, dataset.len())
        }
        "C" => {
            let mut strategy = RegimeHybridStrategy::default();
            sim.run(&mut strategy, &dataset.bars, 65, dataset.len())
        }
        _ => return Err("Ruta no reconocida".into()),
    };

    println!("\n  📊 RESULTADOS CUANTITATIVOS:");
    report.print_summary_card();

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

/// Ejecuta la comparativa simultánea de las 3 arquitecturas (A vs B vs C vs Benchmarks)
pub fn run_comparative_leaderboard(conn: &Connection) -> Result<(), Box<dyn Error + Send + Sync>> {
    let tf = prompt_timeframe();
    println!("\n  🔄 Cargando y sincronizando dataset multiactivo...");
    let dataset = MultiAssetLoader::load_dataset(conn, &tf)?;

    let config = prompt_swap_config();
    let sim = SwapSimulator::new(config);

    println!("\n  ⚡ Evaluando las 3 arquitecturas simultáneamente sobre el mismo histórico causal...");

    let mut strat_a = SwapGbdtStrategy::default();
    let rep_a = sim.run(&mut strat_a, &dataset.bars, 65, dataset.len());

    let mut strat_b = DualMomentumStrategy::default();
    let rep_b = sim.run(&mut strat_b, &dataset.bars, 65, dataset.len());

    let mut strat_c = RegimeHybridStrategy::default();
    let rep_c = sim.run(&mut strat_c, &dataset.bars, 65, dataset.len());

    let mut leaderboard = vec![
        ("Ruta A: SWAP GBDT Tri-Activo (ML)", rep_a),
        ("Ruta B: SWAP Dual Momentum", rep_b),
        ("Ruta C: SWAP Híbrido de Regímenes", rep_c),
    ];

    // Ordenar por Sharpe Ratio descendente
    leaderboard.sort_by(|a, b| {
        b.1.sharpe_ratio
            .partial_cmp(&a.1.sharpe_ratio)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    println!();
    println!("  \x1b[1;36m╔══════════════════════════════════════════════════════════════════════════════════════════════════════════╗\x1b[0m");
    println!("  \x1b[1;36m║\x1b[0m                  \x1b[1;33m🏆 LEADERBOARD COMPARATIVO — ARQUITECTURAS DE SWAP TRI-ACTIVO 🏆\x1b[0m                 \x1b[1;36m║\x1b[0m");
    println!("  \x1b[1;36m╠══════════════════════════════════════════════════════════════════════════════════════════════════════════╣\x1b[0m");
    println!("  \x1b[1;36m║\x1b[0m  Puesto  Estrategia                        Retorno %    CAGR %   Sharpe   Max DD %   Swaps   WinRate %  \x1b[1;36m║\x1b[0m");
    println!("  \x1b[1;36m╠══════════════════════════════════════════════════════════════════════════════════════════════════════════╣\x1b[0m");

    for (rank, (name, rep)) in leaderboard.iter().enumerate() {
        let medal = match rank {
            0 => "🥇 1°",
            1 => "🥈 2°",
            2 => "🥉 3°",
            _ => "   ",
        };
        println!(
            "  \x1b[1;36m║\x1b[0m  {:<4}  {:<32} {:>+10.2}% {:>8.2}%  {:>6.2}   {:>7.2}%   {:>5}   {:>7.1}%  \x1b[1;36m║\x1b[0m",
            medal,
            name,
            rep.total_return_pct,
            rep.cagr_pct,
            rep.sharpe_ratio,
            rep.max_drawdown_pct,
            rep.total_swaps,
            rep.win_rate_swaps_pct
        );
    }

    println!("  \x1b[1;36m╠══════════════════════════════════════════════════════════════════════════════════════════════════════════╣\x1b[0m");
    println!("  \x1b[1;36m║\x1b[0m  BENCHMARKS PASIVOS DE REFERENCIA (BUY & HOLD):                                                          \x1b[1;36m║\x1b[0m");
    if let Some((_, ref_rep)) = leaderboard.first() {
        println!(
            "  \x1b[1;36m║\x1b[0m   • B&H Bitcoin: {:>+8.2}%  |  • B&H Oro: {:>+8.2}%  |  • B&H S&P 500: {:>+8.2}%                 \x1b[1;36m║\x1b[0m",
            ref_rep.benchmark_btc_return_pct,
            ref_rep.benchmark_gold_return_pct,
            ref_rep.benchmark_sp500_return_pct
        );
        println!(
            "  \x1b[1;36m║\x1b[0m   • Cartera Equitativa (1/3 c/u): {:>+8.2}%                                                       \x1b[1;36m║\x1b[0m",
            ref_rep.benchmark_equal_weight_return_pct
        );
    }
    println!("  \x1b[1;36m╚══════════════════════════════════════════════════════════════════════════════════════════════════════════╝\x1b[0m");
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
    println!("  [3] 📈 Descargar Historial Real de SPY (S&P 500 ETF / Yahoo Finance 10 Años)");
    println!("  [0] Volver al Menú Principal");
    print!("\n  👉 Selecciona un activo (0-3) [Por defecto '3']: ");
    let _ = io::stdout().flush();

    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);

    match input.trim() {
        "1" => {
            let tf = prompt_timeframe();
            println!("\n  ⚙️ Modo de Descarga para PAXGUSDT [{}]:", tf);
            println!("  [1] 🌟 Completa desde el Génesis (Descarga TODOS los lotes históricos)");
            println!("  [2] 🔄 Incremental (Desde la última vela registrada)");
            print!("\n  👉 Selecciona modo (1-2) [Por defecto '1']: ");
            let _ = io::stdout().flush();
            input.clear();
            let _ = io::stdin().read_line(&mut input);
            let from_scratch = !input.trim().eq("2");

            let total = download_all_historical_binance_klines(conn, client, "PAXGUSDT", &tf, from_scratch).await?;
            println!("\n  ✅ ¡Proceso finalizado! Total de velas de Oro guardadas: {}", total);
        }
        "2" => {
            let tf = prompt_timeframe();
            println!("\n  ⚙️ Modo de Descarga para BTCUSDT [{}]:", tf);
            println!("  [1] 🌟 Completa desde el Génesis");
            println!("  [2] 🔄 Incremental");
            print!("\n  👉 Selecciona modo (1-2) [Por defecto '1']: ");
            let _ = io::stdout().flush();
            input.clear();
            let _ = io::stdin().read_line(&mut input);
            let from_scratch = !input.trim().eq("2");

            let total = download_all_historical_binance_klines(conn, client, "BTCUSDT", &tf, from_scratch).await?;
            println!("\n  ✅ ¡Proceso finalizado! Total de velas de BTC guardadas: {}", total);
        }
        "3" | "" => {
            println!("\n  🌐 Conectando a Yahoo Finance API para SPY (S&P 500 10 Años)...");
            let klines = download_yahoo_finance_klines(client, "SPY", "10y", "1d").await?;
            println!("  📥 Recibidas {} velas diarias reales del S&P 500.", klines.len());
            if let (Some(first), Some(last)) = (klines.first(), klines.last()) {
                println!("     Rango temporal SPY: {} → {}", first.date_utc_string(), last.date_utc_string());
            }

            let inserted = insert_multi_asset_klines(conn, "SP500", "1D", &klines)?;
            println!("  💾 Almacenadas {} velas de SP500 en candles.db.", inserted);
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
    println!("  {:<4} | {:<10} | {:<18} | {:<12} | {:<12} | {:<9} | {}",
        "N°", "Fecha", "Rotación", "Cap Antes", "Cap Después", "Comisión", "Razón"
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
    println!("\n  🔄 Cargando dataset de INTERSECCIÓN REAL (BTC, PAXG, SP500)...");
    let dataset = MultiAssetLoader::load_real_intersection_dataset(conn, &tf)?;

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
    if let Ok(_) = io::stdin().read_line(&mut input) {
        if let Ok(val) = input.trim().parse::<f64>() {
            if val > 0.0 {
                initial_capital = val;
            }
        }
    }

    println!("\n  🧬 Iniciando Auto-Optimización Institucional SWAP EVO...");
    println!("     Métrica: SWAP Fitness (50% Cap/DD + 25% R² Compuesto + 25% Slope IS/OOS)");
    println!("     Ventana de Optimización: 70% In-Sample / 30% Out-Of-Sample Causal\n");

    let results = run_swap_intersection_optimization(&dataset.bars, initial_capital);

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

    for (idx, res) in results.iter().take(10).enumerate() {
        let medal = match idx {
            0 => "🥇 1°",
            1 => "🥈 2°",
            2 => "🥉 3°",
            _ => "   ",
        };
        let swap_rank = res.rank_score.as_ref().map(|r| r.swap_rank_fitness).unwrap_or(0.0);
        println!(
            "  \x1b[1;36m║\x1b[0m  {:<4}  {:<21}  {:<16}  {:>7.2}%   {:>8.2}  {:>6.2}   {:>6.3}    {:>6.3}    {:>6.2}   {:>6.2}% \x1b[1;36m║\x1b[0m",
            medal,
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

    pause();
    Ok(())
}
