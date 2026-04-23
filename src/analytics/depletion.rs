use crate::analytics::quota::{Severity, severity_for_pct};
use crate::models::{DepletionForecast, DepletionForecastSignal};
use chrono::Utc;

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

pub fn billing_block_signal(
    title: &str,
    used_percent: f64,
    projected_percent: Option<f64>,
    remaining_tokens: Option<i64>,
    remaining_percent: Option<f64>,
    end_time: Option<String>,
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
}
