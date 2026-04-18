// Heimdall deliberately does not integrate claudestatus.com because it exposes
// no API and its RSS feed is redundant with status.claude.com. The rolling
// uptime signal is computed from owned data (agent_status_history table) instead.

pub mod client;
pub mod filter;
pub mod models;

use crate::agent_status::client::parse_injected;
use crate::agent_status::models::{AgentStatusSnapshot, InjectedResponses};
use crate::config::AgentStatusConfig;

/// Poll both providers and return a fresh snapshot plus Claude's new ETag.
///
/// Each provider fetch is guarded by its enable flag — disabling OpenAI means
/// no network call is made for OpenAI.
pub fn poll(
    config: &AgentStatusConfig,
    cached_etag: Option<&str>,
) -> (AgentStatusSnapshot, Option<String>) {
    let mut snapshot = AgentStatusSnapshot {
        fetched_at: chrono::Utc::now().to_rfc3339(),
        ..Default::default()
    };
    let mut new_etag: Option<String> = cached_etag.map(|s| s.to_owned());

    if config.claude_enabled {
        match client::fetch_claude(cached_etag) {
            Some((status, etag)) => {
                new_etag = etag.or_else(|| cached_etag.map(|s| s.to_owned()));
                snapshot.claude = Some(status);
            }
            None => {
                tracing::debug!("Claude status fetch returned None (304 or error)");
            }
        }
    }

    if config.openai_enabled {
        match client::fetch_openai() {
            Some(status) => {
                snapshot.openai = Some(status);
            }
            None => {
                tracing::debug!("OpenAI status fetch returned None (error)");
            }
        }
    }

    (snapshot, new_etag)
}

/// Test seam — accepts pre-canned JSON bodies and returns the aggregated
/// snapshot without making any network calls.
///
/// Mirrors `currency::convert_with_snapshot` and `litellm::run_refresh_with_snapshot`.
pub fn poll_with_injection(injected: InjectedResponses) -> AgentStatusSnapshot {
    let (claude, openai) = parse_injected(&injected);
    AgentStatusSnapshot {
        claude,
        openai,
        fetched_at: chrono::Utc::now().to_rfc3339(),
    }
}

/// Severity-floor alert check.
///
/// Returns `true` when the transition from `previous` to `current` crosses the
/// Major threshold in either direction (degraded or restored).
pub fn is_alert_transition(
    previous_degraded: Option<bool>,
    current_indicator: &models::StatusIndicator,
    min_severity: &models::StatusIndicator,
) -> Option<AlertDirection> {
    let _ = min_severity; // Reserved for configurable floor; currently fixed at Major.
    let is_now_degraded = current_indicator.is_alert_worthy();

    match previous_degraded {
        Some(was_degraded) if was_degraded == is_now_degraded => None,
        Some(false) if is_now_degraded => Some(AlertDirection::Degraded),
        Some(true) if !is_now_degraded => Some(AlertDirection::Restored),
        None => {
            // First observation — set the baseline; don't fire.
            None
        }
        _ => None,
    }
}

/// Direction of an agent-status alert transition.
#[derive(Debug, PartialEq, Eq)]
pub enum AlertDirection {
    /// Crossed from non-alert to Major/Critical.
    Degraded,
    /// Crossed from Major/Critical back to non-alert.
    Restored,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_status::models::StatusIndicator;

    #[test]
    fn test_alert_transition_op_to_major() {
        let dir = is_alert_transition(
            Some(false),
            &StatusIndicator::Major,
            &StatusIndicator::Major,
        );
        assert_eq!(dir, Some(AlertDirection::Degraded));
    }

    #[test]
    fn test_alert_transition_major_to_op() {
        let dir = is_alert_transition(Some(true), &StatusIndicator::None, &StatusIndicator::Major);
        assert_eq!(dir, Some(AlertDirection::Restored));
    }

    #[test]
    fn test_no_transition_same_state() {
        let dir = is_alert_transition(
            Some(true),
            &StatusIndicator::Critical,
            &StatusIndicator::Major,
        );
        assert_eq!(dir, None);
        let dir2 =
            is_alert_transition(Some(false), &StatusIndicator::None, &StatusIndicator::Major);
        assert_eq!(dir2, None);
    }

    #[test]
    fn test_no_fire_on_first_observation() {
        let dir = is_alert_transition(None, &StatusIndicator::Major, &StatusIndicator::Major);
        assert_eq!(dir, None);
    }

    #[test]
    fn test_op_to_minor_does_not_cross_threshold() {
        // Minor is below the Major floor — no alert.
        let dir = is_alert_transition(
            Some(false),
            &StatusIndicator::Minor,
            &StatusIndicator::Major,
        );
        assert_eq!(dir, None); // Minor is NOT alert-worthy
    }

    #[test]
    fn test_major_to_minor_is_restored() {
        // Major -> Minor crosses back below the threshold (restored).
        let dir = is_alert_transition(Some(true), &StatusIndicator::Minor, &StatusIndicator::Major);
        assert_eq!(dir, Some(AlertDirection::Restored));
    }

    #[test]
    fn test_op_to_critical_fires_degraded() {
        let dir = is_alert_transition(
            Some(false),
            &StatusIndicator::Critical,
            &StatusIndicator::Major,
        );
        assert_eq!(dir, Some(AlertDirection::Degraded));
    }

    #[test]
    fn test_poll_with_injection_claude_operational() {
        let fixture =
            include_str!("../../tests/fixtures/agent_status/claude_summary_operational.json");
        let snapshot = poll_with_injection(InjectedResponses {
            claude_summary: Some(fixture.to_string()),
            openai_status: None,
            openai_incidents: None,
        });
        let claude = snapshot.claude.expect("claude should be present");
        assert_eq!(claude.indicator, StatusIndicator::None);
        assert!(!snapshot.fetched_at.is_empty());
    }

    #[test]
    fn test_poll_with_injection_claude_major() {
        let fixture =
            include_str!("../../tests/fixtures/agent_status/claude_summary_major_incident.json");
        let snapshot = poll_with_injection(InjectedResponses {
            claude_summary: Some(fixture.to_string()),
            openai_status: None,
            openai_incidents: None,
        });
        let claude = snapshot.claude.expect("claude should be present");
        assert_eq!(claude.indicator, StatusIndicator::Major);
        assert!(!claude.active_incidents.is_empty());
    }

    #[test]
    fn test_poll_with_injection_openai_operational() {
        let status_fixture =
            include_str!("../../tests/fixtures/agent_status/openai_status_operational.json");
        let incidents_fixture =
            include_str!("../../tests/fixtures/agent_status/openai_incidents_operational.json");
        let snapshot = poll_with_injection(InjectedResponses {
            claude_summary: None,
            openai_status: Some(status_fixture.to_string()),
            openai_incidents: Some(incidents_fixture.to_string()),
        });
        let openai = snapshot.openai.expect("openai should be present");
        assert_eq!(openai.indicator, StatusIndicator::None);
        assert!(openai.active_incidents.is_empty());
    }

    #[test]
    fn test_poll_with_injection_openai_degraded() {
        let status_fixture =
            include_str!("../../tests/fixtures/agent_status/openai_status_degraded.json");
        let incidents_fixture =
            include_str!("../../tests/fixtures/agent_status/openai_incidents_degraded.json");
        let snapshot = poll_with_injection(InjectedResponses {
            claude_summary: None,
            openai_status: Some(status_fixture.to_string()),
            openai_incidents: Some(incidents_fixture.to_string()),
        });
        let openai = snapshot.openai.expect("openai should be present");
        assert!(openai.indicator.is_alert_worthy());
        assert!(!openai.active_incidents.is_empty());
    }
}
