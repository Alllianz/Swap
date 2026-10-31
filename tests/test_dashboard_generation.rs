use swap::dashboard::{build_swap_dashboard_payload, generate_swap_dashboard, StrategyInput};
use swap::data::db::init_candles_db;
use swap::data::loader::MultiAssetLoader;
use swap::engine::swap::types::SwapConfig;
use swap::engine::swap::SwapSimulator;
use swap::metrics::swap_fitness::evaluate_swap_fitness;
use swap::strategies::{DualMomentumStrategy, RegimeHybridStrategy, SwapGbdtStrategy};
use std::fs;

#[test]
fn test_generate_real_multi_asset_swap_dashboard() {
    let db_path = "candles.db";
    let conn = init_candles_db(db_path).expect("Error al abrir candles.db");
    let dataset = MultiAssetLoader::load_real_intersection_dataset(&conn, "1D")
        .expect("Error al cargar dataset de intersección real");

    assert!(dataset.len() >= 2000, "Dataset debe tener más de 2000 barras sincronizadas");

    let total_bars = dataset.len();
    let is_split_idx = (total_bars as f64 * 0.70) as usize;
    let initial_capital = 10_000.0;

    let sim = SwapSimulator::new(SwapConfig {
        initial_capital,
        swap_fee_rate: 0.0010,
        slippage_rate: 0.0005,
        min_holding_bars: 5,
        hysteresis_threshold: 0.015,
    });

    // 1. Simulación Ruta A: SWAP GBDT ML
    let mut strat_a = SwapGbdtStrategy::new(-0.15);
    let rep_a = sim.run(&mut strat_a, &dataset.bars, 65, total_bars);
    let fit_a = evaluate_swap_fitness(&rep_a, is_split_idx);

    // 2. Simulación Ruta B: SWAP Dual Momentum Sistemático
    let mut strat_b = DualMomentumStrategy::new(28, 84, 70);
    let rep_b = sim.run(&mut strat_b, &dataset.bars, 65, total_bars);
    let fit_b = evaluate_swap_fitness(&rep_b, is_split_idx);

    // 3. Simulación Ruta C: SWAP Macro Regime Hybrid
    let mut strat_c = RegimeHybridStrategy::new();
    let rep_c = sim.run(&mut strat_c, &dataset.bars, 65, total_bars);
    let fit_c = evaluate_swap_fitness(&rep_c, is_split_idx);

    let inputs = vec![
        StrategyInput {
            id: "swap_dual_momentum".to_string(),
            name: "SWAP Dual Momentum (S28/M84/A70)".to_string(),
            architecture: "Dual Momentum".to_string(),
            color: "#00E676".to_string(), // Emerald Green
            report: &rep_b,
            fitness: &fit_b,
        },
        StrategyInput {
            id: "swap_regime_hybrid".to_string(),
            name: "SWAP Macro Regime Hybrid".to_string(),
            architecture: "Regime Hybrid".to_string(),
            color: "#00B0FF".to_string(), // Vivid Blue
            report: &rep_c,
            fitness: &fit_c,
        },
        StrategyInput {
            id: "swap_gbdt_ml".to_string(),
            name: "SWAP GBDT Machine Learning".to_string(),
            architecture: "GBDT ML".to_string(),
            color: "#FF9100".to_string(), // Amber Orange
            report: &rep_a,
            fitness: &fit_a,
        },
    ];

    let payload = build_swap_dashboard_payload(&dataset.bars, is_split_idx, &inputs);

    assert_eq!(payload.strategies.len(), 3);
    assert_eq!(payload.benchmarks.len(), 4); // BTC, GOLD, SP500, EQUAL-WEIGHT
    assert_eq!(payload.total_bars, dataset.len());

    let out_dir = "dashboard";
    let file_path = generate_swap_dashboard(&payload, out_dir).expect("Fallo al generar HTML");

    assert!(fs::metadata(&file_path).is_ok(), "El archivo HTML debe existir");
    let content = fs::read_to_string(&file_path).expect("No se pudo leer el archivo HTML");

    // Verificaciones cuantitativas
    assert!(content.contains("SWAP QUANT ENGINE"), "Debe contener el encabezado SWAP");
    assert!(content.contains("Bitcoin (Buy & Hold)"), "Debe comparar contra BTC");
    assert!(content.contains("Oro / PAXG (Buy & Hold)"), "Debe comparar contra ORO");
    assert!(content.contains("S&P 500 (Buy & Hold)"), "Debe comparar contra SP500");
    assert!(content.contains("Cartera Equitativa (1/3 c/u)"), "Debe comparar contra Cartera Equitativa");
    assert!(content.contains("chart-curvas"), "Debe contener el canvas de curvas C1");
    assert!(content.contains("chart-drawdowns"), "Debe contener el canvas de drawdowns C2");
    assert!(content.contains("chart-asignacion"), "Debe contener el canvas de asignación de activos C3");
    assert!(content.contains("tabla-swaps"), "Debe contener la tabla de rotaciones históricas");

    println!("  ✅ Test de Generación del Dashboard Multiactivo SWAP completado con éxito: {}", file_path);
}
