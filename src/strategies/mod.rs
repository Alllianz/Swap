pub mod dual_directional;
pub mod dual_momentum;
pub mod ema_cross;
pub mod regime_hybrid;
pub mod rsi_reversion;
pub mod swap_gbdt;

pub use dual_directional::DualDirectionalModel;
pub use dual_momentum::DualMomentumStrategy;
pub use ema_cross::EmaCrossStrategy;
pub use regime_hybrid::RegimeHybridStrategy;
pub use rsi_reversion::RsiReversionStrategy;
pub use swap_gbdt::SwapGbdtStrategy;
