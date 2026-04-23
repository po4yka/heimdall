use chrono::{DateTime, Duration, Utc};
use serde::Serialize;

use crate::analytics::blocks::BillingBlock;
use crate::analytics::burn_rate::{BurnRateConfig, BurnRateTier, tier as burn_rate_tier};

const LIMIT_HIT_THRESHOLD_FRACTION: f64 = 0.90;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PredictiveInsights {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rolling_hour_burn: Option<RollingBurnRate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub historical_envelope: Option<HistoricalEnvelope>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_hit_analysis: Option<LimitHitAnalysis>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RollingBurnRate {
    pub tokens_per_min: f64,
    pub cost_per_hour_nanos: i64,
    pub coverage_minutes: i64,
    pub tier: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct IntegerPercentiles {
    pub average: i64,
    pub p50: i64,
    pub p75: i64,
    pub p90: i64,
    pub p95: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct FloatPercentiles {
    pub average: f64,
    pub p50: f64,
    pub p75: f64,
    pub p90: f64,
    pub p95: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct HistoricalEnvelope {
    pub sample_count: usize,
    pub tokens: IntegerPercentiles,
    pub cost_usd: FloatPercentiles,
    pub turns: IntegerPercentiles,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct LimitHitAnalysis {
    pub sample_count: usize,
    pub hit_count: usize,
    pub hit_rate: f64,
    pub threshold_tokens: i64,
    pub threshold_percent: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_current_hit: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_projected_hit: Option<bool>,
    pub risk_level: String,
    pub summary_label: String,
}

pub fn compute_predictive_insights(
    blocks: &[BillingBlock],
    token_limit: Option<i64>,
    active_projected_tokens: Option<i64>,
    now: DateTime<Utc>,
) -> Option<PredictiveInsights> {
    let completed = completed_blocks(blocks);
    let active_block = blocks.iter().find(|block| block.is_active && !block.is_gap);
    let rolling_hour_burn = compute_rolling_hour_burn(blocks, now);
    let historical_envelope = compute_historical_envelope(&completed);
    let limit_hit_analysis = compute_limit_hit_analysis(
        &completed,
        token_limit,
        active_block,
        active_projected_tokens,
    );

    let insights = PredictiveInsights {
        rolling_hour_burn,
        historical_envelope,
        limit_hit_analysis,
    };

    (insights.rolling_hour_burn.is_some()
        || insights.historical_envelope.is_some()
        || insights.limit_hit_analysis.is_some())
    .then_some(insights)
}

pub fn compute_rolling_hour_burn(
    blocks: &[BillingBlock],
    now: DateTime<Utc>,
) -> Option<RollingBurnRate> {
    let window_start = now - Duration::hours(1);
    let mut total_tokens = 0.0;
    let mut total_cost_nanos = 0.0;
    let mut coverage_minutes = 0.0;

    for block in blocks.iter().filter(|block| !block.is_gap) {
        let actual_start = block.first_timestamp;
        let actual_end = if block.is_active {
            now
        } else {
            block.last_timestamp
        };
        if actual_end <= window_start || actual_end <= actual_start {
            continue;
        }

        let overlap_start = actual_start.max(window_start);
        let overlap_end = actual_end.min(now);
        let overlap_seconds = (overlap_end - overlap_start).num_seconds();
        if overlap_seconds <= 0 {
            continue;
        }

        let actual_seconds = (actual_end - actual_start).num_seconds();
        if actual_seconds <= 0 {
            continue;
        }

        let fraction = overlap_seconds as f64 / actual_seconds as f64;
        total_tokens += block.tokens.total() as f64 * fraction;
        total_cost_nanos += block.cost_nanos as f64 * fraction;
        coverage_minutes += overlap_seconds as f64 / 60.0;
    }

    if total_tokens <= 0.0 && total_cost_nanos <= 0.0 {
        return None;
    }

    let tokens_per_min = total_tokens / 60.0;
    let cost_per_hour_nanos = total_cost_nanos.round() as i64;
    let tier = match burn_rate_tier(tokens_per_min, &BurnRateConfig::default()) {
        BurnRateTier::Normal => "normal",
        BurnRateTier::Moderate => "moderate",
        BurnRateTier::High => "high",
    };

    Some(RollingBurnRate {
        tokens_per_min,
        cost_per_hour_nanos,
        coverage_minutes: coverage_minutes.round() as i64,
        tier: tier.into(),
    })
}

pub fn compute_historical_envelope(blocks: &[BillingBlock]) -> Option<HistoricalEnvelope> {
    if blocks.is_empty() {
        return None;
    }

    let tokens = blocks
        .iter()
        .map(|block| block.tokens.total())
        .collect::<Vec<_>>();
    let costs = blocks
        .iter()
        .map(|block| block.cost_nanos as f64 / 1_000_000_000.0)
        .collect::<Vec<_>>();
    let turns = blocks
        .iter()
        .map(|block| i64::from(block.entry_count))
        .collect::<Vec<_>>();

    Some(HistoricalEnvelope {
        sample_count: blocks.len(),
        tokens: integer_percentiles(&tokens),
        cost_usd: float_percentiles(&costs),
        turns: integer_percentiles(&turns),
    })
}

pub fn compute_limit_hit_analysis(
    completed_blocks: &[BillingBlock],
    token_limit: Option<i64>,
    active_block: Option<&BillingBlock>,
    active_projected_tokens: Option<i64>,
) -> Option<LimitHitAnalysis> {
    let limit_tokens = token_limit.filter(|limit| *limit > 0)?;
    if completed_blocks.is_empty() {
        return None;
    }

    let threshold_tokens = (limit_tokens as f64 * LIMIT_HIT_THRESHOLD_FRACTION).round() as i64;
    let hit_count = completed_blocks
        .iter()
        .filter(|block| block.tokens.total() >= threshold_tokens)
        .count();
    let sample_count = completed_blocks.len();
    let hit_rate = hit_count as f64 / sample_count as f64;
    let active_current_hit = active_block.map(|block| block.tokens.total() >= threshold_tokens);
    let active_projected_hit = active_projected_tokens.map(|tokens| tokens >= threshold_tokens);

    let risk_level = if active_projected_hit == Some(true) || hit_rate >= 0.5 {
        "high"
    } else if active_current_hit == Some(true) || hit_rate >= 0.2 {
        "elevated"
    } else {
        "low"
    };

    let mut summary_label = format!(
        "{} of {} completed blocks reached {:.0}% of the configured limit",
        hit_count,
        sample_count,
        LIMIT_HIT_THRESHOLD_FRACTION * 100.0
    );
    if active_projected_hit == Some(true) {
        summary_label.push_str(" · active block is on pace to join them");
    } else if active_current_hit == Some(true) {
        summary_label.push_str(" · active block is already near that threshold");
    }

    Some(LimitHitAnalysis {
        sample_count,
        hit_count,
        hit_rate,
        threshold_tokens,
        threshold_percent: LIMIT_HIT_THRESHOLD_FRACTION * 100.0,
        active_current_hit,
        active_projected_hit,
        risk_level: risk_level.into(),
        summary_label,
    })
}

pub(crate) fn completed_blocks(blocks: &[BillingBlock]) -> Vec<BillingBlock> {
    blocks
        .iter()
        .filter(|block| !block.is_gap && !block.is_active)
        .cloned()
        .collect()
}

fn integer_percentiles(values: &[i64]) -> IntegerPercentiles {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    IntegerPercentiles {
        average: average_i64(&sorted),
        p50: nearest_rank_i64(&sorted, 0.50),
        p75: nearest_rank_i64(&sorted, 0.75),
        p90: nearest_rank_i64(&sorted, 0.90),
        p95: nearest_rank_i64(&sorted, 0.95),
    }
}

fn float_percentiles(values: &[f64]) -> FloatPercentiles {
    let mut sorted = values.to_vec();
    sorted.sort_by(|lhs, rhs| lhs.partial_cmp(rhs).unwrap_or(std::cmp::Ordering::Equal));
    FloatPercentiles {
        average: average_f64(&sorted),
        p50: nearest_rank_f64(&sorted, 0.50),
        p75: nearest_rank_f64(&sorted, 0.75),
        p90: nearest_rank_f64(&sorted, 0.90),
        p95: nearest_rank_f64(&sorted, 0.95),
    }
}

fn nearest_rank_i64(sorted_values: &[i64], quantile: f64) -> i64 {
    let len = sorted_values.len();
    let rank = ((quantile * len as f64).ceil() as usize).clamp(1, len);
    sorted_values[rank - 1]
}

fn nearest_rank_f64(sorted_values: &[f64], quantile: f64) -> f64 {
    let len = sorted_values.len();
    let rank = ((quantile * len as f64).ceil() as usize).clamp(1, len);
    sorted_values[rank - 1]
}

fn average_i64(values: &[i64]) -> i64 {
    if values.is_empty() {
        return 0;
    }
    (values.iter().sum::<i64>() as f64 / values.len() as f64).round() as i64
}

fn average_f64(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analytics::blocks::{BillingBlock, TokenBreakdown};

    fn block(
        now: DateTime<Utc>,
        total_tokens: i64,
        cost_nanos: i64,
        entry_count: u32,
        active: bool,
        first_offset_minutes: i64,
        last_offset_minutes: i64,
    ) -> BillingBlock {
        let first_timestamp = now + Duration::minutes(first_offset_minutes);
        let last_timestamp = now + Duration::minutes(last_offset_minutes);
        BillingBlock {
            start: first_timestamp,
            end: if active {
                now + Duration::minutes(120)
            } else {
                last_timestamp
            },
            tokens: TokenBreakdown {
                input: total_tokens,
                output: 0,
                cache_read: 0,
                cache_creation: 0,
                reasoning_output: 0,
            },
            cost_nanos,
            models: vec!["claude-sonnet-4-5".into()],
            is_active: active,
            entry_count,
            first_timestamp,
            last_timestamp,
            is_gap: false,
            kind: "block",
        }
    }

    #[test]
    fn rolling_hour_burn_returns_none_without_recent_usage() {
        let now = Utc::now();
        let stale_block = block(now, 100_000, 2_000_000_000, 10, false, -180, -120);
        assert!(compute_rolling_hour_burn(&[stale_block], now).is_none());
    }

    #[test]
    fn rolling_hour_burn_prorates_overlap_with_last_hour() {
        let now = Utc::now();
        let spanning_block = block(now, 120_000, 3_600_000_000, 12, false, -90, -30);
        let burn = compute_rolling_hour_burn(&[spanning_block], now).expect("burn");

        assert!((burn.tokens_per_min - 1000.0).abs() < 0.5);
        assert_eq!(burn.cost_per_hour_nanos, 1_800_000_000);
        assert_eq!(burn.coverage_minutes, 30);
    }

    #[test]
    fn historical_envelope_uses_completed_blocks_only() {
        let now = Utc::now();
        let completed = vec![
            block(now, 10, 1_000_000_000, 2, false, -180, -160),
            block(now, 20, 2_000_000_000, 4, false, -140, -120),
            block(now, 30, 3_000_000_000, 6, false, -100, -80),
            block(now, 40, 4_000_000_000, 8, false, -60, -40),
        ];
        let envelope = compute_historical_envelope(&completed).expect("envelope");

        assert_eq!(envelope.sample_count, 4);
        assert_eq!(envelope.tokens.p50, 20);
        assert_eq!(envelope.tokens.p90, 40);
        assert_eq!(envelope.cost_usd.p75, 3.0);
        assert_eq!(envelope.turns.p95, 8);
    }

    #[test]
    fn limit_hit_analysis_tracks_completed_and_active_block_risk() {
        let now = Utc::now();
        let completed = vec![
            block(now, 900, 0, 2, false, -180, -160),
            block(now, 1000, 0, 3, false, -140, -120),
            block(now, 300, 0, 1, false, -100, -90),
        ];
        let active = block(now, 850, 0, 4, true, -20, 0);
        let analysis =
            compute_limit_hit_analysis(&completed, Some(1_000), Some(&active), Some(950))
                .expect("analysis");

        assert_eq!(analysis.hit_count, 2);
        assert_eq!(analysis.sample_count, 3);
        assert_eq!(analysis.active_current_hit, Some(false));
        assert_eq!(analysis.active_projected_hit, Some(true));
        assert_eq!(analysis.risk_level, "high");
    }

    #[test]
    fn predictive_insights_returns_none_without_any_signal() {
        assert!(compute_predictive_insights(&[], None, None, Utc::now()).is_none());
    }
}
