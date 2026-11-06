use super::types::{AssetId, MarginMode, SwapTradeRecord};
use serde::{Deserialize, Serialize};

/// Reporte cuantitativo integral de rendimiento para una estrategia de Swap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapReport {
    pub strategy_name: String,
    pub margin_mode: MarginMode,
    pub leverage: f64,
    pub initial_capital: f64,
    pub final_capital: f64,
    pub net_profit: f64,
    pub total_return_pct: f64,
    pub cagr_pct: f64,
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub max_drawdown_pct: f64,
    pub max_drawdown_duration_bars: usize,
    pub calmar_ratio: f64,
    pub total_swaps: usize,
    pub total_fees_paid: f64,
    pub profitable_swaps: usize,
    pub win_rate_swaps_pct: f64,

    // Distribución porcentual del tiempo de exposición Long
    pub exposure_btc_pct: f64,
    pub exposure_gold_pct: f64,
    pub exposure_sp500_pct: f64,

    // Distribución porcentual del tiempo de exposición Short
    pub exposure_short_btc_pct: f64,
    pub exposure_short_gold_pct: f64,
    pub exposure_short_sp500_pct: f64,

    // Preservación en Cash
    pub exposure_cash_pct: f64,

    // Comparativas con Benchmarks pasivos
    pub benchmark_btc_return_pct: f64,
    pub benchmark_gold_return_pct: f64,
    pub benchmark_sp500_return_pct: f64,
    pub benchmark_equal_weight_return_pct: f64,
    pub alpha_vs_sp500: f64,
    pub alpha_vs_btc: f64,

    // Curva de equidad temporal (timestamp, capital)
    pub equity_curve: Vec<(i64, f64)>,
    // Historial completo de operaciones de swap
    pub trades: Vec<SwapTradeRecord>,
    // Historial de asignaciones por barra: (asset_1, weight_1, asset_2, weight_2)
    #[serde(default)]
    pub daily_allocations: Vec<(AssetId, f64, Option<AssetId>, f64)>,
}

impl SwapReport {
    pub fn print_summary_card(&self) {
        println!("  ┌────────────────────────────────────────────────────────────────────────┐");
        println!("  │  ESTRATEGIA: {:<58}│", self.strategy_name);
        println!(
            "  │  MODO CAPITAL: {:<56}│",
            format!("{} ({:.1}X)", self.margin_mode.name(), self.leverage)
        );
        println!("  ├────────────────────────────────────────────────────────────────────────┤");
        println!(
            "  │  Capital Inicial:   ${:<14.2} │ Capital Final:     ${:<15.2}│",
            self.initial_capital, self.final_capital
        );
        println!(
            "  │  Retorno Total:     {:>+13.2}% │ CAGR Anualizado:   {:>+14.2}%│",
            self.total_return_pct, self.cagr_pct
        );
        println!(
            "  │  Sharpe Ratio:      {:>14.2}  │ Sortino Ratio:     {:>15.2} │",
            self.sharpe_ratio, self.sortino_ratio
        );
        println!(
            "  │  Max Drawdown:      {:>13.2}% │ Calmar Ratio:      {:>15.2} │",
            self.max_drawdown_pct, self.calmar_ratio
        );
        println!(
            "  │  Rotaciones (Swaps):{:>14}  │ Win Rate Swaps:    {:>14.2}%│",
            self.total_swaps, self.win_rate_swaps_pct
        );
        println!(
            "  │  Comisiones Totales:${:<14.2} │ Max DD Duración:   {:>11} velas│",
            self.total_fees_paid, self.max_drawdown_duration_bars
        );
        println!("  ├────────────────────────────────────────────────────────────────────────┤");
        println!("  │  DISTRIBUCIÓN DE EXPOSICIÓN (LONG / SHORT / CASH):                    │");
        println!(
            "  │  • BTC:   Long {:>4.1}% | Short {:>4.1}%   • ORO: Long {:>4.1}% | Short {:>4.1}%  │",
            self.exposure_btc_pct, self.exposure_short_btc_pct,
            self.exposure_gold_pct, self.exposure_short_gold_pct
        );
        println!(
            "  │  • SP500: Long {:>4.1}% | Short {:>4.1}%   • CASH: {:>5.1}%                     │",
            self.exposure_sp500_pct, self.exposure_short_sp500_pct,
            self.exposure_cash_pct
        );
        println!("  ├────────────────────────────────────────────────────────────────────────┤");
        println!("  │  BENCHMARKS PASIVOS (BUY & HOLD):                                      │");
        println!(
            "  │  • B&H BTC:   {:>+8.2}%   • B&H ORO:   {:>+8.2}%   • B&H SP500: {:>+8.2}%   │",
            self.benchmark_btc_return_pct,
            self.benchmark_gold_return_pct,
            self.benchmark_sp500_return_pct
        );
        println!(
            "  │  • Cartera Equitativa (1/3 c/u): {:>+8.2}%  │ Alfa vs SP500: {:>+8.2}% │",
            self.benchmark_equal_weight_return_pct, self.alpha_vs_sp500
        );
        println!("  └────────────────────────────────────────────────────────────────────────┘");
    }
}
