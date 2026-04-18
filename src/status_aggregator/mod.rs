pub mod backend;
pub mod models;
pub mod statusgator;

use crate::config::AggregatorConfig;

use self::backend::StatusAggregatorBackend;
use self::models::CommunitySignal;
use self::statusgator::StatusGatorBackend;

/// Poll the configured community-signal backend and return a `CommunitySignal`.
///
/// This is a synchronous function intended to be called from
/// `tokio::task::spawn_blocking`.
pub fn poll(config: &AggregatorConfig) -> CommunitySignal {
    match config.provider.as_str() {
        "statusgator" => StatusGatorBackend.fetch(config),
        other => {
            tracing::warn!(
                "Unknown status_aggregator provider '{}'; returning disabled signal",
                other
            );
            CommunitySignal::disabled()
        }
    }
}

/// Test seam: inject a pre-built `CommunitySignal` and return it unchanged.
///
/// Mirrors `agent_status::poll_with_injection` so server tests can exercise
/// the handler's cache/webhook logic without network calls.
pub fn poll_with_injection(signal: CommunitySignal) -> CommunitySignal {
    signal
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AggregatorConfig;

    fn enabled_config() -> AggregatorConfig {
        AggregatorConfig {
            enabled: true,
            provider: "statusgator".into(),
            api_key_env: "STATUSGATOR_API_KEY".into(),
            refresh_interval: 300,
            claude_services: vec!["claude-ai".into()],
            openai_services: vec!["openai".into()],
            spike_webhook: true,
        }
    }

    #[test]
    fn test_poll_unknown_provider_returns_disabled() {
        let mut cfg = enabled_config();
        cfg.provider = "nonexistent_provider".into();
        let signal = poll(&cfg);
        assert!(!signal.enabled);
    }

    #[test]
    fn test_poll_with_injection_round_trips() {
        let signal = CommunitySignal::disabled();
        let returned = poll_with_injection(signal.clone());
        assert_eq!(returned.enabled, signal.enabled);
    }

    #[test]
    fn test_poll_with_injection_enabled_signal() {
        use super::models::{ServiceSignal, SignalLevel};
        let signal = CommunitySignal {
            fetched_at: "2026-04-17T10:00:00Z".into(),
            enabled: true,
            claude: vec![ServiceSignal {
                slug: "claude-ai".into(),
                name: "Claude AI".into(),
                level: SignalLevel::Normal,
                report_count_last_hour: None,
                report_baseline: None,
                detail: "normal activity".into(),
                source_url: "https://statusgator.com/services/claude-ai".into(),
            }],
            openai: vec![],
        };
        let returned = poll_with_injection(signal);
        assert!(returned.enabled);
        assert_eq!(returned.claude.len(), 1);
        assert_eq!(returned.claude[0].level, SignalLevel::Normal);
    }
}
