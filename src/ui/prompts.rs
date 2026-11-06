use crate::engine::swap::types::{MarginMode, SwapConfig};
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
    if io::stdin().read_line(&mut input).is_ok()
        && let Ok(val) = input.trim().parse::<f64>()
        && val > 0.0
    {
        cfg.initial_capital = val;
    }

    print!("  💸 Comisión de Swap por operación (%) [Por defecto 0.10%]: ");
    let _ = io::stdout().flush();
    input.clear();
    if io::stdin().read_line(&mut input).is_ok()
        && let Ok(val) = input.trim().parse::<f64>()
        && val >= 0.0
    {
        cfg.swap_fee_rate = val / 100.0;
    }

    print!("  ⏳ Barras mínimas de tenencia antes de rotar [Por defecto 5 barras]: ");
    let _ = io::stdout().flush();
    input.clear();
    if io::stdin().read_line(&mut input).is_ok()
        && let Ok(val) = input.trim().parse::<usize>()
        && val >= 1
    {
        cfg.min_holding_bars = val;
    }

    cfg.margin_mode = prompt_margin_mode();
    cfg.leverage = prompt_leverage_value(5.0);

    cfg
}

/// Solicita el modo de gestión de capital (Cruzado o Aislado)
pub fn prompt_margin_mode() -> MarginMode {
    println!("\n  🛡️ Modo de Gestión de Capital:");
    println!("     [1] 🔄 Capital Cruzado (Cross Margin — la pérdida va contra toda la cuenta)");
    println!("     [2] 🔒 Margen Aislado (Isolated Margin — la pérdida se limita al capital de la operación)");
    print!("  👉 Selecciona modo (1-2) [Por defecto '1']: ");
    let _ = io::stdout().flush();
    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_ok() && input.trim() == "2" {
        MarginMode::Isolated
    } else {
        MarginMode::Cross
    }
}

/// Solicita un valor único de apalancamiento
pub fn prompt_leverage_value(default_val: f64) -> f64 {
    print!("  ⚡ Apalancamiento de Cartera (Leverage) [Por defecto {:.1}X]: ", default_val);
    let _ = io::stdout().flush();
    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_ok() {
        let clean = input.trim().trim_end_matches(['X', 'x']);
        if let Ok(val) = clean.parse::<f64>()
            && (1.0..=100.0).contains(&val)
        {
            return val;
        }
    }
    default_val
}

/// Solicita uno o varios apalancamientos para pruebas comparativas
pub fn prompt_leverage_multi(default_levels: &[f64]) -> Vec<f64> {
    let default_str = default_levels
        .iter()
        .map(|v| format!("{:.0}X", v))
        .collect::<Vec<_>>()
        .join(", ");

    println!("\n  ⚡ Configuración de Apalancamiento(s) de Cartera (Leverage):");
    println!("     • Ingresa un valor único (ej. '5' o '5X') para simular solo ese nivel");
    println!("     • O varios separados por coma/espacio para sweep comparativo (ej. '1, 2, 3, 5, 10')");
    print!("  👉 Apalancamiento(s) [Por defecto '{}']: ", default_str);
    let _ = io::stdout().flush();
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return default_levels.to_vec();
    }

    let mut levels = Vec::new();
    for part in trimmed.split([',', ' ', ';']) {
        let clean = part.trim().trim_end_matches(['X', 'x']);
        if let Ok(val) = clean.parse::<f64>()
            && (1.0..=100.0).contains(&val)
            && !levels.iter().any(|&v: &f64| (v - val).abs() < 1e-4)
        {
            levels.push(val);
        }
    }

    if levels.is_empty() {
        default_levels.to_vec()
    } else {
        levels
    }
}
