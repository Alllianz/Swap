pub mod dashboard;
pub mod data;
pub mod engine;
pub mod metrics;
pub mod strategies;
pub mod ui;

pub use dashboard::{
    build_swap_dashboard_payload, build_swap_dashboard_payload_with_meta,
    generate_optimization_dashboards, generate_swap_dashboard, generate_swap_dashboard_to_file,
};
pub use data::{
    get_all_klines_closed_only, init_candles_db, Kline, MultiAssetDataset, MultiAssetLoader,
};
pub use engine::{
    AssetId, CandidateOptimizationResult, MultiAssetSynchronizer, SwapConfig, SwapDecision,
    SwapReport, SwapSimulator, SwapTradeRecord, SynchronizedBar, TriAssetModel,
};
pub use strategies::{DualMomentumStrategy, RegimeHybridStrategy, SwapGbdtStrategy};
