pub mod header;
pub mod prompts;
pub mod session;

pub use header::print_swap_header;
pub use prompts::{pause, prompt_swap_config, prompt_timeframe};
pub use session::{
    run_comparative_leaderboard, run_download_menu, run_optimization_session,
    run_single_swap_strategy,
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
        println!("  [3] 🌲 Simular RUTA A: SWAP GBDT Tri-Activo (Scoring & Ranking ML)");
        println!("  [4] 📈 Simular RUTA B: Dual Momentum Sistemático (Relative + Absolute + Vol-Target)");
        println!("  [5] 🧬 Simular RUTA C: Híbrido de Regímenes Macroeconómicos (Risk-On / Growth / Defensive)");
        println!();
        println!("  ── 🏆 BENCHMARKING Y OPTIMIZACIÓN INSTITUCIONAL SWAP ──");
        println!("  [6] 🏆 Leaderboard Comparativo Simultáneo (A vs B vs C vs Benchmarks Buy & Hold)");
        println!("  [7] 🧬 Auto-Optimización SWAP EVO Max (Métrica SWAP Fitness en Intersección Real)");
        println!();
        println!("  [0] 🚪 Salir");
        print!("\n  👉 Selecciona una opción (0-7) [Por defecto '7']: ");
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
                run_single_swap_strategy(&conn, "C")?;
            }
            "6" => {
                run_comparative_leaderboard(&conn)?;
            }
            "7" | "" => {
                run_optimization_session(&conn)?;
            }
            _ => {
                println!("  ⚠️ Opción no válida.");
                pause();
            }
        }
    }

    Ok(())
}
