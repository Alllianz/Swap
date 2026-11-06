pub mod drawdown;
pub mod neighborhood;
pub mod ranking;
pub mod smoothness;
pub mod stagnation;
pub mod swap_fitness;
pub mod yearly_returns;

pub use drawdown::{compute_drawdown_series, DrawdownPoint, DrawdownSummary};
pub use neighborhood::{NeighborhoodCell, NeighborhoodGrid};
pub use ranking::{compute_multicriteria_rankings, CandidateRankScore};
pub use smoothness::{calculate_equity_smoothness, SmoothnessMetrics};
pub use stagnation::{calculate_stagnation_episodes, StagnationEpisode};
pub use swap_fitness::{
    calculate_autotuning_score, calculate_capital_max_dd_ratio, calculate_series_r2,
    calculate_slope_ratio, evaluate_equity_linearity, evaluate_swap_fitness,
    LinearityMetrics, SwapFitnessResult,
};
pub use yearly_returns::{calculate_yearly_returns, YearlyPerformance};
