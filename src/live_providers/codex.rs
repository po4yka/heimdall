use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow, bail};
use regex::Regex;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::models::{LiveProviderIdentity, LiveRateWindow};

const DEFAULT_USAGE_URL: &str = "https://chatgpt.com/backend-api/wham/usage";

#[derive(Debug, Clone)]
pub struct CodexAuth {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    pub account_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CodexUsageResponse {
    pub plan_type: Option<String>,
    pub rate_limit: Option<CodexRateLimit>,
    pub credits: Option<CodexCredits>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CodexRateLimit {
    pub primary_window: Option<CodexWindow>,
    pub secondary_window: Option<CodexWindow>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CodexWindow {
    pub used_percent: f64,
    pub reset_at: i64,
    pub limit_window_seconds: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CodexCredits {
    pub has_credits: bool,
    pub unlimited: bool,
    pub balance: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct RpcAccountResponse {
    pub account: Option<RpcAccountDetails>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum RpcAccountDetails {
    #[serde(rename = "apikey")]
    ApiKey,
    #[serde(rename = "chatgpt")]
    ChatGpt {
        email: Option<String>,
        #[serde(rename = "planType")]
        plan_type: Option<String>,
    },
}

#[derive(Debug, Deserialize)]
pub struct RpcRateLimitsResponse {
    #[serde(rename = "rateLimits")]
    pub rate_limits: RpcRateLimitSnapshot,
}

#[derive(Debug, Deserialize)]
pub struct RpcRateLimitSnapshot {
    pub primary: Option<RpcRateLimitWindow>,
    pub secondary: Option<RpcRateLimitWindow>,
    pub credits: Option<RpcCreditsSnapshot>,
}

#[derive(Debug, Deserialize)]
pub struct RpcRateLimitWindow {
    #[serde(rename = "usedPercent")]
    pub used_percent: f64,
    #[serde(rename = "windowDurationMins")]
    pub window_duration_mins: Option<i64>,
    #[serde(rename = "resetsAt")]
    pub resets_at: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct RpcCreditsSnapshot {
    #[serde(rename = "hasCredits")]
    pub has_credits: bool,
    pub unlimited: bool,
    pub balance: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CliStatusSnapshot {
    pub credits: Option<f64>,
    pub primary: Option<LiveRateWindow>,
    pub secondary: Option<LiveRateWindow>,
}

pub fn load_auth(env: &[(String, String)]) -> Result<CodexAuth> {
    let path = auth_file_path(env);
    let data = std::fs::read(&path)
        .with_context(|| format!("failed to read Codex auth file at {}", path.display()))?;
    parse_auth(&data)
}

pub fn parse_auth(data: &[u8]) -> Result<CodexAuth> {
    let json: Value = serde_json::from_slice(data).context("invalid Codex auth.json")?;

    if let Some(api_key) = json
        .get("OPENAI_API_KEY")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
    {
        return Ok(CodexAuth {
            access_token: api_key.to_string(),
            refresh_token: None,
            id_token: None,
            account_id: None,
        });
    }

    let tokens = json
        .get("tokens")
        .and_then(Value::as_object)
        .ok_or_else(|| anyhow!("Codex auth.json has no tokens object"))?;

    let access_token = string_value(tokens, "access_token", "accessToken")
        .ok_or_else(|| anyhow!("Codex auth.json has no access token"))?;

    Ok(CodexAuth {
        access_token,
        refresh_token: string_value(tokens, "refresh_token", "refreshToken"),
        id_token: string_value(tokens, "id_token", "idToken"),
        account_id: string_value(tokens, "account_id", "accountId"),
    })
}

pub async fn fetch_oauth_usage(auth: &CodexAuth) -> Result<CodexUsageResponse> {
    let client = reqwest::Client::builder()
        .user_agent("claude-usage-tracker/0.1")
        .build()
        .context("failed to build Codex OAuth client")?;

    let mut request = client
        .get(DEFAULT_USAGE_URL)
        .bearer_auth(&auth.access_token)
        .header(reqwest::header::ACCEPT, "application/json");

    if let Some(account_id) = auth.account_id.as_deref().filter(|id| !id.is_empty()) {
        request = request.header("ChatGPT-Account-Id", account_id);
    }

    let response = request
        .send()
        .await
        .context("failed to fetch Codex OAuth usage")?
        .error_for_status()
        .context("Codex OAuth usage request failed")?;

    response
        .json::<CodexUsageResponse>()
        .await
        .context("failed to decode Codex OAuth usage response")
}

pub fn decode_identity(auth: &CodexAuth) -> Option<LiveProviderIdentity> {
    let id_token = auth.id_token.as_deref()?;
    let payload = decode_jwt_payload(id_token).ok()?;
    let profile = payload
        .get("https://api.openai.com/profile")
        .and_then(Value::as_object);
    let auth_info = payload
        .get("https://api.openai.com/auth")
        .and_then(Value::as_object);

    let email = payload
        .get("email")
        .and_then(Value::as_str)
        .or_else(|| profile.and_then(|profile| profile.get("email").and_then(Value::as_str)))
        .map(ToOwned::to_owned);

    let plan = auth_info
        .and_then(|info| info.get("chatgpt_plan_type").and_then(Value::as_str))
        .or_else(|| payload.get("chatgpt_plan_type").and_then(Value::as_str))
        .map(ToOwned::to_owned);

    Some(LiveProviderIdentity {
        provider: "codex".into(),
        account_email: email,
        account_organization: None,
        login_method: Some("chatgpt".into()),
        plan,
    })
}

pub fn oauth_window_to_live(window: &CodexWindow) -> LiveRateWindow {
    let resets_at = chrono::DateTime::from_timestamp(window.reset_at, 0).map(|ts| ts.to_rfc3339());
    let resets_in_minutes = chrono::DateTime::from_timestamp(window.reset_at, 0).map(|ts| {
        ts.signed_duration_since(chrono::Utc::now())
            .num_minutes()
            .max(0)
    });

    LiveRateWindow {
        used_percent: window.used_percent,
        resets_at,
        resets_in_minutes,
        window_minutes: Some(window.limit_window_seconds / 60),
        reset_label: None,
    }
}

pub fn rpc_window_to_live(window: &RpcRateLimitWindow) -> LiveRateWindow {
    let resets_at = window
        .resets_at
        .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0).map(|ts| ts.to_rfc3339()));
    let resets_in_minutes = window.resets_at.and_then(|ts| {
        chrono::DateTime::from_timestamp(ts, 0).map(|ts| {
            ts.signed_duration_since(chrono::Utc::now())
                .num_minutes()
                .max(0)
        })
    });

    LiveRateWindow {
        used_percent: window.used_percent,
        resets_at,
        resets_in_minutes,
        window_minutes: window.window_duration_mins,
        reset_label: None,
    }
}

pub fn rpc_credits_to_f64(credits: &RpcCreditsSnapshot) -> Option<f64> {
    credits.balance.as_deref()?.parse::<f64>().ok()
}

pub fn oauth_credits_to_f64(credits: &CodexCredits) -> Option<f64> {
    if !credits.has_credits || credits.unlimited {
        return None;
    }

    match credits.balance.as_ref() {
        Some(Value::Number(value)) => value.as_f64(),
        Some(Value::String(value)) => value.parse::<f64>().ok(),
        _ => None,
    }
}

pub fn fetch_rpc_snapshot(
    timeout: Duration,
) -> Result<(Option<RpcAccountResponse>, RpcRateLimitsResponse)> {
    let mut child = Command::new("codex")
        .args(["-s", "read-only", "-a", "untrusted", "app-server"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("failed to launch codex app-server")?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("codex app-server stdout unavailable"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow!("codex app-server stderr unavailable"))?;
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| anyhow!("codex app-server stdin unavailable"))?;

    let (tx, rx) = mpsc::channel::<String>();
    thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines().map_while(|line| line.ok()) {
            let _ = tx.send(line);
        }
    });

    thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().map_while(|line| line.ok()) {
            tracing::debug!("codex rpc stderr: {}", line);
        }
    });

    let mut next_id = 1_i64;
    send_rpc_payload(
        &mut stdin,
        &json!({"id": next_id, "method": "initialize", "params": {"clientInfo": {"name": "heimdall", "version": "0.1.0"}}}),
    )?;
    wait_for_rpc_id(&rx, next_id, timeout)?;
    send_rpc_payload(&mut stdin, &json!({"method": "initialized", "params": {}}))?;

    next_id += 1;
    send_rpc_payload(
        &mut stdin,
        &json!({"id": next_id, "method": "account/read", "params": {}}),
    )?;
    let account_message = wait_for_rpc_id(&rx, next_id, timeout).ok();

    next_id += 1;
    send_rpc_payload(
        &mut stdin,
        &json!({"id": next_id, "method": "account/rateLimits/read", "params": {}}),
    )?;
    let limits_message = wait_for_rpc_id(&rx, next_id, timeout)?;

    let _ = child.kill();
    let _ = child.wait();

    let account = account_message
        .as_ref()
        .and_then(|value| value.get("result"))
        .map(|value| serde_json::from_value::<RpcAccountResponse>(value.clone()))
        .transpose()
        .context("failed to decode codex rpc account response")?;

    let limits = limits_message
        .get("result")
        .ok_or_else(|| anyhow!("codex rpc rateLimits response missing result"))?;
    let limits = serde_json::from_value::<RpcRateLimitsResponse>(limits.clone())
        .context("failed to decode codex rpc rateLimits response")?;

    Ok((account, limits))
}

pub fn fetch_cli_status(timeout: Duration) -> Result<CliStatusSnapshot> {
    let mut child = Command::new("/usr/bin/script")
        .args([
            "-q",
            "/dev/null",
            "codex",
            "-s",
            "read-only",
            "-a",
            "untrusted",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .context("failed to launch codex CLI pty probe")?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| anyhow!("codex CLI stdin unavailable"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("codex CLI stdout unavailable"))?;

    stdin
        .write_all(b"/status\n")
        .context("failed to send /status to codex CLI")?;
    stdin.flush().ok();

    let start = Instant::now();
    let reader = BufReader::new(stdout);
    let mut output = String::new();
    for line in reader.lines().map_while(|line| line.ok()) {
        output.push_str(&line);
        output.push('\n');
        if output.contains("5h limit") || output.contains("Weekly limit") {
            break;
        }
        if start.elapsed() > timeout {
            break;
        }
    }

    let _ = child.kill();
    let _ = child.wait();
    parse_cli_status(&output)
}

pub fn parse_cli_status(text: &str) -> Result<CliStatusSnapshot> {
    let credits = Regex::new(r"Credits:\s*([0-9][0-9.,]*)")
        .unwrap()
        .captures(text)
        .and_then(|captures| captures.get(1))
        .and_then(|value| value.as_str().replace(',', "").parse::<f64>().ok());

    let five_line = Regex::new(r"5h limit[^\n]*")
        .unwrap()
        .find(text)
        .map(|m| m.as_str().to_string());
    let week_line = Regex::new(r"Weekly limit[^\n]*")
        .unwrap()
        .find(text)
        .map(|m| m.as_str().to_string());

    let primary = five_line.as_deref().map(parse_cli_window).transpose()?;
    let secondary = week_line.as_deref().map(parse_cli_window).transpose()?;

    if credits.is_none() && primary.is_none() && secondary.is_none() {
        bail!("could not parse Codex CLI status output");
    }

    Ok(CliStatusSnapshot {
        credits,
        primary,
        secondary,
    })
}

fn parse_cli_window(line: &str) -> Result<LiveRateWindow> {
    let percent_left = Regex::new(r"([0-9]{1,3})%\s*left")
        .unwrap()
        .captures(line)
        .and_then(|captures| captures.get(1))
        .and_then(|value| value.as_str().parse::<f64>().ok())
        .ok_or_else(|| anyhow!("could not parse percent in {}", line))?;

    let reset_label = line
        .split("left")
        .nth(1)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);

    Ok(LiveRateWindow {
        used_percent: (100.0 - percent_left).clamp(0.0, 100.0),
        resets_at: None,
        resets_in_minutes: None,
        window_minutes: None,
        reset_label,
    })
}

fn auth_file_path(env: &[(String, String)]) -> PathBuf {
    let env_map = env
        .iter()
        .cloned()
        .collect::<std::collections::HashMap<_, _>>();
    if let Some(codex_home) = env_map.get("CODEX_HOME").filter(|value| !value.is_empty()) {
        return PathBuf::from(codex_home).join("auth.json");
    }

    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join(".codex")
        .join("auth.json")
}

fn string_value(
    map: &serde_json::Map<String, Value>,
    snake_case: &str,
    camel_case: &str,
) -> Option<String> {
    map.get(snake_case)
        .and_then(Value::as_str)
        .or_else(|| map.get(camel_case).and_then(Value::as_str))
        .map(ToOwned::to_owned)
}

fn decode_jwt_payload(token: &str) -> Result<Value> {
    let payload = token
        .split('.')
        .nth(1)
        .ok_or_else(|| anyhow!("invalid JWT payload"))?;
    let bytes = decode_base64url(payload)?;
    serde_json::from_slice(&bytes).context("invalid Codex JWT payload")
}

fn decode_base64url(input: &str) -> Result<Vec<u8>> {
    let mut normalized = input.replace('-', "+").replace('_', "/");
    while normalized.len() % 4 != 0 {
        normalized.push('=');
    }
    base64_decode(&normalized)
}

fn base64_decode(input: &str) -> Result<Vec<u8>> {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = Vec::with_capacity(input.len() * 3 / 4);
    let mut chunk = [0_u8; 4];
    let mut chunk_len = 0_usize;

    for byte in input.bytes() {
        if byte == b'=' {
            break;
        }
        let value = TABLE
            .iter()
            .position(|candidate| *candidate == byte)
            .ok_or_else(|| anyhow!("invalid base64 character"))? as u8;
        chunk[chunk_len] = value;
        chunk_len += 1;
        if chunk_len == 4 {
            output.push((chunk[0] << 2) | (chunk[1] >> 4));
            output.push((chunk[1] << 4) | (chunk[2] >> 2));
            output.push((chunk[2] << 6) | chunk[3]);
            chunk_len = 0;
        }
    }

    match chunk_len {
        2 => {
            output.push((chunk[0] << 2) | (chunk[1] >> 4));
        }
        3 => {
            output.push((chunk[0] << 2) | (chunk[1] >> 4));
            output.push((chunk[1] << 4) | (chunk[2] >> 2));
        }
        _ => {}
    }

    Ok(output)
}

fn send_rpc_payload(stdin: &mut std::process::ChildStdin, payload: &Value) -> Result<()> {
    let bytes = serde_json::to_vec(payload).context("failed to encode rpc payload")?;
    stdin.write_all(&bytes)?;
    stdin.write_all(b"\n")?;
    stdin.flush().ok();
    Ok(())
}

fn wait_for_rpc_id(rx: &mpsc::Receiver<String>, id: i64, timeout: Duration) -> Result<Value> {
    let start = Instant::now();
    while start.elapsed() < timeout {
        let remaining = timeout.saturating_sub(start.elapsed());
        let line = rx
            .recv_timeout(remaining.min(Duration::from_millis(250)))
            .context("timed out waiting for codex rpc response")?;
        let value: Value = match serde_json::from_str(&line) {
            Ok(value) => value,
            Err(_) => continue,
        };

        if value.get("id").and_then(Value::as_i64) != Some(id) {
            continue;
        }
        if let Some(error) = value.get("error") {
            bail!("codex rpc request failed: {}", error);
        }
        return Ok(value);
    }
    bail!("timed out waiting for codex rpc response");
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_auth_supports_token_object() {
        let data = br#"{
          "tokens": {
            "access_token": "access",
            "refresh_token": "refresh",
            "id_token": "header.eyJlbWFpbCI6InRlc3RAZXhhbXBsZS5jb20ifQ.sig",
            "account_id": "acct_123"
          }
        }"#;

        let auth = parse_auth(data).expect("parse auth");
        assert_eq!(auth.access_token, "access");
        assert_eq!(auth.refresh_token.as_deref(), Some("refresh"));
        assert_eq!(auth.account_id.as_deref(), Some("acct_123"));
    }

    #[test]
    fn auth_file_path_prefers_codex_home() {
        let env = vec![("CODEX_HOME".to_string(), "/tmp/codex-home".to_string())];
        let path = auth_file_path(&env);
        assert_eq!(path, PathBuf::from("/tmp/codex-home/auth.json"));
    }

    #[test]
    fn auth_file_path_defaults_to_dot_codex() {
        let path = auth_file_path(&[]);
        assert!(path.ends_with(".codex/auth.json"));
    }

    #[test]
    fn parse_cli_status_extracts_limits() {
        let snapshot = parse_cli_status(
            "Credits: 123.4\n5h limit 72% left resets 14:00\nWeekly limit 41% left resets Fri 09:00\n",
        )
        .expect("parse cli");

        assert_eq!(snapshot.credits, Some(123.4));
        assert_eq!(
            snapshot.primary.expect("primary").used_percent.round() as i64,
            28
        );
        assert_eq!(
            snapshot.secondary.expect("secondary").used_percent.round() as i64,
            59
        );
    }

    #[test]
    fn fixture_auth_payload_decodes_identity_fields() {
        let auth = parse_auth(include_bytes!("../../tests/fixtures/codex/auth_tokens.json"))
            .expect("fixture auth parses");

        assert_eq!(auth.access_token, "access_fixture_token");
        assert_eq!(
            auth.refresh_token.as_deref(),
            Some("refresh_fixture_token")
        );
        assert_eq!(auth.account_id.as_deref(), Some("acct_fixture_123"));

        let identity = decode_identity(&auth).expect("fixture identity");
        assert_eq!(identity.account_email.as_deref(), Some("fixture@example.com"));
        assert_eq!(identity.plan.as_deref(), Some("pro"));
    }

    #[test]
    fn fixture_oauth_payload_decodes_windows_and_credits() {
        let response: CodexUsageResponse = serde_json::from_str(include_str!(
            "../../tests/fixtures/codex/oauth_usage.json"
        ))
        .expect("fixture oauth usage parses");

        assert_eq!(response.plan_type.as_deref(), Some("pro"));
        assert_eq!(
            oauth_credits_to_f64(response.credits.as_ref().expect("credits")),
            Some(17.25)
        );

        let primary = oauth_window_to_live(
            response
                .rate_limit
                .as_ref()
                .and_then(|limit| limit.primary_window.as_ref())
                .expect("primary"),
        );
        let secondary = oauth_window_to_live(
            response
                .rate_limit
                .as_ref()
                .and_then(|limit| limit.secondary_window.as_ref())
                .expect("secondary"),
        );
        assert_eq!(primary.used_percent, 38.5);
        assert_eq!(primary.window_minutes, Some(300));
        assert_eq!(secondary.used_percent, 61.0);
        assert_eq!(secondary.window_minutes, Some(10_080));
    }

    #[test]
    fn fixture_rpc_payloads_decode_account_limits_and_credits() {
        let account: RpcAccountResponse = serde_json::from_str(include_str!(
            "../../tests/fixtures/codex/rpc_account.json"
        ))
        .expect("fixture rpc account parses");
        let limits: RpcRateLimitsResponse = serde_json::from_str(include_str!(
            "../../tests/fixtures/codex/rpc_rate_limits.json"
        ))
        .expect("fixture rpc limits parses");

        match account.account.expect("account") {
            RpcAccountDetails::ChatGpt { email, plan_type } => {
                assert_eq!(email.as_deref(), Some("rpc-fixture@example.com"));
                assert_eq!(plan_type.as_deref(), Some("team"));
            }
            RpcAccountDetails::ApiKey => panic!("expected chatgpt account"),
        }

        let primary = rpc_window_to_live(limits.rate_limits.primary.as_ref().expect("primary"));
        let secondary = rpc_window_to_live(limits.rate_limits.secondary.as_ref().expect("secondary"));
        assert_eq!(primary.used_percent, 42.0);
        assert_eq!(primary.window_minutes, Some(300));
        assert_eq!(secondary.used_percent, 73.0);
        assert_eq!(secondary.window_minutes, Some(10_080));
        assert_eq!(
            rpc_credits_to_f64(limits.rate_limits.credits.as_ref().expect("credits")),
            Some(11.9)
        );
    }

    #[test]
    fn fixture_cli_status_output_extracts_limits() {
        let snapshot = parse_cli_status(include_str!("../../tests/fixtures/codex/cli_status.txt"))
            .expect("fixture cli parses");

        assert_eq!(snapshot.credits, Some(123.4));
        assert_eq!(
            snapshot.primary.expect("primary").used_percent.round() as i64,
            28
        );
        assert_eq!(
            snapshot.secondary.expect("secondary").used_percent.round() as i64,
            59
        );
    }

    #[test]
    fn fixture_status_shape_matches_expected_rpc_result_wrapper() {
        let payload = json!({
            "result": serde_json::from_str::<Value>(include_str!(
                "../../tests/fixtures/codex/rpc_rate_limits.json"
            ))
            .expect("fixture value")
        });
        let result = serde_json::from_value::<RpcRateLimitsResponse>(
            payload.get("result").cloned().expect("result"),
        )
        .expect("wrapped rpc limits parse");
        assert!(result.rate_limits.primary.is_some());
        assert!(result.rate_limits.secondary.is_some());
    }
}
