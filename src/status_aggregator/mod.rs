pub mod models;
pub mod statusgator;

use crate::config::AggregatorConfig;

use self::models::CommunitySignal;

/// Poll the configured community-signal backend and return a `CommunitySignal`.
///
/// This is a synchronous function intended to be called from
/// `tokio::task::spawn_blocking`.
pub fn poll(config: &AggregatorConfig) -> CommunitySignal {
    match config.provider.as_str() {
        "statusgator" => statusgator::fetch(config),
        other => {
            tracing::warn!(
                "Unknown status_aggregator provider '{}'; returning disabled signal",
                other
            );
            CommunitySignal::disabled()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AggregatorConfig;

    fn enabled_config() -> AggregatorConfig {
        AggregatorConfig {
            enabled: true,
            provider: "statusgator".into(),
            key_env_var: "STATUSGATOR_COMMUNITY_TOKEN".into(),
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
}
