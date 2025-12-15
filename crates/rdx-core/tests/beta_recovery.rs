use rand::prelude::*;
use rdx_core::preferences::beta_from_alpha_to_base;
use rdx_core::math::normalize;

fn dirichlet_like(rng: &mut StdRng, n: usize) -> Vec<f64> {
    let mut v: Vec<f64> = (0..n)
        .map(|_| {
            let u: f64 = rng.gen::<f64>().max(1e-12);
            (-u.ln()).max(1e-12)
        })
        .collect();
    normalize(&mut v);
    v
}

#[test]
fn beta_alpha_base_roundtrip() {
    let mut rng = StdRng::seed_from_u64(123);
    let n = 15;
    let base = 0;

    for _ in 0..200 {
        let beta = dirichlet_like(&mut rng, n);
        let beta_b = beta[base].max(1e-12);

        // build alpha_to_base from beta
        let mut alpha_to_base = vec![0.5; n];
        for k in 0..n {
            if k == base { continue; }
            let a = beta[k] / (beta[k] + beta_b);
            alpha_to_base[k] = a.clamp(1e-6, 1.0 - 1e-6);
        }

        // recover beta'
        let beta2 = beta_from_alpha_to_base(&alpha_to_base, base, 1e-6);

        // Compare up to tolerance (they should match closely after normalization)
        let err: f64 = beta.iter().zip(beta2.iter()).map(|(x,y)| (x-y).abs()).sum();
        assert!(err < 1e-6, "err too large: {err}");
    }
}
