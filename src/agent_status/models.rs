use serde::{Deserialize, Serialize};

/// Overall severity indicator (mirrors Atlassian Statuspage convention).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StatusIndicator {
    /// All systems operational.
    #[default]
    None,
    /// Minor degradation.
    Minor,
    /// Major outage.
    Major,
    /// Critical / full outage.
    Critical,
    /// Scheduled maintenance window.
    Maintenance,
    /// Unrecognised value from the wire.
    Unknown,
}

impl StatusIndicator {
    /// Parse from a Statuspage indicator string.
    pub fn parse_indicator(s: &str) -> Self {
        match s {
            "none" => StatusIndicator::None,
            "minor" => StatusIndicator::Minor,
            "major" => StatusIndicator::Major,
            "critical" => StatusIndicator::Critical,
            "maintenance" => StatusIndicator::Maintenance,
            other => {
                tracing::warn!("unknown status indicator: {}", other);
                StatusIndicator::Unknown
            }
        }
    }

    /// Returns true when the indicator is Major or Critical (alert-worthy).
    pub fn is_alert_worthy(&self) -> bool {
        matches!(self, StatusIndicator::Major | StatusIndicator::Critical)
    }

    /// Returns true when the indicator represents normal/non-degraded operation.
    pub fn is_operational(&self) -> bool {
        matches!(
            self,
            StatusIndicator::None | StatusIndicator::Maintenance | StatusIndicator::Unknown
        )
    }
}

/// Per-component health snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentStatus {
    /// Stable component identifier used as the history key.
    /// For Claude: the Statuspage component UUID (e.g. `yyzkbfz2thpt`).
    /// For OpenAI: the component name (no stable UUID available from the shim).
    pub id: String,
    pub name: String,
    pub status: String,
    /// Rolling 30-day uptime percentage (0.0–1.0). `None` until ≥10 samples exist in the window.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uptime_30d: Option<f64>,
    /// Rolling 7-day uptime percentage (0.0–1.0). `None` until ≥10 samples exist in the window.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uptime_7d: Option<f64>,
}

/// Summary of an active incident.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IncidentSummary {
    pub name: String,
    pub impact: String,
    pub status: String,
    pub shortlink: Option<String>,
    pub started_at: String,
}

/// Aggregated health for one provider.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProviderStatus {
    pub indicator: StatusIndicator,
    pub description: String,
    pub components: Vec<ComponentStatus>,
    pub active_incidents: Vec<IncidentSummary>,
    pub page_url: String,
}

/// Top-level snapshot returned by `GET /api/agent-status`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentStatusSnapshot {
    pub claude: Option<ProviderStatus>,
    pub openai: Option<ProviderStatus>,
    /// RFC 3339 timestamp.
    pub fetched_at: String,
}

// ── Raw Statuspage wire shapes ──────────────────────────────────────────────

/// `GET /api/v2/summary.json` — used by Claude (status.claude.com).
#[derive(Debug, Deserialize)]
pub struct StatuspageSummary {
    pub status: StatuspageStatus,
    pub components: Vec<StatuspageComponent>,
    pub incidents: Vec<StatuspageIncident>,
}

#[derive(Debug, Deserialize)]
pub struct StatuspageStatus {
    pub indicator: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct StatuspageComponent {
    pub id: String,
    pub name: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct StatuspageIncident {
    pub name: String,
    pub impact: String,
    pub status: String,
    pub shortlink: Option<String>,
    pub created_at: String,
}

/// `GET /api/v2/status.json` — used by OpenAI (status.openai.com).
#[derive(Debug, Deserialize)]
pub struct OpenAiStatusResponse {
    pub status: StatuspageStatus,
}

/// `GET /api/v2/incidents.json` — used by OpenAI (status.openai.com).
#[derive(Debug, Deserialize)]
pub struct OpenAiIncidentsResponse {
    pub incidents: Vec<StatuspageIncident>,
}

/// Injected responses for the test seam (mirrors currency/litellm pattern).
pub struct InjectedResponses {
    /// Raw JSON body for Claude `GET /api/v2/summary.json`.
    pub claude_summary: Option<String>,
    /// Raw JSON body for OpenAI `GET /api/v2/status.json`.
    pub openai_status: Option<String>,
    /// Raw JSON body for OpenAI `GET /api/v2/incidents.json`.
    pub openai_incidents: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indicator_from_str_all_variants() {
        assert_eq!(
            StatusIndicator::parse_indicator("none"),
            StatusIndicator::None
        );
        assert_eq!(
            StatusIndicator::parse_indicator("minor"),
            StatusIndicator::Minor
        );
        assert_eq!(
            StatusIndicator::parse_indicator("major"),
            StatusIndicator::Major
        );
        assert_eq!(
            StatusIndicator::parse_indicator("critical"),
            StatusIndicator::Critical
        );
        assert_eq!(
            StatusIndicator::parse_indicator("maintenance"),
            StatusIndicator::Maintenance
        );
        assert_eq!(
            StatusIndicator::parse_indicator("unknown_value"),
            StatusIndicator::Unknown
        );
    }

    #[test]
    fn test_indicator_is_alert_worthy() {
        assert!(!StatusIndicator::None.is_alert_worthy());
        assert!(!StatusIndicator::Minor.is_alert_worthy());
        assert!(StatusIndicator::Major.is_alert_worthy());
        assert!(StatusIndicator::Critical.is_alert_worthy());
        assert!(!StatusIndicator::Maintenance.is_alert_worthy());
        assert!(!StatusIndicator::Unknown.is_alert_worthy());
    }

    #[test]
    fn test_indicator_is_operational() {
        assert!(StatusIndicator::None.is_operational());
        assert!(!StatusIndicator::Minor.is_operational());
        assert!(!StatusIndicator::Major.is_operational());
        assert!(!StatusIndicator::Critical.is_operational());
        assert!(StatusIndicator::Maintenance.is_operational());
        assert!(StatusIndicator::Unknown.is_operational());
    }

    #[test]
    fn test_snapshot_default() {
        let snap = AgentStatusSnapshot::default();
        assert!(snap.claude.is_none());
        assert!(snap.openai.is_none());
        assert!(snap.fetched_at.is_empty());
    }

    #[test]
    fn test_claude_summary_operational_parse() {
        let fixture =
            include_str!("../../tests/fixtures/agent_status/claude_summary_operational.json");
        let summary: StatuspageSummary =
            serde_json::from_str(fixture).expect("should parse operational fixture");
        assert_eq!(summary.status.indicator, "none");
        assert_eq!(summary.status.description, "All Systems Operational");
        assert!(summary.incidents.is_empty());
        // Should have 3 components in the fixture
        assert_eq!(summary.components.len(), 3);
    }

    #[test]
    fn test_claude_summary_major_parse() {
        let fixture =
            include_str!("../../tests/fixtures/agent_status/claude_summary_major_incident.json");
        let summary: StatuspageSummary =
            serde_json::from_str(fixture).expect("should parse major fixture");
        assert_eq!(summary.status.indicator, "major");
        assert_eq!(summary.incidents.len(), 1);
        let inc = &summary.incidents[0];
        assert_eq!(inc.impact, "major");
        assert_eq!(inc.status, "investigating");
        assert!(inc.shortlink.is_some());
    }

    #[test]
    fn test_openai_status_operational_parse() {
        let fixture =
            include_str!("../../tests/fixtures/agent_status/openai_status_operational.json");
        let resp: super::OpenAiStatusResponse =
            serde_json::from_str(fixture).expect("should parse openai status fixture");
        assert_eq!(resp.status.indicator, "none");
    }

    #[test]
    fn test_openai_incidents_degraded_parse() {
        let fixture =
            include_str!("../../tests/fixtures/agent_status/openai_incidents_degraded.json");
        let resp: super::OpenAiIncidentsResponse =
            serde_json::from_str(fixture).expect("should parse openai incidents fixture");
        assert_eq!(resp.incidents.len(), 1);
        assert_eq!(resp.incidents[0].impact, "major");
        assert_ne!(resp.incidents[0].status, "resolved");
    }
}
