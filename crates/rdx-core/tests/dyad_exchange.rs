use rdx_core::pareto_oracle::{ParetoOracle, CobbDouglasWalrasOracle};

#[test]
fn walras_oracle_clears_market() {
    let oracle = CobbDouglasWalrasOracle;

    let alpha_i = 0.7;
    let alpha_j = 0.2;

    let ai = 2.0;
    let bi = 1.0;
    let aj = 1.5;
    let bj = 3.0;

    let ex = oracle.solve_two_good_exchange(alpha_i, ai, bi, alpha_j, aj, bj, 1e-9, 80);

    // Conservation
    let ta0 = ai + aj;
    let tb0 = bi + bj;

    let ta1 = ex.ai_post + ex.aj_post;
    let tb1 = ex.bi_post + ex.bj_post;

    assert!((ta0 - ta1).abs() < 1e-6);
    assert!((tb0 - tb1).abs() < 1e-6);
    assert!(ex.q_ab > 0.0);
}
