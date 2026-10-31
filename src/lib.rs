pub mod dashboard;
pub mod data;
pub mod engine;
pub mod features;
pub mod metrics;
pub mod strategies;
pub mod ui;

pub use dashboard::{build_swap_dashboard_payload, generate_swap_dashboard};
pub use data::{
    get_all_klines_closed_only, init_candles_db, Kline, MultiAssetDataset, MultiAssetLoader,
};
pub use engine::{
    AssetId, BacktestConfig, BacktestSimulator, EquationModel, LiquidationCalculator,
    MultiAssetSynchronizer, SignalAction, SwapConfig, SwapDecision, SwapReport, SwapSimulator,
    SwapTradeRecord, SynchronizedBar, TriAssetModel,
};
pub use metrics::{calculate_metrics, BacktestReport};
pub use strategies::{DualMomentumStrategy, RegimeHybridStrategy, SwapGbdtStrategy};
