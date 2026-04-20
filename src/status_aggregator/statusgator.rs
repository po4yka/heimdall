use std::time::Duration;

use tracing::warn;

use crate::config::AggregatorConfig;

use super::models::{CommunitySignal, ServiceSignal, SignalLevel};

const STATUSGATOR_BASE: &str = "https://statusgator.com/api/v3/services";
const REQUEST_TIMEOUT_SECS: u64 = 5;

/// Fetch a fresh community signal snapshot from StatusGator.
pub fn fetch(config: &AggregatorConfig) -> CommunitySignal {
    let api_key = match std::env::var(&config.key_env_var) {
        Ok(k) if !k.trim().is_empty() => k.trim().to_owned(),
        _ => {
            warn!(
                "StatusGator: env var '{}' not set or empty; returning Unknown signals",
                config.key_env_var
            );
            return build_unknown_signal(config);
        }
    };

    fetch_live(config, &api_key)
}

/// Blocking wrapper that drives async HTTP fetches on a single-thread runtime.
///
/// Uses the same pattern as `currency::fetch_from_url` and
/// `litellm::fetch_from_url` to avoid the `reqwest/blocking` feature.
pub(crate) fn fetch_live(config: &AggregatorConfig, api_key: &str) -> CommunitySignal {
    fetch_live_from_base(config, api_key, STATUSGATOR_BASE)
}

fn fetch_live_from_base(
    config: &AggregatorConfig,
    api_key: &str,
    base_url: &str,
) -> CommunitySignal {
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(r) => r,
        Err(e) => {
            warn!("StatusGator: failed to build tokio runtime: {}", e);
            return build_unknown_signal(config);
        }
    };

    let claude_slugs = config.claude_services.clone();
    let openai_slugs = config.openai_services.clone();
    let api_key_owned = api_key.to_owned();
    let key_env_var = config.key_env_var.clone();

    rt.block_on(async move {
        let client = match reqwest::Client::builder()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .user_agent(format!(
                "heimdall-community-signal/{}",
                env!("CARGO_PKG_VERSION")
            ))
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                warn!("StatusGator: failed to build HTTP client: {}", e);
                let detail = format!("set {} to enable community signal", key_env_var);
                return build_unknown_signal_from_slugs(&claude_slugs, &openai_slugs, &detail);
            }
        };

        let mut claude_signals = Vec::with_capacity(claude_slugs.len());
        for slug in &claude_slugs {
            claude_signals
                .push(fetch_slug_async_from_base(&client, slug, &api_key_owned, base_url).await);
        }

        let mut openai_signals = Vec::with_capacity(openai_slugs.len());
        for slug in &openai_slugs {
            openai_signals
                .push(fetch_slug_async_from_base(&client, slug, &api_key_owned, base_url).await);
        }

        CommunitySignal {
            fetched_at: chrono::Utc::now().to_rfc3339(),
            claude: claude_signals,
            openai: openai_signals,
            enabled: true,
        }
    })
}

async fn fetch_slug_async_from_base(
    client: &reqwest::Client,
    slug: &str,
    api_key: &str,
    base_url: &str,
) -> ServiceSignal {
    let url = format!("{}/{}/status", base_url.trim_end_matches('/'), slug);
    let source_url = format!("https://statusgator.com/services/{}", slug);

    let response = match client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            warn!("StatusGator: network error for slug '{}': {}", slug, e);
            return unknown_signal(slug, &source_url, "network error");
        }
    };

    let status = response.status();

    if status == reqwest::StatusCode::UNAUTHORIZED {
        warn!(
            "StatusGator: 401 Unauthorized for slug '{}' — check API key",
            slug
        );
        return unknown_signal(slug, &source_url, "unauthorized (check API key)");
    }

    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        warn!("StatusGator: 429 rate limited for slug '{}'", slug);
        return unknown_signal(slug, &source_url, "rate limited");
    }

    if status.is_server_error() {
        warn!("StatusGator: server error {} for slug '{}'", status, slug);
        return unknown_signal(slug, &source_url, &format!("server error {}", status));
    }

    if !status.is_success() {
        warn!(
            "StatusGator: unexpected status {} for slug '{}'",
            status, slug
        );
        return unknown_signal(slug, &source_url, &format!("HTTP {}", status));
    }

    let body = match response.text().await {
        Ok(t) => t,
        Err(e) => {
            warn!(
                "StatusGator: failed to read body for slug '{}': {}",
                slug, e
            );
            return unknown_signal(slug, &source_url, "failed to read body");
        }
    };

    parse_service_response(slug, &source_url, &body)
}

/// Parse a StatusGator service status JSON body into a `ServiceSignal`.
///
/// Uses permissive `serde_json::Value` first to avoid hard failures on
/// unexpected response shapes, then falls back to raw field extraction.
pub fn parse_service_response(slug: &str, source_url: &str, body: &str) -> ServiceSignal {
    let raw: serde_json::Value = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(e) => {
            warn!("StatusGator: invalid JSON for slug '{}': {}", slug, e);
            return unknown_signal(slug, source_url, "invalid JSON");
        }
    };

    // Extract name from service object.
    let name = raw
        .get("service")
        .and_then(|s| s.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or(slug)
        .to_owned();

    // Extract status string; try multiple plausible field names.
    let status_str = raw
        .get("service")
        .and_then(|s| {
            s.get("current_status")
                .or_else(|| s.get("status"))
                .or_else(|| s.get("state"))
        })
        .and_then(|s| s.as_str())
        .unwrap_or("unknown");

    // Derive level from status string first.
    let level_from_status = SignalLevel::from_statusgator_str(status_str);

    // Extract optional report counts (best-effort; field names vary).
    let report_count = raw
        .get("service")
        .and_then(|s| {
            s.get("reports_last_hour")
                .or_else(|| s.get("report_count_last_hour"))
                .or_else(|| s.get("reports"))
        })
        .and_then(|v| v.as_i64());

    let report_baseline = raw
        .get("service")
        .and_then(|s| {
            s.get("reports_baseline")
                .or_else(|| s.get("report_baseline"))
                .or_else(|| s.get("baseline"))
        })
        .and_then(|v| v.as_i64());

    // If status string gave Unknown and we have counts, derive from counts.
    let level = if level_from_status == SignalLevel::Unknown && report_count.is_some() {
        let count_level = SignalLevel::from_counts(report_count, report_baseline);
        if count_level != SignalLevel::Unknown {
            tracing::debug!(
                "StatusGator: slug '{}' status '{}' → Unknown from string, derived {:?} from counts",
                slug,
                status_str,
                count_level
            );
            count_level
        } else {
            tracing::debug!(
                "StatusGator: slug '{}' status '{}' → Unknown (no usable counts)",
                slug,
                status_str
            );
            SignalLevel::Unknown
        }
    } else {
        level_from_status
    };

    let detail = match &level {
        SignalLevel::Spike => format!("{}: spike in community reports", name),
        SignalLevel::Elevated => format!("{}: elevated community reports", name),
        SignalLevel::Normal => format!("{}: normal activity", name),
        SignalLevel::Unknown => format!("{}: status unknown", name),
    };

    ServiceSignal {
        slug: slug.to_owned(),
        name,
        level,
        report_count_last_hour: report_count,
        report_baseline,
        detail,
        source_url: source_url.to_owned(),
    }
}

fn unknown_signal(slug: &str, source_url: &str, reason: &str) -> ServiceSignal {
    ServiceSignal {
        slug: slug.to_owned(),
        name: slug.to_owned(),
        level: SignalLevel::Unknown,
        report_count_last_hour: None,
        report_baseline: None,
        detail: format!("{}: {}", slug, reason),
        source_url: source_url.to_owned(),
    }
}

fn build_unknown_signal(config: &AggregatorConfig) -> CommunitySignal {
    let detail = format!("set {} to enable community signal", config.key_env_var);
    build_unknown_signal_from_slugs(&config.claude_services, &config.openai_services, &detail)
}

fn build_unknown_signal_from_slugs(
    claude_slugs: &[String],
    openai_slugs: &[String],
    detail: &str,
) -> CommunitySignal {
    let make_signals = |slugs: &[String]| -> Vec<ServiceSignal> {
        slugs
            .iter()
            .map(|slug| ServiceSignal {
                slug: slug.clone(),
                name: slug.clone(),
                level: SignalLevel::Unknown,
                report_count_last_hour: None,
                report_baseline: None,
                detail: detail.to_owned(),
                source_url: format!("https://statusgator.com/services/{}", slug),
            })
            .collect()
    };

    CommunitySignal {
        fetched_at: chrono::Utc::now().to_rfc3339(),
        claude: make_signals(claude_slugs),
        openai: make_signals(openai_slugs),
        enabled: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex, OnceLock};
    use std::thread;

    struct MockResponse {
        status_line: &'static str,
        headers: Vec<(&'static str, String)>,
        body: String,
    }

    impl MockResponse {
        fn json(status_line: &'static str, body: &str) -> Self {
            Self {
                status_line,
                headers: vec![
                    ("Content-Type", "application/json".to_owned()),
                    ("Content-Length", body.len().to_string()),
                    ("Connection", "close".to_owned()),
                ],
                body: body.to_owned(),
            }
        }
    }

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn test_config() -> AggregatorConfig {
        AggregatorConfig {
            enabled: true,
            provider: "statusgator".to_owned(),
            key_env_var: "HEIMDALL_STATUSGATOR_TEST_TOKEN_ENV".to_owned(),
            refresh_interval: 300,
            claude_services: vec!["claude-ai".to_owned()],
            openai_services: vec!["openai".to_owned()],
            spike_webhook: true,
        }
    }

    fn build_client() -> reqwest::Client {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .build()
            .expect("client should build")
    }

    fn read_http_request(stream: &mut std::net::TcpStream) -> String {
        let mut data = Vec::new();
        let mut buf = [0_u8; 1024];
        loop {
            let read = stream.read(&mut buf).expect("request should read");
            if read == 0 {
                break;
            }
            data.extend_from_slice(&buf[..read]);
            if data.windows(4).any(|window| window == b"\r\n\r\n") {
                break;
            }
        }
        String::from_utf8(data).expect("request should be utf8")
    }

    fn spawn_http_server(
        responses: Vec<MockResponse>,
    ) -> (String, Arc<Mutex<Vec<String>>>, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let addr = listener.local_addr().expect("addr should resolve");
        let requests = Arc::new(Mutex::new(Vec::new()));
        let requests_for_thread = Arc::clone(&requests);
        let handle = thread::spawn(move || {
            for response in responses {
                let (mut stream, _) = listener.accept().expect("accept should succeed");
                let request = read_http_request(&mut stream);
                requests_for_thread
                    .lock()
                    .expect("lock should succeed")
                    .push(request);

                let mut raw = format!("HTTP/1.1 {}\r\n", response.status_line);
                for (name, value) in response.headers {
                    raw.push_str(&format!("{name}: {value}\r\n"));
                }
                raw.push_str("\r\n");
                raw.push_str(&response.body);
                stream
                    .write_all(raw.as_bytes())
                    .expect("response should write");
            }
        });

        (format!("http://{}", addr), requests, handle)
    }

    fn spawn_truncated_body_server(
        content_length: usize,
        actual_body: &'static str,
    ) -> (String, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let addr = listener.local_addr().expect("addr should resolve");
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept should succeed");
            let _request = read_http_request(&mut stream);
            let raw = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                content_length, actual_body
            );
            stream
                .write_all(raw.as_bytes())
                .expect("response should write");
        });
        (format!("http://{}", addr), handle)
    }

    fn source_url(slug: &str) -> String {
        format!("https://statusgator.com/services/{}", slug)
    }

    #[test]
    fn test_parse_service_response_operational() {
        let body = r#"{"service":{"name":"Claude AI","slug":"claude-ai","current_status":"up"}}"#;
        let sig = parse_service_response("claude-ai", &source_url("claude-ai"), body);
        assert_eq!(sig.level, SignalLevel::Normal);
        assert_eq!(sig.name, "Claude AI");
        assert_eq!(sig.slug, "claude-ai");
    }

    #[test]
    fn test_parse_service_response_degraded() {
        let body =
            r#"{"service":{"name":"Claude AI","slug":"claude-ai","current_status":"degraded"}}"#;
        let sig = parse_service_response("claude-ai", &source_url("claude-ai"), body);
        assert_eq!(sig.level, SignalLevel::Elevated);
    }

    #[test]
    fn test_parse_service_response_down() {
        let body = r#"{"service":{"name":"OpenAI","slug":"openai","current_status":"down"}}"#;
        let sig = parse_service_response("openai", &source_url("openai"), body);
        assert_eq!(sig.level, SignalLevel::Spike);
    }

    #[test]
    fn test_parse_service_response_unknown_status_string() {
        let body =
            r#"{"service":{"name":"SomeService","slug":"some","current_status":"investigating"}}"#;
        let sig = parse_service_response("some", &source_url("some"), body);
        assert_eq!(sig.level, SignalLevel::Unknown);
    }

    #[test]
    fn test_parse_service_response_missing_service_object() {
        let body = r#"{"data":null}"#;
        let sig = parse_service_response("claude-ai", &source_url("claude-ai"), body);
        assert_eq!(sig.level, SignalLevel::Unknown);
    }

    #[test]
    fn test_parse_service_response_invalid_json() {
        let body = r#"not json at all {"#;
        let sig = parse_service_response("claude-ai", &source_url("claude-ai"), body);
        assert_eq!(sig.level, SignalLevel::Unknown);
        assert!(sig.detail.contains("invalid JSON"));
    }

    #[test]
    fn test_parse_service_response_uses_slug_as_name_when_missing() {
        let body = r#"{"service":{"current_status":"up"}}"#;
        let sig = parse_service_response("claude-ai", &source_url("claude-ai"), body);
        assert_eq!(sig.name, "claude-ai");
        assert_eq!(sig.level, SignalLevel::Normal);
    }

    #[test]
    fn test_parse_service_response_with_report_counts_spike() {
        // Status unknown but counts say spike.
        let body = r#"{"service":{"name":"Claude AI","current_status":"unknown_status","reports_last_hour":40,"reports_baseline":10}}"#;
        let sig = parse_service_response("claude-ai", &source_url("claude-ai"), body);
        assert_eq!(sig.level, SignalLevel::Spike);
        assert_eq!(sig.report_count_last_hour, Some(40));
        assert_eq!(sig.report_baseline, Some(10));
    }

    #[test]
    fn test_parse_service_response_missing_env_var_detail() {
        // When slug has no usable data at all.
        let body = r#"{"service":{}}"#;
        let sig = parse_service_response("claude-ai", &source_url("claude-ai"), body);
        // Empty current_status → "unknown" string → Unknown level
        assert_eq!(sig.level, SignalLevel::Unknown);
    }

    #[test]
    fn fetch_returns_unknown_signal_when_api_key_missing() {
        let _env_guard = env_lock().lock().expect("env lock should succeed");
        let config = test_config();
        unsafe {
            std::env::remove_var(&config.key_env_var);
        }

        let signal = fetch(&config);

        assert!(signal.enabled);
        assert_eq!(signal.claude.len(), 1);
        assert_eq!(signal.openai.len(), 1);
        assert_eq!(signal.claude[0].level, SignalLevel::Unknown);
        assert!(
            signal.claude[0]
                .detail
                .contains("set HEIMDALL_STATUSGATOR_TEST_TOKEN_ENV to enable community signal")
        );
    }

    #[test]
    fn fetch_live_from_base_maps_successful_service_responses_and_sends_auth_header() {
        let (base_url, requests, server) = spawn_http_server(vec![
            MockResponse::json(
                "200 OK",
                r#"{"service":{"name":"Claude AI","current_status":"up","reports_last_hour":4,"reports_baseline":4}}"#,
            ),
            MockResponse::json(
                "200 OK",
                r#"{"service":{"name":"OpenAI","current_status":"down","reports_last_hour":12,"reports_baseline":3}}"#,
            ),
        ]);
        let config = test_config();

        let signal = fetch_live_from_base(&config, "secret-token", &base_url);

        server.join().expect("server should finish");
        let requests = requests.lock().expect("lock should succeed");
        assert_eq!(requests.len(), 2);
        assert!(requests[0].starts_with("GET /claude-ai/status HTTP/1.1\r\n"));
        assert!(requests[0].contains("authorization: Bearer secret-token\r\n"));
        assert!(requests[1].starts_with("GET /openai/status HTTP/1.1\r\n"));

        assert_eq!(signal.claude[0].name, "Claude AI");
        assert_eq!(signal.claude[0].level, SignalLevel::Normal);
        assert_eq!(signal.openai[0].name, "OpenAI");
        assert_eq!(signal.openai[0].level, SignalLevel::Spike);
    }

    #[test]
    fn fetch_live_from_base_maps_unauthorized_to_unknown_signal() {
        let (base_url, _requests, server) = spawn_http_server(vec![MockResponse::json(
            "401 Unauthorized",
            r#"{"error":"bad key"}"#,
        )]);
        let mut config = test_config();
        config.openai_services.clear();

        let signal = fetch_live_from_base(&config, "bad-token", &base_url);

        server.join().expect("server should finish");
        assert_eq!(signal.claude[0].level, SignalLevel::Unknown);
        assert!(signal.claude[0].detail.contains("unauthorized"));
    }

    #[test]
    fn fetch_live_from_base_maps_rate_limit_to_unknown_signal() {
        let (base_url, _requests, server) = spawn_http_server(vec![MockResponse::json(
            "429 Too Many Requests",
            r#"{"error":"slow down"}"#,
        )]);
        let mut config = test_config();
        config.openai_services.clear();

        let signal = fetch_live_from_base(&config, "rate-limited", &base_url);

        server.join().expect("server should finish");
        assert_eq!(signal.claude[0].level, SignalLevel::Unknown);
        assert!(signal.claude[0].detail.contains("rate limited"));
    }

    #[test]
    fn fetch_live_from_base_maps_server_error_to_unknown_signal() {
        let (base_url, _requests, server) = spawn_http_server(vec![MockResponse::json(
            "503 Service Unavailable",
            r#"{"error":"outage"}"#,
        )]);
        let mut config = test_config();
        config.openai_services.clear();

        let signal = fetch_live_from_base(&config, "token", &base_url);

        server.join().expect("server should finish");
        assert_eq!(signal.claude[0].level, SignalLevel::Unknown);
        assert!(
            signal.claude[0]
                .detail
                .contains("server error 503 Service Unavailable")
        );
    }

    #[test]
    fn fetch_live_from_base_maps_network_failures_to_unknown_signal() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let addr = listener.local_addr().expect("addr should resolve");
        drop(listener);

        let mut config = test_config();
        config.openai_services.clear();

        let signal = fetch_live_from_base(&config, "token", &format!("http://{}", addr));

        assert_eq!(signal.claude[0].level, SignalLevel::Unknown);
        assert!(signal.claude[0].detail.contains("network error"));
    }

    #[test]
    fn fetch_live_from_base_maps_body_read_failures_to_unknown_signal() {
        let (base_url, server) = spawn_truncated_body_server(64, r#"{"service":{"name":"Claude"#);
        let mut config = test_config();
        config.openai_services.clear();

        let signal = fetch_live_from_base(&config, "token", &base_url);

        server.join().expect("server should finish");
        assert_eq!(signal.claude[0].level, SignalLevel::Unknown);
        assert!(signal.claude[0].detail.contains("failed to read body"));
    }

    #[test]
    fn fetch_slug_async_from_base_maps_unexpected_status_to_http_detail() {
        let (base_url, _requests, server) = spawn_http_server(vec![MockResponse::json(
            "404 Not Found",
            r#"{"error":"missing"}"#,
        )]);
        let client = build_client();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime should build");

        let signal = rt.block_on(async {
            fetch_slug_async_from_base(&client, "claude-ai", "token", &base_url).await
        });

        server.join().expect("server should finish");
        assert_eq!(signal.level, SignalLevel::Unknown);
        assert!(signal.detail.contains("HTTP 404 Not Found"));
    }
}
