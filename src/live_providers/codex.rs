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
use toml::Value as TomlValue;

use crate::models::{
    LiveProviderAuth, LiveProviderIdentity, LiveProviderRecoveryAction, LiveRateWindow,
};

const DEFAULT_USAGE_URL: &str = "https://chatgpt.com/backend-api/wham/usage";

// Codex CLI OAuth refresh — wire format taken from openai/codex
// `codex-rs/login/src/auth/manager.rs`:
//   - endpoint: https://auth.openai.com/oauth/token, JSON body
//   - client_id: app_EMoamEEZ73f0CkXaXp7hrann
//   - body: { client_id, grant_type: "refresh_token", refresh_token }
//   - response: { access_token, refresh_token, id_token }
const CODEX_REFRESH_ENDPOINT: &str = "https://auth.openai.com/oauth/token";
const CODEX_CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
const CODEX_REFRESH_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone)]
pub struct CodexAuth {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    pub account_id: Option<String>,
    pub auth_mode: Option<String>,
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
    #[serde(rename = "requiresOpenaiAuth")]
    pub requires_openai_auth: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum RpcAccountDetails {
    #[serde(rename = "apiKey", alias = "apikey")]
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

#[derive(Debug, Clone, Default)]
pub struct CodexConfigFacts {
    pub credential_store: Option<String>,
    pub forced_login_method: Option<String>,
    pub forced_chatgpt_workspace_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedCodexCredentialStore {
    File,
    Keyring,
    AutoFile,
    AutoKeyring,
}

impl ResolvedCodexCredentialStore {
    pub fn backend_label(self) -> &'static str {
        match self {
            Self::File => "file",
            Self::Keyring => "keyring",
            Self::AutoFile => "auto-file",
            Self::AutoKeyring => "auto-keyring",
        }
    }

    pub fn persists_to_file(self) -> bool {
        matches!(self, Self::File | Self::AutoFile)
    }
}

#[derive(Debug, Clone)]
pub struct CodexBootstrapAuth {
    pub auth: Option<CodexAuth>,
    pub credential_store: ResolvedCodexCredentialStore,
    pub auth_file_path: PathBuf,
    pub load_error: Option<String>,
}

pub struct CodexAuthHealthInput<'a> {
    pub credential_store: ResolvedCodexCredentialStore,
    pub auth: Option<&'a CodexAuth>,
    pub identity: Option<&'a LiveProviderIdentity>,
    pub available: bool,
    pub bootstrap_error: Option<&'a str>,
    pub error: Option<&'a str>,
}

pub fn resolve_credential_store(
    env: &[(String, String)],
    config: &CodexConfigFacts,
) -> ResolvedCodexCredentialStore {
    let auth_path = auth_file_path(env);
    match config.credential_store.as_deref() {
        Some("file") => ResolvedCodexCredentialStore::File,
        Some("keyring") => ResolvedCodexCredentialStore::Keyring,
        Some("auto") | None => {
            if auth_path.exists() {
                ResolvedCodexCredentialStore::AutoFile
            } else {
                ResolvedCodexCredentialStore::AutoKeyring
            }
        }
        Some(_) => {
            if auth_path.exists() {
                ResolvedCodexCredentialStore::AutoFile
            } else {
                ResolvedCodexCredentialStore::AutoKeyring
            }
        }
    }
}

pub fn resolve_bootstrap_auth(
    env: &[(String, String)],
    config: &CodexConfigFacts,
) -> CodexBootstrapAuth {
    let credential_store = resolve_credential_store(env, config);
    let auth_file_path = auth_file_path(env);
    let (auth, load_error) = if credential_store.persists_to_file() {
        match load_auth(env) {
            Ok(auth) => (Some(auth), None),
            Err(err) => (None, Some(err.to_string())),
        }
    } else {
        (None, None)
    };

    CodexBootstrapAuth {
        auth,
        credential_store,
        auth_file_path,
        load_error,
    }
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
            auth_mode: Some("api-key".into()),
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
        auth_mode: json
            .get("auth_mode")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
    })
}

#[derive(Debug, Clone, Default)]
pub struct CodexRefreshedTokens {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
}

/// POST a refresh_token grant to Codex's OAuth provider and return the new
/// tokens. On any failure (transport, non-2xx, malformed JSON) returns an
/// error; the caller is responsible for falling back to the pre-refresh auth.
pub async fn refresh_oauth_token(refresh_token: &str) -> Result<CodexRefreshedTokens> {
    let client = reqwest::Client::builder()
        .user_agent("claude-usage-tracker/0.1")
        .timeout(CODEX_REFRESH_TIMEOUT)
        .build()
        .context("failed to build Codex refresh client")?;

    let body = json!({
        "client_id": CODEX_CLIENT_ID,
        "grant_type": "refresh_token",
        "refresh_token": refresh_token,
    });

    let response = client
        .post(CODEX_REFRESH_ENDPOINT)
        .json(&body)
        .send()
        .await
        .context("failed to POST Codex token refresh")?;

    let status = response.status();
    let body_text = response.text().await.unwrap_or_default();
    if !status.is_success() {
        bail!(
            "Codex token refresh failed ({}): {}",
            status.as_u16(),
            body_text.chars().take(200).collect::<String>()
        );
    }
    parse_refresh_response(&body_text)
}

pub fn parse_refresh_response(body: &str) -> Result<CodexRefreshedTokens> {
    let parsed: Value =
        serde_json::from_str(body).context("invalid Codex refresh response JSON")?;
    Ok(CodexRefreshedTokens {
        access_token: parsed
            .get("access_token")
            .and_then(Value::as_str)
            .map(String::from),
        refresh_token: parsed
            .get("refresh_token")
            .and_then(Value::as_str)
            .map(String::from),
        id_token: parsed
            .get("id_token")
            .and_then(Value::as_str)
            .map(String::from),
    })
}

/// Return a new CodexAuth with refreshed access/refresh/id tokens merged in.
/// Missing response fields fall back to the prior auth values.
pub fn apply_refreshed_tokens(auth: &CodexAuth, tokens: &CodexRefreshedTokens) -> CodexAuth {
    CodexAuth {
        access_token: tokens
            .access_token
            .clone()
            .unwrap_or_else(|| auth.access_token.clone()),
        refresh_token: tokens
            .refresh_token
            .clone()
            .or_else(|| auth.refresh_token.clone()),
        id_token: tokens.id_token.clone().or_else(|| auth.id_token.clone()),
        account_id: auth.account_id.clone(),
        auth_mode: auth.auth_mode.clone(),
    }
}

/// Update `~/.codex/auth.json` (or `$CODEX_HOME/auth.json`) with the refreshed
/// tokens, preserving other top-level keys (OPENAI_API_KEY, auth_mode, etc.)
/// and bumping `last_refresh` to the current time.
pub fn persist_refreshed_tokens_to_disk(
    env: &[(String, String)],
    tokens: &CodexRefreshedTokens,
) -> Result<()> {
    let path = auth_file_path(env);
    let existing = std::fs::read(&path)
        .with_context(|| format!("read Codex auth file at {}", path.display()))?;
    let mut json: Value = serde_json::from_slice(&existing).context("invalid Codex auth.json")?;

    let root = json
        .as_object_mut()
        .ok_or_else(|| anyhow!("Codex auth.json root must be an object"))?;
    let tokens_entry = root
        .entry("tokens".to_string())
        .or_insert_with(|| Value::Object(Default::default()));
    let tokens_obj = tokens_entry
        .as_object_mut()
        .ok_or_else(|| anyhow!("Codex auth.json tokens must be an object"))?;
    if let Some(at) = tokens.access_token.as_deref() {
        tokens_obj.insert("access_token".into(), Value::String(at.to_string()));
    }
    if let Some(rt) = tokens.refresh_token.as_deref() {
        tokens_obj.insert("refresh_token".into(), Value::String(rt.to_string()));
    }
    if let Some(it) = tokens.id_token.as_deref() {
        tokens_obj.insert("id_token".into(), Value::String(it.to_string()));
    }
    root.insert(
        "last_refresh".into(),
        Value::String(chrono::Utc::now().to_rfc3339()),
    );

    let rendered = serde_json::to_string_pretty(&json).context("serialize Codex auth.json")?;
    std::fs::write(&path, rendered)
        .with_context(|| format!("write Codex auth file at {}", path.display()))?;
    Ok(())
}

/// Heuristic: does this fetch error look like an auth failure worth retrying
/// after refresh? We don't get typed errors out of fetch_oauth_usage (it goes
/// through reqwest::Response::error_for_status), so we sniff the message.
pub fn looks_like_oauth_auth_error(err: &anyhow::Error) -> bool {
    let s = format!("{err:#}");
    s.contains("401")
        || s.contains("403")
        || s.to_lowercase().contains("unauthorized")
        || s.to_lowercase().contains("invalid_token")
        || s.to_lowercase().contains("token expired")
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
        login_method: auth.auth_mode.clone().or_else(|| Some("chatgpt".into())),
        plan,
    })
}

pub fn rpc_account_to_identity(account: &RpcAccountResponse) -> Option<LiveProviderIdentity> {
    match account.account.as_ref()? {
        RpcAccountDetails::ApiKey => Some(LiveProviderIdentity {
            provider: "codex".into(),
            account_email: None,
            account_organization: None,
            login_method: Some("api-key".into()),
            plan: None,
        }),
        RpcAccountDetails::ChatGpt { email, plan_type } => Some(LiveProviderIdentity {
            provider: "codex".into(),
            account_email: email.clone(),
            account_organization: None,
            login_method: Some("chatgpt".into()),
            plan: plan_type.clone(),
        }),
    }
}

fn normalized_codex_login_method(value: Option<&str>) -> Option<&'static str> {
    match value.map(str::trim).filter(|value| !value.is_empty()) {
        Some(value)
            if value.eq_ignore_ascii_case("chatgpt")
                || value.eq_ignore_ascii_case("chatgptdevicecode") =>
        {
            Some("chatgpt")
        }
        Some(value)
            if value.eq_ignore_ascii_case("api")
                || value.eq_ignore_ascii_case("api-key")
                || value.eq_ignore_ascii_case("apikey")
                || value.eq_ignore_ascii_case("apiKey") =>
        {
            Some("api")
        }
        _ => None,
    }
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
        &json!({"id": next_id, "method": "account/read", "params": {"refreshToken": true}}),
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

fn config_file_path(env: &[(String, String)]) -> PathBuf {
    let env_map = env
        .iter()
        .cloned()
        .collect::<std::collections::HashMap<_, _>>();
    if let Some(codex_home) = env_map.get("CODEX_HOME").filter(|value| !value.is_empty()) {
        return PathBuf::from(codex_home).join("config.toml");
    }

    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join(".codex")
        .join("config.toml")
}

pub fn load_config_facts(env: &[(String, String)]) -> CodexConfigFacts {
    let path = config_file_path(env);
    let Ok(contents) = std::fs::read_to_string(path) else {
        return CodexConfigFacts::default();
    };
    let Ok(value) = contents.parse::<TomlValue>() else {
        return CodexConfigFacts::default();
    };
    CodexConfigFacts {
        credential_store: value
            .get("cli_auth_credentials_store")
            .and_then(TomlValue::as_str)
            .map(ToOwned::to_owned),
        forced_login_method: value
            .get("forced_login_method")
            .and_then(TomlValue::as_str)
            .map(ToOwned::to_owned),
        forced_chatgpt_workspace_id: value
            .get("forced_chatgpt_workspace_id")
            .and_then(TomlValue::as_str)
            .map(ToOwned::to_owned),
    }
}

pub fn build_auth_health(
    env: &[(String, String)],
    config: &CodexConfigFacts,
    input: CodexAuthHealthInput<'_>,
) -> LiveProviderAuth {
    let CodexAuthHealthInput {
        credential_store,
        auth,
        identity,
        available,
        bootstrap_error,
        error,
    } = input;
    let validated_at = Some(chrono::Utc::now().to_rfc3339());
    let headless = env_has(env, "SSH_CONNECTION")
        || env_has(env, "CI")
        || env_has(env, "GITHUB_ACTIONS")
        || env_has(env, "CODESPACES");
    let preferred_login_command = if headless {
        "codex login --device-auth"
    } else {
        "codex login"
    };
    let mut recovery_actions = vec![
        recovery_action(
            if headless {
                "Run codex login --device-auth"
            } else {
                "Run codex login"
            },
            if headless {
                "codex-login-device"
            } else {
                "codex-login"
            },
            Some(preferred_login_command),
            None,
        ),
        recovery_action(
            "Run codex login --device-auth",
            "codex-login-device",
            Some("codex login --device-auth"),
            Some("Preferred on remote or headless machines.".into()),
        ),
        recovery_action(
            "Open login diagnostics",
            "codex-login-diagnostics",
            Some("codex login --help"),
            None,
        ),
    ];

    if let Some(store) = config.credential_store.as_deref() {
        recovery_actions.push(recovery_action(
            "Explain storage mode",
            "codex-explain-storage",
            None,
            Some(format!(
                "Codex CLI credential storage is configured as `{store}`."
            )),
        ));
    }
    if config.forced_login_method.is_some() || config.forced_chatgpt_workspace_id.is_some() {
        recovery_actions.push(recovery_action(
            "Explain managed restriction mismatch",
            "codex-explain-managed-policy",
            None,
            Some(format!(
                "Managed policy: login_method={:?}, workspace_id={:?}",
                config.forced_login_method, config.forced_chatgpt_workspace_id
            )),
        ));
    }

    if env_has(env, "OPENAI_API_KEY") {
        return LiveProviderAuth {
            login_method: Some("api-key".into()),
            credential_backend: Some("env".into()),
            auth_mode: Some("api-key".into()),
            is_authenticated: true,
            is_refreshable: false,
            is_source_compatible: false,
            requires_relogin: false,
            managed_restriction: None,
            diagnostic_code: Some("env-override".into()),
            failure_reason: Some(
                "OPENAI_API_KEY is active, so Codex is authenticated in API-key mode rather than ChatGPT subscription mode."
                    .into(),
            ),
            last_validated_at: validated_at,
            recovery_actions,
        };
    }

    let login_method = auth
        .and_then(|auth| auth.auth_mode.clone())
        .or_else(|| identity.and_then(|identity| identity.login_method.clone()))
        .unwrap_or_else(|| "chatgpt".into());
    let normalized_login_method =
        normalized_codex_login_method(Some(&login_method)).unwrap_or("chatgpt");
    let managed_restriction = if let Some(forced_login_method) = &config.forced_login_method {
        if normalized_codex_login_method(Some(forced_login_method.as_str()))
            != Some(normalized_login_method)
        {
            Some(format!("forced_login_method={forced_login_method}"))
        } else {
            None
        }
    } else {
        config
            .forced_chatgpt_workspace_id
            .as_ref()
            .map(|workspace_id| format!("forced_chatgpt_workspace_id={workspace_id}"))
    };

    let is_refreshable = auth
        .and_then(|auth| auth.refresh_token.as_deref())
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false);
    let is_authenticated = auth.is_some() || available;
    let source_compatible = normalized_login_method == "chatgpt" && managed_restriction.is_none();
    let requires_relogin = !is_authenticated
        || error
            .map(|message| message.to_lowercase().contains("expired"))
            .unwrap_or(false);
    let diagnostic_code = if managed_restriction.is_some() {
        Some("managed-policy".into())
    } else if !is_authenticated {
        Some("missing-credentials".into())
    } else if !source_compatible {
        Some("authenticated-incompatible-source".into())
    } else if requires_relogin && is_refreshable {
        Some("expired-refreshable".into())
    } else if requires_relogin {
        Some("requires-relogin".into())
    } else {
        Some("authenticated-compatible".into())
    };
    let failure_reason = if let Some(managed_restriction) = &managed_restriction {
        Some(format!(
            "Codex auth does not satisfy managed policy `{managed_restriction}`."
        ))
    } else if !is_authenticated {
        bootstrap_error.map(ToOwned::to_owned).or_else(|| {
            Some(match credential_store {
                ResolvedCodexCredentialStore::File => {
                    "Codex file-backed credentials are unavailable, and no active CLI session was detected.".into()
                }
                ResolvedCodexCredentialStore::Keyring => {
                    "Codex keyring-backed credentials are unavailable, and no active CLI session was detected.".into()
                }
                ResolvedCodexCredentialStore::AutoFile => {
                    "Codex auto storage resolved to file-backed auth, but no usable auth.json or active CLI session was found.".into()
                }
                ResolvedCodexCredentialStore::AutoKeyring => {
                    "Codex auto storage resolved to keyring-backed auth, but no active CLI session was detected.".into()
                }
            })
        })
    } else if !source_compatible {
        Some("Codex is authenticated with API key semantics, so ChatGPT credits and subscription quota features are unavailable.".into())
    } else if requires_relogin && is_refreshable {
        Some("Codex access token expired, but refresh should be possible.".into())
    } else if requires_relogin {
        Some("Codex requires a fresh login.".into())
    } else {
        error.map(ToOwned::to_owned)
    };

    LiveProviderAuth {
        login_method: Some(login_method.clone()),
        credential_backend: Some(credential_store.backend_label().into()),
        auth_mode: Some(login_method),
        is_authenticated,
        is_refreshable,
        is_source_compatible: source_compatible,
        requires_relogin,
        managed_restriction,
        diagnostic_code,
        failure_reason,
        last_validated_at: validated_at,
        recovery_actions,
    }
}

fn recovery_action(
    label: &str,
    action_id: &str,
    command: Option<&str>,
    detail: Option<String>,
) -> LiveProviderRecoveryAction {
    LiveProviderRecoveryAction {
        label: label.into(),
        action_id: action_id.into(),
        command: command.map(ToOwned::to_owned),
        detail,
    }
}

fn env_has(env: &[(String, String)], key: &str) -> bool {
    env.iter()
        .find(|(candidate, _)| candidate == key)
        .map(|(_, value)| !value.trim().is_empty())
        .unwrap_or(false)
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
    while !normalized.len().is_multiple_of(4) {
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
    fn parse_refresh_response_extracts_all_fields() {
        let body = r#"{
            "access_token": "at_new",
            "refresh_token": "rt_new",
            "id_token": "it_new"
        }"#;
        let tokens = parse_refresh_response(body).expect("parse refresh");
        assert_eq!(tokens.access_token.as_deref(), Some("at_new"));
        assert_eq!(tokens.refresh_token.as_deref(), Some("rt_new"));
        assert_eq!(tokens.id_token.as_deref(), Some("it_new"));
    }

    #[test]
    fn parse_refresh_response_handles_partial_payload() {
        let body = r#"{ "access_token": "at_new" }"#;
        let tokens = parse_refresh_response(body).expect("parse partial");
        assert_eq!(tokens.access_token.as_deref(), Some("at_new"));
        assert!(tokens.refresh_token.is_none());
        assert!(tokens.id_token.is_none());
    }

    #[test]
    fn apply_refreshed_tokens_merges_with_existing_auth() {
        let auth = CodexAuth {
            access_token: "old_at".into(),
            refresh_token: Some("old_rt".into()),
            id_token: Some("old_it".into()),
            account_id: Some("acct_1".into()),
            auth_mode: Some("chatgpt".into()),
        };
        // Only access_token is rotated — refresh/id fall back to prior values.
        let tokens = CodexRefreshedTokens {
            access_token: Some("new_at".into()),
            refresh_token: None,
            id_token: None,
        };
        let merged = apply_refreshed_tokens(&auth, &tokens);
        assert_eq!(merged.access_token, "new_at");
        assert_eq!(merged.refresh_token.as_deref(), Some("old_rt"));
        assert_eq!(merged.id_token.as_deref(), Some("old_it"));
        assert_eq!(merged.account_id.as_deref(), Some("acct_1"));
    }

    #[test]
    fn persist_refreshed_tokens_updates_auth_json() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let codex_home = tmp.path().to_path_buf();
        let auth_path = codex_home.join("auth.json");
        std::fs::write(
            &auth_path,
            r#"{
                "OPENAI_API_KEY": "sk-keep-me",
                "auth_mode": "chatgpt",
                "tokens": {
                    "access_token": "old_at",
                    "refresh_token": "old_rt",
                    "id_token": "old_it",
                    "account_id": "acct_preserve"
                }
            }"#,
        )
        .expect("seed auth.json");

        let env = vec![(
            "CODEX_HOME".to_string(),
            codex_home.to_string_lossy().into_owned(),
        )];
        let tokens = CodexRefreshedTokens {
            access_token: Some("fresh_at".into()),
            refresh_token: Some("fresh_rt".into()),
            id_token: None,
        };
        persist_refreshed_tokens_to_disk(&env, &tokens).expect("persist");

        let written = std::fs::read_to_string(&auth_path).expect("read back");
        let parsed: Value = serde_json::from_str(&written).expect("valid json");
        let tokens_obj = parsed
            .get("tokens")
            .and_then(Value::as_object)
            .expect("tokens object");
        assert_eq!(
            tokens_obj.get("access_token").and_then(Value::as_str),
            Some("fresh_at")
        );
        assert_eq!(
            tokens_obj.get("refresh_token").and_then(Value::as_str),
            Some("fresh_rt")
        );
        // id_token was not in the refresh response — the old value stays.
        assert_eq!(
            tokens_obj.get("id_token").and_then(Value::as_str),
            Some("old_it")
        );
        // account_id is preserved verbatim.
        assert_eq!(
            tokens_obj.get("account_id").and_then(Value::as_str),
            Some("acct_preserve")
        );
        // Sibling top-level keys are preserved.
        assert_eq!(
            parsed.get("OPENAI_API_KEY").and_then(Value::as_str),
            Some("sk-keep-me")
        );
        assert_eq!(
            parsed.get("auth_mode").and_then(Value::as_str),
            Some("chatgpt")
        );
        // last_refresh is stamped.
        assert!(parsed.get("last_refresh").and_then(Value::as_str).is_some());
    }

    #[test]
    fn looks_like_oauth_auth_error_recognizes_401() {
        let err = anyhow!("request failed: 401 Unauthorized");
        assert!(looks_like_oauth_auth_error(&err));

        let err = anyhow!("timeout waiting for response");
        assert!(!looks_like_oauth_auth_error(&err));
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
    fn resolve_credential_store_honors_config_and_file_presence() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let codex_home = tmp.path().to_path_buf();
        let env = vec![(
            "CODEX_HOME".to_string(),
            codex_home.to_string_lossy().into_owned(),
        )];

        assert_eq!(
            resolve_credential_store(&env, &CodexConfigFacts::default()),
            ResolvedCodexCredentialStore::AutoKeyring
        );

        std::fs::write(codex_home.join("auth.json"), "{}").expect("seed auth file");
        assert_eq!(
            resolve_credential_store(&env, &CodexConfigFacts::default()),
            ResolvedCodexCredentialStore::AutoFile
        );
        assert_eq!(
            resolve_credential_store(
                &env,
                &CodexConfigFacts {
                    credential_store: Some("keyring".into()),
                    ..CodexConfigFacts::default()
                }
            ),
            ResolvedCodexCredentialStore::Keyring
        );
        assert_eq!(
            resolve_credential_store(
                &env,
                &CodexConfigFacts {
                    credential_store: Some("file".into()),
                    ..CodexConfigFacts::default()
                }
            ),
            ResolvedCodexCredentialStore::File
        );
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
        let auth = parse_auth(include_bytes!(
            "../../tests/fixtures/codex/auth_tokens.json"
        ))
        .expect("fixture auth parses");

        assert_eq!(auth.access_token, "access_fixture_token");
        assert_eq!(auth.refresh_token.as_deref(), Some("refresh_fixture_token"));
        assert_eq!(auth.account_id.as_deref(), Some("acct_fixture_123"));

        let identity = decode_identity(&auth).expect("fixture identity");
        assert_eq!(
            identity.account_email.as_deref(),
            Some("fixture@example.com")
        );
        assert_eq!(identity.plan.as_deref(), Some("pro"));
    }

    #[test]
    fn fixture_oauth_payload_decodes_windows_and_credits() {
        let response: CodexUsageResponse =
            serde_json::from_str(include_str!("../../tests/fixtures/codex/oauth_usage.json"))
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
        let account: RpcAccountResponse =
            serde_json::from_str(include_str!("../../tests/fixtures/codex/rpc_account.json"))
                .expect("fixture rpc account parses");
        let limits: RpcRateLimitsResponse = serde_json::from_str(include_str!(
            "../../tests/fixtures/codex/rpc_rate_limits.json"
        ))
        .expect("fixture rpc limits parses");

        match account.account.as_ref().expect("account") {
            RpcAccountDetails::ChatGpt { email, plan_type } => {
                assert_eq!(email.as_deref(), Some("rpc-fixture@example.com"));
                assert_eq!(plan_type.as_deref(), Some("team"));
            }
            RpcAccountDetails::ApiKey => panic!("expected chatgpt account"),
        }

        let primary = rpc_window_to_live(limits.rate_limits.primary.as_ref().expect("primary"));
        let secondary =
            rpc_window_to_live(limits.rate_limits.secondary.as_ref().expect("secondary"));
        assert_eq!(primary.used_percent, 42.0);
        assert_eq!(primary.window_minutes, Some(300));
        assert_eq!(secondary.used_percent, 73.0);
        assert_eq!(secondary.window_minutes, Some(10_080));
        assert_eq!(
            rpc_credits_to_f64(limits.rate_limits.credits.as_ref().expect("credits")),
            Some(11.9)
        );

        let identity = rpc_account_to_identity(&account).expect("rpc identity");
        assert_eq!(
            identity.account_email.as_deref(),
            Some("rpc-fixture@example.com")
        );
        assert_eq!(identity.login_method.as_deref(), Some("chatgpt"));
        assert_eq!(identity.plan.as_deref(), Some("team"));
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

    #[test]
    fn build_auth_health_marks_env_override_as_incompatible() {
        let env = vec![("OPENAI_API_KEY".to_string(), "sk-live".to_string())];
        let health = build_auth_health(
            &env,
            &CodexConfigFacts::default(),
            CodexAuthHealthInput {
                credential_store: ResolvedCodexCredentialStore::File,
                auth: None,
                identity: None,
                available: false,
                bootstrap_error: None,
                error: None,
            },
        );

        assert_eq!(health.diagnostic_code.as_deref(), Some("env-override"));
        assert_eq!(health.login_method.as_deref(), Some("api-key"));
        assert_eq!(health.credential_backend.as_deref(), Some("env"));
        assert!(health.is_authenticated);
        assert!(!health.is_source_compatible);
    }

    #[test]
    fn build_auth_health_prefers_device_auth_recovery_when_headless() {
        let env = vec![("SSH_CONNECTION".to_string(), "1".to_string())];
        let health = build_auth_health(
            &env,
            &CodexConfigFacts {
                credential_store: Some("auto".into()),
                ..CodexConfigFacts::default()
            },
            CodexAuthHealthInput {
                credential_store: ResolvedCodexCredentialStore::AutoKeyring,
                auth: None,
                identity: None,
                available: false,
                bootstrap_error: None,
                error: Some("expired token"),
            },
        );

        assert_eq!(
            health.diagnostic_code.as_deref(),
            Some("missing-credentials")
        );
        assert_eq!(
            health
                .recovery_actions
                .first()
                .and_then(|action| action.command.as_deref()),
            Some("codex login --device-auth")
        );
    }

    #[test]
    fn build_auth_health_normalizes_api_managed_policy() {
        let health = build_auth_health(
            &[],
            &CodexConfigFacts {
                forced_login_method: Some("api".into()),
                ..CodexConfigFacts::default()
            },
            CodexAuthHealthInput {
                credential_store: ResolvedCodexCredentialStore::File,
                auth: Some(&CodexAuth {
                    access_token: "sk-test".into(),
                    refresh_token: None,
                    id_token: None,
                    account_id: None,
                    auth_mode: Some("api-key".into()),
                }),
                identity: None,
                available: true,
                bootstrap_error: None,
                error: None,
            },
        );

        assert!(health.managed_restriction.is_none());
        assert_eq!(
            health.diagnostic_code.as_deref(),
            Some("authenticated-incompatible-source")
        );
    }
}
