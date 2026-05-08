use chrono::{Duration, Utc};
use tracing::warn;

use crate::models::{CostForecastSummary, CostRegression, CostTrend, DailyCostPoint};
use crate::scanner::db::query_cost_by_day_tz;
use crate::tz::TzParams;

pub fn compute_cost_forecast(conn: &rusqlite::Connection, tz: &TzParams) -> CostForecastSummary {
    let now = Utc::now();
    let cutoff = (now.date_naive() - Duration::days(29)).to_string();

    let raw_map = match query_cost_by_day_tz(conn, tz, &cutoff) {
        Ok(m) => m,
        Err(e) => {
            warn!("cost_forecast query failed: {e}");
            return CostForecastSummary::default();
        }
    };

    // Zero-filled 30-day array, oldest first.
    let mut days: Vec<DailyCostPoint> = Vec::with_capacity(30);
    for i in 0..30i64 {
        let day = (now.date_naive() - Duration::days(29 - i)).to_string();
        let cost_nanos = *raw_map.get(&day).unwrap_or(&0);
        days.push(DailyCostPoint { day, cost_nanos });
    }

    compute_forecast_from_days(days, now.to_rfc3339())
}

/// Pure math: build the summary from a pre-filled 30-day slice.
/// Extracted so unit tests can drive it without a DB.
pub(crate) fn compute_forecast_from_days(
    days: Vec<DailyCostPoint>,
    generated_at: String,
) -> CostForecastSummary {
    let n = days.len() as i64;
    if n == 0 {
        return CostForecastSummary {
            generated_at,
            ..Default::default()
        };
    }

    // Rolling means.
    let rolling_7d_avg_nanos = if n >= 7 {
        days[(n - 7) as usize..].iter().map(|d| d.cost_nanos).sum::<i64>() / 7
    } else {
        days.iter().map(|d| d.cost_nanos).sum::<i64>() / n
    };
    let rolling_30d_avg_nanos = days.iter().map(|d| d.cost_nanos).sum::<i64>() / n;

    // OLS on non-zero days (require ≥ 7 to fit).
    let nonzero: Vec<(f64, f64)> = days
        .iter()
        .enumerate()
        .filter(|(_, d)| d.cost_nanos > 0)
        .map(|(i, d)| (i as f64, d.cost_nanos as f64))
        .collect();

    let regression = fit_ols(&nonzero);

    let projected_month_nanos = if let Some(ref reg) = regression {
        // Sum regression estimates for the next 30 days (indices n..n+30).
        let total: i64 = (n..n + 30)
            .map(|i| reg.intercept_nanos + reg.slope_nanos_per_day * i)
            .filter(|&v| v > 0)
            .sum();
        total.max(30 * rolling_30d_avg_nanos).max(0)
    } else {
        (30 * rolling_30d_avg_nanos).max(0)
    };

    let trend = classify_trend(&regression, rolling_30d_avg_nanos);

    CostForecastSummary {
        days,
        rolling_7d_avg_nanos,
        rolling_30d_avg_nanos,
        regression,
        projected_month_nanos,
        trend,
        generated_at,
    }
}

fn fit_ols(nonzero: &[(f64, f64)]) -> Option<CostRegression> {
    if nonzero.len() < 7 {
        return None;
    }
    let n = nonzero.len() as f64;
    let mean_x: f64 = nonzero.iter().map(|(x, _)| x).sum::<f64>() / n;
    let mean_y: f64 = nonzero.iter().map(|(_, y)| y).sum::<f64>() / n;
    let ss_xy: f64 = nonzero.iter().map(|(x, y)| (x - mean_x) * (y - mean_y)).sum();
    let ss_xx: f64 = nonzero.iter().map(|(x, _)| (x - mean_x).powi(2)).sum();
    let ss_yy: f64 = nonzero.iter().map(|(_, y)| (y - mean_y).powi(2)).sum();

    if ss_xx < 1.0 {
        return None;
    }
    let slope = ss_xy / ss_xx;
    let intercept = mean_y - slope * mean_x;
    let r_squared = if ss_yy < 1.0 {
        0.0_f32
    } else {
        ((ss_xy * ss_xy) / (ss_xx * ss_yy)).clamp(0.0, 1.0) as f32
    };
    Some(CostRegression {
        slope_nanos_per_day: slope.round() as i64,
        intercept_nanos: intercept.round() as i64,
        r_squared,
        sample_size: nonzero.len() as u32,
    })
}

fn classify_trend(regression: &Option<CostRegression>, rolling_30d_avg_nanos: i64) -> CostTrend {
    match regression {
        None => CostTrend::Insufficient,
        Some(reg) => {
            let daily_avg = rolling_30d_avg_nanos.max(1);
            let slope_pct = reg.slope_nanos_per_day as f64 / daily_avg as f64;
            if slope_pct >= 0.05 {
                CostTrend::Rising
            } else if slope_pct <= -0.05 {
                CostTrend::Falling
            } else {
                CostTrend::Flat
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn days_uniform(nanos: i64, count: usize) -> Vec<DailyCostPoint> {
        (0..count)
            .map(|i| DailyCostPoint {
                day: format!("2024-01-{:02}", i + 1),
                cost_nanos: nanos,
            })
            .collect()
    }

    fn days_linear(start: i64, step: i64, count: usize) -> Vec<DailyCostPoint> {
        (0..count)
            .map(|i| DailyCostPoint {
                day: format!("2024-01-{:02}", i + 1),
                cost_nanos: start + step * i as i64,
            })
            .collect()
    }

    fn sparse_days(count: usize, nonzero: &[(usize, i64)]) -> Vec<DailyCostPoint> {
        let mut v: Vec<DailyCostPoint> = (0..count)
            .map(|i| DailyCostPoint {
                day: format!("2024-01-{:02}", i + 1),
                cost_nanos: 0,
            })
            .collect();
        for &(idx, nanos) in nonzero {
            v[idx].cost_nanos = nanos;
        }
        v
    }

    #[test]
    fn flat_history_flat_trend() {
        let nanos = 1_000_000_000i64;
        let summary = compute_forecast_from_days(days_uniform(nanos, 30), String::new());
        assert_eq!(summary.trend, CostTrend::Flat);
        assert!(summary.regression.is_some());
        let slope = summary.regression.as_ref().unwrap().slope_nanos_per_day;
        assert!(slope.abs() < 1_000, "slope should be ~0, got {slope}");
        let expected_month = 30 * nanos;
        let delta = (summary.projected_month_nanos - expected_month).abs();
        assert!(delta < expected_month / 100, "projected {}", summary.projected_month_nanos);
    }

    #[test]
    fn rising_history_rising_trend() {
        // slope=10M, avg≈155M → slope/avg≈6.5% > 5% threshold
        let summary = compute_forecast_from_days(days_linear(10_000_000, 10_000_000, 30), String::new());
        assert_eq!(summary.trend, CostTrend::Rising);
        let reg = summary.regression.as_ref().expect("should have regression");
        assert!(reg.slope_nanos_per_day > 0);
        assert!(reg.r_squared > 0.95, "r²={}", reg.r_squared);
    }

    #[test]
    fn falling_history_falling_trend() {
        // slope=-20M, avg≈310M → slope/avg≈-6.5% < -5% threshold
        let summary = compute_forecast_from_days(days_linear(600_000_000, -20_000_000, 30), String::new());
        assert_eq!(summary.trend, CostTrend::Falling);
    }

    #[test]
    fn sparse_history_insufficient() {
        let days = sparse_days(30, &[(0, 1_000_000_000), (15, 2_000_000_000), (29, 500_000_000)]);
        let summary = compute_forecast_from_days(days, String::new());
        assert_eq!(summary.trend, CostTrend::Insufficient);
        assert!(summary.regression.is_none());
        assert!(summary.projected_month_nanos >= 0);
    }

    #[test]
    fn all_zero_no_panic() {
        let summary = compute_forecast_from_days(days_uniform(0, 30), String::new());
        assert_eq!(summary.trend, CostTrend::Insufficient);
        assert!(summary.regression.is_none());
        assert_eq!(summary.rolling_7d_avg_nanos, 0);
        assert_eq!(summary.rolling_30d_avg_nanos, 0);
        assert_eq!(summary.projected_month_nanos, 0);
    }

    #[test]
    fn negative_slope_projection_clamped_to_zero() {
        // Very steep decline → projection should never go negative
        let summary = compute_forecast_from_days(days_linear(1_000_000_000, -50_000_000, 30), String::new());
        assert!(summary.projected_month_nanos >= 0);
    }
}
