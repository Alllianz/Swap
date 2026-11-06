/// Normalización y utilidades para temporalidades de trading
pub fn normalize_timeframe(tf: &str) -> String {
    let clean = tf.trim();
    if clean.is_empty() {
        return "1D".to_string();
    }
    let lower = clean.to_lowercase();
    if lower.ends_with('m') {
        lower
    } else {
        clean.to_uppercase()
    }
}

/// Convierte el string de temporalidad a duración en milisegundos
pub fn tf_to_duration_millis(tf: &str) -> i64 {
    let lower = tf.to_lowercase();
    match lower.as_str() {
        "1m" => 60 * 1000,
        "3m" => 3 * 60 * 1000,
        "5m" => 5 * 60 * 1000,
        "15m" => 15 * 60 * 1000,
        "30m" => 30 * 60 * 1000,
        "1h" => 60 * 60 * 1000,
        "2h" => 2 * 60 * 60 * 1000,
        "4h" => 4 * 60 * 60 * 1000,
        "6h" => 6 * 60 * 60 * 1000,
        "8h" => 8 * 60 * 60 * 1000,
        "12h" => 12 * 60 * 60 * 1000,
        "1d" => 24 * 60 * 60 * 1000,
        _ => 24 * 60 * 60 * 1000,
    }
}
