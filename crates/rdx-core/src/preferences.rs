use crate::math::normalize;

/// Build an aggregated Cobb–Douglas exponent vector beta from per-good alphas
/// against a fixed base good B (numeraire).
///
/// For each k != B:
///   alpha_{kB} = beta_k / (beta_k + beta_B)
/// => beta_k = (alpha_{kB}/(1-alpha_{kB})) * beta_B.
///
/// This construction is inherently cycle-consistent because all ratios are anchored
/// to the same base good.
pub fn beta_from_alpha_to_base(alpha_to_base: &[f64], base: usize, min_alpha: f64) -> Vec<f64> {
    let n = alpha_to_base.len();
    assert!(base < n);

    // set beta_B = 1 for convenience, then scale others by ratio
    let beta_b = 1.0;
    let mut beta = vec![0.0; n];
    beta[base] = beta_b;

    for k in 0..n {
        if k == base { continue; }
        let a = alpha_to_base[k].clamp(min_alpha, 1.0 - min_alpha);
        let ratio = a / (1.0 - a);
        beta[k] = ratio * beta_b;
    }

    normalize(&mut beta);
    beta
}

/// Given full beta, derive the implied pairwise alpha_{AB} for a dyadic (A,B) evaluation:
///
/// alpha_{AB} = beta_A / (beta_A + beta_B)
pub fn alpha_from_beta(beta: &[f64], a: usize, b: usize, min_alpha: f64) -> f64 {
    let ba = beta[a].max(0.0);
    let bb = beta[b].max(0.0);
    let denom = (ba + bb).max(1e-18);
    (ba / denom).clamp(min_alpha, 1.0 - min_alpha)
}

/// Cobb–Douglas utility over n goods.
pub fn cd_utility(beta: &[f64], x: &[f64], min_qty: f64) -> f64 {
    // compute exp(sum beta_k log x_k)
    let mut s = 0.0;
    for (b, &xi) in beta.iter().zip(x.iter()) {
        s += b * (xi.max(min_qty)).ln();
    }
    s.exp()
}
