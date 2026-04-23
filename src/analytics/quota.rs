use serde::Serialize;

use crate::analytics::blocks::{BillingBlock, Projection};

const LOW_HISTORY_THRESHOLD: usize = 10;

/// Severity level for quota thresholds.
///
/// Matches ccusage: <50% → Ok, 50–80% → Warn, >80% → Danger.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Ok,
    Warn,
    Danger,
}

/// Derive severity from a fractional percentage (0.0 = 0%, 1.0 = 100%).
///
/// - `pct < 0.5` (or non-finite / negative) → `Ok`
/// - `0.5 <= pct < 0.8` → `Warn`
/// - `pct >= 0.8` → `Danger`
pub fn severity_for_pct(pct: f64) -> Severity {
    if !pct.is_finite() || pct < 0.5 {
        Severity::Ok
    } else if pct < 0.8 {
        Severity::Warn
    } else {
        Severity::Danger
    }
}

/// Quota metadata computed for one billing block against a token limit.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct QuotaMeta {
    pub limit_tokens: i64,
    pub used_tokens: i64,
    pub projected_tokens: i64,
    /// `used / limit`; can exceed 1.0 when over quota.
    pub current_pct: f64,
    /// `projected / limit`; can exceed 1.0 when over quota.
    pub projected_pct: f64,
    /// `limit - used`; negative when over quota.
    pub remaining_tokens: i64,
    pub current_severity: Severity,
    pub projected_severity: Severity,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct QuotaSuggestionLevel {
    pub key: String,
    pub label: String,
    pub limit_tokens: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct QuotaSuggestions {
    pub sample_count: usize,
    pub population_count: usize,
    pub recommended_key: String,
    pub sample_strategy: String,
    pub sample_label: String,
    pub levels: Vec<QuotaSuggestionLevel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Compute quota metadata for `block` against `limit_tokens`.
///
/// Returns `None` when `limit_tokens <= 0` (no valid quota configured).
pub fn compute_quota(
    block: &BillingBlock,
    projection: &Projection,
    limit_tokens: i64,
) -> Option<QuotaMeta> {
    if limit_tokens <= 0 {
        return None;
    }

    // invariant: limit_tokens > 0
    let used_tokens = block.tokens.total();
    let projected_tokens = projection.projected_tokens as i64;
    let denom = limit_tokens as f64;
    let current_pct = used_tokens as f64 / denom;
    let projected_pct = projected_tokens as f64 / denom;
    let remaining_tokens = limit_tokens - used_tokens;

    Some(QuotaMeta {
        limit_tokens,
        used_tokens,
        projected_tokens,
        current_pct,
        projected_pct,
        remaining_tokens,
        current_severity: severity_for_pct(current_pct),
        projected_severity: severity_for_pct(projected_pct),
    })
}

const LIMIT_HIT_SAMPLE_THRESHOLD_FRACTION: f64 = 0.90;

pub fn compute_quota_suggestions(
    blocks: &[BillingBlock],
    configured_limit_tokens: Option<i64>,
) -> Option<QuotaSuggestions> {
    let completed = blocks
        .iter()
        .filter(|block| !block.is_gap && !block.is_active)
        .collect::<Vec<_>>();
    let population_count = completed.len();

    let sample_blocks = configured_limit_tokens
        .filter(|limit| *limit > 0)
        .map(|limit| {
            let threshold = (limit as f64 * LIMIT_HIT_SAMPLE_THRESHOLD_FRACTION).round() as i64;
            completed
                .iter()
                .copied()
                .filter(|block| block.tokens.total() >= threshold)
                .collect::<Vec<_>>()
        })
        .filter(|hits| !hits.is_empty())
        .unwrap_or_else(|| completed.clone());

    let mut totals = sample_blocks
        .iter()
        .map(|block| block.tokens.total())
        .collect::<Vec<_>>();

    if totals.is_empty() {
        return None;
    }

    totals.sort_unstable();
    let sample_count = totals.len();
    let used_limit_hits = sample_count != population_count;

    Some(QuotaSuggestions {
        sample_count,
        population_count,
        recommended_key: "p90".into(),
        sample_strategy: if used_limit_hits {
            "near_limit_hits".into()
        } else {
            "completed_blocks".into()
        },
        sample_label: if used_limit_hits {
            format!("{sample_count} near-limit completed blocks")
        } else {
            format!("{sample_count} completed blocks")
        },
        levels: vec![
            QuotaSuggestionLevel {
                key: "p90".into(),
                label: "P90".into(),
                limit_tokens: nearest_rank(&totals, 0.90),
            },
            QuotaSuggestionLevel {
                key: "p95".into(),
                label: "P95".into(),
                limit_tokens: nearest_rank(&totals, 0.95),
            },
            QuotaSuggestionLevel {
                key: "max".into(),
                label: "Max".into(),
                limit_tokens: *totals.last().unwrap_or(&0),
            },
        ],
        note: (sample_count < LOW_HISTORY_THRESHOLD).then_some(if used_limit_hits {
            "Based on fewer than 10 near-limit completed blocks.".into()
        } else {
            "Based on fewer than 10 completed blocks.".into()
        }),
    })
}

fn nearest_rank(sorted_values: &[i64], quantile: f64) -> i64 {
    let len = sorted_values.len();
    let rank = ((quantile * len as f64).ceil() as usize).clamp(1, len);
    sorted_values[rank - 1]
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analytics::blocks::{BillingBlock, Projection, TokenBreakdown};
    use chrono::{Duration, Utc};

    fn make_block(total_tokens: i64) -> BillingBlock {
        let now = Utc::now();
        BillingBlock {
            start: now,
            end: now + Duration::hours(5),
            tokens: TokenBreakdown {
                input: total_tokens,
                output: 0,
                cache_read: 0,
                cache_creation: 0,
                reasoning_output: 0,
            },
            cost_nanos: 0,
            models: vec![],
            is_active: true,
            entry_count: 1,
            first_timestamp: now,
            last_timestamp: now,
            is_gap: false,
            kind: "block",
        }
    }

    // ── severity_for_pct boundary tests ──────────────────────────────────────

    #[test]
    fn severity_zero_is_ok() {
        assert_eq!(severity_for_pct(0.0), Severity::Ok);
    }

    #[test]
    fn severity_just_below_warn_threshold_is_ok() {
        assert_eq!(severity_for_pct(0.499), Severity::Ok);
    }

    #[test]
    fn severity_at_warn_threshold_is_warn() {
        assert_eq!(severity_for_pct(0.5), Severity::Warn);
    }

    #[test]
    fn severity_just_below_danger_threshold_is_warn() {
        assert_eq!(severity_for_pct(0.799), Severity::Warn);
    }

    #[test]
    fn severity_at_danger_threshold_is_danger() {
        assert_eq!(severity_for_pct(0.8), Severity::Danger);
    }

    #[test]
    fn severity_over_100pct_is_danger() {
        assert_eq!(severity_for_pct(1.5), Severity::Danger);
    }

    #[test]
    fn severity_nan_is_ok() {
        assert_eq!(severity_for_pct(f64::NAN), Severity::Ok);
    }

    #[test]
    fn severity_negative_is_ok() {
        assert_eq!(severity_for_pct(-0.1), Severity::Ok);
    }

    // ── compute_quota tests ───────────────────────────────────────────────────

    #[test]
    fn compute_quota_normal_case() {
        // 500k used, 1M limit, 900k projected → current 50% Warn, projected 90% Danger
        let block = make_block(500_000);
        let proj = Projection {
            projected_cost_nanos: 0,
            projected_tokens: 900_000,
        };
        let q = compute_quota(&block, &proj, 1_000_000).expect("limit > 0 must return Some");

        assert_eq!(q.limit_tokens, 1_000_000);
        assert_eq!(q.used_tokens, 500_000);
        assert_eq!(q.projected_tokens, 900_000);
        assert!((q.current_pct - 0.5).abs() < 1e-9);
        assert!((q.projected_pct - 0.9).abs() < 1e-9);
        assert_eq!(q.remaining_tokens, 500_000);
        assert_eq!(q.current_severity, Severity::Warn);
        assert_eq!(q.projected_severity, Severity::Danger);
    }

    #[test]
    fn compute_quota_limit_zero_returns_none() {
        let block = make_block(500_000);
        let proj = Projection {
            projected_cost_nanos: 0,
            projected_tokens: 900_000,
        };
        assert!(compute_quota(&block, &proj, 0).is_none());
    }

    #[test]
    fn compute_quota_limit_negative_returns_none() {
        let block = make_block(500_000);
        let proj = Projection {
            projected_cost_nanos: 0,
            projected_tokens: 900_000,
        };
        assert!(compute_quota(&block, &proj, -1).is_none());
    }

    #[test]
    fn compute_quota_over_quota() {
        // 1.2M used, 1M limit → remaining = -200_000, severity Danger
        let block = make_block(1_200_000);
        let proj = Projection {
            projected_cost_nanos: 0,
            projected_tokens: 1_200_000,
        };
        let q = compute_quota(&block, &proj, 1_000_000).expect("limit > 0 must return Some");
        assert_eq!(q.remaining_tokens, -200_000);
        assert!(q.current_pct > 1.0);
        assert_eq!(q.current_severity, Severity::Danger);
    }

    #[test]
    fn compute_quota_suggestions_returns_none_without_completed_history() {
        let active = make_block(400_000);
        assert!(compute_quota_suggestions(&[active], None).is_none());
    }

    #[test]
    fn compute_quota_suggestions_single_completed_block_returns_identical_levels() {
        let mut block = make_block(750_000);
        block.is_active = false;

        let suggestions = compute_quota_suggestions(&[block], None).expect("suggestions");
        assert_eq!(suggestions.sample_count, 1);
        assert_eq!(suggestions.population_count, 1);
        assert_eq!(suggestions.recommended_key, "p90");
        assert_eq!(suggestions.sample_strategy, "completed_blocks");
        assert_eq!(suggestions.levels.len(), 3);
        assert!(suggestions.note.is_some());
        assert_eq!(suggestions.levels[0].limit_tokens, 750_000);
        assert_eq!(suggestions.levels[1].limit_tokens, 750_000);
        assert_eq!(suggestions.levels[2].limit_tokens, 750_000);
    }

    #[test]
    fn compute_quota_suggestions_uses_nearest_rank_for_p90_and_p95() {
        let totals = [
            100_000, 200_000, 300_000, 400_000, 500_000, 600_000, 700_000, 800_000, 900_000,
            1_000_000,
        ];
        let blocks = totals
            .iter()
            .map(|total| {
                let mut block = make_block(*total);
                block.is_active = false;
                block
            })
            .collect::<Vec<_>>();

        let suggestions = compute_quota_suggestions(&blocks, None).expect("suggestions");
        assert_eq!(suggestions.note, None);
        assert_eq!(suggestions.levels[0].limit_tokens, 900_000);
        assert_eq!(suggestions.levels[1].limit_tokens, 1_000_000);
        assert_eq!(suggestions.levels[2].limit_tokens, 1_000_000);
    }

    #[test]
    fn compute_quota_suggestions_excludes_active_and_gap_blocks() {
        let mut historical = make_block(400_000);
        historical.is_active = false;

        let mut active = make_block(950_000);
        active.is_active = true;

        let mut gap = make_block(1_200_000);
        gap.is_active = false;
        gap.is_gap = true;

        let suggestions =
            compute_quota_suggestions(&[historical, active, gap], None).expect("suggestions");
        assert_eq!(suggestions.sample_count, 1);
        assert_eq!(suggestions.levels[2].limit_tokens, 400_000);
    }

    #[test]
    fn compute_quota_suggestions_note_only_appears_for_sparse_history() {
        let sparse = (0..9)
            .map(|idx| {
                let mut block = make_block((idx + 1) as i64 * 100_000);
                block.is_active = false;
                block
            })
            .collect::<Vec<_>>();
        let dense = (0..10)
            .map(|idx| {
                let mut block = make_block((idx + 1) as i64 * 100_000);
                block.is_active = false;
                block
            })
            .collect::<Vec<_>>();

        assert!(
            compute_quota_suggestions(&sparse, None)
                .expect("sparse suggestions")
                .note
                .is_some()
        );
        assert!(
            compute_quota_suggestions(&dense, None)
                .expect("dense suggestions")
                .note
                .is_none()
        );
    }

    #[test]
    fn compute_quota_suggestions_prefers_near_limit_completed_blocks() {
        let blocks = [300_000, 890_000, 920_000, 980_000]
            .into_iter()
            .map(|total| {
                let mut block = make_block(total);
                block.is_active = false;
                block
            })
            .collect::<Vec<_>>();

        let suggestions = compute_quota_suggestions(&blocks, Some(1_000_000)).expect("suggestions");

        assert_eq!(suggestions.population_count, 4);
        assert_eq!(suggestions.sample_count, 2);
        assert_eq!(suggestions.sample_strategy, "near_limit_hits");
        assert_eq!(suggestions.sample_label, "2 near-limit completed blocks");
        assert_eq!(suggestions.levels[0].limit_tokens, 980_000);
    }
}
