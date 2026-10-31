use serde::{Deserialize, Serialize};

/// Punto de asignación de activos en una barra temporal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyAllocationPointDto {
    pub timestamp: i64,
    pub date: String,
    pub asset: String,      // "BTC", "GOLD", "SP500", "CASH"
    pub equity: f64,
    pub drawdown_pct: f64,
}

/// Registro de una rotación o swap de activos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapTradeDto {
    pub swap_index: usize,
    pub timestamp: i64,
    pub date: String,
    pub from_asset: String,
    pub to_asset: String,
    pub exit_price: f64,
    pub entry_price: f64,
    pub capital_before: f64,
    pub capital_after: f64,
    pub pnl_pct: f64,
    pub fee_paid: f64,
    pub holding_bars: usize,
    pub reason: String,
}

/// Rendimiento anual comparativo de una estrategia frente a los 3 activos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YearlyPerformanceDto {
    pub year: i32,
    pub strategy_return_pct: f64,
    pub btc_return_pct: f64,
    pub gold_return_pct: f64,
    pub sp500_return_pct: f64,
    pub diff_vs_btc: f64,
    pub diff_vs_sp500: f64,
    pub beat_btc: bool,
    pub beat_sp500: bool,
}

/// Bloque de métricas financieras de una estrategia de Swap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapMetricsDto {
    pub total_return_pct: f64,
    pub cagr_pct: f64,
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub max_drawdown_pct: f64,
    pub max_drawdown_bars: usize,
    pub calmar_ratio: f64,
    pub total_swaps: usize,
    pub total_fees_paid: f64,
    pub win_rate_swaps_pct: f64,
    pub exposure_btc_pct: f64,
    pub exposure_gold_pct: f64,
    pub exposure_sp500_pct: f64,
    pub exposure_cash_pct: f64,
    pub swap_fitness_score: f64,
    pub capital_dd_ratio: f64,
    pub composite_r2: f64,
    pub slope_ratio: f64,
}

/// Representación DTO de una estrategia en el Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapDashboardStrategyDto {
    pub id: String,
    pub name: String,
    pub architecture: String,
    pub color: String,
    pub initial_capital: f64,
    pub final_capital: f64,
    pub net_profit: f64,
    pub metrics: SwapMetricsDto,
    pub equity_curve: Vec<f64>,
    pub drawdown_curve: Vec<f64>,
    pub allocations: Vec<StrategyAllocationPointDto>,
    pub trades: Vec<SwapTradeDto>,
    pub yearly_returns: Vec<YearlyPerformanceDto>,
}

/// Benchmark pasivo individual (BTC, ORO, SPY o Cartera Equitativa)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetBenchmarkDto {
    pub id: String,
    pub name: String,
    pub symbol: String,
    pub color: String,
    pub initial_price: f64,
    pub final_price: f64,
    pub total_return_pct: f64,
    pub cagr_pct: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown_pct: f64,
    pub equity_curve: Vec<f64>,
    pub drawdown_curve: Vec<f64>,
}

/// Payload global para la generación del Dashboard HTML de SWAP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapDashboardPayload {
    pub title: String,
    pub timeframe: String,
    pub generated_at_utc: String,
    pub common_start_date: String,
    pub common_end_date: String,
    pub total_bars: usize,
    pub is_split_idx: usize,
    pub timestamps: Vec<i64>,
    pub dates: Vec<String>,
    pub benchmarks: Vec<AssetBenchmarkDto>, // [BTC, ORO, SPY, Equal-Weight]
    pub strategies: Vec<SwapDashboardStrategyDto>,
}
