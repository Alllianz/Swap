use reqwest::Client;
use swap::data::db::init_candles_db;
use swap::data::downloader::download_all_historical_binance_klines;
use swap::data::kline::Kline;
use swap::data::loader::MultiAssetLoader;
use swap::engine::swap::simulator::SwapSimulator;
use swap::engine::swap::synchronizer::MultiAssetSynchronizer;
use swap::engine::swap::types::{MarginMode, SwapConfig};
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
        margin_mode: MarginMode::Cross,
        leverage: 1.0,
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

#[test]
fn test_hourly_synchronization_and_backtest_from_zero() {
    // Generar exactamente 720 velas sintéticas de 1H (~30 días continuos)
    let mut btc_klines = Vec::with_capacity(720);
    let mut gold_klines = Vec::with_capacity(720);
    let mut sp500_klines = Vec::with_capacity(720);

    let start_ts = 1700000000000i64; // Unix epoch ms
    let hour_ms = 3_600_000i64;

    for i in 0..720 {
        let ts = start_ts + (i as i64) * hour_ms;

        btc_klines.push(Kline {
            timestamp: ts,
            open: 35000.0 + (i as f64) * 10.0,
            high: 35100.0 + (i as f64) * 10.0,
            low: 34900.0 + (i as f64) * 10.0,
            close: 35050.0 + (i as f64) * 10.0,
            volume: 150.0,
        });

        if (i % 24) >= 13 && (i % 24) <= 20 {
            gold_klines.push(Kline {
                timestamp: ts,
                open: 1980.0 + (i as f64) * 0.5,
                high: 1985.0 + (i as f64) * 0.5,
                low: 1975.0 + (i as f64) * 0.5,
                close: 1982.0 + (i as f64) * 0.5,
                volume: 50.0,
            });

            sp500_klines.push(Kline {
                timestamp: ts,
                open: 440.0 + (i as f64) * 0.1,
                high: 442.0 + (i as f64) * 0.1,
                low: 439.0 + (i as f64) * 0.1,
                close: 441.0 + (i as f64) * 0.1,
                volume: 50000.0,
            });
        }
    }

    // Sincronizar en temporalidad 1H
    let synchronized = MultiAssetSynchronizer::synchronize("1H", &btc_klines, &gold_klines, &sp500_klines);
    assert_eq!(synchronized.len(), 720, "Deben existir exactamente 720 barras sincronizadas de 1H");

    let config = SwapConfig {
        initial_capital: 10_000.0,
        swap_fee_rate: 0.0010,
        slippage_rate: 0.0005,
        min_holding_bars: 5,
        hysteresis_threshold: 0.015,
        margin_mode: MarginMode::Cross,
        leverage: 1.0,
    };
    let sim = SwapSimulator::new(config);

    let mut dm_strategy = DualMomentumStrategy::new(14, 45, 40);
    let report = sim.run(&mut dm_strategy, &synchronized, 0, 720);

    assert_eq!(report.equity_curve.first().unwrap().0, synchronized[0].timestamp, "El backtest debe iniciar en la vela 0");
    assert_eq!(report.equity_curve.len(), 720, "La curva de equity debe contener exactamente las 720 velas de 1H");
    assert!(!report.final_capital.is_nan());
    assert!(report.final_capital > 0.0);
    println!("✅ Backtest de 1H verificado exitosamente desde vela 0 hasta 720 (Capital Final: ${:.2})", report.final_capital);
}

#[test]
fn test_4h_synchronization_and_backtest_from_zero() {
    // Generar exactamente 720 velas sintéticas de 4H (~120 días continuos)
    let mut btc_klines = Vec::with_capacity(720);
    let mut gold_klines = Vec::with_capacity(720);
    let mut sp500_klines = Vec::with_capacity(720);

    let start_ts = 1700000000000i64;
    let four_hour_ms = 4 * 3_600_000i64; // 14,400,000 ms

    for i in 0..720 {
        let ts = start_ts + (i as i64) * four_hour_ms;

        btc_klines.push(Kline {
            timestamp: ts,
            open: 36000.0 + (i as f64) * 20.0,
            high: 36200.0 + (i as f64) * 20.0,
            low: 35800.0 + (i as f64) * 20.0,
            close: 36100.0 + (i as f64) * 20.0,
            volume: 500.0,
        });

        // Velas de Oro y SP500 durante sus bloques horarios hábiles
        if (i % 6) == 3 || (i % 6) == 4 {
            gold_klines.push(Kline {
                timestamp: ts,
                open: 2000.0 + (i as f64) * 1.0,
                high: 2010.0 + (i as f64) * 1.0,
                low: 1990.0 + (i as f64) * 1.0,
                close: 2005.0 + (i as f64) * 1.0,
                volume: 120.0,
            });

            sp500_klines.push(Kline {
                timestamp: ts,
                open: 450.0 + (i as f64) * 0.2,
                high: 453.0 + (i as f64) * 0.2,
                low: 448.0 + (i as f64) * 0.2,
                close: 451.0 + (i as f64) * 0.2,
                volume: 150000.0,
            });
        }
    }

    // Sincronizar en temporalidad 4H
    let synchronized = MultiAssetSynchronizer::synchronize("4H", &btc_klines, &gold_klines, &sp500_klines);
    assert_eq!(synchronized.len(), 720, "Deben existir exactamente 720 barras sincronizadas de 4H");

    let config = SwapConfig {
        initial_capital: 10_000.0,
        swap_fee_rate: 0.0010,
        slippage_rate: 0.0005,
        min_holding_bars: 5,
        hysteresis_threshold: 0.015,
        margin_mode: MarginMode::Cross,
        leverage: 1.0,
    };
    let sim = SwapSimulator::new(config);

    let mut dm_strategy = DualMomentumStrategy::new(14, 45, 40);
    let report = sim.run(&mut dm_strategy, &synchronized, 0, 720);

    assert_eq!(report.equity_curve.first().unwrap().0, synchronized[0].timestamp, "El backtest de 4H debe iniciar en la vela 0");
    assert_eq!(report.equity_curve.len(), 720, "La curva de equity debe contener exactamente las 720 velas de 4H");
    assert!(!report.final_capital.is_nan());
    assert!(report.final_capital > 0.0);
    println!("✅ Backtest de 4H verificado exitosamente desde vela 0 hasta 720 (Capital Final: ${:.2})", report.final_capital);
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

    let results = swap::engine::swap::run_swap_intersection_optimization(&dataset.bars, 10_000.0, 5.0);
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

#[test]
fn test_short_assets_math_and_execution() {
    use swap::engine::swap::model::{SwapDecision, TriAssetModel};
    use swap::engine::swap::types::{AssetId, SynchronizedBar};

    // 1. Verificación de propiedades de tipos AssetId
    assert!(AssetId::ShortBtc.is_short());
    assert!(AssetId::ShortGold.is_short());
    assert!(AssetId::ShortSp500.is_short());
    assert!(!AssetId::Btc.is_short());
    assert!(!AssetId::Cash.is_short());

    assert_eq!(AssetId::Btc.to_short(), Some(AssetId::ShortBtc));
    assert_eq!(AssetId::Gold.to_short(), Some(AssetId::ShortGold));
    assert_eq!(AssetId::Sp500.to_short(), Some(AssetId::ShortSp500));
    assert_eq!(AssetId::Cash.to_short(), None);
    assert_eq!(AssetId::ShortBtc.to_long(), Some(AssetId::Btc));
    assert_eq!(AssetId::ShortGold.to_long(), Some(AssetId::Gold));
    assert_eq!(AssetId::ShortSp500.to_long(), Some(AssetId::Sp500));
    assert_eq!(AssetId::Cash.to_long(), Some(AssetId::Cash));

    // 2. Crear una serie sintética donde el precio de BTC cae un 10% por barra
    // Día 0: 100.0, Día 1: 90.0 (-10%), Día 2: 81.0 (-10%)
    let mut bars = Vec::new();
    let p0 = 100.0;
    let p1 = 90.0;
    let p2 = 81.0;

    bars.push(SynchronizedBar {
        timestamp: 1600000000000,
        btc_open: p0,
        btc_high: p0,
        btc_low: p0,
        btc_close: p0,
        btc_volume: 1.0,
        gold_open: 2000.0,
        gold_high: 2000.0,
        gold_low: 2000.0,
        gold_close: 2000.0,
        gold_volume: 1.0,
        sp500_open: 400.0,
        sp500_high: 400.0,
        sp500_low: 400.0,
        sp500_close: 400.0,
        sp500_volume: 1.0,
    });

    bars.push(SynchronizedBar {
        timestamp: 1600000000000 + 86400000,
        btc_open: p0,
        btc_high: p0,
        btc_low: p1,
        btc_close: p1,
        btc_volume: 1.0,
        gold_open: 2000.0,
        gold_high: 2000.0,
        gold_low: 2000.0,
        gold_close: 2000.0,
        gold_volume: 1.0,
        sp500_open: 400.0,
        sp500_high: 400.0,
        sp500_low: 400.0,
        sp500_close: 400.0,
        sp500_volume: 1.0,
    });

    bars.push(SynchronizedBar {
        timestamp: 1600000000000 + 86400000 * 2,
        btc_open: p1,
        btc_high: p1,
        btc_low: p2,
        btc_close: p2,
        btc_volume: 1.0,
        gold_open: 2000.0,
        gold_high: 2000.0,
        gold_low: 2000.0,
        gold_close: 2000.0,
        gold_volume: 1.0,
        sp500_open: 400.0,
        sp500_high: 400.0,
        sp500_low: 400.0,
        sp500_close: 400.0,
        sp500_volume: 1.0,
    });

    // Estrategia sintética que abre Short BTC inmediatamente
    struct AlwaysShortBtcStrategy;
    impl TriAssetModel for AlwaysShortBtcStrategy {
        fn name(&self) -> &str { "AlwaysShortBtc" }
        fn evaluate(&mut self, _history: &[SynchronizedBar], _current_step: usize) -> SwapDecision {
            SwapDecision {
                target_asset: AssetId::ShortBtc,
                secondary_asset: None,
                confidence_score: 1.0,
                explanation: "Tactical Short BTC".to_string(),
            }
        }
    }

    let config = SwapConfig {
        initial_capital: 10_000.0,
        swap_fee_rate: 0.0,     // 0 fees para validar matemática pura
        slippage_rate: 0.0,     // 0 slippage
        min_holding_bars: 1,
        hysteresis_threshold: 0.0,
        margin_mode: MarginMode::Cross,
        leverage: 1.0,
    };

    let sim = SwapSimulator::new(config);
    let mut strat = AlwaysShortBtcStrategy;
    let rep = sim.run(&mut strat, &bars, 0, bars.len());

    // Al estar short en un activo que cae -10% y luego -10%:
    // Retorno día 1: -(90/100 - 1) = +10% -> Capital = $11,000
    // Retorno día 2: -(81/90 - 1) = +10% -> Capital = $12,100
    println!("Final capital short BTC: ${:.2}", rep.final_capital);
    assert!((rep.final_capital - 12_100.0).abs() < 1e-4, "Capital final debe ser $12,100 tras dos caídas del 10% en BTC");
    assert_eq!(rep.trades.len(), 1);
    assert_eq!(rep.trades[0].to_asset, AssetId::ShortBtc);
    assert!(rep.exposure_short_btc_pct > 0.0);
    assert_eq!(rep.exposure_btc_pct, 0.0);

    // 3. Verificación de Short SP500 y Short Gold
    struct AlwaysShortSp500Strategy;
    impl TriAssetModel for AlwaysShortSp500Strategy {
        fn name(&self) -> &str { "AlwaysShortSp500" }
        fn evaluate(&mut self, _history: &[SynchronizedBar], _current_step: usize) -> SwapDecision {
            SwapDecision {
                target_asset: AssetId::ShortSp500,
                secondary_asset: None,
                confidence_score: 1.0,
                explanation: "Tactical Short SP500".to_string(),
            }
        }
    }
    let mut strat_sp = AlwaysShortSp500Strategy;
    let rep_sp = sim.run(&mut strat_sp, &bars, 0, bars.len());
    assert_eq!(rep_sp.trades[0].to_asset, AssetId::ShortSp500);
    assert!(rep_sp.exposure_short_sp500_pct > 0.0);

    struct AlwaysShortGoldStrategy;
    impl TriAssetModel for AlwaysShortGoldStrategy {
        fn name(&self) -> &str { "AlwaysShortGold" }
        fn evaluate(&mut self, _history: &[SynchronizedBar], _current_step: usize) -> SwapDecision {
            SwapDecision {
                target_asset: AssetId::ShortGold,
                secondary_asset: None,
                confidence_score: 1.0,
                explanation: "Tactical Short Gold".to_string(),
            }
        }
    }
    let mut strat_gold = AlwaysShortGoldStrategy;
    let rep_gold = sim.run(&mut strat_gold, &bars, 0, bars.len());
    assert_eq!(rep_gold.trades[0].to_asset, AssetId::ShortGold);
    assert!(rep_gold.exposure_short_gold_pct > 0.0);
}
