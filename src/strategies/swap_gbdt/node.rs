use super::features::TOTAL_SWAP_FEATURES;

/// Nodo de árbol de decisión del motor SWAP
#[derive(Clone, Debug)]
pub enum SwapTreeNode {
    Leaf {
        weight: f32,
        samples: usize,
    },
    Internal {
        feature_idx: usize,
        threshold: f32,
        left: Box<SwapTreeNode>,
        right: Box<SwapTreeNode>,
    },
}

impl SwapTreeNode {
    pub fn new_leaf(weight: f32) -> Self {
        SwapTreeNode::Leaf { weight, samples: 0 }
    }

    pub fn predict(&self, features: &[f32; TOTAL_SWAP_FEATURES]) -> f32 {
        match self {
            SwapTreeNode::Leaf { weight, .. } => *weight,
            SwapTreeNode::Internal {
                feature_idx,
                threshold,
                left,
                right,
            } => {
                let val = if *feature_idx < TOTAL_SWAP_FEATURES {
                    features[*feature_idx]
                } else {
                    0.0
                };

                if val <= *threshold {
                    left.predict(features)
                } else {
                    right.predict(features)
                }
            }
        }
    }
}
