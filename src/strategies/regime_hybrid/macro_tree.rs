use crate::engine::swap::types::SynchronizedBar;
use super::regime::MarketRegime;

/// Árbol de Decisión de Regímenes Macroeconómicos SWAP.
/// Evalúa de forma 100% causal las interacciones multiactivo para determinar
/// el estado dominante del mercado financiero.
pub struct MacroRegimeClassifier;

impl MacroRegimeClassifier {
    /// Clasifica el régimen del mercado usando estrictamente datos hasta `current_idx` (0% Look-Ahead)
    pub fn classify(history: &[SynchronizedBar], current_idx: usize) -> (MarketRegime, f64, String) {
        if current_idx < 60 {
            return (
                MarketRegime::SteadyGrowthEquities,
                0.5,
                "Inicializando régimen por defecto".to_string(),
            );
        }

        let curr = &history[current_idx];

        // 1. Métricas de S&P 500
        let sp500_sma50 = calc_sma(history, current_idx, 50, |b| b.sp500_close);
        let sp500_sma200 = calc_sma(
            history,
            current_idx,
            current_idx.min(150),
            |b| b.sp500_close,
        );
        let sp500_vol30 = calc_realized_vol(history, current_idx, 30, |b| b.sp500_close);
        let sp500_dist_sma200 = (curr.sp500_close / sp500_sma200.max(1e-6)) - 1.0;
        let sp500_ret_30 = (curr.sp500_close / history[current_idx - 30].sp500_close.max(1e-6)) - 1.0;

        // 2. Métricas de Bitcoin
        let btc_ret_30 = (curr.btc_close / history[current_idx - 30].btc_close.max(1e-6)) - 1.0;
        let btc_ret_14 = (curr.btc_close / history[current_idx - 14].btc_close.max(1e-6)) - 1.0;
        let btc_sma30 = calc_sma(history, current_idx, 30, |b| b.btc_close);
        let btc_above_sma = curr.btc_close > btc_sma30;

        // 3. Métricas de Oro
        let gold_ret_30 = (curr.gold_close / history[current_idx - 30].gold_close.max(1e-6)) - 1.0;
        let gold_sma30 = calc_sma(history, current_idx, 30, |b| b.gold_close);
        let gold_above_sma = curr.gold_close > gold_sma30;

        // 4. Ratio BTC / Oro (Termómetro de Apetito por Riesgo vs Aversión)
        let ratio_curr = curr.btc_close / curr.gold_close.max(1e-6);
        let ratio_30d = history[current_idx - 30].btc_close
            / history[current_idx - 30].gold_close.max(1e-6);
        let btc_gold_ratio_momentum = (ratio_curr / ratio_30d.max(1e-6)) - 1.0;

        // --- NODO 1: Detección de Pánico Sistémico / Crash ---
        // S&P 500 desplomándose con alta volatilidad Y Bitcoin también cayendo
        if (sp500_vol30 > 0.28 || sp500_dist_sma200 < -0.06) && sp500_ret_30 < -0.04 && btc_ret_30 < -0.08 {
            if gold_above_sma && gold_ret_30 > 0.01 {
                return (
                    MarketRegime::DefensiveGoldFlight,
                    0.85,
                    format!(
                        "Pánico en activos de riesgo (SP500 Vol: {:.1}%, Ret30: {:.1}%, BTC Ret30: {:.1}%). Oro actuando como desacoplador defensivo (+{:.1}%)",
                        sp500_vol30 * 100.0, sp500_ret_30 * 100.0, btc_ret_30 * 100.0, gold_ret_30 * 100.0
                    ),
                );
            } else {
                return (
                    MarketRegime::SystemicPanicCash,
                    0.90,
                    format!(
                        "Shock de correlación negativa (SP500 Vol: {:.1}%, Ret30: {:.1}% | BTC Ret30: {:.1}%). Fuga a CASH",
                        sp500_vol30 * 100.0, sp500_ret_30 * 100.0, btc_ret_30 * 100.0
                    ),
                );
            }
        }

        // --- NODO 2: Detección de Régimen Risk-On / Cripto Expansión ---
        // Ratio BTC/Oro expandiéndose con BTC sobre su media y retornos positivos
        if btc_gold_ratio_momentum > 0.05 && btc_above_sma && btc_ret_30 > 0.03 && btc_ret_14 > -0.02 {
            let conf = (btc_gold_ratio_momentum * 2.0).clamp(0.4, 0.95);
            return (
                MarketRegime::RiskOnLiquidity,
                conf,
                format!(
                    "Expansión de liquidez y aceleración cripto (Ratio BTC/Oro +{:.1}%, BTC Ret30: +{:.1}%, sobre SMA30)",
                    btc_gold_ratio_momentum * 100.0, btc_ret_30 * 100.0
                ),
            );
        }

        // --- NODO 3: Detección de Régimen Renta Variable / Crecimiento Estable ---
        // S&P 500 en tendencia alcista tranquila (baja volatilidad, por encima de SMA50)
        if curr.sp500_close >= sp500_sma50 && sp500_ret_30 >= 0.0 && sp500_vol30 <= 0.20 {
            let conf = 0.70;
            return (
                MarketRegime::SteadyGrowthEquities,
                conf,
                format!(
                    "Bursátil estable en EE.UU. (SP500 sobre SMA50, Vol baja: {:.1}%, Ret30: +{:.1}%)",
                    sp500_vol30 * 100.0, sp500_ret_30 * 100.0
                ),
            );
        }

        // --- NODO 4: Régimen Refugio / Oro si supera a la renta variable ---
        if gold_above_sma && (gold_ret_30 > sp500_ret_30 || sp500_ret_30 < -0.01) {
            return (
                MarketRegime::DefensiveGoldFlight,
                0.65,
                format!(
                    "Mercado defensivo con debilidad bursátil (Oro Ret30: +{:.1}% vs SP500: {:.1}%)",
                    gold_ret_30 * 100.0, sp500_ret_30 * 100.0
                ),
            );
        }

        // Fallback dinámico por fuerza relativa
        if btc_ret_30 > sp500_ret_30 && btc_ret_30 > gold_ret_30 && btc_above_sma {
            (
                MarketRegime::RiskOnLiquidity,
                0.55,
                "Liderazgo relativo de Bitcoin en régimen mixto".to_string(),
            )
        } else if sp500_ret_30 >= gold_ret_30 {
            (
                MarketRegime::SteadyGrowthEquities,
                0.55,
                "Prevalencia de Renta Variable en régimen neutro".to_string(),
            )
        } else {
            (
                MarketRegime::DefensiveGoldFlight,
                0.55,
                "Prevalencia de Oro físico en régimen neutro".to_string(),
            )
        }
    }
}

fn calc_sma<F: Fn(&SynchronizedBar) -> f64>(
    history: &[SynchronizedBar],
    curr: usize,
    window: usize,
    extractor: F,
) -> f64 {
    let start = curr.saturating_sub(window - 1);
    let count = (curr - start + 1) as f64;
    let sum: f64 = history[start..=curr].iter().map(|b| extractor(b)).sum();
    sum / count.max(1.0)
}

fn calc_realized_vol<F: Fn(&SynchronizedBar) -> f64>(
    history: &[SynchronizedBar],
    curr: usize,
    window: usize,
    extractor: F,
) -> f64 {
    let start = curr.saturating_sub(window);
    let mut log_rets = Vec::with_capacity(window);
    for i in start..curr {
        let p0 = extractor(&history[i]);
        let p1 = extractor(&history[i + 1]);
        if p0 > 1e-6 && p1 > 1e-6 {
            log_rets.push((p1 / p0).ln());
        }
    }
    if log_rets.len() < 2 {
        return 0.15;
    }
    let m = log_rets.iter().sum::<f64>() / (log_rets.len() as f64);
    let var = log_rets.iter().map(|r| (r - m).powi(2)).sum::<f64>() / (log_rets.len() - 1) as f64;
    (var.sqrt() * (365.25f64).sqrt()).max(0.01)
}
