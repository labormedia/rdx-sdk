use crate::math::clamp01;

/// Output of dyadic exchange oracle for two goods (A,B) between two agents i and j.
#[derive(Clone, Debug)]
pub struct DyadExchange {
    /// Implied exchange rate / quotient Q_AB (interpreted here as price ratio pA/pB).
    pub q_ab: f64,
    /// Post-trade quantities for agent i: (a_i', b_i')
    pub ai_post: f64,
    pub bi_post: f64,
    /// Post-trade quantities for agent j: (a_j', b_j')
    pub aj_post: f64,
    pub bj_post: f64,
}

/// Trait boundary representing the endogenous functions:
///
/// Q_AB = f(alpha_i, a_i, b_i, alpha_j, a_j, b_j)
/// (e1_AB, e2_AB) = Exchange(..., Q_AB)
///
/// Implementations must return a Pareto-optimal allocation for the two-good exchange.
pub trait ParetoOracle: Send + Sync {
    fn solve_two_good_exchange(
        &self,
        alpha_i: f64, ai: f64, bi: f64,
        alpha_j: f64, aj: f64, bj: f64,
        min_qty: f64,
        iters: usize,
    ) -> DyadExchange;
}

/// Default implementation: compute a Walrasian equilibrium for a 2-good exchange economy
/// with Cobb–Douglas preferences:
///   u_i = a^{alpha_i} b^{1-alpha_i}
///   u_j = a^{alpha_j} b^{1-alpha_j}
///
/// This equilibrium is Pareto efficient. We find price ratio p = pA/pB by bisection on
/// excess demand for good A, then compute final allocations via Marshallian demands.
pub struct CobbDouglasWalrasOracle;

impl CobbDouglasWalrasOracle {
    fn excess_demand_a(alpha_i: f64, ai: f64, bi: f64, alpha_j: f64, aj: f64, bj: f64, p: f64) -> f64 {
        // numeraire pB = 1, so prices: pA = p, pB = 1.
        let wi = p * ai + bi;
        let wj = p * aj + bj;

        let di_a = alpha_i * wi / p;
        let dj_a = alpha_j * wj / p;

        (di_a + dj_a) - (ai + aj)
    }
}

impl ParetoOracle for CobbDouglasWalrasOracle {
    fn solve_two_good_exchange(
        &self,
        alpha_i: f64, ai: f64, bi: f64,
        alpha_j: f64, aj: f64, bj: f64,
        min_qty: f64,
        iters: usize,
    ) -> DyadExchange {
        // Guard rails
        let ai = ai.max(min_qty);
        let bi = bi.max(min_qty);
        let aj = aj.max(min_qty);
        let bj = bj.max(min_qty);

        let a_i = clamp01(alpha_i);
        let a_j = clamp01(alpha_j);

        // Bracket pA/pB. We search p in [p_lo, p_hi] such that excess demand changes sign.
        let mut p_lo: f64 = 1e-6;
        let mut p_hi: f64 = 1e6;

        // Bisection on p. (Excess demand is decreasing in p.)
        for _ in 0..iters {
            let p_mid = (p_lo * p_hi).sqrt(); // geometric mid improves scaling across magnitudes
            let z = Self::excess_demand_a(a_i, ai, bi, a_j, aj, bj, p_mid);
            if z > 0.0 {
                // demand > supply => p too low
                p_lo = p_mid;
            } else {
                // demand < supply => p too high
                p_hi = p_mid;
            }
        }
        let p = (p_lo * p_hi).sqrt();

        // Compute allocations at p, pB=1
        let wi = p * ai + bi;
        let wj = p * aj + bj;

        let ai_post = (a_i * wi / p).max(min_qty);
        let bi_post = ((1.0 - a_i) * wi).max(min_qty);

        let aj_post = (a_j * wj / p).max(min_qty);
        let bj_post = ((1.0 - a_j) * wj).max(min_qty);

        DyadExchange { q_ab: p, ai_post, bi_post, aj_post, bj_post }
    }
}
