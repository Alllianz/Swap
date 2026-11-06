pub mod header;
pub mod prompts;
pub mod session;

pub use header::print_swap_header;
pub use prompts::{pause, prompt_swap_config, prompt_timeframe};
pub use session::{
    run_comparative_leaderboard, run_download_menu, run_leverage_sweep_session,
    run_optimization_session, run_ruta_b_population_session, run_single_swap_strategy,
};

use crate::data::db::{init_candles_db, print_db_summary};
use reqwest::Client;
use std::error::Error;
use std::io::{self, Write};

pub async fn run_interactive_app() -> Result<(), Box<dyn Error + Send + Sync>> {
    let db_path = "candles.db";
    let mut conn = init_candles_db(db_path)?;
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap_or_else(|_| Client::new());

    loop {
        print_swap_header();
        println!("  ── 📊 GESTIÓN DE DATOS MULTIACTIVO ──");
        println!("  [1] 📊 Ver Resumen de Velas en Base de Datos (candles.db)");
        println!("  [2] 📥 Descargar/Actualizar Velas (PAXG / BTC en Binance & SPY en Yahoo)");
        println!();
        println!("  ── 🚀 ARQUITECTURAS DE SWAP TRI-ACTIVO (BTC | ORO | SP500) ──");
        println!("  [3] 🌲 Simular GBDT Tri-Activo (Scoring & Ranking ML)");
        println!("  [4] 📈 Simular Dual Momentum 50/50 (Individuos y Población Cross-Asset)");
        println!();
        println!("  ── 🏆 BENCHMARKING Y OPTIMIZACIÓN INSTITUCIONAL SWAP ──");
        println!("  [5] ⚔️ Comparativa Mano a Mano (2 o 3 Estrategias vs Benchmarks Buy & Hold)");
        println!("  [6] 🧬 Auto-Optimización SWAP EVO Max (Métrica SWAP Fitness en Intersección Real)");
        println!("  [7] 📈 Población Exclusiva Dual Momentum (18 Individuos 50/50 en Dashboard)");
        println!("  [8] ⚡ Estudio Multi-Apalancamiento (Sweep 1X a 10X en Consola y Dashboard)");
        println!();
        println!("  [0] 🚪 Salir");
        print!("\n  👉 Selecciona una opción (0-8) [Por defecto '8']: ");
        let _ = io::stdout().flush();

        let mut input = String::new();
        let _ = io::stdin().read_line(&mut input);
        let choice = input.trim();

        match choice {
            "0" => {
                println!("\n  👋 Finalizando sesión en Swap Quant Engine. ¡Hasta pronto, Allianz!\n");
                break;
            }
            "1" => {
                print_db_summary(&conn)?;
                pause();
            }
            "2" => {
                run_download_menu(&mut conn, &client).await?;
            }
            "3" => {
                run_single_swap_strategy(&conn, "A")?;
            }
            "4" => {
                run_single_swap_strategy(&conn, "B")?;
            }
            "5" => {
                run_comparative_leaderboard(&conn)?;
            }
            "6" => {
                run_optimization_session(&conn)?;
            }
            "7" => {
                run_ruta_b_population_session(&conn)?;
            }
            "8" | "" => {
                run_leverage_sweep_session(&conn, None)?;
            }
            _ => {
                println!("  ⚠️ Opción no válida.");
                pause();
            }
        }
    }

    Ok(())
}
