use crate::reaction::ReactionRuleSpec;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Agent {
    /// Endowment vector across goods (length = n).
    pub e: Vec<f64>,
    /// Aggregated Cobb–Douglas exponents (length = n, sum = 1).
    pub beta: Vec<f64>,
    /// Pairwise preference parameters versus base good:
    /// alpha_to_base[k] = alpha_{k,base} in (0,1); alpha_to_base[base]=0.5 by convention.
    pub alpha_to_base: Vec<f64>,
    /// Endogenous transformations (reaction term) applied before diffusion/trading.
    #[serde(default)]
    pub reaction_rules: Vec<ReactionRuleSpec>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TradeEvent {
    pub round: usize,
    pub i: usize,
    pub j: usize,
    pub good_a: usize,
    pub good_b: usize,
    pub q_ab: f64,
    pub delta_a_i: f64,
    pub delta_b_i: f64,
    pub delta_u_i: f64,
    pub delta_u_j: f64,
}

/// How to choose candidate good-pairs to evaluate in each P2P encounter.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PairingMode {
    /// Evaluate every good A against the base good B only.
    AgainstBase,
    /// Evaluate all ordered pairs (A,B) but only within a pruned candidate set
    /// of size `candidate_goods_k` (plus the base good).
    AllPairsPruned,
}

impl Default for PairingMode {
    fn default() -> Self { PairingMode::AgainstBase }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SimConfig {
    pub seed: u64,
    pub num_agents: usize,
    pub rounds: usize,
    pub p2p_encounters_per_round: usize,
    pub base_good: usize,

    pub initial_endowment_scale: f64,
    pub alpha_low: f64,
    pub alpha_high: f64,

    pub trade_step_cap_frac: f64,
    pub min_qty: f64,
    pub oracle_bisect_iters: usize,

    #[serde(default)]
    pub pairing_mode: PairingMode,
    /// Used only when `pairing_mode = all_pairs_pruned`.
    #[serde(default = "default_candidate_goods_k")]
    pub candidate_goods_k: usize,
    
    // Incorporates Goods as config parameters
    #[serde(default)]
    pub base_goods: Vec<String>,
    #[serde(default)]
    pub base_goods_quantity: usize,
    pub reaction_rules: Vec<ReactionRuleSpec>,
}

fn default_candidate_goods_k() -> usize { 12 }
