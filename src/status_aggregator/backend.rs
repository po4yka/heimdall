use super::models::CommunitySignal;
use crate::config::AggregatorConfig;

/// Trait for community-signal backends.
///
/// A backend fetches crowd-sourced service health data for all configured
/// slugs and returns a `CommunitySignal` snapshot. The trait allows future
/// backends (IsDown, etc.) to be added without touching call sites.
pub trait StatusAggregatorBackend {
    /// Fetch a fresh `CommunitySignal` snapshot.
    ///
    /// This is a synchronous call intended to be wrapped in
    /// `tokio::task::spawn_blocking` at the call site.
    fn fetch(&self, config: &AggregatorConfig) -> CommunitySignal;

    /// Human-readable backend name for logging.
    fn name(&self) -> &'static str;
}
