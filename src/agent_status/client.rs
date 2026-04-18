use std::time::Duration;

use crate::agent_status::filter::{claude_component_allowed, openai_component_allowed};
use crate::agent_status::models::{
    ComponentStatus, IncidentSummary, InjectedResponses, OpenAiIncidentsResponse,
    OpenAiStatusResponse, ProviderStatus, StatusIndicator, StatuspageSummary,
};

const CLAUDE_BASE: &str = "https://status.claude.com";
const OPENAI_BASE: &str = "https://status.openai.com";
const FETCH_TIMEOUT_SECS: u64 = 10;
const USER_AGENT: &str = concat!("heimdall-status-monitor/", env!("CARGO_PKG_VERSION"));

/// Fetch Claude provider status using the Statuspage summary endpoint.
///
/// Returns `(ProviderStatus, new_etag)`. Supports conditional GET via
/// `If-None-Match` — callers pass their stored ETag and receive `None` on 304.
/// Returns `None` on any network or parse error.
pub fn fetch_claude(cached_etag: Option<&str>) -> Option<(ProviderStatus, Option<String>)> {
    fetch_claude_from(CLAUDE_BASE, cached_etag, None)
}

/// Internal fetch with URL and optional injected body (for testing).
pub fn fetch_claude_from(
    base_url: &str,
    cached_etag: Option<&str>,
    injected_body: Option<String>,
) -> Option<(ProviderStatus, Option<String>)> {
    if let Some(body) = injected_body {
        let summary: StatuspageSummary = serde_json::from_str(&body).ok()?;
        return Some((build_claude_status(summary), None));
    }

    let url = format!("{}/api/v2/summary.json", base_url);
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .ok()?;
    rt.block_on(async {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(FETCH_TIMEOUT_SECS))
            .user_agent(USER_AGENT)
            .build()
            .ok()?;

        let mut req = client.get(&url);
        if let Some(etag) = cached_etag {
            req = req.header("If-None-Match", etag);
        }

        let resp = req.send().await.ok()?;

        if resp.status() == reqwest::StatusCode::NOT_MODIFIED {
            tracing::debug!("Claude status: 304 Not Modified (cache hit)");
            return None;
        }

        let new_etag = resp
            .headers()
            .get("etag")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_owned());

        let body = resp.text().await.ok()?;
        let summary: StatuspageSummary = serde_json::from_str(&body)
            .map_err(|e| tracing::debug!("Claude status parse error: {}", e))
            .ok()?;

        Some((build_claude_status(summary), new_etag))
    })
}

fn build_claude_status(summary: StatuspageSummary) -> ProviderStatus {
    let indicator = StatusIndicator::parse_indicator(&summary.status.indicator);

    let components: Vec<ComponentStatus> = summary
        .components
        .into_iter()
        .filter(|c| claude_component_allowed(&c.id))
        .map(|c| ComponentStatus {
            id: c.id,
            name: c.name,
            status: c.status,
            uptime_30d: None,
            uptime_7d: None,
        })
        .collect();

    let active_incidents: Vec<IncidentSummary> = summary
        .incidents
        .into_iter()
        .filter(|i| i.status != "resolved")
        .map(|i| IncidentSummary {
            name: i.name,
            impact: i.impact,
            status: i.status,
            shortlink: i.shortlink,
            started_at: i.created_at,
        })
        .collect();

    ProviderStatus {
        indicator,
        description: summary.status.description,
        components,
        active_incidents,
        page_url: "https://status.claude.com".to_string(),
    }
}

/// Fetch OpenAI provider status using two separate endpoints.
/// Returns `None` on any network or parse error.
pub fn fetch_openai() -> Option<ProviderStatus> {
    fetch_openai_from(OPENAI_BASE, None, None)
}

/// Internal fetch with base URL and optional injected bodies (for testing).
pub fn fetch_openai_from(
    base_url: &str,
    injected_status: Option<String>,
    injected_incidents: Option<String>,
) -> Option<ProviderStatus> {
    let status_body = if let Some(body) = injected_status {
        body
    } else {
        fetch_openai_raw(base_url, "status")?
    };

    let incidents_body = if let Some(body) = injected_incidents {
        body
    } else {
        fetch_openai_raw(base_url, "incidents")?
    };

    let status_resp: OpenAiStatusResponse = serde_json::from_str(&status_body)
        .map_err(|e| tracing::debug!("OpenAI status parse error: {}", e))
        .ok()?;
    let incidents_resp: OpenAiIncidentsResponse = serde_json::from_str(&incidents_body)
        .map_err(|e| tracing::debug!("OpenAI incidents parse error: {}", e))
        .ok()?;

    let indicator = StatusIndicator::parse_indicator(&status_resp.status.indicator);

    let active_incidents: Vec<IncidentSummary> = incidents_resp
        .incidents
        .into_iter()
        .filter(|i| i.status != "resolved")
        .map(|i| IncidentSummary {
            name: i.name,
            impact: i.impact,
            status: i.status,
            shortlink: i.shortlink,
            started_at: i.created_at,
        })
        .collect();

    Some(ProviderStatus {
        indicator,
        description: status_resp.status.description,
        components: vec![], // OpenAI components come from incidents, not a summary endpoint
        active_incidents,
        page_url: "https://status.openai.com".to_string(),
    })
}

fn fetch_openai_raw(base_url: &str, endpoint: &str) -> Option<String> {
    let url = format!("{}/api/v2/{}.json", base_url, endpoint);
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .ok()?;
    rt.block_on(async {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(FETCH_TIMEOUT_SECS))
            .user_agent(USER_AGENT)
            .build()
            .ok()?;
        let resp = client.get(&url).send().await.ok()?;
        resp.text().await.ok()
    })
}

/// Fetch only component names from OpenAI incidents for the allowlist.
pub fn extract_openai_affected_components(incidents: &[IncidentSummary]) -> Vec<ComponentStatus> {
    // OpenAI's incident.io shim does not expose a separate components endpoint
    // in the same shape as Statuspage; derive components from incident impact.
    let _ = incidents;
    vec![]
}

/// Parse OpenAI status JSON for the test seam (accepts injected bodies).
pub fn parse_injected(
    injected: &InjectedResponses,
) -> (Option<ProviderStatus>, Option<ProviderStatus>) {
    let claude = injected.claude_summary.as_deref().and_then(|body| {
        let summary: StatuspageSummary = serde_json::from_str(body)
            .map_err(|e| tracing::debug!("injected Claude parse error: {}", e))
            .ok()?;
        Some(build_claude_status(summary))
    });

    let openai = match (&injected.openai_status, &injected.openai_incidents) {
        (Some(s), Some(i)) => fetch_openai_from("", Some(s.clone()), Some(i.clone())),
        _ => None,
    };

    (claude, openai)
}

/// Filter OpenAI component list by allowlist.
pub fn filter_openai_components(components: Vec<ComponentStatus>) -> Vec<ComponentStatus> {
    components
        .into_iter()
        .filter(|c| openai_component_allowed(&c.name))
        .collect()
}
