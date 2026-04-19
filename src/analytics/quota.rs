use serde::Serialize;

use crate::analytics::blocks::{BillingBlock, Projection};

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
}
