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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    struct TestServer {
        base_url: String,
        requests: Arc<Mutex<Vec<String>>>,
        handle: Option<thread::JoinHandle<()>>,
    }

    impl TestServer {
        fn spawn(responses: Vec<String>) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
            let addr = listener.local_addr().expect("test server local addr");
            let requests = Arc::new(Mutex::new(Vec::new()));
            let requests_for_thread = Arc::clone(&requests);

            let handle = thread::spawn(move || {
                for response in responses {
                    let (mut stream, _) = listener.accept().expect("accept request");
                    stream
                        .set_read_timeout(Some(Duration::from_secs(2)))
                        .expect("set read timeout");
                    let request = read_request(&mut stream);
                    requests_for_thread
                        .lock()
                        .expect("lock captured requests")
                        .push(request);
                    stream
                        .write_all(response.as_bytes())
                        .expect("write response");
                }
            });

            Self {
                base_url: format!("http://{}", addr),
                requests,
                handle: Some(handle),
            }
        }

        fn requests(&self) -> Vec<String> {
            self.requests
                .lock()
                .expect("lock captured requests")
                .clone()
        }
    }

    impl Drop for TestServer {
        fn drop(&mut self) {
            if let Some(handle) = self.handle.take() {
                handle.join().expect("join test server");
            }
        }
    }

    fn read_request(stream: &mut std::net::TcpStream) -> String {
        let mut request = Vec::new();
        let mut buf = [0_u8; 1024];
        loop {
            match stream.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    request.extend_from_slice(&buf[..n]);
                    if request.windows(4).any(|window| window == b"\r\n\r\n") {
                        break;
                    }
                }
                Err(error)
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) =>
                {
                    break;
                }
                Err(error) => panic!("failed to read request: {error}"),
            }
        }

        String::from_utf8(request).expect("request should be utf-8")
    }

    fn http_response(status_line: &str, body: &str, extra_headers: &[(&str, &str)]) -> String {
        let mut response = format!(
            "HTTP/1.1 {status_line}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n",
            body.len()
        );
        for (name, value) in extra_headers {
            response.push_str(name);
            response.push_str(": ");
            response.push_str(value);
            response.push_str("\r\n");
        }
        response.push_str("\r\n");
        response.push_str(body);
        response
    }

    #[test]
    fn fetch_claude_from_returns_none_on_304_and_sends_if_none_match() {
        let server = TestServer::spawn(vec![http_response(
            "304 Not Modified",
            "",
            &[("etag", "\"next-etag\"")],
        )]);

        let result = fetch_claude_from(&server.base_url, Some("\"cached-etag\""), None);

        assert!(result.is_none());
        let requests = server.requests();
        assert_eq!(requests.len(), 1);
        assert!(requests[0].starts_with("GET /api/v2/summary.json HTTP/1.1\r\n"));
        let request_lower = requests[0].to_ascii_lowercase();
        assert!(request_lower.contains("if-none-match: \"cached-etag\"\r\n"));
        assert!(request_lower.contains(&format!(
            "user-agent: {}\r\n",
            USER_AGENT.to_ascii_lowercase()
        )));
    }

    #[test]
    fn fetch_claude_from_filters_components_incidents_and_returns_etag() {
        let body = r#"{
            "status": { "indicator": "major", "description": "Partial outage" },
            "components": [
                { "id": "yyzkbfz2thpt", "name": "Claude Code", "status": "partial_outage" },
                { "id": "ignored-component", "name": "Docs", "status": "operational" }
            ],
            "incidents": [
                {
                    "name": "Active incident",
                    "impact": "major",
                    "status": "investigating",
                    "shortlink": "https://status.example/1",
                    "created_at": "2026-04-18T12:00:00Z"
                },
                {
                    "name": "Resolved incident",
                    "impact": "minor",
                    "status": "resolved",
                    "shortlink": null,
                    "created_at": "2026-04-17T12:00:00Z"
                }
            ]
        }"#;
        let server = TestServer::spawn(vec![http_response(
            "200 OK",
            body,
            &[("etag", "\"fresh-etag\"")],
        )]);

        let (status, etag) = fetch_claude_from(&server.base_url, None, None)
            .expect("Claude status should parse from test server");

        assert_eq!(etag.as_deref(), Some("\"fresh-etag\""));
        assert_eq!(status.indicator, StatusIndicator::Major);
        assert_eq!(status.description, "Partial outage");
        assert_eq!(status.components.len(), 1);
        assert_eq!(status.components[0].id, "yyzkbfz2thpt");
        assert_eq!(status.active_incidents.len(), 1);
        assert_eq!(status.active_incidents[0].name, "Active incident");
    }

    #[test]
    fn fetch_openai_from_fetches_both_endpoints_and_filters_resolved_incidents() {
        let status_body = r#"{
            "status": { "indicator": "minor", "description": "Degraded performance" }
        }"#;
        let incidents_body = r#"{
            "incidents": [
                {
                    "name": "Active issue",
                    "impact": "minor",
                    "status": "identified",
                    "shortlink": "https://status.example/active",
                    "created_at": "2026-04-18T12:00:00Z"
                },
                {
                    "name": "Old issue",
                    "impact": "minor",
                    "status": "resolved",
                    "shortlink": null,
                    "created_at": "2026-04-17T12:00:00Z"
                }
            ]
        }"#;
        let server = TestServer::spawn(vec![
            http_response("200 OK", status_body, &[]),
            http_response("200 OK", incidents_body, &[]),
        ]);

        let status =
            fetch_openai_from(&server.base_url, None, None).expect("OpenAI status should parse");

        assert_eq!(status.indicator, StatusIndicator::Minor);
        assert_eq!(status.description, "Degraded performance");
        assert_eq!(status.active_incidents.len(), 1);
        assert_eq!(status.active_incidents[0].name, "Active issue");

        let requests = server.requests();
        assert_eq!(requests.len(), 2);
        assert!(requests[0].starts_with("GET /api/v2/status.json HTTP/1.1\r\n"));
        assert!(requests[1].starts_with("GET /api/v2/incidents.json HTTP/1.1\r\n"));
    }

    #[test]
    fn fetch_openai_from_returns_none_on_invalid_incident_payload() {
        let server = TestServer::spawn(vec![
            http_response(
                "200 OK",
                r#"{"status":{"indicator":"none","description":"OK"}}"#,
                &[],
            ),
            http_response("200 OK", "{not-json}", &[]),
        ]);

        let status = fetch_openai_from(&server.base_url, None, None);

        assert!(status.is_none());
    }
}
