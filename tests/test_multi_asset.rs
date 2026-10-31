use reqwest::Client;
use swap::data::db::init_candles_db;
use swap::data::downloader::download_all_historical_binance_klines;
use swap::data::loader::MultiAssetLoader;
use swap::engine::swap::simulator::SwapSimulator;
use swap::engine::swap::types::SwapConfig;
use swap::strategies::dual_momentum::DualMomentumStrategy;
use swap::strategies::regime_hybrid::RegimeHybridStrategy;
use swap::strategies::swap_gbdt::SwapGbdtStrategy;

#[test]
fn test_inspect_btc_and_simulate_all_routes() {
    let conn = init_candles_db("candles.db").expect("candles.db debe abrir");
    let dataset = MultiAssetLoader::load_dataset(&conn, "1D")
        .expect("debe cargar dataset multiactivo sincronizado");

    assert!(dataset.len() >= 500, "Debe haber al menos 500 barras de datos");
    println!("Barras multiactivo sincronizadas: {}", dataset.len());

    let config = SwapConfig {
        initial_capital: 10_000.0,
        swap_fee_rate: 0.0010,
        slippage_rate: 0.0005,
        min_holding_bars: 5,
        hysteresis_threshold: 0.015,
    };
    let sim = SwapSimulator::new(config);

    // 1. Simulación Ruta A (SWAP GBDT Tri-Activo)
    let mut strat_a = SwapGbdtStrategy::default();
    let rep_a = sim.run(&mut strat_a, &dataset.bars, 65, dataset.len());
    println!("\n=== REPORTE RUTA A (SWAP GBDT) ===");
    rep_a.print_summary_card();
    assert!(rep_a.final_capital > 0.0);
    assert!(!rep_a.sharpe_ratio.is_nan());

    // 2. Simulación Ruta B (Dual Momentum Sistemático)
    let mut strat_b = DualMomentumStrategy::default();
    let rep_b = sim.run(&mut strat_b, &dataset.bars, 65, dataset.len());
    println!("\n=== REPORTE RUTA B (Dual Momentum) ===");
    rep_b.print_summary_card();
    assert!(rep_b.final_capital > 0.0);
    assert!(!rep_b.sharpe_ratio.is_nan());

    // 3. Simulación Ruta C (Híbrido de Regímenes Macroeconómicos SWAP)
    let mut strat_c = RegimeHybridStrategy::default();
    let rep_c = sim.run(&mut strat_c, &dataset.bars, 65, dataset.len());
    println!("\n=== REPORTE RUTA C (Híbrido de Regímenes) ===");
    rep_c.print_summary_card();
    assert!(rep_c.final_capital > 0.0);
    assert!(!rep_c.sharpe_ratio.is_nan());

    // Verificación de consistencia causal
    assert_eq!(rep_a.initial_capital, 10_000.0);
    assert_eq!(rep_b.initial_capital, 10_000.0);
    assert_eq!(rep_c.initial_capital, 10_000.0);
}

#[tokio::test]
async fn test_download_all_historical_paxg() {
    let mut conn = init_candles_db("candles.db").expect("candles.db debe abrir");
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap();

    let count = download_all_historical_binance_klines(&mut conn, &client, "PAXGUSDT", "1D", true)
        .await
        .expect("debe descargar el historial completo de PAXGUSDT");

    println!("Total velas PAXGUSDT descargadas: {}", count);
    assert!(count > 1000, "Debe descargar más de 1000 velas de PAXGUSDT (actualmente cotiza desde 2020)");
}

#[tokio::test]
async fn test_download_real_spy_data() {
    let mut conn = init_candles_db("candles.db").expect("candles.db debe abrir");
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .unwrap();

    let klines = swap::data::downloader::download_yahoo_finance_klines(&client, "SPY", "10y", "1d")
        .await
        .expect("debe descargar klines reales de SPY desde Yahoo Finance");

    println!("Total velas reales descargadas de SPY (10y): {}", klines.len());
    assert!(klines.len() > 2000, "Debe haber más de 2000 velas en 10 años de SPY");

    let inserted = swap::data::downloader::insert_multi_asset_klines(&mut conn, "SP500", "1D", &klines)
        .expect("debe insertar SP500 en candles.db");
    println!("Total velas de SP500 guardadas en candles.db: {}", inserted);
    assert_eq!(inserted, klines.len());
}

#[test]
fn test_swap_intersection_optimization() {
    let conn = init_candles_db("candles.db").expect("candles.db debe abrir");
    let dataset = MultiAssetLoader::load_real_intersection_dataset(&conn, "1D")
        .expect("debe cargar la intersección real de los 3 activos");

    assert!(dataset.len() >= 2000, "Debe haber más de 2000 barras en la intersección real");
    let first = dataset.bars.first().unwrap();
    let last = dataset.bars.last().unwrap();
    println!("Intersección real 3 activos: {} barras | Rango: {} → {}", dataset.len(), first.date_string(), last.date_string());
    assert_eq!(first.date_string(), "2020-08-28", "Debe iniciar exactamente el 28 de agosto de 2020");

    let results = swap::engine::swap::run_swap_intersection_optimization(&dataset.bars, 10_000.0);
    assert!(!results.is_empty(), "Debe retornar candidatos evaluados");

    let winner = &results[0];
    println!("\n=== GANADOR DE OPTIMIZACIÓN SWAP EVO ===");
    println!("ID: {}", winner.candidate_id);
    println!("Arquitectura: {}", winner.architecture);
    println!("SWAP Fitness: {:.2} puntos", winner.swap_fitness.swap_fitness_score);
    println!("Capital/DD Ratio: {:.2}", winner.swap_fitness.capital_dd_ratio);
    println!("R² Compuesto: {:.3}", winner.swap_fitness.composite_linearity);
    println!("Slope Ratio: {:.3}", winner.swap_fitness.slope_ratio);
    println!("Retorno: {:+.2}%", winner.report.total_return_pct);
    println!("Sharpe: {:.2}", winner.report.sharpe_ratio);
    println!("Max DD: {:.2}%", winner.report.max_drawdown_pct);

    assert!(winner.swap_fitness.swap_fitness_score > 0.0);
}
