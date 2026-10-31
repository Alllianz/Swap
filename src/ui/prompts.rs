use crate::engine::swap::types::SwapConfig;
use std::io::{self, Write};

pub fn pause() {
    print!("\n  ⏎ Presiona [Enter] para continuar...");
    let _ = io::stdout().flush();
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
}

pub fn prompt_timeframe() -> String {
    print!("  👉 Selecciona el timeframe para el swap (1D, 4H, 1H) [Por defecto '1D']: ");
    let _ = io::stdout().flush();
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
    let trimmed = input.trim();
    if trimmed.is_empty() {
        "1D".to_string()
    } else {
        trimmed.to_uppercase()
    }
}

pub fn prompt_swap_config() -> SwapConfig {
    let mut cfg = SwapConfig::default();

    print!("  💵 Capital Inicial en USD [Por defecto $10,000]: ");
    let _ = io::stdout().flush();
    let mut input = String::new();
    if let Ok(_) = io::stdin().read_line(&mut input) {
        if let Ok(val) = input.trim().parse::<f64>() {
            if val > 0.0 {
                cfg.initial_capital = val;
            }
        }
    }

    print!("  💸 Comisión de Swap por operación (%) [Por defecto 0.10%]: ");
    let _ = io::stdout().flush();
    input.clear();
    if let Ok(_) = io::stdin().read_line(&mut input) {
        if let Ok(val) = input.trim().parse::<f64>() {
            if val >= 0.0 {
                cfg.swap_fee_rate = val / 100.0;
            }
        }
    }

    print!("  ⏳ Barras mínimas de tenencia antes de rotar [Por defecto 5 barras]: ");
    let _ = io::stdout().flush();
    input.clear();
    if let Ok(_) = io::stdin().read_line(&mut input) {
        if let Ok(val) = input.trim().parse::<usize>() {
            if val >= 1 {
                cfg.min_holding_bars = val;
            }
        }
    }

    cfg
}
