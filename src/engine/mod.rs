pub mod liquidation;
pub mod model;
pub mod partition;
pub mod position;
pub mod simulator;
pub mod swap;
pub mod trade_logger;
pub mod types;

pub use liquidation::LiquidationCalculator;
pub use model::EquationModel;
pub use partition::{get_dataset_partition_indices, MANDATORY_OOS_START_TS};
pub use position::Position;
pub use simulator::BacktestSimulator;
pub use swap::{
    run_swap_intersection_optimization, AssetId,
    CandidateOptimizationResult, MultiAssetSynchronizer, SwapConfig, SwapDecision, SwapReport,
    SwapSimulator, SwapTradeRecord, SynchronizedBar, TriAssetModel,
};
pub use trade_logger::TradeLog;
pub use types::{BacktestConfig, ExitReason, SignalAction};
