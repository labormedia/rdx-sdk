pub fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(x,y)| x*y).sum()
}

pub fn normalize(v: &mut [f64]) {
    let s: f64 = v.iter().sum();
    if s > 0.0 {
        for x in v.iter_mut() { *x /= s; }
    }
}

pub fn clamp01(x: f64) -> f64 {
    if x < 0.0 { 0.0 } else if x > 1.0 { 1.0 } else { x }
}

pub fn safe_log(x: f64, min_qty: f64) -> f64 {
    (x.max(min_qty)).ln()
}
