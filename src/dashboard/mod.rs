pub mod components;
pub mod exporter;
pub mod generator;
pub mod models;

pub use exporter::{build_swap_dashboard_payload, StrategyInput};
pub use generator::generate_swap_dashboard;
pub use models::{
    AssetBenchmarkDto, StrategyAllocationPointDto, SwapDashboardPayload, SwapDashboardStrategyDto,
    SwapMetricsDto, SwapTradeDto, YearlyPerformanceDto,
};
