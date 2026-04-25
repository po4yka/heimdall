use std::time::Duration;

use tracing::warn;

use super::models::{BudgetInfo, Identity, OAuthUsageResponse, UsageWindowsResponse, WindowInfo};

const USAGE_ENDPOINT: &str = "https://api.anthropic.com/api/oauth/usage";
const BETA_HEADER: &str = "oauth-2025-04-20";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Fetch usage windows from the Claude OAuth API.
pub async fn fetch_usage(access_token: &str) -> UsageWindowsResponse {
    fetch_usage_from(USAGE_ENDPOINT, access_token).await
}

async fn fetch_usage_from(endpoint: &str, access_token: &str) -> UsageWindowsResponse {
    match fetch_usage_inner(endpoint, access_token).await {
        Ok(resp) => resp,
        Err(e) => {
            warn!("OAuth usage fetch failed: {}", e);
            UsageWindowsResponse::with_error(e.to_string())
        }
    }
}

async fn fetch_usage_inner(
    endpoint: &str,
    access_token: &str,
) -> anyhow::Result<UsageWindowsResponse> {
    let client = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()?;

    let resp = client
        .get(endpoint)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Accept", "application/json")
        .header("anthropic-beta", BETA_HEADER)
        .send()
        .await?;

    let status = resp.status();
    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Ok(UsageWindowsResponse::with_error(
            "OAuth token expired. Run `claude login` to refresh.".into(),
        ));
    }
    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        // Transient — Anthropic throttles companion apps that poll the usage
        // endpoint too frequently. Don't dump the raw JSON body into the UI;
        // just say 'retrying' so the user knows cached data will clear soon.
        let _body = resp.text().await.unwrap_or_default();
        return Ok(UsageWindowsResponse::with_error(
            "Anthropic API rate-limited us — retrying on next poll.".into(),
        ));
    }
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Ok(UsageWindowsResponse::with_error(format!(
            "API returned HTTP {}: {}",
            status,
            body.chars().take(200).collect::<String>()
        )));
    }

    let data: OAuthUsageResponse = resp.json().await?;
    Ok(build_response(data))
}

fn build_response(data: OAuthUsageResponse) -> UsageWindowsResponse {
    UsageWindowsResponse {
        available: true,
        source: "oauth".into(),
        session: data.five_hour.as_ref().map(WindowInfo::from_usage_window),
        weekly: data.seven_day.as_ref().map(WindowInfo::from_usage_window),
        weekly_opus: data
            .seven_day_opus
            .as_ref()
            .map(WindowInfo::from_usage_window),
        weekly_sonnet: data
            .seven_day_sonnet
            .as_ref()
            .map(WindowInfo::from_usage_window),
        budget: data
            .extra_usage
            .as_ref()
            .and_then(BudgetInfo::from_extra_usage),
        identity: None, // filled in by the caller from credentials
        admin_fallback: None,
        error: None,
    }
}

/// Build a complete response by combining API data with identity from credentials.
pub fn with_identity(mut resp: UsageWindowsResponse, identity: Identity) -> UsageWindowsResponse {
    resp.identity = Some(identity);
    resp
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oauth::models::{ExtraUsage, UsageWindow};
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration as StdDuration;

    struct TestServer {
        url: String,
        requests: Arc<Mutex<Vec<String>>>,
        handle: Option<thread::JoinHandle<()>>,
    }

    impl TestServer {
        fn spawn(path: &str, responses: Vec<String>) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
            let addr = listener.local_addr().expect("test server local addr");
            let requests = Arc::new(Mutex::new(Vec::new()));
            let requests_for_thread = Arc::clone(&requests);
            let path = path.to_string();

            let handle = thread::spawn(move || {
                for response in responses {
                    let (mut stream, _) = listener.accept().expect("accept request");
                    stream
                        .set_read_timeout(Some(StdDuration::from_secs(2)))
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
                url: format!("http://{}{}", addr, path),
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

    fn http_response(status_line: &str, body: &str) -> String {
        format!(
            "HTTP/1.1 {status_line}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        )
    }

    #[test]
    fn test_build_response_full() {
        // Anthropic's /api/oauth/usage returns `utilization` as a 0-100
        // percentage; mirror that in fixtures so assertions exercise the
        // production code path (no fraction→percent conversion).
        let data = OAuthUsageResponse {
            five_hour: Some(UsageWindow {
                utilization: Some(45.0),
                resets_at: Some("2099-01-01T00:00:00Z".into()),
            }),
            seven_day: Some(UsageWindow {
                utilization: Some(60.0),
                resets_at: Some("2099-01-08T00:00:00Z".into()),
            }),
            seven_day_oauth_apps: None,
            seven_day_opus: Some(UsageWindow {
                utilization: Some(30.0),
                resets_at: None,
            }),
            seven_day_sonnet: None,
            iguana_necktie: None,
            extra_usage: Some(ExtraUsage {
                is_enabled: Some(true),
                monthly_limit: Some(100.0),
                used_credits: Some(45.5),
                utilization: Some(45.5),
                currency: Some("USD".into()),
            }),
        };

        let resp = build_response(data);
        assert!(resp.available);
        assert!((resp.session.as_ref().unwrap().used_percent - 45.0).abs() < 0.01);
        assert!((resp.weekly.as_ref().unwrap().used_percent - 60.0).abs() < 0.01);
        assert!(resp.weekly_opus.is_some());
        assert!(resp.weekly_sonnet.is_none());
        assert!((resp.budget.as_ref().unwrap().used - 45.5).abs() < 0.01);
    }

    #[test]
    fn test_build_response_empty() {
        let data = OAuthUsageResponse {
            five_hour: None,
            seven_day: None,
            seven_day_oauth_apps: None,
            seven_day_opus: None,
            seven_day_sonnet: None,
            iguana_necktie: None,
            extra_usage: None,
        };

        let resp = build_response(data);
        assert!(resp.available);
        assert!(resp.session.is_none());
        assert!(resp.weekly.is_none());
        assert!(resp.budget.is_none());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_fetch_usage_success_sends_expected_headers() {
        // Live Anthropic /oauth/usage returns utilization as 0-100 percent;
        // mirror that here so the response builder receives realistic input.
        let body = r#"{
            "five_hour": { "utilization": 25.0, "resets_at": "2099-01-01T00:00:00Z" },
            "extra_usage": {
                "is_enabled": true,
                "monthly_limit": 40.0,
                "used_credits": 10.0,
                "utilization": 25.0,
                "currency": "USD"
            }
        }"#;
        let server = TestServer::spawn("/oauth/usage", vec![http_response("200 OK", body)]);

        let response = fetch_usage_inner(&server.url, "secret-token")
            .await
            .expect("successful OAuth response");

        assert!(response.available);
        assert!((response.session.as_ref().expect("session").used_percent - 25.0).abs() < 0.01);
        assert_eq!(response.budget.as_ref().expect("budget").limit, 40.0);

        let requests = server.requests();
        assert_eq!(requests.len(), 1);
        let request = &requests[0];
        assert!(request.starts_with("GET /oauth/usage HTTP/1.1\r\n"));
        let request_lower = request.to_ascii_lowercase();
        assert!(request_lower.contains("authorization: bearer secret-token\r\n"));
        assert!(request_lower.contains("accept: application/json\r\n"));
        assert!(request_lower.contains(&format!("anthropic-beta: {}\r\n", BETA_HEADER)));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_fetch_usage_unauthorized_returns_refresh_message() {
        let server = TestServer::spawn(
            "/oauth/usage",
            vec![http_response("401 Unauthorized", r#"{"error":"expired"}"#)],
        );

        let response = fetch_usage_inner(&server.url, "expired-token")
            .await
            .expect("401 path should return an error response payload");

        assert!(!response.available);
        assert_eq!(
            response.error.as_deref(),
            Some("OAuth token expired. Run `claude login` to refresh.")
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_fetch_usage_server_error_truncates_body() {
        let long_body = "x".repeat(260);
        let server = TestServer::spawn(
            "/oauth/usage",
            vec![http_response("502 Bad Gateway", &long_body)],
        );

        let response = fetch_usage_inner(&server.url, "token")
            .await
            .expect("HTTP error path should return an error response payload");

        let error = response.error.expect("error message");
        assert!(error.starts_with("API returned HTTP 502 Bad Gateway: "));
        let suffix = error
            .split_once(": ")
            .map(|(_, body)| body)
            .expect("split truncated body");
        assert_eq!(suffix.len(), 200);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_fetch_usage_wraps_json_decode_failures() {
        let server = TestServer::spawn("/oauth/usage", vec![http_response("200 OK", "{not-json}")]);

        let response = fetch_usage_from(&server.url, "token").await;

        assert!(!response.available);
        assert!(!response.error.as_deref().unwrap_or_default().is_empty());
    }
}
