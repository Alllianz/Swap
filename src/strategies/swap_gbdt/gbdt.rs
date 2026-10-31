use super::features::TOTAL_SWAP_FEATURES;
use super::node::SwapTreeNode;
use super::tree::SwapDecisionTree;

/// Ensamble GBDT institucional del motor SWAP
#[derive(Clone, Debug)]
pub struct SwapGbdtEnsemble {
    pub name: String,
    pub base_score: f32,
    pub learning_rate: f32,
    pub trees: Vec<SwapDecisionTree>,
}

impl SwapGbdtEnsemble {
    #[inline(always)]
    pub fn predict(&self, features: &[f32; TOTAL_SWAP_FEATURES]) -> f32 {
        let mut score = self.base_score;
        for tree in &self.trees {
            score += self.learning_rate * tree.predict(features);
        }
        score
    }

    pub fn build_btc_model() -> Self {
        let mut trees = Vec::new();

        // Árbol 1: Momentum de 30D y ratio BTC/Gold
        trees.push(SwapDecisionTree::new(
            SwapTreeNode::Internal {
                feature_idx: 1, // BTC ret 30D
                threshold: 0.05,
                left: Box::new(SwapTreeNode::Internal {
                    feature_idx: 3, // BTC dist SMA30
                    threshold: -0.02,
                    left: Box::new(SwapTreeNode::new_leaf(-0.65)),
                    right: Box::new(SwapTreeNode::new_leaf(-0.10)),
                }),
                right: Box::new(SwapTreeNode::Internal {
                    feature_idx: 13, // BTC / Gold momentum
                    threshold: 0.02,
                    left: Box::new(SwapTreeNode::new_leaf(0.35)),
                    right: Box::new(SwapTreeNode::new_leaf(0.85)),
                }),
            },
            0,
        ));

        // Árbol 2: Volatilidad y ratio BTC/SP500
        trees.push(SwapDecisionTree::new(
            SwapTreeNode::Internal {
                feature_idx: 12, // BTC / SP500 momentum
                threshold: 0.04,
                left: Box::new(SwapTreeNode::Internal {
                    feature_idx: 0, // BTC ret 14D
                    threshold: -0.03,
                    left: Box::new(SwapTreeNode::new_leaf(-0.50)),
                    right: Box::new(SwapTreeNode::new_leaf(0.05)),
                }),
                right: Box::new(SwapTreeNode::Internal {
                    feature_idx: 15, // Spread de vol BTC vs SP500
                    threshold: 4.5,
                    left: Box::new(SwapTreeNode::new_leaf(0.70)),
                    right: Box::new(SwapTreeNode::new_leaf(0.40)),
                }),
            },
            1,
        ));

        // Árbol 3: Tendencia de corto plazo
        trees.push(SwapDecisionTree::new(
            SwapTreeNode::Internal {
                feature_idx: 0, // BTC ret 14D
                threshold: 0.02,
                left: Box::new(SwapTreeNode::new_leaf(-0.25)),
                right: Box::new(SwapTreeNode::new_leaf(0.45)),
            },
            2,
        ));

        Self {
            name: "SWAP GBDT BTC Scorer".to_string(),
            base_score: 0.0,
            learning_rate: 0.15,
            trees,
        }
    }

    pub fn build_gold_model() -> Self {
        let mut trees = Vec::new();

        // Árbol 1: Ratio Gold / SP500 y distancia SMA30
        trees.push(SwapDecisionTree::new(
            SwapTreeNode::Internal {
                feature_idx: 14, // Gold / SP500 ratio
                threshold: 0.01,
                left: Box::new(SwapTreeNode::Internal {
                    feature_idx: 5, // Gold ret 30D
                    threshold: -0.01,
                    left: Box::new(SwapTreeNode::new_leaf(-0.40)),
                    right: Box::new(SwapTreeNode::new_leaf(0.10)),
                }),
                right: Box::new(SwapTreeNode::Internal {
                    feature_idx: 7, // Gold dist SMA30
                    threshold: 0.01,
                    left: Box::new(SwapTreeNode::new_leaf(0.45)),
                    right: Box::new(SwapTreeNode::new_leaf(0.80)),
                }),
            },
            0,
        ));

        // Árbol 2: Volatilidad de SP500 y retornos de BTC
        trees.push(SwapDecisionTree::new(
            SwapTreeNode::Internal {
                feature_idx: 10, // SP500 vol
                threshold: 0.22,
                left: Box::new(SwapTreeNode::Internal {
                    feature_idx: 4, // Gold ret 14D
                    threshold: 0.01,
                    left: Box::new(SwapTreeNode::new_leaf(-0.10)),
                    right: Box::new(SwapTreeNode::new_leaf(0.30)),
                }),
                right: Box::new(SwapTreeNode::Internal {
                    feature_idx: 1, // BTC ret 30D
                    threshold: -0.05,
                    left: Box::new(SwapTreeNode::new_leaf(0.90)),
                    right: Box::new(SwapTreeNode::new_leaf(0.40)),
                }),
            },
            1,
        ));

        Self {
            name: "SWAP GBDT Gold Scorer".to_string(),
            base_score: 0.0,
            learning_rate: 0.15,
            trees,
        }
    }

    pub fn build_sp500_model() -> Self {
        let mut trees = Vec::new();

        // Árbol 1: Volatilidad SP500 y retorno 30D
        trees.push(SwapDecisionTree::new(
            SwapTreeNode::Internal {
                feature_idx: 10, // SP500 vol
                threshold: 0.18,
                left: Box::new(SwapTreeNode::Internal {
                    feature_idx: 9, // SP500 ret 30D
                    threshold: 0.01,
                    left: Box::new(SwapTreeNode::new_leaf(0.20)),
                    right: Box::new(SwapTreeNode::new_leaf(0.75)),
                }),
                right: Box::new(SwapTreeNode::Internal {
                    feature_idx: 11, // SP500 dist SMA30
                    threshold: -0.02,
                    left: Box::new(SwapTreeNode::new_leaf(-0.60)),
                    right: Box::new(SwapTreeNode::new_leaf(-0.10)),
                }),
            },
            0,
        ));

        // Árbol 2: Retorno a 14D y relación con Oro
        trees.push(SwapDecisionTree::new(
            SwapTreeNode::Internal {
                feature_idx: 8, // SP500 ret 14D
                threshold: 0.005,
                left: Box::new(SwapTreeNode::new_leaf(-0.30)),
                right: Box::new(SwapTreeNode::Internal {
                    feature_idx: 14, // Gold / SP500
                    threshold: -0.01,
                    left: Box::new(SwapTreeNode::new_leaf(0.60)),
                    right: Box::new(SwapTreeNode::new_leaf(0.25)),
                }),
            },
            1,
        ));

        Self {
            name: "SWAP GBDT SP500 Scorer".to_string(),
            base_score: 0.0,
            learning_rate: 0.15,
            trees,
        }
    }
}
