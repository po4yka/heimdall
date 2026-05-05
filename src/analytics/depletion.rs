use crate::analytics::blocks::{BillingBlock, BurnRate};
use crate::analytics::quota::{QuotaMeta, Severity, severity_for_pct};
use crate::models::{DepletionForecast, DepletionForecastSignal};
use chrono::{DateTime, Duration, Utc};

const BILLING_BLOCK_KIND: &str = "billing_block";
const PRIMARY_WINDOW_KIND: &str = "primary_window";
const SECONDARY_WINDOW_KIND: &str = "secondary_window";

#[derive(Debug, Clone)]
struct RankedSignal {
    signal: DepletionForecastSignal,
    pressure_percent: f64,
    reset_sort_key: i64,
    priority_rank: u8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TokenRunoutForecast {
    pub runout_in_minutes: i64,
    pub runout_at: DateTime<Utc>,
    pub will_run_out_before_reset: bool,
}

pub fn estimate_token_runout(
    block: &BillingBlock,
    quota: &QuotaMeta,
    rate: Option<BurnRate>,
    now: DateTime<Utc>,
) -> Option<TokenRunoutForecast> {
    if quota.remaining_tokens <= 0 {
        return Some(TokenRunoutForecast {
            runout_in_minutes: 0,
            runout_at: now,
            will_run_out_before_reset: true,
        });
    }

    let rate = rate?;
    if !rate.tokens_per_min.is_finite() || rate.tokens_per_min <= 0.0 {
        return None;
    }

    let seconds_to_runout = ((quota.remaining_tokens as f64 / rate.tokens_per_min) * 60.0).ceil();
    if !seconds_to_runout.is_finite()
        || seconds_to_runout < 0.0
        || seconds_to_runout > i64::MAX as f64
    {
        return None;
    }

    let seconds = seconds_to_runout as i64;
    let runout_at = now + Duration::seconds(seconds);
    Some(TokenRunoutForecast {
        runout_in_minutes: ((seconds + 59) / 60).max(0),
        runout_at,
        will_run_out_before_reset: runout_at <= block.end,
    })
}

pub fn billing_block_signal(
    title: &str,
    used_percent: f64,
    projected_percent: Option<f64>,
    remaining_tokens: Option<i64>,
    remaining_percent: Option<f64>,
    end_time: Option<String>,
) -> DepletionForecastSignal {
    billing_block_signal_with_runout(
        title,
        used_percent,
        projected_percent,
        remaining_tokens,
        remaining_percent,
        end_time,
        None,
    )
}

pub fn billing_block_signal_with_runout(
    title: &str,
    used_percent: f64,
    projected_percent: Option<f64>,
    remaining_tokens: Option<i64>,
    remaining_percent: Option<f64>,
    end_time: Option<String>,
    runout: Option<TokenRunoutForecast>,
) -> DepletionForecastSignal {
    DepletionForecastSignal {
        kind: BILLING_BLOCK_KIND.into(),
        title: title.into(),
        used_percent,
        projected_percent,
        remaining_tokens,
        remaining_percent,
        resets_in_minutes: None,
        pace_label: Some(pace_label_for_remaining_percent(
            remaining_percent.unwrap_or(0.0),
        )),
        end_time,
        runout_in_minutes: runout.map(|value| value.runout_in_minutes),
        runout_at: runout.map(|value| value.runout_at.to_rfc3339()),
        will_run_out_before_reset: runout.map(|value| value.will_run_out_before_reset),
    }
}

pub fn primary_window_signal(
    used_percent: f64,
    remaining_percent: Option<f64>,
    resets_in_minutes: Option<i64>,
    pace_label: Option<String>,
    end_time: Option<String>,
) -> DepletionForecastSignal {
    DepletionForecastSignal {
        kind: PRIMARY_WINDOW_KIND.into(),
        title: "Primary window".into(),
        used_percent,
        projected_percent: None,
        remaining_tokens: None,
        remaining_percent,
        resets_in_minutes,
        pace_label: Some(
            pace_label.unwrap_or_else(|| {
                pace_label_for_remaining_percent(remaining_percent.unwrap_or(0.0))
            }),
        ),
        end_time,
        runout_in_minutes: None,
        runout_at: None,
        will_run_out_before_reset: None,
    }
}

pub fn secondary_window_signal(
    used_percent: f64,
    remaining_percent: Option<f64>,
    resets_in_minutes: Option<i64>,
    pace_label: Option<String>,
    end_time: Option<String>,
) -> DepletionForecastSignal {
    DepletionForecastSignal {
        kind: SECONDARY_WINDOW_KIND.into(),
        title: "Secondary window".into(),
        used_percent,
        projected_percent: None,
        remaining_tokens: None,
        remaining_percent,
        resets_in_minutes,
        pace_label: Some(
            pace_label.unwrap_or_else(|| {
                pace_label_for_remaining_percent(remaining_percent.unwrap_or(0.0))
            }),
        ),
        end_time,
        runout_in_minutes: None,
        runout_at: None,
        will_run_out_before_reset: None,
    }
}

pub fn build_depletion_forecast<I>(signals: I) -> Option<DepletionForecast>
where
    I: IntoIterator<Item = DepletionForecastSignal>,
{
    let mut ranked = signals.into_iter().map(rank_signal).collect::<Vec<_>>();

    if ranked.is_empty() {
        return None;
    }

    ranked.sort_by(compare_ranked_signals);
    let primary = ranked.remove(0);
    let severity = severity_label(severity_for_primary(&primary.signal)).to_string();
    let summary_label = summary_label(&primary.signal);
    let secondary_signals = ranked
        .into_iter()
        .map(|ranked| ranked.signal)
        .collect::<Vec<_>>();

    Some(DepletionForecast {
        primary_signal: primary.signal,
        secondary_signals,
        summary_label,
        severity,
        note: None,
    })
}

fn rank_signal(signal: DepletionForecastSignal) -> RankedSignal {
    RankedSignal {
        pressure_percent: signal.projected_percent.unwrap_or(signal.used_percent),
        reset_sort_key: reset_sort_key(&signal),
        priority_rank: signal_priority(&signal.kind),
        signal,
    }
}

fn compare_ranked_signals(lhs: &RankedSignal, rhs: &RankedSignal) -> std::cmp::Ordering {
    rhs.pressure_percent
        .partial_cmp(&lhs.pressure_percent)
        .unwrap_or(std::cmp::Ordering::Equal)
        .then_with(|| lhs.reset_sort_key.cmp(&rhs.reset_sort_key))
        .then_with(|| lhs.priority_rank.cmp(&rhs.priority_rank))
}

fn signal_priority(kind: &str) -> u8 {
    match kind {
        BILLING_BLOCK_KIND => 0,
        PRIMARY_WINDOW_KIND => 1,
        SECONDARY_WINDOW_KIND => 2,
        _ => 3,
    }
}

fn reset_sort_key(signal: &DepletionForecastSignal) -> i64 {
    if let Some(resets_in_minutes) = signal.resets_in_minutes {
        return resets_in_minutes;
    }
    signal
        .end_time
        .as_deref()
        .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
        .map(|end_time| {
            (end_time.with_timezone(&Utc) - Utc::now())
                .num_minutes()
                .max(0)
        })
        .unwrap_or(i64::MAX)
}

fn severity_for_primary(signal: &DepletionForecastSignal) -> Severity {
    severity_for_pct(signal.projected_percent.unwrap_or(signal.used_percent) / 100.0)
}

fn severity_label(severity: Severity) -> &'static str {
    match severity {
        Severity::Ok => "ok",
        Severity::Warn => "warn",
        Severity::Danger => "danger",
    }
}

fn summary_label(signal: &DepletionForecastSignal) -> String {
    if signal.will_run_out_before_reset == Some(true) {
        return match signal.runout_in_minutes {
            Some(0) => format!("{} limit is already exhausted", signal.title),
            Some(minutes) => format!(
                "{} runs out in {} at current burn",
                signal.title,
                duration_label(minutes)
            ),
            None => format!("{} runs out before reset", signal.title),
        };
    }

    if signal.will_run_out_before_reset == Some(false) {
        return format!("{} resets before projected runout", signal.title);
    }

    let percent = signal
        .projected_percent
        .unwrap_or(signal.used_percent)
        .round() as i64;
    let verb = if signal.projected_percent.is_some() {
        "projected to reach"
    } else {
        "currently at"
    };
    match signal.kind.as_str() {
        BILLING_BLOCK_KIND => format!("Billing block {verb} {percent}% before reset"),
        PRIMARY_WINDOW_KIND => format!("Primary window {verb} {percent}% used"),
        SECONDARY_WINDOW_KIND => format!("Secondary window {verb} {percent}% used"),
        _ => format!("Depletion signal {verb} {percent}%"),
    }
}

fn duration_label(total_minutes: i64) -> String {
    if total_minutes <= 0 {
        return "now".into();
    }
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;
    if hours == 0 {
        return format!("{minutes}m");
    }
    if minutes == 0 {
        return format!("{hours}h");
    }
    format!("{hours}h {minutes}m")
}

fn pace_label_for_remaining_percent(remaining_percent: f64) -> String {
    match remaining_percent {
        value if value < 15.0 => "Critical".into(),
        value if value < 35.0 => "Heavy".into(),
        value if value < 65.0 => "Steady".into(),
        _ => "Comfortable".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analytics::blocks::{BillingBlock, TokenBreakdown};
    use crate::analytics::quota::QuotaMeta;
    use chrono::Duration;

    fn ts(s: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
    }

    fn active_block(now: DateTime<Utc>) -> BillingBlock {
        BillingBlock {
            start: now - Duration::hours(1),
            end: now + Duration::hours(4),
            tokens: TokenBreakdown {
                input: 600_000,
                output: 0,
                cache_read: 0,
                cache_creation: 0,
                reasoning_output: 0,
            },
            cost_nanos: 0,
            models: vec![],
            is_active: true,
            entry_count: 2,
            first_timestamp: now - Duration::hours(1),
            last_timestamp: now,
            is_gap: false,
            kind: "block",
        }
    }

    fn quota(remaining_tokens: i64) -> QuotaMeta {
        QuotaMeta {
            limit_tokens: 1_000_000,
            used_tokens: 1_000_000 - remaining_tokens,
            projected_tokens: 1_100_000,
            current_pct: 0.6,
            projected_pct: 1.1,
            remaining_tokens,
            current_severity: Severity::Warn,
            projected_severity: Severity::Danger,
        }
    }

    #[test]
    fn no_candidates_returns_none() {
        assert!(build_depletion_forecast(Vec::new()).is_none());
    }

    #[test]
    fn billing_block_projection_beats_weaker_window_signal() {
        let forecast = build_depletion_forecast([
            primary_window_signal(54.0, Some(46.0), Some(90), None, None),
            billing_block_signal(
                "Billing block",
                62.0,
                Some(88.0),
                Some(120_000),
                Some(38.0),
                Some("2026-04-23T12:00:00Z".into()),
            ),
        ])
        .unwrap();

        assert_eq!(forecast.primary_signal.kind, BILLING_BLOCK_KIND);
        assert_eq!(forecast.secondary_signals.len(), 1);
    }

    #[test]
    fn window_signal_wins_when_no_active_block_projection_exists() {
        let forecast = build_depletion_forecast([
            primary_window_signal(84.0, Some(16.0), Some(30), None, None),
            secondary_window_signal(55.0, Some(45.0), Some(180), None, None),
        ])
        .unwrap();

        assert_eq!(forecast.primary_signal.kind, PRIMARY_WINDOW_KIND);
        assert_eq!(forecast.severity, "danger");
    }

    #[test]
    fn tie_breaking_prefers_earlier_reset_then_stable_priority() {
        let earlier_reset = primary_window_signal(72.0, Some(28.0), Some(45), None, None);
        let later_reset = secondary_window_signal(72.0, Some(28.0), Some(120), None, None);
        let stable_priority = billing_block_signal(
            "Billing block",
            72.0,
            None,
            Some(80_000),
            Some(28.0),
            Some("2026-04-23T12:00:00Z".into()),
        );

        let forecast =
            build_depletion_forecast([later_reset.clone(), earlier_reset.clone()]).unwrap();
        assert_eq!(forecast.primary_signal.kind, PRIMARY_WINDOW_KIND);

        let forecast = build_depletion_forecast([
            stable_priority,
            primary_window_signal(72.0, Some(28.0), None, None, None),
        ])
        .unwrap();
        assert_eq!(forecast.primary_signal.kind, BILLING_BLOCK_KIND);
    }

    #[test]
    fn secondary_signals_are_ordered_after_primary_selection() {
        let forecast = build_depletion_forecast([
            primary_window_signal(66.0, Some(34.0), Some(80), None, None),
            secondary_window_signal(78.0, Some(22.0), Some(200), None, None),
            billing_block_signal(
                "Billing block",
                44.0,
                Some(91.0),
                Some(140_000),
                Some(56.0),
                Some("2026-04-23T12:00:00Z".into()),
            ),
        ])
        .unwrap();

        assert_eq!(forecast.primary_signal.kind, BILLING_BLOCK_KIND);
        assert_eq!(
            forecast
                .secondary_signals
                .iter()
                .map(|signal| signal.kind.as_str())
                .collect::<Vec<_>>(),
            vec![SECONDARY_WINDOW_KIND, PRIMARY_WINDOW_KIND]
        );
    }

    #[test]
    fn runout_estimate_returns_eta_when_rate_crosses_limit_before_reset() {
        let now = ts("2026-04-23T10:00:00Z");
        let block = active_block(now);
        let forecast = estimate_token_runout(
            &block,
            &quota(120_000),
            Some(BurnRate {
                tokens_per_min: 2_000.0,
                cost_per_hour_nanos: 0,
            }),
            now,
        )
        .unwrap();

        assert_eq!(forecast.runout_in_minutes, 60);
        assert_eq!(forecast.runout_at, ts("2026-04-23T11:00:00Z"));
        assert!(forecast.will_run_out_before_reset);
    }

    #[test]
    fn runout_estimate_marks_reset_before_late_crossing() {
        let now = ts("2026-04-23T10:00:00Z");
        let block = active_block(now);
        let forecast = estimate_token_runout(
            &block,
            &quota(300_000),
            Some(BurnRate {
                tokens_per_min: 1_000.0,
                cost_per_hour_nanos: 0,
            }),
            now,
        )
        .unwrap();

        assert_eq!(forecast.runout_in_minutes, 300);
        assert!(!forecast.will_run_out_before_reset);
    }

    #[test]
    fn runout_estimate_reports_now_when_quota_is_already_exhausted() {
        let now = ts("2026-04-23T10:00:00Z");
        let block = active_block(now);
        let forecast = estimate_token_runout(&block, &quota(-10_000), None, now).unwrap();

        assert_eq!(forecast.runout_in_minutes, 0);
        assert_eq!(forecast.runout_at, now);
        assert!(forecast.will_run_out_before_reset);
    }

    #[test]
    fn runout_estimate_returns_none_without_positive_rate() {
        let now = ts("2026-04-23T10:00:00Z");
        let block = active_block(now);
        assert!(estimate_token_runout(&block, &quota(120_000), None, now).is_none());
        assert!(
            estimate_token_runout(
                &block,
                &quota(120_000),
                Some(BurnRate {
                    tokens_per_min: 0.0,
                    cost_per_hour_nanos: 0,
                }),
                now,
            )
            .is_none()
        );
    }

    #[test]
    fn billing_block_summary_prefers_runout_eta() {
        let now = ts("2026-04-23T10:00:00Z");
        let forecast = build_depletion_forecast([billing_block_signal_with_runout(
            "Billing block",
            70.0,
            Some(110.0),
            Some(120_000),
            Some(30.0),
            Some("2026-04-23T14:00:00Z".into()),
            Some(TokenRunoutForecast {
                runout_in_minutes: 75,
                runout_at: now + Duration::minutes(75),
                will_run_out_before_reset: true,
            }),
        )])
        .unwrap();

        assert_eq!(
            forecast.summary_label,
            "Billing block runs out in 1h 15m at current burn"
        );
    }
}
