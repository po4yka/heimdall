use std::collections::HashMap;

use anyhow::Result;
use rusqlite::Connection;
use serde::Serialize;
use tracing::{debug, error, info};

use crate::config::WebhookConfig;
use crate::models::AgentStopReasonTransition;
use crate::scanner::db;

/// Default allowlist for `agent_stop_reason_transition` events. When
/// `WebhookConfig::agent_stop_reason_filter` is `None` this list is used
/// verbatim — both values indicate that an agent ran out of context or was
/// refused, both of which usually mean lost work or a config issue worth
/// investigating.
pub const DEFAULT_STOP_REASON_FILTER: &[&str] = &["max_tokens", "refusal"];

/// A webhook event payload sent to the configured URL.
#[derive(Debug, Serialize)]
pub struct WebhookEvent {
    pub event_type: String,
    pub message: String,
    pub details: serde_json::Value,
    pub timestamp: String,
}

#[derive(Debug, Default)]
pub struct WebhookState {
    pub session_depleted: Option<bool>,
    pub last_cost_threshold_day: Option<String>,
    /// Track whether Claude was last seen degraded (Major/Critical).
    pub claude_degraded: Option<bool>,
    /// Track whether OpenAI was last seen degraded (Major/Critical).
    pub openai_degraded: Option<bool>,
    /// Track whether Claude community signal was last seen as a spike while
    /// official status was below Major (leading-indicator dedup).
    pub claude_community_spike: Option<bool>,
    /// Track whether OpenAI community signal was last seen as a spike while
    /// official status was below Major (leading-indicator dedup).
    pub openai_community_spike: Option<bool>,
    /// Active subscription-cap shift state keyed by `provider:window_type`.
    /// The value is `"increase"` or `"decrease"`; absence means no active
    /// material cap change was last observed for that window.
    pub cap_change_shifts: HashMap<String, String>,
}

/// POST a webhook event to the given URL. Fire-and-forget via `tokio::spawn`.
pub fn send_webhook(url: &str, event: &WebhookEvent) {
    let url = url.to_owned();
    let body = match serde_json::to_string(event) {
        Ok(b) => b,
        Err(e) => {
            error!("Failed to serialize webhook event: {}", e);
            return;
        }
    };

    tokio::spawn(async move {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build();

        let client = match client {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to build HTTP client for webhook: {}", e);
                return;
            }
        };

        match client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
        {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    info!("Webhook sent successfully to {}", url);
                } else {
                    error!("Webhook to {} returned status {}", url, status);
                }
            }
            Err(e) => {
                error!("Failed to send webhook to {}: {}", url, e);
            }
        }
    });
}

/// Check if webhooks are configured and the event type is enabled, then send.
pub fn notify_if_configured(config: &WebhookConfig, event: WebhookEvent) {
    let url = match &config.url {
        Some(u) => u,
        None => {
            debug!("Webhook URL not configured, skipping notification");
            return;
        }
    };

    if !url.starts_with("https://") && !url.starts_with("http://") {
        tracing::warn!("Webhook URL must use http(s) scheme: {}", url);
        return;
    }

    let enabled = match event.event_type.as_str() {
        "session_depleted" | "session_restored" => config.session_depleted,
        "cost_threshold" => config.cost_threshold.is_some(),
        "agent_status_degraded" | "agent_status_restored" => config.agent_status,
        "community_signal_spike" => config.spike_webhook,
        "subscription_cap_changed" => config.cap_changes,
        "agent_stop_reason_transition" => config.agent_stop_reason,
        _ => {
            debug!("Unknown webhook event type: {}", event.event_type);
            false
        }
    };

    if !enabled {
        debug!(
            "Webhook event type '{}' is not enabled, skipping",
            event.event_type
        );
        return;
    }

    send_webhook(url, &event);
}

pub fn session_transition_event(
    config: &WebhookConfig,
    state: &mut WebhookState,
    used_percent: f64,
    resets_in_minutes: Option<i64>,
) -> Option<WebhookEvent> {
    if !config.session_depleted {
        state.session_depleted = Some(used_percent >= 99.99);
        return None;
    }

    let is_depleted = used_percent >= 99.99;
    let previous = state.session_depleted.replace(is_depleted);

    match previous {
        Some(prev) if prev == is_depleted => None,
        Some(false) if is_depleted => Some(WebhookEvent {
            event_type: "session_depleted".to_string(),
            message: match resets_in_minutes {
                Some(minutes) => format!("Claude session depleted. Resets in {} minutes.", minutes),
                None => "Claude session depleted.".to_string(),
            },
            details: serde_json::json!({
                "used_percent": used_percent,
                "resets_in_minutes": resets_in_minutes,
            }),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }),
        Some(true) if !is_depleted => Some(WebhookEvent {
            event_type: "session_restored".to_string(),
            message: "Claude session restored.".to_string(),
            details: serde_json::json!({
                "used_percent": used_percent,
                "resets_in_minutes": resets_in_minutes,
            }),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }),
        _ => None,
    }
}

/// Produce an agent-status transition webhook event when the severity threshold
/// is crossed, with deduplication so the same edge only fires once.
///
/// `provider` is a display name like `"Claude"` or `"OpenAI"`.
/// `indicator` is the current `StatusIndicator`.
/// `degraded_state` is the mutable per-provider dedup flag on `WebhookState`.
pub fn agent_status_transition_event(
    config: &WebhookConfig,
    degraded_state: &mut Option<bool>,
    provider: &str,
    indicator: &crate::agent_status::models::StatusIndicator,
    active_incident_count: usize,
) -> Option<WebhookEvent> {
    if !config.agent_status {
        // Still update the state so the dedup is consistent.
        *degraded_state = Some(indicator.is_alert_worthy());
        return None;
    }

    let is_now_degraded = indicator.is_alert_worthy();
    let previous = degraded_state.replace(is_now_degraded);

    match previous {
        // Same state — no transition.
        Some(prev) if prev == is_now_degraded => None,
        // Operational → degraded.
        Some(false) if is_now_degraded => Some(WebhookEvent {
            event_type: "agent_status_degraded".to_string(),
            message: format!(
                "{} is experiencing a major outage or service degradation.",
                provider
            ),
            details: serde_json::json!({
                "provider": provider,
                "indicator": format!("{:?}", indicator).to_lowercase(),
                "active_incidents": active_incident_count,
            }),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }),
        // Degraded → operational/minor/maintenance.
        Some(true) if !is_now_degraded => Some(WebhookEvent {
            event_type: "agent_status_restored".to_string(),
            message: format!("{} service has been restored.", provider),
            details: serde_json::json!({
                "provider": provider,
                "indicator": format!("{:?}", indicator).to_lowercase(),
                "active_incidents": active_incident_count,
            }),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }),
        // First observation — set baseline, don't fire.
        None => None,
        _ => None,
    }
}

pub fn cost_threshold_event(
    config: &WebhookConfig,
    state: &mut WebhookState,
    day: &str,
    daily_cost: f64,
) -> Option<WebhookEvent> {
    let threshold = config.cost_threshold?;
    if daily_cost <= threshold {
        return None;
    }
    if state.last_cost_threshold_day.as_deref() == Some(day) {
        return None;
    }

    state.last_cost_threshold_day = Some(day.to_string());
    Some(WebhookEvent {
        event_type: "cost_threshold".to_string(),
        message: format!("Daily cost exceeded ${threshold:.2} on {day}."),
        details: serde_json::json!({
            "day": day,
            "daily_cost": daily_cost,
            "threshold": threshold,
        }),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

#[derive(Debug, Clone, Copy)]
pub struct CapChangeObservation<'a> {
    pub provider: &'a str,
    pub service_label: &'a str,
    pub window_type: &'a str,
    pub window_label: &'a str,
    pub cap_shift: Option<&'a str>,
    pub estimated_cap_tokens: i64,
    pub smoothed_cap_tokens: Option<i64>,
    pub observed_tokens: i64,
    pub used_percent: f64,
    pub confidence: f64,
    pub sample_count: Option<u32>,
    pub plan: Option<&'a str>,
    pub source_used: &'a str,
}

/// Produce a webhook event when a subscription cap estimate enters or flips a
/// material shift state for one provider/window. The state is cleared once the
/// estimator no longer reports a shift, so future changes can notify again.
pub fn cap_change_transition_event(
    config: &WebhookConfig,
    state: &mut WebhookState,
    observation: CapChangeObservation<'_>,
) -> Option<WebhookEvent> {
    let state_key = format!("{}:{}", observation.provider, observation.window_type);
    let cap_shift = observation.cap_shift.filter(|value| {
        value.eq_ignore_ascii_case("increase") || value.eq_ignore_ascii_case("decrease")
    });

    if !config.cap_changes {
        match cap_shift {
            Some(shift) => {
                state
                    .cap_change_shifts
                    .insert(state_key, shift.to_ascii_lowercase());
            }
            None => {
                state.cap_change_shifts.remove(&state_key);
            }
        }
        return None;
    }

    let Some(shift) = cap_shift else {
        state.cap_change_shifts.remove(&state_key);
        return None;
    };
    let shift = shift.to_ascii_lowercase();
    let previous = state.cap_change_shifts.insert(state_key, shift.clone());
    if previous.as_deref() == Some(shift.as_str()) {
        return None;
    }

    let direction = if shift == "increase" {
        "increased"
    } else {
        "decreased"
    };

    Some(WebhookEvent {
        event_type: "subscription_cap_changed".to_string(),
        message: format!(
            "{} {} cap estimate {}.",
            observation.service_label, observation.window_label, direction
        ),
        details: serde_json::json!({
            "provider": observation.provider,
            "service": observation.service_label,
            "window_type": observation.window_type,
            "window_label": observation.window_label,
            "cap_shift": shift,
            "estimated_cap_tokens": observation.estimated_cap_tokens,
            "smoothed_cap_tokens": observation.smoothed_cap_tokens,
            "observed_tokens": observation.observed_tokens,
            "used_percent": observation.used_percent,
            "confidence": observation.confidence,
            "sample_count": observation.sample_count,
            "plan": observation.plan,
            "source_used": observation.source_used,
        }),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

/// Produce a `community_signal_spike` webhook event when the crowd signal for a
/// provider transitions to Spike AND the official indicator is below Major
/// (i.e., the crowd sees something the official status page hasn't confirmed).
///
/// - Fires only on `false → true` transition (dedup via `spike_state`).
/// - Clears the spike state when either the crowd signal normalises or the
///   official page catches up to Major/Critical (official_is_major = true).
///
/// `provider` is a display name like `"Claude"` or `"OpenAI"`.
/// `is_crowd_spike` is whether any slug for this provider is at Spike level.
/// `official_is_major` is whether the official status is already Major/Critical.
/// `spike_state` is the mutable per-provider dedup flag on `WebhookState`.
pub fn community_signal_spike_event(
    config: &WebhookConfig,
    spike_state: &mut Option<bool>,
    provider: &str,
    is_crowd_spike: bool,
    official_is_major: bool,
) -> Option<WebhookEvent> {
    // Leading-indicator condition: crowd=Spike AND official < Major.
    let is_leading_spike = is_crowd_spike && !official_is_major;

    if !config.spike_webhook {
        *spike_state = Some(is_leading_spike);
        return None;
    }

    let previous = spike_state.replace(is_leading_spike);

    match previous {
        // Same state — no transition.
        Some(prev) if prev == is_leading_spike => None,
        // false → true: crowd detected something official hasn't confirmed.
        Some(false) if is_leading_spike => Some(WebhookEvent {
            event_type: "community_signal_spike".to_string(),
            message: format!(
                "{} community reports spike while official status is nominal — possible leading indicator.",
                provider
            ),
            details: serde_json::json!({
                "provider": provider,
                "crowd_spike": true,
                "official_is_major": false,
            }),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }),
        // First observation or true→false — no event.
        _ => None,
    }
}

/// Produce an `agent_stop_reason_transition` event when a subagent's terminal
/// `stop_reason` lands on the configured allowlist AND represents a real
/// change versus the prior emission.
///
/// Dedup happens at three layers:
///   1. The `agent_stop_reason` config flag (returns `Ok(None)` when off).
///   2. The allowlist filter (default `["max_tokens", "refusal"]`).
///   3. The persistent `webhook_emitted` table — last emitted value for the
///      `(source, project, agent_id)` triple is compared against `new_reason`,
///      and identical values are suppressed even across process restart.
///
/// On a real transition the helper writes the new value into
/// `webhook_emitted` BEFORE returning the event, so concurrent writers will
/// see the latest value on their next read. The actual webhook dispatch is
/// the caller's responsibility (`notify_if_configured`).
pub fn agent_stop_reason_transition_event(
    config: &WebhookConfig,
    conn: &Connection,
    transition: &AgentStopReasonTransition,
) -> Result<Option<WebhookEvent>> {
    if !config.agent_stop_reason {
        return Ok(None);
    }

    let new_reason = match transition.record.stop_reason.as_deref() {
        Some(r) if !r.is_empty() => r,
        _ => return Ok(None),
    };

    let in_filter = match &config.agent_stop_reason_filter {
        Some(list) => list.iter().any(|s| s == new_reason),
        None => DEFAULT_STOP_REASON_FILTER.contains(&new_reason),
    };
    if !in_filter {
        return Ok(None);
    }

    if transition.prev_stop_reason.as_deref() == Some(new_reason) {
        return Ok(None);
    }

    let r = &transition.record;
    let dedup_key = format!("{}|{}|{}", r.source.as_str(), r.project, r.agent_id);

    let already = db::get_webhook_emitted(conn, "agent_stop_reason_transition", &dedup_key)?;
    if already.as_deref() == Some(new_reason) {
        return Ok(None);
    }

    db::upsert_webhook_emitted(
        conn,
        "agent_stop_reason_transition",
        &dedup_key,
        Some(new_reason),
    )?;

    Ok(Some(WebhookEvent {
        event_type: "agent_stop_reason_transition".to_string(),
        message: format!(
            "Agent {} ({}) ended with stop_reason={}",
            r.agent_id, r.role, new_reason,
        ),
        details: serde_json::json!({
            "agent_id": r.agent_id,
            "source": r.source.as_str(),
            "role": r.role,
            "project": r.project,
            "session_id": r.session_id,
            "model": r.model,
            "stop_reason": new_reason,
            "prev_stop_reason": transition.prev_stop_reason,
            "total_tokens": r.total_tokens,
            "cost_usd": r.cost_nanos as f64 / 1e9,
            "duration_s": r.duration_s,
            "ts_start": r.ts_start,
        }),
        timestamp: chrono::Utc::now().to_rfc3339(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_webhook_event_serialization() {
        let event = WebhookEvent {
            event_type: "cost_threshold".to_string(),
            message: "Daily cost exceeded $50.00".to_string(),
            details: json!({"daily_cost": 52.30, "threshold": 50.0}),
            timestamp: "2026-04-09T12:00:00Z".to_string(),
        };

        let serialized = serde_json::to_string(&event).expect("should serialize");
        let deserialized: serde_json::Value =
            serde_json::from_str(&serialized).expect("should be valid JSON");

        assert_eq!(deserialized["event_type"], "cost_threshold");
        assert_eq!(deserialized["message"], "Daily cost exceeded $50.00");
        assert_eq!(deserialized["details"]["daily_cost"], 52.30);
        assert_eq!(deserialized["timestamp"], "2026-04-09T12:00:00Z");
    }

    #[test]
    fn test_notify_no_url_configured() {
        let config = WebhookConfig {
            url: None,
            cost_threshold: Some(50.0),
            session_depleted: true,
            agent_status: true,
            spike_webhook: true,
            cap_changes: true,
            agent_stop_reason: true,
            agent_stop_reason_filter: None,
        };

        let event = WebhookEvent {
            event_type: "cost_threshold".to_string(),
            message: "test".to_string(),
            details: json!({}),
            timestamp: "2026-04-09T12:00:00Z".to_string(),
        };

        // Should not panic when no URL is configured
        notify_if_configured(&config, event);
    }

    #[test]
    fn test_notify_event_type_disabled() {
        let config = WebhookConfig {
            url: Some("https://example.com/hook".to_string()),
            cost_threshold: Some(50.0),
            session_depleted: false, // session events disabled
            agent_status: true,
            spike_webhook: true,
            cap_changes: true,
            agent_stop_reason: true,
            agent_stop_reason_filter: None,
        };

        let event = WebhookEvent {
            event_type: "session_depleted".to_string(),
            message: "test".to_string(),
            details: json!({}),
            timestamp: "2026-04-09T12:00:00Z".to_string(),
        };

        // Should not send (session_depleted is false) and should not panic
        notify_if_configured(&config, event);
    }

    #[test]
    fn test_notify_cost_threshold_disabled() {
        let config = WebhookConfig {
            url: Some("https://example.com/hook".to_string()),
            cost_threshold: None, // cost threshold not set
            session_depleted: true,
            agent_status: true,
            spike_webhook: true,
            cap_changes: true,
            agent_stop_reason: true,
            agent_stop_reason_filter: None,
        };

        let event = WebhookEvent {
            event_type: "cost_threshold".to_string(),
            message: "test".to_string(),
            details: json!({}),
            timestamp: "2026-04-09T12:00:00Z".to_string(),
        };

        // Should not send (cost_threshold is None) and should not panic
        notify_if_configured(&config, event);
    }

    #[test]
    fn test_notify_unknown_event_type() {
        let config = WebhookConfig {
            url: Some("https://example.com/hook".to_string()),
            cost_threshold: Some(50.0),
            session_depleted: true,
            agent_status: true,
            spike_webhook: true,
            cap_changes: true,
            agent_stop_reason: true,
            agent_stop_reason_filter: None,
        };

        let event = WebhookEvent {
            event_type: "unknown_event".to_string(),
            message: "test".to_string(),
            details: json!({}),
            timestamp: "2026-04-09T12:00:00Z".to_string(),
        };

        // Should not panic for unknown event types
        notify_if_configured(&config, event);
    }

    #[test]
    fn test_session_transition_event_depleted_then_restored() {
        let config = WebhookConfig {
            url: Some("https://example.com/hook".to_string()),
            cost_threshold: None,
            session_depleted: true,
            agent_status: true,
            spike_webhook: true,
            cap_changes: true,
            agent_stop_reason: true,
            agent_stop_reason_filter: None,
        };
        let mut state = WebhookState::default();

        assert!(session_transition_event(&config, &mut state, 50.0, Some(60)).is_none());

        let depleted = session_transition_event(&config, &mut state, 100.0, Some(30))
            .expect("depletion event");
        assert_eq!(depleted.event_type, "session_depleted");

        let restored =
            session_transition_event(&config, &mut state, 25.0, Some(120)).expect("restored event");
        assert_eq!(restored.event_type, "session_restored");
    }

    #[test]
    fn test_cost_threshold_event_only_once_per_day() {
        let config = WebhookConfig {
            url: Some("https://example.com/hook".to_string()),
            cost_threshold: Some(50.0),
            session_depleted: false,
            agent_status: true,
            spike_webhook: true,
            cap_changes: true,
            agent_stop_reason: true,
            agent_stop_reason_filter: None,
        };
        let mut state = WebhookState::default();

        let first =
            cost_threshold_event(&config, &mut state, "2026-04-10", 60.0).expect("threshold event");
        assert_eq!(first.event_type, "cost_threshold");
        assert!(cost_threshold_event(&config, &mut state, "2026-04-10", 70.0).is_none());
        assert!(cost_threshold_event(&config, &mut state, "2026-04-11", 70.0).is_some());
    }

    fn cap_change_obs<'a>(
        provider: &'a str,
        service_label: &'a str,
        window_type: &'a str,
        cap_shift: Option<&'a str>,
    ) -> CapChangeObservation<'a> {
        CapChangeObservation {
            provider,
            service_label,
            window_type,
            window_label: "5-hour window",
            cap_shift,
            estimated_cap_tokens: 1_300_000,
            smoothed_cap_tokens: Some(1_050_000),
            observed_tokens: 650_000,
            used_percent: 50.0,
            confidence: 1.0,
            sample_count: Some(7),
            plan: Some("pro"),
            source_used: "oauth",
        }
    }

    #[test]
    fn test_cap_change_event_fires_once_per_active_shift() {
        let config = WebhookConfig {
            url: Some("https://example.com/hook".to_string()),
            cost_threshold: None,
            session_depleted: false,
            agent_status: true,
            spike_webhook: true,
            cap_changes: true,
            agent_stop_reason: true,
            agent_stop_reason_filter: None,
        };
        let mut state = WebhookState::default();

        let event = cap_change_transition_event(
            &config,
            &mut state,
            cap_change_obs("claude", "Claude Code", "five_hour", Some("increase")),
        )
        .expect("first active shift should fire");
        assert_eq!(event.event_type, "subscription_cap_changed");
        assert_eq!(event.details["provider"], "claude");
        assert_eq!(event.details["service"], "Claude Code");
        assert_eq!(event.details["cap_shift"], "increase");

        assert!(
            cap_change_transition_event(
                &config,
                &mut state,
                cap_change_obs("claude", "Claude Code", "five_hour", Some("increase")),
            )
            .is_none(),
            "same active shift should be deduped"
        );

        assert!(
            cap_change_transition_event(
                &config,
                &mut state,
                cap_change_obs("claude", "Claude Code", "five_hour", None),
            )
            .is_none(),
            "clearing the shift should not fire"
        );

        assert!(
            cap_change_transition_event(
                &config,
                &mut state,
                cap_change_obs("claude", "Claude Code", "five_hour", Some("increase")),
            )
            .is_some(),
            "a later shift after clear should fire again"
        );
    }

    #[test]
    fn test_cap_change_event_tracks_windows_independently_and_honors_disable() {
        let config = WebhookConfig {
            url: Some("https://example.com/hook".to_string()),
            cost_threshold: None,
            session_depleted: false,
            agent_status: true,
            spike_webhook: true,
            cap_changes: true,
            agent_stop_reason: true,
            agent_stop_reason_filter: None,
        };
        let disabled = WebhookConfig {
            cap_changes: false,
            ..config.clone()
        };
        let mut state = WebhookState::default();

        assert!(
            cap_change_transition_event(
                &config,
                &mut state,
                cap_change_obs("codex", "Codex", "codex_primary", Some("decrease")),
            )
            .is_some()
        );
        assert!(
            cap_change_transition_event(
                &config,
                &mut state,
                cap_change_obs("codex", "Codex", "codex_secondary", Some("decrease")),
            )
            .is_some(),
            "a different window should notify independently"
        );
        assert!(
            cap_change_transition_event(
                &disabled,
                &mut state,
                cap_change_obs("codex", "Codex", "codex_primary", Some("increase")),
            )
            .is_none()
        );
        assert_eq!(
            state.cap_change_shifts.get("codex:codex_primary"),
            Some(&"increase".to_string()),
            "disabled config should still update dedup state"
        );
    }

    // ── Agent status webhook tests ───────────────────────────────────────────

    fn agent_status_config_enabled() -> WebhookConfig {
        WebhookConfig {
            url: Some("https://example.com/hook".to_string()),
            cost_threshold: None,
            session_depleted: false,
            agent_status: true,
            spike_webhook: true,
            cap_changes: true,
            agent_stop_reason: true,
            agent_stop_reason_filter: None,
        }
    }

    fn agent_status_config_disabled() -> WebhookConfig {
        WebhookConfig {
            url: Some("https://example.com/hook".to_string()),
            cost_threshold: None,
            session_depleted: false,
            agent_status: false,
            spike_webhook: true,
            cap_changes: true,
            agent_stop_reason: true,
            agent_stop_reason_filter: None,
        }
    }

    #[test]
    fn test_agent_status_op_to_major_fires_degraded() {
        use crate::agent_status::models::StatusIndicator;

        let config = agent_status_config_enabled();
        let mut degraded: Option<bool> = Some(false);

        let event = agent_status_transition_event(
            &config,
            &mut degraded,
            "Claude",
            &StatusIndicator::Major,
            1,
        )
        .expect("should fire on op->major");

        assert_eq!(event.event_type, "agent_status_degraded");
        assert!(event.details["provider"] == "Claude");
        assert_eq!(degraded, Some(true));
    }

    #[test]
    fn test_agent_status_major_to_op_fires_restored() {
        use crate::agent_status::models::StatusIndicator;

        let config = agent_status_config_enabled();
        let mut degraded: Option<bool> = Some(true);

        let event = agent_status_transition_event(
            &config,
            &mut degraded,
            "OpenAI",
            &StatusIndicator::None,
            0,
        )
        .expect("should fire on major->op");

        assert_eq!(event.event_type, "agent_status_restored");
        assert_eq!(degraded, Some(false));
    }

    #[test]
    fn test_agent_status_no_change_no_fire() {
        use crate::agent_status::models::StatusIndicator;

        let config = agent_status_config_enabled();
        let mut degraded: Option<bool> = Some(true);

        // major->major: no transition
        let event = agent_status_transition_event(
            &config,
            &mut degraded,
            "Claude",
            &StatusIndicator::Critical,
            2,
        );
        assert!(event.is_none(), "major->critical should not fire");
    }

    #[test]
    fn test_agent_status_first_observation_no_fire() {
        use crate::agent_status::models::StatusIndicator;

        let config = agent_status_config_enabled();
        let mut degraded: Option<bool> = None;

        let event = agent_status_transition_event(
            &config,
            &mut degraded,
            "Claude",
            &StatusIndicator::Major,
            1,
        );
        assert!(event.is_none(), "first observation should not fire");
        assert_eq!(degraded, Some(true));
    }

    #[test]
    fn test_agent_status_op_to_minor_does_not_fire() {
        // Minor is BELOW the Major threshold — no alert.
        use crate::agent_status::models::StatusIndicator;

        let config = agent_status_config_enabled();
        let mut degraded: Option<bool> = Some(false);

        let event = agent_status_transition_event(
            &config,
            &mut degraded,
            "Claude",
            &StatusIndicator::Minor,
            0,
        );
        assert!(
            event.is_none(),
            "op->minor must not fire (below Major floor)"
        );
        // State should still update (minor is not alert-worthy).
        assert_eq!(degraded, Some(false));
    }

    #[test]
    fn test_agent_status_minor_to_major_fires_degraded() {
        // minor->major crosses the threshold — fires degraded.
        use crate::agent_status::models::StatusIndicator;

        let config = agent_status_config_enabled();
        let mut degraded: Option<bool> = Some(false); // minor was not alert-worthy

        let event = agent_status_transition_event(
            &config,
            &mut degraded,
            "Claude",
            &StatusIndicator::Major,
            1,
        )
        .expect("minor->major should fire degraded");

        assert_eq!(event.event_type, "agent_status_degraded");
    }

    #[test]
    fn test_agent_status_major_to_minor_fires_restored() {
        // major->minor crosses back below the threshold — fires restored.
        use crate::agent_status::models::StatusIndicator;

        let config = agent_status_config_enabled();
        let mut degraded: Option<bool> = Some(true);

        let event = agent_status_transition_event(
            &config,
            &mut degraded,
            "Claude",
            &StatusIndicator::Minor,
            0,
        )
        .expect("major->minor should fire restored");

        assert_eq!(event.event_type, "agent_status_restored");
        assert_eq!(degraded, Some(false));
    }

    #[test]
    fn test_agent_status_disabled_config_no_fire() {
        use crate::agent_status::models::StatusIndicator;

        let config = agent_status_config_disabled();
        let mut degraded: Option<bool> = Some(false);

        let event = agent_status_transition_event(
            &config,
            &mut degraded,
            "Claude",
            &StatusIndicator::Major,
            1,
        );
        // agent_status = false → no event regardless of transition.
        assert!(event.is_none());
    }

    #[tokio::test]
    async fn test_notify_agent_status_event_type_routing() {
        let config = agent_status_config_enabled();
        let event = WebhookEvent {
            event_type: "agent_status_degraded".to_string(),
            message: "test".to_string(),
            details: json!({}),
            timestamp: "2026-04-17T10:00:00Z".to_string(),
        };
        // Should not panic (URL is https, event type is enabled).
        // The actual HTTP call will fail (no server) — that's fine; we only
        // test that the routing logic doesn't blow up.
        notify_if_configured(&config, event);
    }

    // ── agent_stop_reason_transition_event tests ─────────────────────────────

    fn stop_reason_test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        crate::scanner::db::init_db(&conn).unwrap();
        conn
    }

    fn make_transition(
        agent_id: &str,
        stop_reason: Option<&str>,
        prev: Option<&str>,
    ) -> AgentStopReasonTransition {
        use crate::models::{AgentSessionRecord, AgentSource, RoleConfidence};
        AgentStopReasonTransition {
            record: AgentSessionRecord {
                agent_id: agent_id.into(),
                source: AgentSource::Subagent,
                project: "my-project".into(),
                session_id: Some("sess-1".into()),
                ts_start: "2026-05-01T10:00:00Z".into(),
                ts_start_epoch: 1_746_093_600,
                duration_s: 30,
                role: "executor".into(),
                role_confidence: RoleConfidence::Meta,
                description: "test agent".into(),
                model: "claude-sonnet-4-5".into(),
                input_tokens: 1000,
                cache_create_tokens: 0,
                cache_read_tokens: 0,
                output_tokens: 500,
                total_tokens: 1500,
                cost_nanos: 50_000_000,
                api_calls: 3,
                tool_uses: 1,
                tools_json: "{}".into(),
                prompt_id: None,
                stop_reason: stop_reason.map(str::to_string),
                source_path: format!("/tmp/{agent_id}.jsonl"),
            },
            prev_stop_reason: prev.map(str::to_string),
        }
    }

    fn stop_reason_config(agent_stop_reason: bool, filter: Option<Vec<String>>) -> WebhookConfig {
        WebhookConfig {
            url: Some("https://example.com/hook".to_string()),
            cost_threshold: None,
            session_depleted: false,
            agent_status: false,
            spike_webhook: false,
            cap_changes: false,
            agent_stop_reason,
            agent_stop_reason_filter: filter,
        }
    }

    #[test]
    fn agent_stop_reason_event_disabled_when_flag_off() {
        let conn = stop_reason_test_conn();
        let config = stop_reason_config(false, None);
        let t = make_transition("a1", Some("max_tokens"), None);
        let event = agent_stop_reason_transition_event(&config, &conn, &t).unwrap();
        assert!(event.is_none(), "flag off must suppress event");
    }

    #[test]
    fn agent_stop_reason_event_default_filter_only_max_tokens_and_refusal() {
        let conn = stop_reason_test_conn();
        let config = stop_reason_config(true, None);

        // end_turn must NOT fire (not on default allowlist).
        let t = make_transition("a1", Some("end_turn"), Some("tool_use"));
        assert!(
            agent_stop_reason_transition_event(&config, &conn, &t)
                .unwrap()
                .is_none(),
            "end_turn is not on the default allowlist",
        );

        // tool_use must NOT fire.
        let t = make_transition("a2", Some("tool_use"), None);
        assert!(
            agent_stop_reason_transition_event(&config, &conn, &t)
                .unwrap()
                .is_none(),
            "tool_use is not on the default allowlist",
        );

        // max_tokens fires.
        let t = make_transition("a3", Some("max_tokens"), Some("tool_use"));
        let event = agent_stop_reason_transition_event(&config, &conn, &t)
            .unwrap()
            .expect("max_tokens must fire");
        assert_eq!(event.event_type, "agent_stop_reason_transition");
        assert_eq!(event.details["stop_reason"], "max_tokens");

        // refusal fires.
        let t = make_transition("a4", Some("refusal"), Some("tool_use"));
        let event = agent_stop_reason_transition_event(&config, &conn, &t)
            .unwrap()
            .expect("refusal must fire");
        assert_eq!(event.details["stop_reason"], "refusal");
    }

    #[test]
    fn agent_stop_reason_event_custom_filter_overrides_default() {
        let conn = stop_reason_test_conn();
        let config =
            stop_reason_config(true, Some(vec!["stop_sequence".into(), "tool_use".into()]));

        // max_tokens (a default value) is NOT in the custom list → no fire.
        let t = make_transition("a1", Some("max_tokens"), Some("end_turn"));
        assert!(
            agent_stop_reason_transition_event(&config, &conn, &t)
                .unwrap()
                .is_none(),
            "custom filter excludes max_tokens",
        );

        // tool_use IS in the custom list → fires.
        let t = make_transition("a2", Some("tool_use"), Some("end_turn"));
        let event = agent_stop_reason_transition_event(&config, &conn, &t)
            .unwrap()
            .expect("tool_use must fire under custom filter");
        assert_eq!(event.details["stop_reason"], "tool_use");
    }

    #[test]
    fn agent_stop_reason_event_skips_when_value_unchanged() {
        let conn = stop_reason_test_conn();
        let config = stop_reason_config(true, None);
        // prev == new → no transition.
        let t = make_transition("a1", Some("max_tokens"), Some("max_tokens"));
        assert!(
            agent_stop_reason_transition_event(&config, &conn, &t)
                .unwrap()
                .is_none(),
            "unchanged value must not fire",
        );
    }

    #[test]
    fn agent_stop_reason_event_skips_when_persistent_dedup_matches() {
        let conn = stop_reason_test_conn();
        let config = stop_reason_config(true, None);

        // First emission writes the value into webhook_emitted.
        let t = make_transition("a1", Some("max_tokens"), None);
        let _ = agent_stop_reason_transition_event(&config, &conn, &t)
            .unwrap()
            .expect("first emission fires");

        // Same agent_id with same new value but a different `prev` (e.g. the
        // in-row prev got cleared) — persistent dedup must still suppress it.
        let t = make_transition("a1", Some("max_tokens"), Some("end_turn"));
        assert!(
            agent_stop_reason_transition_event(&config, &conn, &t)
                .unwrap()
                .is_none(),
            "persistent dedup must suppress identical re-emission",
        );
    }

    #[test]
    fn agent_stop_reason_event_fires_on_first_emission_if_in_filter() {
        let conn = stop_reason_test_conn();
        let config = stop_reason_config(true, None);
        let t = make_transition("brand-new-agent", Some("max_tokens"), None);
        let event = agent_stop_reason_transition_event(&config, &conn, &t)
            .unwrap()
            .expect("first emission must fire");
        assert_eq!(event.event_type, "agent_stop_reason_transition");
        assert_eq!(event.details["agent_id"], "brand-new-agent");
        assert_eq!(event.details["prev_stop_reason"], serde_json::Value::Null);

        // The dedup row must have been written.
        let stored = db::get_webhook_emitted(
            &conn,
            "agent_stop_reason_transition",
            "subagent|my-project|brand-new-agent",
        )
        .unwrap();
        assert_eq!(stored.as_deref(), Some("max_tokens"));
    }

    #[test]
    fn agent_stop_reason_event_fires_again_when_value_transitions_to_new_filtered_value() {
        let conn = stop_reason_test_conn();
        let config = stop_reason_config(true, None);

        // First: max_tokens.
        let t = make_transition("a1", Some("max_tokens"), None);
        let _ = agent_stop_reason_transition_event(&config, &conn, &t)
            .unwrap()
            .expect("max_tokens must fire");

        // Then: refusal — different value, also on allowlist → fires again.
        let t = make_transition("a1", Some("refusal"), Some("max_tokens"));
        let event = agent_stop_reason_transition_event(&config, &conn, &t)
            .unwrap()
            .expect("refusal after max_tokens must fire");
        assert_eq!(event.details["stop_reason"], "refusal");

        // The dedup row was updated.
        let stored = db::get_webhook_emitted(
            &conn,
            "agent_stop_reason_transition",
            "subagent|my-project|a1",
        )
        .unwrap();
        assert_eq!(stored.as_deref(), Some("refusal"));
    }
}
