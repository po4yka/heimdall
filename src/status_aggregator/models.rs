use serde::{Deserialize, Serialize};

/// Multiplier above baseline to classify as Spike.
pub const SPIKE_MULTIPLIER: f64 = 3.0;
/// Multiplier above baseline to classify as Elevated.
pub const ELEVATED_MULTIPLIER: f64 = 1.5;

/// Crowd-sourced signal level for a service.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalLevel {
    Normal,
    Elevated,
    Spike,
    Unknown,
}

impl SignalLevel {
    /// Derive a level from the current report count and baseline.
    ///
    /// - `count > baseline * SPIKE_MULTIPLIER` → Spike
    /// - `count > baseline * ELEVATED_MULTIPLIER` → Elevated
    /// - otherwise → Normal
    ///
    /// Returns `Unknown` when baseline is None or zero.
    pub fn from_counts(count: Option<i64>, baseline: Option<i64>) -> Self {
        match (count, baseline) {
            (Some(c), Some(b)) if b > 0 => {
                let ratio = c as f64 / b as f64;
                if ratio > SPIKE_MULTIPLIER {
                    Self::Spike
                } else if ratio > ELEVATED_MULTIPLIER {
                    Self::Elevated
                } else {
                    Self::Normal
                }
            }
            (Some(_c), Some(0)) => Self::Normal,
            _ => Self::Unknown,
        }
    }

    /// Parse the string level that StatusGator returns in its `status` field.
    pub fn from_statusgator_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "up" | "operational" | "normal" => Self::Normal,
            "degraded" | "issues" | "elevated" => Self::Elevated,
            "down" | "outage" | "spike" => Self::Spike,
            _ => Self::Unknown,
        }
    }
}

/// Signal for a single monitored service (one StatusGator slug).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceSignal {
    pub slug: String,
    pub name: String,
    pub level: SignalLevel,
    pub report_count_last_hour: Option<i64>,
    pub report_baseline: Option<i64>,
    pub detail: String,
    pub source_url: String,
}

/// Aggregated community signal across all configured services.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunitySignal {
    /// RFC 3339 timestamp of when this snapshot was fetched.
    pub fetched_at: String,
    /// Per-slug signals for Claude services.
    pub claude: Vec<ServiceSignal>,
    /// Per-slug signals for OpenAI services.
    pub openai: Vec<ServiceSignal>,
    /// Whether the aggregator is enabled in config.
    pub enabled: bool,
}

impl CommunitySignal {
    /// Construct a sentinel value used when the aggregator is disabled.
    pub fn disabled() -> Self {
        Self {
            fetched_at: chrono::Utc::now().to_rfc3339(),
            claude: vec![],
            openai: vec![],
            enabled: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_level_from_counts_spike() {
        // 31/10 = 3.1 > SPIKE_MULTIPLIER(3.0) → Spike
        assert_eq!(
            SignalLevel::from_counts(Some(31), Some(10)),
            SignalLevel::Spike
        );
    }

    #[test]
    fn test_signal_level_from_counts_elevated() {
        // 2x baseline → Elevated (> 1.5, ≤ 3.0)
        assert_eq!(
            SignalLevel::from_counts(Some(20), Some(10)),
            SignalLevel::Elevated
        );
    }

    #[test]
    fn test_signal_level_from_counts_normal() {
        // Same as baseline → Normal
        assert_eq!(
            SignalLevel::from_counts(Some(10), Some(10)),
            SignalLevel::Normal
        );
    }

    #[test]
    fn test_signal_level_from_counts_zero_baseline_is_normal() {
        assert_eq!(
            SignalLevel::from_counts(Some(5), Some(0)),
            SignalLevel::Normal
        );
    }

    #[test]
    fn test_signal_level_from_counts_none_count_is_unknown() {
        assert_eq!(
            SignalLevel::from_counts(None, Some(10)),
            SignalLevel::Unknown
        );
    }

    #[test]
    fn test_signal_level_from_counts_none_baseline_is_unknown() {
        assert_eq!(
            SignalLevel::from_counts(Some(10), None),
            SignalLevel::Unknown
        );
    }

    #[test]
    fn test_signal_level_from_counts_both_none_is_unknown() {
        assert_eq!(SignalLevel::from_counts(None, None), SignalLevel::Unknown);
    }

    #[test]
    fn test_signal_level_from_counts_boundary_elevated() {
        // Exactly at ELEVATED_MULTIPLIER boundary: 15/10 = 1.5, NOT > 1.5 → Normal
        assert_eq!(
            SignalLevel::from_counts(Some(15), Some(10)),
            SignalLevel::Normal
        );
    }

    #[test]
    fn test_signal_level_from_counts_just_above_elevated() {
        // 16/10 = 1.6 > 1.5 → Elevated
        assert_eq!(
            SignalLevel::from_counts(Some(16), Some(10)),
            SignalLevel::Elevated
        );
    }

    #[test]
    fn test_signal_level_from_counts_boundary_spike() {
        // Exactly at SPIKE_MULTIPLIER boundary: 30/10 = 3.0, NOT > 3.0 → Elevated
        assert_eq!(
            SignalLevel::from_counts(Some(30), Some(10)),
            SignalLevel::Elevated
        );
    }

    #[test]
    fn test_signal_level_from_statusgator_str_up() {
        assert_eq!(SignalLevel::from_statusgator_str("up"), SignalLevel::Normal);
        assert_eq!(
            SignalLevel::from_statusgator_str("operational"),
            SignalLevel::Normal
        );
        assert_eq!(
            SignalLevel::from_statusgator_str("normal"),
            SignalLevel::Normal
        );
    }

    #[test]
    fn test_signal_level_from_statusgator_str_degraded() {
        assert_eq!(
            SignalLevel::from_statusgator_str("degraded"),
            SignalLevel::Elevated
        );
        assert_eq!(
            SignalLevel::from_statusgator_str("issues"),
            SignalLevel::Elevated
        );
    }

    #[test]
    fn test_signal_level_from_statusgator_str_down() {
        assert_eq!(
            SignalLevel::from_statusgator_str("down"),
            SignalLevel::Spike
        );
        assert_eq!(
            SignalLevel::from_statusgator_str("outage"),
            SignalLevel::Spike
        );
    }

    #[test]
    fn test_signal_level_from_statusgator_str_unknown() {
        assert_eq!(SignalLevel::from_statusgator_str(""), SignalLevel::Unknown);
        assert_eq!(
            SignalLevel::from_statusgator_str("maintenance"),
            SignalLevel::Unknown
        );
    }

    #[test]
    fn test_signal_level_serde_roundtrip() {
        let levels = [
            SignalLevel::Normal,
            SignalLevel::Elevated,
            SignalLevel::Spike,
            SignalLevel::Unknown,
        ];
        for level in &levels {
            let json = serde_json::to_string(level).expect("serialize");
            let back: SignalLevel = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(*level, back);
        }
    }

    #[test]
    fn test_community_signal_disabled() {
        let sig = CommunitySignal::disabled();
        assert!(!sig.enabled);
        assert!(sig.claude.is_empty());
        assert!(sig.openai.is_empty());
    }
}
