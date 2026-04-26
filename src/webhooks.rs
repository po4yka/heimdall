use serde::Serialize;
use tracing::{debug, error, info};

use crate::config::WebhookConfig;

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
        };
        let mut state = WebhookState::default();

        let first =
            cost_threshold_event(&config, &mut state, "2026-04-10", 60.0).expect("threshold event");
        assert_eq!(first.event_type, "cost_threshold");
        assert!(cost_threshold_event(&config, &mut state, "2026-04-10", 70.0).is_none());
        assert!(cost_threshold_event(&config, &mut state, "2026-04-11", 70.0).is_some());
    }

    // ── Agent status webhook tests ───────────────────────────────────────────

    fn agent_status_config_enabled() -> WebhookConfig {
        WebhookConfig {
            url: Some("https://example.com/hook".to_string()),
            cost_threshold: None,
            session_depleted: false,
            agent_status: true,
            spike_webhook: true,
        }
    }

    fn agent_status_config_disabled() -> WebhookConfig {
        WebhookConfig {
            url: Some("https://example.com/hook".to_string()),
            cost_threshold: None,
            session_depleted: false,
            agent_status: false,
            spike_webhook: true,
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
}
