use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;

use crate::models::ClaudeAdminSummary;

const ORG_URL: &str = "https://api.anthropic.com/v1/organizations/me";
const ANALYTICS_URL: &str = "https://api.anthropic.com/v1/organizations/usage_report/claude_code";
const API_VERSION: &str = "2023-06-01";
const USER_AGENT: &str = "claude-usage-tracker/0.1";
const DATA_LATENCY_NOTE: &str = "Org-wide · UTC daily aggregation · up to 1 hour delayed";

#[derive(Debug, Deserialize)]
struct OrganizationResponse {
    name: String,
}

#[derive(Debug, Deserialize)]
struct AnalyticsPage {
    data: Vec<AnalyticsRecord>,
    has_more: bool,
    next_page: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AnalyticsRecord {
    date: String,
    core_metrics: Option<CoreMetrics>,
    model_breakdown: Option<Vec<ModelBreakdown>>,
}

#[derive(Debug, Deserialize)]
struct CoreMetrics {
    num_sessions: Option<i64>,
    lines_of_code: Option<LinesOfCode>,
}

#[derive(Debug, Deserialize)]
struct LinesOfCode {
    added: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct ModelBreakdown {
    tokens: Option<ModelTokens>,
    estimated_cost: Option<EstimatedCost>,
}

#[derive(Debug, Deserialize)]
struct ModelTokens {
    input: Option<i64>,
    output: Option<i64>,
    cache_read: Option<i64>,
    cache_creation: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct EstimatedCost {
    amount: Option<i64>,
}

pub async fn fetch_org_claude_code_summary(
    admin_key: &str,
    lookback_days: i64,
) -> ClaudeAdminSummary {
    fetch_org_claude_code_summary_from(ORG_URL, ANALYTICS_URL, admin_key, lookback_days).await
}

async fn fetch_org_claude_code_summary_from(
    org_url: &str,
    analytics_url: &str,
    admin_key: &str,
    lookback_days: i64,
) -> ClaudeAdminSummary {
    match fetch_org_claude_code_summary_inner(org_url, analytics_url, admin_key, lookback_days)
        .await
    {
        Ok(summary) => summary,
        Err(error) => {
            let end = chrono::Utc::now().date_naive();
            let start = end - chrono::Duration::days(lookback_days.saturating_sub(1));
            ClaudeAdminSummary {
                lookback_days,
                start_date: start.to_string(),
                end_date: end.to_string(),
                data_latency_note: DATA_LATENCY_NOTE.into(),
                error: Some(error.to_string()),
                ..ClaudeAdminSummary::default()
            }
        }
    }
}

async fn fetch_org_claude_code_summary_inner(
    org_url: &str,
    analytics_url: &str,
    admin_key: &str,
    lookback_days: i64,
) -> Result<ClaudeAdminSummary> {
    let client = Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .context("failed to build Anthropic admin client")?;

    let end = chrono::Utc::now().date_naive();
    let start = end - chrono::Duration::days(lookback_days.saturating_sub(1));
    let organization_name = fetch_organization_name(&client, org_url, admin_key).await?;

    let mut today_active_users = 0_i64;
    let mut today_sessions = 0_i64;
    let mut lookback_lines_accepted = 0_i64;
    let mut lookback_estimated_cost_usd = 0.0_f64;
    let mut lookback_input_tokens = 0_i64;
    let mut lookback_output_tokens = 0_i64;
    let mut lookback_cache_read_tokens = 0_i64;
    let mut lookback_cache_creation_tokens = 0_i64;

    for day_offset in 0..lookback_days.max(1) {
        let day = start + chrono::Duration::days(day_offset);
        let mut page: Option<String> = None;
        loop {
            let mut request = client
                .get(analytics_url)
                .header("anthropic-version", API_VERSION)
                .header("x-api-key", admin_key)
                .query(&[
                    ("starting_at", day.to_string()),
                    ("limit", "1000".to_string()),
                ]);
            if let Some(cursor) = page.as_deref() {
                request = request.query(&[("page", cursor)]);
            }

            let response = request
                .send()
                .await
                .context("failed to fetch Claude Code admin analytics")?
                .error_for_status()
                .context("Claude Code admin analytics request failed")?;
            let payload: AnalyticsPage = response
                .json()
                .await
                .context("failed to decode Claude Code admin analytics response")?;

            for record in payload.data {
                let record_day = record.date.split('T').next().unwrap_or_default();
                let core_metrics = record.core_metrics.unwrap_or(CoreMetrics {
                    num_sessions: Some(0),
                    lines_of_code: None,
                });
                if record_day == end.to_string() {
                    today_active_users += 1;
                    today_sessions += core_metrics.num_sessions.unwrap_or(0);
                }
                lookback_lines_accepted += core_metrics
                    .lines_of_code
                    .and_then(|lines| lines.added)
                    .unwrap_or(0);
                for model in record.model_breakdown.unwrap_or_default() {
                    if let Some(tokens) = model.tokens {
                        lookback_input_tokens += tokens.input.unwrap_or(0);
                        lookback_output_tokens += tokens.output.unwrap_or(0);
                        lookback_cache_read_tokens += tokens.cache_read.unwrap_or(0);
                        lookback_cache_creation_tokens += tokens.cache_creation.unwrap_or(0);
                    }
                    lookback_estimated_cost_usd += model
                        .estimated_cost
                        .and_then(|cost| cost.amount)
                        .map(|cents| cents as f64 / 100.0)
                        .unwrap_or(0.0);
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
    }

    Ok(ClaudeAdminSummary {
        organization_name,
        lookback_days,
        start_date: start.to_string(),
        end_date: end.to_string(),
        data_latency_note: DATA_LATENCY_NOTE.into(),
        today_active_users,
        today_sessions,
        lookback_lines_accepted,
        lookback_estimated_cost_usd,
        lookback_input_tokens,
        lookback_output_tokens,
        lookback_cache_read_tokens,
        lookback_cache_creation_tokens,
        error: None,
    })
}

async fn fetch_organization_name(
    client: &Client,
    org_url: &str,
    admin_key: &str,
) -> Result<String> {
    let response = client
        .get(org_url)
        .header("anthropic-version", API_VERSION)
        .header("x-api-key", admin_key)
        .send()
        .await
        .context("failed to fetch Anthropic organization info")?
        .error_for_status()
        .context("Anthropic organization info request failed")?;
    let payload: OrganizationResponse = response
        .json()
        .await
        .context("failed to decode Anthropic organization info")?;
    Ok(payload.name)
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
            let addr = listener.local_addr().expect("addr");
            let requests = Arc::new(Mutex::new(Vec::new()));
            let requests_for_thread = Arc::clone(&requests);
            let handle = thread::spawn(move || {
                for response in responses {
                    let (mut stream, _) = listener.accept().expect("accept");
                    stream
                        .set_read_timeout(Some(Duration::from_secs(2)))
                        .expect("timeout");
                    let request = read_request(&mut stream);
                    requests_for_thread.lock().expect("lock").push(request);
                    stream.write_all(response.as_bytes()).expect("write");
                }
            });
            Self {
                base_url: format!("http://{}", addr),
                requests,
                handle: Some(handle),
            }
        }

        fn requests(&self) -> Vec<String> {
            self.requests.lock().expect("lock").clone()
        }
    }

    impl Drop for TestServer {
        fn drop(&mut self) {
            if let Some(handle) = self.handle.take() {
                handle.join().expect("join");
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
        String::from_utf8(request).expect("utf-8")
    }

    fn http_response(status_line: &str, body: &str) -> String {
        format!(
            "HTTP/1.1 {status_line}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        )
    }

    #[tokio::test(flavor = "current_thread")]
    async fn aggregates_multiple_days_and_pages() {
        let end = chrono::Utc::now().date_naive();
        let yesterday = end - chrono::Duration::days(1);
        let org_body = r#"{"name":"Acme Org"}"#;
        let today_page_one = format!(
            r#"{{
                "data":[{{"date":"{}T00:00:00Z","core_metrics":{{"num_sessions":2,"lines_of_code":{{"added":10}}}},"model_breakdown":[{{"tokens":{{"input":100,"output":50,"cache_read":20,"cache_creation":10}},"estimated_cost":{{"amount":250,"currency":"USD"}}}}]}}],
                "has_more":true,
                "next_page":"cursor-2"
            }}"#,
            end
        );
        let today_page_two = format!(
            r#"{{
                "data":[{{"date":"{}T00:00:00Z","core_metrics":{{"num_sessions":3,"lines_of_code":{{"added":4}}}},"model_breakdown":[{{"tokens":{{"input":11,"output":7,"cache_read":3,"cache_creation":1}},"estimated_cost":{{"amount":75,"currency":"USD"}}}}]}}],
                "has_more":false,
                "next_page":null
            }}"#,
            end
        );
        let yesterday_page = format!(
            r#"{{
                "data":[{{"date":"{}T00:00:00Z","core_metrics":{{"num_sessions":5,"lines_of_code":{{"added":6}}}},"model_breakdown":[{{"tokens":{{"input":30,"output":12,"cache_read":5,"cache_creation":2}},"estimated_cost":{{"amount":125,"currency":"USD"}}}}]}}],
                "has_more":false,
                "next_page":null
            }}"#,
            yesterday
        );
        let server = TestServer::spawn(vec![
            http_response("200 OK", org_body),
            http_response("200 OK", &today_page_one),
            http_response("200 OK", &today_page_two),
            http_response("200 OK", &yesterday_page),
        ]);

        let summary = fetch_org_claude_code_summary_from(
            &format!("{}/org", server.base_url),
            &format!("{}/analytics", server.base_url),
            "sk-ant-admin-test",
            2,
        )
        .await;

        assert_eq!(summary.organization_name, "Acme Org");
        assert_eq!(summary.today_active_users, 2);
        assert_eq!(summary.today_sessions, 5);
        assert_eq!(summary.lookback_lines_accepted, 20);
        assert!((summary.lookback_estimated_cost_usd - 4.50).abs() < 0.0001);
        assert_eq!(summary.lookback_input_tokens, 141);
        assert_eq!(summary.lookback_output_tokens, 69);
        assert_eq!(summary.lookback_cache_read_tokens, 28);
        assert_eq!(summary.lookback_cache_creation_tokens, 13);
        assert!(summary.error.is_none());

        let requests = server.requests();
        assert!(requests[0].starts_with("GET /org HTTP/1.1\r\n"));
        assert!(requests[1].contains("/analytics?"));
        assert!(requests[1].contains("starting_at="));
        assert!(requests[2].contains("page=cursor-2"));
        assert!(requests[3].contains("starting_at="));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn returns_error_when_org_name_fetch_fails() {
        let server = TestServer::spawn(vec![http_response("500 Internal Server Error", "{}")]);
        let summary = fetch_org_claude_code_summary_from(
            &format!("{}/org", server.base_url),
            &format!("{}/analytics", server.base_url),
            "sk-ant-admin-test",
            1,
        )
        .await;
        assert!(summary.error.is_some());
        assert!(summary.organization_name.is_empty());
    }
}
