use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeighborhoodCell {
    pub p1_val: usize,
    pub p2_val: usize,
    pub score_is: f64,
    pub score_oos: f64,
    pub score_completo: f64,
    pub retention_ratio: f64,
    pub is_winner: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeighborhoodGrid {
    pub p1_name: String,
    pub p2_name: String,
    pub p1_values: Vec<usize>,
    pub p2_values: Vec<usize>,
    pub cells: Vec<Vec<NeighborhoodCell>>,
    pub winner_coords: (usize, usize),
    pub diagnosis: String,
}

impl NeighborhoodGrid {
    pub fn new(
        p1_name: impl Into<String>,
        p2_name: impl Into<String>,
        p1_values: Vec<usize>,
        p2_values: Vec<usize>,
        cells: Vec<Vec<NeighborhoodCell>>,
        winner_coords: (usize, usize),
    ) -> Self {
        let p1_name = p1_name.into();
        let p2_name = p2_name.into();

        // Evaluar robustez del vecindario
        let win_cell = &cells[winner_coords.0][winner_coords.1];
        let mut neighbor_retentions = Vec::new();

        let rows = cells.len();
        let cols = if rows > 0 { cells[0].len() } else { 0 };

        for dr in -1i32..=1 {
            for dc in -1i32..=1 {
                if dr == 0 && dc == 0 {
                    continue;
                }
                let nr = winner_coords.0 as i32 + dr;
                let nc = winner_coords.1 as i32 + dc;
                if nr >= 0 && nr < rows as i32 && nc >= 0 && nc < cols as i32 {
                    let cell = &cells[nr as usize][nc as usize];
                    if cell.score_is > 0.0 {
                        neighbor_retentions.push(cell.retention_ratio);
                    }
                }
            }
        }

        let avg_neighbor_ret = if !neighbor_retentions.is_empty() {
            neighbor_retentions.iter().sum::<f64>() / neighbor_retentions.len() as f64
        } else {
            0.0
        };

        let diagnosis = if win_cell.retention_ratio >= 0.70 && avg_neighbor_ret >= 0.60 {
            format!(
                "Zona Robusta Confirmada: El ganador tiene Retención {:.2} y sus vecinos promedian {:.2}. Meseta cuantitativa sólida sin overfitting.",
                win_cell.retention_ratio, avg_neighbor_ret
            )
        } else if win_cell.retention_ratio >= 0.40 {
            format!(
                "Zona Prometedora: Retención del ganador {:.2}, vecindario promedio {:.2}. Monitorear degradación fuera de muestra.",
                win_cell.retention_ratio, avg_neighbor_ret
            )
        } else {
            format!(
                "Alerta de Overfitting / Pico Aislado: El óptimo tiene Retención {:.2} pero sus vecinos decaen a {:.2}. Alta probabilidad de curva sobreajustada.",
                win_cell.retention_ratio, avg_neighbor_ret
            )
        };

        Self {
            p1_name,
            p2_name,
            p1_values,
            p2_values,
            cells,
            winner_coords,
            diagnosis,
        }
    }
}
