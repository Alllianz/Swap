pub mod components;
pub mod exporter;
pub mod generator;
pub mod models;

pub use exporter::{
    build_swap_dashboard_payload, build_swap_dashboard_payload_with_meta, StrategyInput,
};
pub use generator::{
    generate_optimization_dashboards, generate_swap_dashboard, generate_swap_dashboard_to_file,
};
pub use models::{
    AssetBenchmarkDto, StrategyAllocationPointDto, SwapDashboardPayload, SwapDashboardStrategyDto,
    SwapMetricsDto, SwapTradeDto, YearlyPerformanceDto,
};
