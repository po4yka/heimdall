use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;

use crate::models::OpenAiReconciliation;
use crate::pricing;

const OPENAI_USAGE_URL: &str = "https://api.openai.com/v1/organization/usage/completions";

#[derive(Debug, Deserialize)]
struct UsagePage {
    data: Vec<UsageBucket>,
    has_more: bool,
    next_page: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UsageBucket {
    results: Vec<UsageResult>,
}

#[derive(Debug, Deserialize)]
struct UsageResult {
    input_tokens: Option<i64>,
    output_tokens: Option<i64>,
    input_cached_tokens: Option<i64>,
    num_model_requests: Option<i64>,
    model: Option<String>,
}

pub async fn fetch_org_usage_reconciliation(
    admin_key: &str,
    lookback_days: i64,
    estimated_local_cost: f64,
) -> OpenAiReconciliation {
    fetch_org_usage_reconciliation_from(
        OPENAI_USAGE_URL,
        admin_key,
        lookback_days,
        estimated_local_cost,
    )
    .await
}

async fn fetch_org_usage_reconciliation_from(
    usage_url: &str,
    admin_key: &str,
    lookback_days: i64,
    estimated_local_cost: f64,
) -> OpenAiReconciliation {
    match fetch_org_usage_reconciliation_inner(
        usage_url,
        admin_key,
        lookback_days,
        estimated_local_cost,
    )
    .await
    {
        Ok(data) => data,
        Err(error) => {
            let end = chrono::Utc::now().date_naive();
            let start = end - chrono::Duration::days(lookback_days.saturating_sub(1));
            OpenAiReconciliation {
                available: false,
                lookback_days,
                start_date: start.to_string(),
                end_date: end.to_string(),
                estimated_local_cost,
                api_usage_cost: 0.0,
                api_input_tokens: 0,
                api_output_tokens: 0,
                api_cached_input_tokens: 0,
                api_requests: 0,
                delta_cost: 0.0,
                error: Some(error.to_string()),
            }
        }
    }
}

async fn fetch_org_usage_reconciliation_inner(
    usage_url: &str,
    admin_key: &str,
    lookback_days: i64,
    estimated_local_cost: f64,
) -> Result<OpenAiReconciliation> {
    let client = Client::builder()
        .user_agent("claude-usage-tracker/0.1")
        .build()
        .context("failed to build OpenAI client")?;

    let end = chrono::Utc::now().date_naive();
    let start = end - chrono::Duration::days(lookback_days.saturating_sub(1));
    let start_time = start
        .and_hms_opt(0, 0, 0)
        .context("invalid OpenAI reconciliation start time")?
        .and_utc()
        .timestamp();
    let end_time = (end + chrono::Duration::days(1))
        .and_hms_opt(0, 0, 0)
        .context("invalid OpenAI reconciliation end time")?
        .and_utc()
        .timestamp();

    let mut page: Option<String> = None;
    let mut input_tokens = 0_i64;
    let mut output_tokens = 0_i64;
    let mut cached_input_tokens = 0_i64;
    let mut api_requests = 0_i64;
    let mut api_usage_cost = 0.0_f64;

    loop {
        let mut request = client.get(usage_url).bearer_auth(admin_key).query(&[
            ("start_time", start_time.to_string()),
            ("end_time", end_time.to_string()),
            ("bucket_width", "1d".to_string()),
            ("limit", (lookback_days + 1).max(1).to_string()),
            ("group_by[]", "model".to_string()),
        ]);

        if let Some(page_cursor) = page.as_deref() {
            request = request.query(&[("page", page_cursor)]);
        }

        let response = request
            .send()
            .await
            .context("failed to fetch OpenAI organization usage")?
            .error_for_status()
            .context("OpenAI organization usage request failed")?;
        let payload: UsagePage = response
            .json()
            .await
            .context("failed to decode OpenAI organization usage response")?;

        for bucket in payload.data {
            for result in bucket.results {
                let Some(model) = result.model.as_deref() else {
                    continue;
                };
                if !is_codex_usage_model(model) {
                    continue;
                }

                let input = result.input_tokens.unwrap_or(0);
                let output = result.output_tokens.unwrap_or(0);
                let cached = result.input_cached_tokens.unwrap_or(0);
                input_tokens += input;
                output_tokens += output;
                cached_input_tokens += cached;
                api_requests += result.num_model_requests.unwrap_or(0);
                api_usage_cost += pricing::calc_cost(model, input, output, cached, 0);
            }
        }

        if !payload.has_more {
            break;
        }
        page = payload.next_page;
        if page.is_none() {
            break;
        }
    }

    Ok(OpenAiReconciliation {
        available: true,
        lookback_days,
        start_date: start.to_string(),
        end_date: end.to_string(),
        estimated_local_cost,
        api_usage_cost,
        api_input_tokens: input_tokens,
        api_output_tokens: output_tokens,
        api_cached_input_tokens: cached_input_tokens,
        api_requests,
        delta_cost: api_usage_cost - estimated_local_cost,
        error: None,
    })
}

fn is_codex_usage_model(model: &str) -> bool {
    let lower = model.to_ascii_lowercase();
    lower.contains("codex") || lower.starts_with("gpt-5.4") || lower.starts_with("gpt-5.3-codex")
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
                    let (mut stream, _) = listener.accept().expect("accept test request");
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
                        .expect("write test response");
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

    #[tokio::test(flavor = "current_thread")]
    async fn fetch_org_usage_reconciliation_follows_pagination_and_filters_models() {
        let first_page = r#"{
            "data": [{
                "results": [
                    {
                        "input_tokens": 1000,
                        "output_tokens": 200,
                        "input_cached_tokens": 100,
                        "num_model_requests": 3,
                        "model": "gpt-5.4"
                    },
                    {
                        "input_tokens": 999,
                        "output_tokens": 888,
                        "input_cached_tokens": 777,
                        "num_model_requests": 9,
                        "model": "text-embedding-3-small"
                    },
                    {
                        "input_tokens": 500,
                        "output_tokens": 50,
                        "input_cached_tokens": 25,
                        "num_model_requests": 1,
                        "model": null
                    }
                ]
            }],
            "has_more": true,
            "next_page": "cursor-2"
        }"#;
        let second_page = r#"{
            "data": [{
                "results": [
                    {
                        "input_tokens": 400,
                        "output_tokens": 150,
                        "input_cached_tokens": 40,
                        "num_model_requests": 2,
                        "model": "codex-mini-latest"
                    }
                ]
            }],
            "has_more": false,
            "next_page": null
        }"#;
        let server = TestServer::spawn(vec![
            http_response("200 OK", first_page, &[]),
            http_response("200 OK", second_page, &[]),
        ]);

        let reconciliation =
            fetch_org_usage_reconciliation_inner(&server.base_url, "test-admin-key", 3, 1.25)
                .await
                .expect("OpenAI reconciliation should succeed");

        let expected_cost = pricing::calc_cost("gpt-5.4", 1000, 200, 100, 0)
            + pricing::calc_cost("codex-mini-latest", 400, 150, 40, 0);
        assert!(reconciliation.available);
        assert_eq!(reconciliation.lookback_days, 3);
        assert_eq!(reconciliation.api_input_tokens, 1400);
        assert_eq!(reconciliation.api_output_tokens, 350);
        assert_eq!(reconciliation.api_cached_input_tokens, 140);
        assert_eq!(reconciliation.api_requests, 5);
        assert!((reconciliation.api_usage_cost - expected_cost).abs() < 1e-12);
        assert!((reconciliation.delta_cost - (expected_cost - 1.25)).abs() < 1e-12);
        assert_eq!(reconciliation.error, None);

        let requests = server.requests();
        assert_eq!(requests.len(), 2);
        assert!(requests[0].starts_with("GET /?"));
        let first_request_lower = requests[0].to_ascii_lowercase();
        assert!(first_request_lower.contains("authorization: bearer test-admin-key\r\n"));
        assert!(requests[0].contains("start_time="));
        assert!(requests[0].contains("end_time="));
        assert!(requests[0].contains("bucket_width=1d"));
        assert!(requests[0].contains("limit=4"));
        assert!(requests[0].contains("group_by%5B%5D=model"));
        assert!(requests[1].contains("page=cursor-2"));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn fetch_org_usage_reconciliation_returns_unavailable_on_http_failure() {
        let server = TestServer::spawn(vec![http_response(
            "500 Internal Server Error",
            r#"{"error":"downstream"}"#,
            &[],
        )]);

        let reconciliation =
            fetch_org_usage_reconciliation_from(&server.base_url, "key", 2, 0.5).await;

        assert!(!reconciliation.available);
        assert_eq!(reconciliation.lookback_days, 2);
        assert_eq!(reconciliation.api_usage_cost, 0.0);
        assert_eq!(reconciliation.api_input_tokens, 0);
        assert!(
            reconciliation
                .error
                .as_deref()
                .expect("error message")
                .contains("OpenAI organization usage request failed")
        );
    }
}
