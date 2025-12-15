use crate::model::{Agent};
use crate::preferences::{cd_utility, alpha_from_beta};
use crate::pareto_oracle::{ParetoOracle, CobbDouglasWalrasOracle};

#[derive(Clone, Debug)]
pub struct TradeCandidate {
    pub good_a: usize,
    pub good_b: usize,
    pub q_ab: f64,
    pub delta_a_i: f64,
    pub delta_b_i: f64,
    pub delta_u_i: f64,
    pub delta_u_j: f64,
}

/// Compute a Cobb–Douglas marginal rate of substitution (price ratio) for good k vs base:
/// MRS_{k,base} = (beta_k/beta_base) * (x_base/x_k).
fn mrs_to_base(beta: &[f64], x: &[f64], k: usize, base: usize, min_qty: f64) -> f64 {
    let bk = beta[k].max(0.0);
    let bb = beta[base].max(1e-18);
    let xb = x[base].max(min_qty);
    let xk = x[k].max(min_qty);
    (bk / bb) * (xb / xk)
}

/// Select a pruned candidate set of goods (excluding base) for a dyad (i,j).
///
/// Heuristic: pick goods with largest disagreement in log(MRS_{k,base}) between agents.
pub fn candidate_goods_pruned(
    i: &Agent,
    j: &Agent,
    base: usize,
    k: usize,
    min_qty: f64,
) -> Vec<usize> {
    let n = i.e.len();
    let mut scored: Vec<(usize, f64)> = Vec::with_capacity(n.saturating_sub(1));

    for g in 0..n {
        if g == base { continue; }
        let mi = mrs_to_base(&i.beta, &i.e, g, base, min_qty).max(1e-18).ln();
        let mj = mrs_to_base(&j.beta, &j.e, g, base, min_qty).max(1e-18).ln();
        scored.push((g, (mi - mj).abs()));
    }

    scored.sort_by(|a,b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(k.min(scored.len()));
    scored.into_iter().map(|(g,_)| g).collect()
}

/// Evaluate a single ordered good-pair (A,B) P2P exchange candidate between agents i and j.
///
/// - Uses dyadic Cobb–Douglas alphas inferred from each agent's beta (or alpha_to_base when B is base).
/// - Calls the oracle to get the Pareto-optimal two-good allocation.
/// - Scores by requiring both delta_u_i > 0 and delta_u_j > 0 (strict improvement).
pub fn evaluate_pairwise_trade(
    i: &Agent,
    j: &Agent,
    good_a: usize,
    good_b: usize,
    base_good: usize,
    min_qty: f64,
    oracle_iters: usize,
    oracle: &dyn ParetoOracle,
) -> Option<TradeCandidate> {
    if good_a == good_b { return None; }
    if good_a >= i.e.len() || good_b >= i.e.len() { return None; }
    if i.e.len() != j.e.len() { return None; }

    // Extract quantities (only A,B change; other goods fixed)
    let ai = i.e[good_a];
    let bi = i.e[good_b];
    let aj = j.e[good_a];
    let bj = j.e[good_b];

    // Determine alpha parameters for dyadic utility u(a,b)=a^alpha b^(1-alpha)
    let min_alpha = 1e-6;
    let alpha_i = if good_b == base_good && i.alpha_to_base.len() == i.e.len() {
        i.alpha_to_base[good_a].clamp(min_alpha, 1.0 - min_alpha)
    } else {
        alpha_from_beta(&i.beta, good_a, good_b, min_alpha)
    };
    let alpha_j = if good_b == base_good && j.alpha_to_base.len() == j.e.len() {
        j.alpha_to_base[good_a].clamp(min_alpha, 1.0 - min_alpha)
    } else {
        alpha_from_beta(&j.beta, good_a, good_b, min_alpha)
    };

    let ex = oracle.solve_two_good_exchange(alpha_i, ai, bi, alpha_j, aj, bj, min_qty, oracle_iters);

    // Build counterfactual post-trade full bundles (n goods), changing only A,B
    let mut xi_post = i.e.clone();
    xi_post[good_a] = ex.ai_post;
    xi_post[good_b] = ex.bi_post;

    let mut xj_post = j.e.clone();
    xj_post[good_a] = ex.aj_post;
    xj_post[good_b] = ex.bj_post;

    // Compute utility deltas using full n-good CD utility
    let ui0 = cd_utility(&i.beta, &i.e, min_qty);
    let uj0 = cd_utility(&j.beta, &j.e, min_qty);

    let ui1 = cd_utility(&i.beta, &xi_post, min_qty);
    let uj1 = cd_utility(&j.beta, &xj_post, min_qty);

    let delta_u_i = ui1 - ui0;
    let delta_u_j = uj1 - uj0;

    if delta_u_i > 0.0 && delta_u_j > 0.0 {
        Some(TradeCandidate {
            good_a,
            good_b,
            q_ab: ex.q_ab,
            delta_a_i: ex.ai_post - ai,
            delta_b_i: ex.bi_post - bi,
            delta_u_i,
            delta_u_j,
        })
    } else {
        None
    }
}

/// Evaluate every good A against the base good B for a P2P encounter, and return the best candidate.
pub fn best_trade_against_base(
    i: &Agent,
    j: &Agent,
    base_good: usize,
    min_qty: f64,
    oracle_iters: usize,
    oracle: &dyn ParetoOracle,
) -> Option<TradeCandidate> {
    let n = i.e.len();
    if n != j.e.len() { return None; }

    let mut best: Option<TradeCandidate> = None;

    for a in 0..n {
        if a == base_good { continue; }
        if let Some(cand) = evaluate_pairwise_trade(
            i, j, a, base_good, base_good, min_qty, oracle_iters, oracle
        ) {
            let score = cand.delta_u_i.min(cand.delta_u_j); // conservative
            match &best {
                None => best = Some(cand),
                Some(bc) => {
                    let bscore = bc.delta_u_i.min(bc.delta_u_j);
                    if score > bscore {
                        best = Some(cand);
                    }
                }
            }
        }
    }

    best
}

/// Evaluate all ordered pairs (A,B) within a pruned candidate set (plus base good), return best.
///
/// This “apply logic to all range of goods vector” while remaining tractable
/// by pruning candidate goods per encounter.
pub fn best_trade_over_all_pairs_pruned(
    i: &Agent,
    j: &Agent,
    base_good: usize,
    candidate_goods_k: usize,
    min_qty: f64,
    oracle_iters: usize,
    oracle: &dyn ParetoOracle,
) -> Option<TradeCandidate> {
    let n = i.e.len();
    if n != j.e.len() { return None; }

    let mut cand_goods = candidate_goods_pruned(i, j, base_good, candidate_goods_k, min_qty);
    // Always include base good in the candidate pool
    cand_goods.push(base_good);

    let mut best: Option<TradeCandidate> = None;

    for &a in cand_goods.iter() {
        for &b in cand_goods.iter() {
            if a == b { continue; }
            if let Some(cand) = evaluate_pairwise_trade(i, j, a, b, base_good, min_qty, oracle_iters, oracle) {
                let score = cand.delta_u_i.min(cand.delta_u_j);
                match &best {
                    None => best = Some(cand),
                    Some(bc) => {
                        let bscore = bc.delta_u_i.min(bc.delta_u_j);
                        if score > bscore {
                            best = Some(cand);
                        }
                    }
                }
            }
        }
    }
    best
}

/// Execute a trade candidate by mutating both agents' endowments for goods (A,B).
pub fn apply_trade(i: &mut Agent, j: &mut Agent, cand: &TradeCandidate, min_qty: f64) {
    let a = cand.good_a;
    let b = cand.good_b;

    // Update i; j gets opposite deltas due to conservation of A and B within the dyad.
    i.e[a] = (i.e[a] + cand.delta_a_i).max(min_qty);
    i.e[b] = (i.e[b] + cand.delta_b_i).max(min_qty);

    j.e[a] = (j.e[a] - cand.delta_a_i).max(min_qty);
    j.e[b] = (j.e[b] - cand.delta_b_i).max(min_qty);
}

/// Convenience: build default oracle
pub fn default_oracle() -> CobbDouglasWalrasOracle {
    CobbDouglasWalrasOracle
}
