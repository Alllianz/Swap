use super::features::TOTAL_SWAP_FEATURES;
use super::node::SwapTreeNode;

/// Árbol de decisión individual del ensamble de boosting de SWAP
#[derive(Clone, Debug)]
pub struct SwapDecisionTree {
    pub root: SwapTreeNode,
    pub tree_idx: usize,
}

impl SwapDecisionTree {
    pub fn new(root: SwapTreeNode, tree_idx: usize) -> Self {
        Self { root, tree_idx }
    }

    #[inline(always)]
    pub fn predict(&self, features: &[f32; TOTAL_SWAP_FEATURES]) -> f32 {
        self.root.predict(features)
    }
}
