/// Calculador oficial de distancia y precio de liquidación con factor de ajuste escalonado
/// según la doctrina cuantitativa de Allianz y especificado en el rediseño.
pub struct LiquidationCalculator;

impl LiquidationCalculator {
    /// Devuelve el factor de ajuste `f` y la distancia de liquidación en porcentaje `d`
    /// Fórmula: Distancia (%) = (100 / palanca) * (1 - f) - 0.045
    pub fn get_liquidation_params(leverage: f64) -> (f64, f64) {
        let f = if leverage <= 5.0 {
            0.05
        } else if leverage <= 10.0 {
            0.08
        } else if leverage <= 15.0 {
            0.10
        } else if leverage <= 20.0 {
            0.12
        } else if leverage <= 30.0 {
            0.15
        } else if leverage <= 35.0 {
            0.18
        } else if leverage <= 45.0 {
            0.20
        } else if leverage <= 50.0 {
            0.25
        } else if leverage <= 100.0 {
            0.30
        } else {
            0.35
        };

        let distance_pct = ((100.0 / leverage) * (1.0 - f)) - 0.045;
        (f, distance_pct.max(0.01))
    }

    /// Calcula el precio exacto de liquidación para Long y Short
    pub fn calculate_liquidation_price(entry_price: f64, leverage: f64, is_long: bool) -> f64 {
        let (_, distance_pct) = Self::get_liquidation_params(leverage);
        let dist_factor = distance_pct / 100.0;

        if is_long {
            (entry_price * (1.0 - dist_factor)).max(0.0)
        } else {
            entry_price * (1.0 + dist_factor)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_liquidation_distance() {
        let (f10, d10) = LiquidationCalculator::get_liquidation_params(10.0);
        assert_eq!(f10, 0.08);
        assert!((d10 - 9.155).abs() < 1e-3);

        let (f20, d20) = LiquidationCalculator::get_liquidation_params(20.0);
        assert_eq!(f20, 0.12);
        assert!((d20 - 4.355).abs() < 1e-3);
    }
}
