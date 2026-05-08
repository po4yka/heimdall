/// Pearson correlation coefficient between two equal-length slices.
///
/// Returns `None` if:
/// - fewer than 5 paired points are supplied, or
/// - either series has zero variance (constant values).
pub fn pearson(xs: &[f64], ys: &[f64]) -> Option<f32> {
    let n = xs.len().min(ys.len());
    if n < 5 {
        return None;
    }

    let n_f = n as f64;
    let mean_x = xs[..n].iter().sum::<f64>() / n_f;
    let mean_y = ys[..n].iter().sum::<f64>() / n_f;

    let mut ss_xy = 0.0f64;
    let mut ss_xx = 0.0f64;
    let mut ss_yy = 0.0f64;

    for i in 0..n {
        let dx = xs[i] - mean_x;
        let dy = ys[i] - mean_y;
        ss_xy += dx * dy;
        ss_xx += dx * dx;
        ss_yy += dy * dy;
    }

    if ss_xx == 0.0 || ss_yy == 0.0 {
        return None;
    }

    let r = ss_xy / (ss_xx.sqrt() * ss_yy.sqrt());
    Some(r.clamp(-1.0, 1.0) as f32)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pearson_perfect_positive() {
        let xs: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let ys: Vec<f64> = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let r = pearson(&xs, &ys).expect("should return Some");
        assert!((r - 1.0_f32).abs() < 0.001, "expected r ≈ 1.0, got {r}");
    }

    #[test]
    fn pearson_perfect_negative() {
        let xs: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let ys: Vec<f64> = vec![10.0, 8.0, 6.0, 4.0, 2.0];
        let r = pearson(&xs, &ys).expect("should return Some");
        assert!((r + 1.0_f32).abs() < 0.001, "expected r ≈ -1.0, got {r}");
    }

    #[test]
    fn pearson_under_min_sample_returns_none() {
        let xs: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0];
        let ys: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0];
        assert!(pearson(&xs, &ys).is_none(), "4 points should return None");
    }

    #[test]
    fn pearson_zero_variance_returns_none() {
        let xs: Vec<f64> = vec![5.0, 5.0, 5.0, 5.0, 5.0];
        let ys: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert!(
            pearson(&xs, &ys).is_none(),
            "constant xs should return None"
        );
    }
}
