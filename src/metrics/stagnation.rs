use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StagnationEpisode {
    pub rank: usize,
    pub start_idx: usize,
    pub end_idx: usize,
    pub start_timestamp: i64,
    pub end_timestamp: i64,
    pub length_bars: usize,
    pub max_dd_pct: f64,
    pub is_open: bool,
}

fn push_episode_if_valid(
    episodes: &mut Vec<StagnationEpisode>,
    peak_idx: usize,
    end_idx: usize,
    peak: f64,
    trough: f64,
    is_open: bool,
    curve: &[(i64, f64)],
) {
    if end_idx > peak_idx {
        let dd_pct = if peak > 0.0 {
            ((peak - trough) / peak) * 100.0
        } else {
            0.0
        };
        episodes.push(StagnationEpisode {
            rank: 0,
            start_idx: peak_idx,
            end_idx,
            start_timestamp: curve[peak_idx].0,
            end_timestamp: curve[end_idx].0,
            length_bars: end_idx - peak_idx,
            max_dd_pct: dd_pct,
            is_open,
        });
    }
}

/// Detecta todos los períodos de estancamiento (stagnation) desde un pico histórico
/// hasta la última vela que no lo supera, siguiendo la metodología cuantitativa de SWAP.
pub fn calculate_stagnation_episodes(curve: &[(i64, f64)]) -> Vec<StagnationEpisode> {
    if curve.is_empty() {
        return Vec::new();
    }

    let n = curve.len();
    let mut episodes = Vec::new();

    let mut peak = curve[0].1;
    let mut peak_idx = 0;
    let mut trough = curve[0].1;

    for i in 1..n {
        let eq = curve[i].1;
        if eq > peak {
            push_episode_if_valid(&mut episodes, peak_idx, i - 1, peak, trough, false, curve);
            peak = eq;
            peak_idx = i;
            trough = eq;
        } else if eq < trough {
            trough = eq;
        }
    }

    push_episode_if_valid(&mut episodes, peak_idx, n - 1, peak, trough, true, curve);

    // Ordenar de mayor a menor duración en velas
    episodes.sort_by_key(|a| std::cmp::Reverse(a.length_bars));

    // Asignar rangos (1º, 2º, 3º...)
    for (i, ep) in episodes.iter_mut().enumerate() {
        ep.rank = i + 1;
    }

    episodes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stagnation_detection() {
        let curve = vec![
            (1000, 100.0), // peak
            (2000, 90.0),
            (3000, 95.0),
            (4000, 105.0), // new peak
            (5000, 100.0),
            (6000, 102.0),
        ];

        let episodes = calculate_stagnation_episodes(&curve);
        assert!(!episodes.is_empty());
        // First episode from 100 to 105 (3 bars: index 0 to 2)
        // Second episode from 105 open (2 bars: index 3 to 5)
        assert_eq!(episodes[0].rank, 1);
    }
}
