use crate::data::kline::Kline;

/// Retornos porcentuales vela a vela causales
pub fn calculate_returns(klines: &[Kline]) -> Vec<f64> {
    let n = klines.len();
    if n == 0 {
        return Vec::new();
    }
    let mut ret = vec![0.0; n];
    for i in 1..n {
        let prev = klines[i - 1].close;
        if prev > 0.0 {
            ret[i] = (klines[i].close - prev) / prev;
        }
    }
    ret
}
