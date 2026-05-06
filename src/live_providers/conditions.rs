use std::sync::Arc;

use anyhow::Result;

use crate::agent_status::models::ProviderStatus;
use crate::models::{LocalNotificationCondition, LocalNotificationState};
use crate::scanner::db;
use crate::server::api::AppState;
use crate::status_aggregator::models::SignalLevel;
use crate::tz::TzParams;

pub(super) fn provider_status_condition(
    id: &str,
    provider: &str,
    service_label: &str,
    status: &ProviderStatus,
) -> LocalNotificationCondition {
    let active = status.indicator.is_alert_worthy();
    let activation_title = if provider == "codex" {
        "Codex service degraded"
    } else {
        "Claude service degraded"
    };
    let recovery_title = if provider == "codex" {
        "Codex service restored"
    } else {
        "Claude service restored"
    };

    LocalNotificationCondition {
        id: id.into(),
        kind: "provider_degraded".into(),
        provider: Some(provider.into()),
        service_label: service_label.into(),
        is_active: active,
        activation_title: activation_title.into(),
        activation_body: if provider == "codex" {
            format!(
                "OpenAI status is degraded for Codex. {}",
                status.description
            )
        } else {
            format!("Claude status is degraded. {}", status.description)
        },
        recovery_title: Some(recovery_title.into()),
        recovery_body: Some(if provider == "codex" {
            "OpenAI status returned to a non-alert state for Codex.".into()
        } else {
            "Claude status returned to a non-alert state.".into()
        }),
        day_key: None,
    }
}

pub(super) fn community_spike_condition(
    id: &str,
    provider: &str,
    service_label: &str,
    signals: &[crate::status_aggregator::models::ServiceSignal],
    official_is_major: bool,
) -> LocalNotificationCondition {
    let is_spike = signals
        .iter()
        .any(|signal| signal.level == SignalLevel::Spike);
    let is_active = is_spike && !official_is_major;
    let activation_title = if provider == "codex" {
        "Codex community spike detected"
    } else {
        "Claude community spike detected"
    };

    LocalNotificationCondition {
        id: id.into(),
        kind: "community_spike".into(),
        provider: Some(provider.into()),
        service_label: service_label.into(),
        is_active,
        activation_title: activation_title.into(),
        activation_body: if provider == "codex" {
            "OpenAI community reports spiked while official status remains below major.".into()
        } else {
            "Claude community reports spiked while official status remains below major.".into()
        },
        recovery_title: None,
        recovery_body: None,
        day_key: None,
    }
}

pub(super) async fn load_today_cost_snapshot(state: &Arc<AppState>) -> Result<(String, f64)> {
    let db_path = state.db_path.clone();
    tokio::task::spawn_blocking(move || -> Result<(String, f64)> {
        let conn = db::open_db(&db_path)?;
        db::init_db(&conn)?;

        let now = chrono::Local::now();
        let tz = TzParams {
            tz_offset_min: Some(now.offset().local_minus_utc() / 60),
            week_starts_on: None,
        };
        let today = now.format("%Y-%m-%d").to_string();
        let data = db::get_dashboard_data(&conn, tz)?;
        let daily_cost_usd = data
            .daily_by_model
            .iter()
            .filter(|row| row.day == today)
            .map(|row| row.cost)
            .sum();

        Ok((today, daily_cost_usd))
    })
    .await
    .map_err(|err| anyhow::anyhow!("failed to load daily cost snapshot: {}", err))?
}

pub(super) async fn build_local_notification_state(
    state: &Arc<AppState>,
    agent_status: Option<&crate::agent_status::models::AgentStatusSnapshot>,
    claude_usage: Option<&crate::oauth::models::UsageWindowsResponse>,
    community_signal: Option<&crate::status_aggregator::models::CommunitySignal>,
) -> Result<LocalNotificationState> {
    let generated_at = chrono::Utc::now().to_rfc3339();
    // Snapshot live settings once; the read guard is dropped before any DB or
    // network awaits below.
    let (cost_threshold_usd, aggregator_enabled) = {
        let live = state.settings.read().await;
        (live.webhooks.cost_threshold, live.aggregator.enabled)
    };
    let mut conditions = Vec::new();

    if let Some(session) = claude_usage.and_then(|usage| usage.session.as_ref()) {
        let depleted = session.used_percent >= 99.99;
        conditions.push(LocalNotificationCondition {
            id: "claude-session-depleted".into(),
            kind: "session_depleted".into(),
            provider: Some("claude".into()),
            service_label: "Claude".into(),
            is_active: depleted,
            activation_title: "Claude session depleted".into(),
            activation_body: match session.resets_in_minutes {
                Some(minutes) => {
                    format!("Claude session is depleted. Resets in {minutes} minutes.")
                }
                None => "Claude session is depleted.".into(),
            },
            recovery_title: Some("Claude session restored".into()),
            recovery_body: Some("Claude session capacity is available again.".into()),
            day_key: None,
        });
    }

    if let Some(status) = agent_status.and_then(|snapshot| snapshot.claude.as_ref()) {
        conditions.push(provider_status_condition(
            "claude-service-degraded",
            "claude",
            "Claude",
            status,
        ));
    }
    if let Some(status) = agent_status.and_then(|snapshot| snapshot.openai.as_ref()) {
        conditions.push(provider_status_condition(
            "codex-service-degraded",
            "codex",
            "OpenAI",
            status,
        ));
    }

    if aggregator_enabled
        && let Some(signal) = community_signal
        && signal.enabled
    {
        let claude_major = agent_status
            .and_then(|snapshot| snapshot.claude.as_ref())
            .map(|status| status.indicator.is_alert_worthy())
            .unwrap_or(false);
        let codex_major = agent_status
            .and_then(|snapshot| snapshot.openai.as_ref())
            .map(|status| status.indicator.is_alert_worthy())
            .unwrap_or(false);

        conditions.push(community_spike_condition(
            "claude-community-spike",
            "claude",
            "Claude",
            &signal.claude,
            claude_major,
        ));
        conditions.push(community_spike_condition(
            "codex-community-spike",
            "codex",
            "OpenAI",
            &signal.openai,
            codex_major,
        ));
    }

    if let Some(threshold) = cost_threshold_usd {
        let (day_key, daily_cost_usd) = load_today_cost_snapshot(state).await?;
        conditions.push(LocalNotificationCondition {
            id: "daily-cost-threshold".into(),
            kind: "daily_cost_threshold".into(),
            provider: None,
            service_label: "Heimdall".into(),
            is_active: daily_cost_usd > threshold,
            activation_title: "Daily cost threshold crossed".into(),
            activation_body: format!(
                "Today's Heimdall cost reached ${daily_cost_usd:.2}, above the configured ${threshold:.2} threshold."
            ),
            recovery_title: None,
            recovery_body: None,
            day_key: Some(day_key),
        });
    }

    Ok(LocalNotificationState {
        generated_at,
        cost_threshold_usd,
        conditions,
    })
}
