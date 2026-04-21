use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::Value;
use tracing::{debug, info, warn};

use crate::models::{LiveProviderAuth, LiveProviderRecoveryAction};

use super::models::{CredentialsFile, Identity, OAuthCredentials, Plan};

const REFRESH_ENDPOINT: &str = "https://platform.claude.com/v1/oauth/token";
const CLIENT_ID: &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e";
const REFRESH_TIMEOUT: Duration = Duration::from_secs(30);
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
const KEYCHAIN_SERVICE_CANDIDATES: &[&str] = &["Claude Code-credentials", "Claude Code"];

#[derive(Debug, Clone)]
pub struct ResolvedClaudeAuth {
    pub credentials: Option<OAuthCredentials>,
    pub identity: Option<Identity>,
    pub health: LiveProviderAuth,
    pub credential_store: Option<ClaudeCredentialStore>,
}

#[derive(Debug, Clone)]
pub enum ClaudeCredentialStore {
    File(PathBuf),
    Keychain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
enum KeychainStatus {
    Available,
    Missing,
    Locked,
    Error,
}

#[derive(Debug, Default)]
struct ClaudeSettingsFacts {
    api_key_helper: bool,
}

fn credentials_path() -> PathBuf {
    credentials_path_from_env(&std::env::vars().collect::<Vec<_>>())
}

fn claude_config_dir(env: &[(String, String)]) -> PathBuf {
    env.iter()
        .find(|(key, _)| key == "CLAUDE_CONFIG_DIR")
        .map(|(_, value)| PathBuf::from(value))
        .unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".claude")
        })
}

fn credentials_path_from_env(env: &[(String, String)]) -> PathBuf {
    claude_config_dir(env).join(".credentials.json")
}

fn settings_path_from_env(env: &[(String, String)]) -> PathBuf {
    claude_config_dir(env).join("settings.json")
}

/// Load OAuth credentials from Claude's credentials file.
/// Returns None if file missing, malformed, or has no OAuth section.
pub fn load_credentials() -> Option<OAuthCredentials> {
    load_credentials_from(&credentials_path())
}

pub fn load_credentials_from(path: &Path) -> Option<OAuthCredentials> {
    let contents = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => {
            debug!("No credentials file at {}", path.display());
            return None;
        }
    };

    let file: CredentialsFile = match serde_json::from_str(&contents) {
        Ok(f) => f,
        Err(e) => {
            warn!("Failed to parse credentials file: {}", e);
            return None;
        }
    };

    file.claude_ai_oauth
}

#[cfg_attr(not(any(target_os = "macos", test)), allow(dead_code))]
fn load_credentials_from_str(contents: &str) -> Option<OAuthCredentials> {
    if let Ok(file) = serde_json::from_str::<CredentialsFile>(contents)
        && file.claude_ai_oauth.is_some()
    {
        return file.claude_ai_oauth;
    }
    serde_json::from_str::<OAuthCredentials>(contents).ok()
}

pub fn resolve_auth(env: &[(String, String)]) -> ResolvedClaudeAuth {
    let settings = load_settings_facts(&settings_path_from_env(env));
    let file_path = credentials_path_from_env(env);
    let file_creds = load_credentials_from(&file_path);
    let (keychain_creds, keychain_status) = load_credentials_from_keychain();
    let validated_at = chrono::Utc::now().to_rfc3339();

    if env_has(env, "ANTHROPIC_API_KEY") {
        return ResolvedClaudeAuth {
            credentials: None,
            identity: None,
            health: auth_health(
                Some("api-key".into()),
                Some("env".into()),
                Some("anthropic-api-key".into()),
                true,
                false,
                false,
                false,
                None,
                Some("env-override".into()),
                Some("ANTHROPIC_API_KEY overrides Claude subscription OAuth.".into()),
                Some(validated_at),
                vec![
                    recovery_action("Run Claude", "claude-run", Some("claude"), None),
                    recovery_action(
                        "Explain env override conflict",
                        "claude-explain-env-override",
                        None,
                        Some(
                            "Unset ANTHROPIC_API_KEY to restore subscription OAuth detection."
                                .into(),
                        ),
                    ),
                ],
            ),
            credential_store: None,
        };
    }

    if env_has(env, "ANTHROPIC_AUTH_TOKEN") {
        return ResolvedClaudeAuth {
            credentials: None,
            identity: None,
            health: auth_health(
                Some("auth-token".into()),
                Some("env".into()),
                Some("anthropic-auth-token".into()),
                true,
                false,
                false,
                false,
                None,
                Some("env-override".into()),
                Some("ANTHROPIC_AUTH_TOKEN overrides Claude subscription OAuth.".into()),
                Some(validated_at),
                vec![
                    recovery_action("Run Claude", "claude-run", Some("claude"), None),
                    recovery_action(
                        "Explain env override conflict",
                        "claude-explain-env-override",
                        None,
                        Some(
                            "Unset ANTHROPIC_AUTH_TOKEN to restore subscription OAuth detection."
                                .into(),
                        ),
                    ),
                ],
            ),
            credential_store: None,
        };
    }

    if settings.api_key_helper {
        return ResolvedClaudeAuth {
            credentials: None,
            identity: None,
            health: auth_health(
                Some("api-key-helper".into()),
                Some("config".into()),
                Some("api-key-helper".into()),
                true,
                false,
                false,
                false,
                None,
                Some("env-override".into()),
                Some("Claude settings enable apiKeyHelper, which masks subscription OAuth.".into()),
                Some(validated_at),
                vec![
                    recovery_action("Run Claude", "claude-run", Some("claude"), None),
                    recovery_action(
                        "Explain env override conflict",
                        "claude-explain-env-override",
                        None,
                        Some(
                            "Disable apiKeyHelper in Claude settings to use subscription OAuth."
                                .into(),
                        ),
                    ),
                ],
            ),
            credential_store: None,
        };
    }

    if claude_cloud_provider(env).is_some() {
        let provider = claude_cloud_provider(env).unwrap_or("cloud");
        return ResolvedClaudeAuth {
            credentials: None,
            identity: None,
            health: auth_health(
                Some(provider.into()),
                Some("env".into()),
                Some(provider.into()),
                true,
                false,
                false,
                false,
                None,
                Some("env-override".into()),
                Some(format!(
                    "Claude appears to be configured for {} auth, not subscription OAuth.",
                    provider
                )),
                Some(validated_at),
                vec![
                    recovery_action("Run Claude", "claude-run", Some("claude"), None),
                    recovery_action(
                        "Explain env override conflict",
                        "claude-explain-env-override",
                        None,
                        Some("Clear cloud-provider auth env vars to restore subscription OAuth detection.".into()),
                    ),
                ],
            ),
            credential_store: None,
        };
    }

    if env_has(env, "CLAUDE_CODE_OAUTH_TOKEN") {
        return ResolvedClaudeAuth {
            credentials: None,
            identity: None,
            health: auth_health(
                Some("oauth-token".into()),
                Some("env".into()),
                Some("oauth-token-env".into()),
                true,
                false,
                true,
                false,
                None,
                Some("headless-oauth-env".into()),
                Some("CLAUDE_CODE_OAUTH_TOKEN is active; account validation is limited compared with stored desktop login.".into()),
                Some(validated_at),
                vec![
                    recovery_action("Run Claude", "claude-run", Some("claude"), None),
                    recovery_action("Run Claude login flow", "claude-login", Some("claude login"), None),
                ],
            ),
            credential_store: None,
        };
    }

    if let Some(creds) = keychain_creds {
        let identity = get_identity(&creds);
        return ResolvedClaudeAuth {
            credentials: Some(creds.clone()),
            identity: Some(identity.clone()),
            health: oauth_health(&creds, "keychain", None, Some(validated_at)),
            credential_store: Some(ClaudeCredentialStore::Keychain),
        };
    }

    if let Some(creds) = file_creds {
        let identity = get_identity(&creds);
        return ResolvedClaudeAuth {
            credentials: Some(creds.clone()),
            identity: Some(identity.clone()),
            health: oauth_health(&creds, "file", None, Some(validated_at)),
            credential_store: Some(ClaudeCredentialStore::File(file_path)),
        };
    }

    let (diagnostic_code, failure_reason) = match keychain_status {
        // User-denied or locked login.keychain-db. Distinct code so the UI
        // can tell them to retry (re-running the helper re-triggers the
        // prompt) instead of lumping with system-level Keychain faults.
        KeychainStatus::Locked => (
            Some("keychain-locked".into()),
            Some("Access to the Claude macOS Keychain item was denied. Retry and approve the prompt, or unlock your login keychain.".into()),
        ),
        KeychainStatus::Error => (
            Some("keychain-unavailable".into()),
            Some("Claude credentials could not be read from macOS Keychain.".into()),
        ),
        _ => (
            Some("missing-credentials".into()),
            Some(format!(
                "No Claude subscription OAuth credentials were found in {}.",
                file_path.display()
            )),
        ),
    };

    let mut recovery_actions = vec![
        recovery_action("Run Claude", "claude-run", Some("claude"), None),
        recovery_action(
            "Run Claude login flow",
            "claude-login",
            Some("claude login"),
            None,
        ),
        recovery_action(
            "Run Claude doctor",
            "claude-doctor",
            Some("claude doctor"),
            None,
        ),
    ];
    if cfg!(target_os = "macos") {
        recovery_actions.push(recovery_action(
            "Show keychain guidance",
            "claude-keychain-guidance",
            None,
            Some("Claude Code stores desktop credentials in macOS Keychain; unlock login.keychain and retry if needed.".into()),
        ));
    }

    ResolvedClaudeAuth {
        credentials: None,
        identity: None,
        health: auth_health(
            Some("subscription-oauth".into()),
            match keychain_status {
                KeychainStatus::Locked | KeychainStatus::Error => Some("keychain".into()),
                _ => Some("file".into()),
            },
            Some("subscription-oauth".into()),
            false,
            false,
            false,
            true,
            None,
            diagnostic_code,
            failure_reason,
            Some(validated_at),
            recovery_actions,
        ),
        credential_store: None,
    }
}

fn oauth_health(
    creds: &OAuthCredentials,
    backend: &str,
    managed_restriction: Option<String>,
    validated_at: Option<String>,
) -> LiveProviderAuth {
    let token_valid = is_token_valid(creds);
    let is_refreshable = creds
        .refresh_token
        .as_deref()
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false);
    let requires_relogin = !token_valid && !is_refreshable;
    let diagnostic_code = if token_valid {
        Some("authenticated-compatible".into())
    } else if is_refreshable {
        Some("expired-refreshable".into())
    } else {
        Some("requires-relogin".into())
    };
    let failure_reason = if token_valid {
        None
    } else if is_refreshable {
        Some("Claude subscription OAuth expired, but Heimdall can attempt token refresh.".into())
    } else {
        Some("Claude subscription OAuth expired and requires a new login.".into())
    };
    let mut recovery_actions = vec![
        recovery_action("Run Claude", "claude-run", Some("claude"), None),
        recovery_action(
            "Run Claude login flow",
            "claude-login",
            Some("claude login"),
            None,
        ),
        recovery_action(
            "Run Claude doctor",
            "claude-doctor",
            Some("claude doctor"),
            None,
        ),
    ];
    if backend == "keychain" {
        recovery_actions.push(recovery_action(
            "Show keychain guidance",
            "claude-keychain-guidance",
            None,
            Some("Claude Code stores desktop credentials in macOS Keychain.".into()),
        ));
    }

    auth_health(
        Some("subscription-oauth".into()),
        Some(backend.into()),
        Some("subscription-oauth".into()),
        token_valid || is_refreshable,
        is_refreshable,
        true,
        requires_relogin,
        managed_restriction,
        diagnostic_code,
        failure_reason,
        validated_at,
        recovery_actions,
    )
}

#[allow(clippy::too_many_arguments)]
fn auth_health(
    login_method: Option<String>,
    credential_backend: Option<String>,
    auth_mode: Option<String>,
    is_authenticated: bool,
    is_refreshable: bool,
    is_source_compatible: bool,
    requires_relogin: bool,
    managed_restriction: Option<String>,
    diagnostic_code: Option<String>,
    failure_reason: Option<String>,
    last_validated_at: Option<String>,
    recovery_actions: Vec<LiveProviderRecoveryAction>,
) -> LiveProviderAuth {
    LiveProviderAuth {
        login_method,
        credential_backend,
        auth_mode,
        is_authenticated,
        is_refreshable,
        is_source_compatible,
        requires_relogin,
        managed_restriction,
        diagnostic_code,
        failure_reason,
        last_validated_at,
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

fn claude_cloud_provider(env: &[(String, String)]) -> Option<&'static str> {
    let has_aws = env
        .iter()
        .any(|(key, value)| key.starts_with("AWS_") && !value.trim().is_empty());
    if has_aws {
        return Some("bedrock");
    }
    let has_vertex = env.iter().any(|(key, value)| {
        (key.starts_with("GOOGLE_") || key.starts_with("VERTEX_")) && !value.trim().is_empty()
    });
    if has_vertex {
        return Some("vertex");
    }
    None
}

#[cfg(target_os = "macos")]
fn load_credentials_from_keychain() -> (Option<OAuthCredentials>, KeychainStatus) {
    for service in KEYCHAIN_SERVICE_CANDIDATES {
        let output = std::process::Command::new("/usr/bin/security")
            .args(["find-generic-password", "-s", service, "-w"])
            .output();
        match output {
            Ok(output) if output.status.success() => {
                let secret = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if let Some(creds) = load_credentials_from_str(&secret) {
                    return (Some(creds), KeychainStatus::Available);
                }
                return (None, KeychainStatus::Error);
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr).to_lowercase();
                if stderr.contains("could not be found") || stderr.contains("item not found") {
                    continue;
                }
                if stderr.contains("interaction is not allowed")
                    || stderr.contains("user interaction is not allowed")
                    || stderr.contains("user canceled")
                {
                    return (None, KeychainStatus::Locked);
                }
                return (None, KeychainStatus::Error);
            }
            Err(_) => return (None, KeychainStatus::Error),
        }
    }
    (None, KeychainStatus::Missing)
}

#[cfg(not(target_os = "macos"))]
fn load_credentials_from_keychain() -> (Option<OAuthCredentials>, KeychainStatus) {
    (None, KeychainStatus::Missing)
}

fn load_settings_facts(path: &Path) -> ClaudeSettingsFacts {
    let Ok(contents) = std::fs::read_to_string(path) else {
        return ClaudeSettingsFacts::default();
    };
    let Ok(json) = serde_json::from_str::<Value>(&contents) else {
        return ClaudeSettingsFacts::default();
    };
    ClaudeSettingsFacts {
        api_key_helper: json_contains_key(&json, "apiKeyHelper"),
    }
}

fn json_contains_key(value: &Value, needle: &str) -> bool {
    match value {
        Value::Object(map) => map
            .iter()
            .any(|(key, value)| key == needle || json_contains_key(value, needle)),
        Value::Array(items) => items.iter().any(|value| json_contains_key(value, needle)),
        _ => false,
    }
}

/// Check if the access token is still valid (not expired).
pub fn is_token_valid(creds: &OAuthCredentials) -> bool {
    let Some(token) = &creds.access_token else {
        return false;
    };
    if token.is_empty() {
        return false;
    }
    let Some(expires_at_ms) = creds.expires_at else {
        return true; // No expiration info, assume valid
    };
    let now_ms = chrono::Utc::now().timestamp_millis();
    now_ms < expires_at_ms
}

/// Extract the access token if valid.
pub fn get_access_token(creds: &OAuthCredentials) -> Option<&str> {
    if is_token_valid(creds) {
        creds.access_token.as_deref()
    } else {
        None
    }
}

/// Extract identity (plan, tier) from credentials.
pub fn get_identity(creds: &OAuthCredentials) -> Identity {
    let plan = creds.rate_limit_tier.as_deref().and_then(Plan::from_tier);
    Identity {
        plan,
        rate_limit_tier: creds.rate_limit_tier.clone(),
    }
}

/// Refresh the OAuth access token using the refresh_token grant.
/// On success, persists updated credentials only to the original secure store and
/// returns the new access token. Returns None on any failure (network, invalid_grant, I/O).
pub async fn refresh_token(
    creds: &OAuthCredentials,
    credential_store: Option<&ClaudeCredentialStore>,
) -> Option<String> {
    refresh_token_with_store(creds, credential_store).await
}

pub async fn refresh_token_to(creds: &OAuthCredentials, creds_path: &Path) -> Option<String> {
    let credential_store = ClaudeCredentialStore::File(creds_path.to_path_buf());
    refresh_token_with_store(creds, Some(&credential_store)).await
}

async fn refresh_token_with_store(
    creds: &OAuthCredentials,
    credential_store: Option<&ClaudeCredentialStore>,
) -> Option<String> {
    let refresh_tok = creds.refresh_token.as_deref()?;

    let client = reqwest::Client::builder()
        .timeout(REFRESH_TIMEOUT)
        .build()
        .ok()?;

    let resp = client
        .post(REFRESH_ENDPOINT)
        .form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_tok),
            ("client_id", CLIENT_ID),
        ])
        .send()
        .await
        .map_err(|e| warn!("Token refresh request failed: {}", e))
        .ok()?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        if body.contains("invalid_grant") {
            warn!("Token refresh returned invalid_grant; not retrying");
        } else {
            warn!("Token refresh failed: {}", body);
        }
        return None;
    }

    let body: Value = resp
        .json()
        .await
        .map_err(|e| warn!("Token refresh response parse failed: {}", e))
        .ok()?;

    let new_access_token = body["access_token"].as_str()?.to_string();
    let new_refresh_token = body["refresh_token"].as_str().map(String::from);
    let expires_in_secs = body["expires_in"].as_i64().unwrap_or(3600);
    let new_expires_at = chrono::Utc::now().timestamp_millis() + (expires_in_secs * 1000);

    if let Err(e) = persist_refreshed_credentials(
        credential_store,
        &new_access_token,
        new_refresh_token.as_deref(),
        new_expires_at,
    ) {
        warn!("Failed to persist refreshed credentials: {}", e);
        // Still return the token -- it's valid even if we couldn't persist it
    }

    info!("OAuth token refreshed successfully");
    Some(new_access_token)
}

/// Read the credentials file, update the OAuth fields, and write it back.
/// Preserves any other top-level keys in the JSON.
fn write_refreshed_credentials(
    path: &Path,
    access_token: &str,
    refresh_token: Option<&str>,
    expires_at: i64,
) -> std::io::Result<()> {
    let contents = std::fs::read_to_string(path).unwrap_or_else(|_| "{}".into());
    let serialized =
        build_refreshed_credentials_json(Some(&contents), access_token, refresh_token, expires_at)?;
    std::fs::write(path, serialized)
}

fn persist_refreshed_credentials(
    credential_store: Option<&ClaudeCredentialStore>,
    access_token: &str,
    refresh_token: Option<&str>,
    expires_at: i64,
) -> std::io::Result<()> {
    match credential_store {
        Some(ClaudeCredentialStore::File(path)) => {
            write_refreshed_credentials(path, access_token, refresh_token, expires_at)
        }
        #[cfg(target_os = "macos")]
        Some(ClaudeCredentialStore::Keychain) => {
            write_refreshed_credentials_to_keychain(access_token, refresh_token, expires_at)
        }
        #[cfg(not(target_os = "macos"))]
        Some(ClaudeCredentialStore::Keychain) => {
            debug!(
                "Keychain-backed Claude credential refresh persistence not supported on this platform"
            );
            Ok(())
        }
        None => Ok(()),
    }
}

/// Locate the existing Claude Keychain entry and rewrite it with refreshed
/// credentials. We respect the user's original backend — when creds were read
/// from Keychain we write them back to Keychain, not to `~/.claude/.credentials.json`.
#[cfg(target_os = "macos")]
fn write_refreshed_credentials_to_keychain(
    access_token: &str,
    refresh_token: Option<&str>,
    expires_at: i64,
) -> std::io::Result<()> {
    for service in KEYCHAIN_SERVICE_CANDIDATES {
        let Some((existing_secret, account)) = read_keychain_entry_meta(service) else {
            continue;
        };
        let new_contents = build_refreshed_credentials_json(
            Some(&existing_secret),
            access_token,
            refresh_token,
            expires_at,
        )?;
        let mut cmd = std::process::Command::new("/usr/bin/security");
        cmd.args(["add-generic-password", "-U", "-s", service]);
        if let Some(account) = account.as_deref() {
            cmd.args(["-a", account]);
        }
        cmd.args(["-w", &new_contents]);
        let output = cmd
            .output()
            .map_err(|e| std::io::Error::other(format!("spawn security: {e}")))?;
        if output.status.success() {
            debug!("Persisted refreshed Claude credentials to Keychain service '{service}'");
            return Ok(());
        }
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(std::io::Error::other(format!(
            "security add-generic-password -U (service '{service}') failed: {stderr}"
        )));
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "No matching Claude Keychain entry found to update",
    ))
}

/// Pull the raw secret and the `acct` attribute from a generic-password
/// Keychain entry. Returns None when the entry doesn't exist or can't be read.
#[cfg(target_os = "macos")]
fn read_keychain_entry_meta(service: &str) -> Option<(String, Option<String>)> {
    let attrs = std::process::Command::new("/usr/bin/security")
        .args(["find-generic-password", "-s", service])
        .output()
        .ok()?;
    if !attrs.status.success() {
        return None;
    }
    let account = parse_account_from_security_output(&String::from_utf8_lossy(&attrs.stdout));

    let pw = std::process::Command::new("/usr/bin/security")
        .args(["find-generic-password", "-s", service, "-w"])
        .output()
        .ok()?;
    if !pw.status.success() {
        return None;
    }
    let secret = String::from_utf8_lossy(&pw.stdout).trim().to_string();
    Some((secret, account))
}

/// Parse the `acct` attribute from `security find-generic-password` output,
/// which prints lines like `"acct"<blob>="<value>"`.
#[cfg(target_os = "macos")]
fn parse_account_from_security_output(output: &str) -> Option<String> {
    for line in output.lines() {
        let trimmed = line.trim();
        let Some(rest) = trimmed.strip_prefix("\"acct\"<blob>=") else {
            continue;
        };
        // Value is either `"..."` or `0x...` (hex) or `<NULL>`.
        if rest == "<NULL>" {
            return None;
        }
        if let Some(quoted) = rest.strip_prefix('"') {
            if let Some(end) = quoted.find('"') {
                return Some(quoted[..end].to_string());
            }
        }
    }
    None
}

fn build_refreshed_credentials_json(
    contents: Option<&str>,
    access_token: &str,
    refresh_token: Option<&str>,
    expires_at: i64,
) -> std::io::Result<String> {
    let mut root: Value = contents
        .and_then(|value| serde_json::from_str(value).ok())
        .unwrap_or_else(|| Value::Object(Default::default()));

    if root.get("claudeAiOauth").is_some() {
        let oauth = root
            .as_object_mut()
            .and_then(|m| m.get_mut("claudeAiOauth"))
            .and_then(|v| v.as_object_mut());

        if let Some(oauth) = oauth {
            oauth.insert(
                "accessToken".into(),
                Value::String(access_token.to_string()),
            );
            oauth.insert("expiresAt".into(), Value::Number(expires_at.into()));
            if let Some(rt) = refresh_token {
                oauth.insert("refreshToken".into(), Value::String(rt.to_string()));
            }
        }
    } else if let Some(root_object) = root.as_object_mut() {
        root_object.insert(
            "accessToken".into(),
            Value::String(access_token.to_string()),
        );
        root_object.insert("expiresAt".into(), Value::Number(expires_at.into()));
        if let Some(rt) = refresh_token {
            root_object.insert("refreshToken".into(), Value::String(rt.to_string()));
        }
    }

    serde_json::to_string_pretty(&root).map_err(std::io::Error::other)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_missing_file() {
        let result = load_credentials_from(Path::new("/nonexistent/creds.json"));
        assert!(result.is_none());
    }

    #[test]
    fn test_empty_file() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("creds.json");
        std::fs::File::create(&path).unwrap();
        let result = load_credentials_from(&path);
        assert!(result.is_none());
    }

    #[test]
    fn test_valid_credentials() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("creds.json");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(
            f,
            r#"{{
                "claudeAiOauth": {{
                    "accessToken": "tok_abc",
                    "refreshToken": "ref_xyz",
                    "expiresAt": {},
                    "rateLimitTier": "claude_max"
                }}
            }}"#,
            chrono::Utc::now().timestamp_millis() + 3_600_000 // 1 hour from now
        )
        .unwrap();

        let creds = load_credentials_from(&path).unwrap();
        assert_eq!(creds.access_token.as_deref(), Some("tok_abc"));
        assert!(is_token_valid(&creds));
        assert_eq!(get_access_token(&creds), Some("tok_abc"));

        let identity = get_identity(&creds);
        assert_eq!(identity.plan, Some(Plan::Max));
    }

    #[test]
    fn test_expired_token() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("creds.json");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(
            f,
            r#"{{
                "claudeAiOauth": {{
                    "accessToken": "tok_old",
                    "expiresAt": 1000000000000
                }}
            }}"#
        )
        .unwrap();

        let creds = load_credentials_from(&path).unwrap();
        assert!(!is_token_valid(&creds));
        assert!(get_access_token(&creds).is_none());
    }

    #[test]
    fn test_no_oauth_section() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("creds.json");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, r#"{{"someOtherKey": {{}}}}"#).unwrap();

        let result = load_credentials_from(&path);
        assert!(result.is_none());
    }

    #[test]
    fn test_identity_pro() {
        let creds = OAuthCredentials {
            access_token: Some("tok".into()),
            refresh_token: None,
            expires_at: None,
            scopes: None,
            rate_limit_tier: Some("claude_pro".into()),
        };
        let identity = get_identity(&creds);
        assert_eq!(identity.plan, Some(Plan::Pro));
        assert_eq!(identity.rate_limit_tier.as_deref(), Some("claude_pro"));
    }

    #[test]
    fn test_identity_unknown_tier() {
        let creds = OAuthCredentials {
            access_token: Some("tok".into()),
            refresh_token: None,
            expires_at: None,
            scopes: None,
            rate_limit_tier: Some("free".into()),
        };
        let identity = get_identity(&creds);
        assert_eq!(identity.plan, None);
    }

    #[test]
    fn test_write_refreshed_credentials_updates_fields() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("creds.json");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(
            f,
            r#"{{
                "claudeAiOauth": {{
                    "accessToken": "old_tok",
                    "refreshToken": "old_ref",
                    "expiresAt": 1000000000000,
                    "rateLimitTier": "claude_pro"
                }}
            }}"#
        )
        .unwrap();

        let new_expires = chrono::Utc::now().timestamp_millis() + 3_600_000;
        write_refreshed_credentials(&path, "new_tok", Some("new_ref"), new_expires).unwrap();

        let creds = load_credentials_from(&path).unwrap();
        assert_eq!(creds.access_token.as_deref(), Some("new_tok"));
        assert_eq!(creds.refresh_token.as_deref(), Some("new_ref"));
        assert_eq!(creds.expires_at, Some(new_expires));
        // Preserved field
        assert_eq!(creds.rate_limit_tier.as_deref(), Some("claude_pro"));
    }

    #[test]
    fn test_write_refreshed_credentials_no_new_refresh_token() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("creds.json");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(
            f,
            r#"{{
                "claudeAiOauth": {{
                    "accessToken": "old_tok",
                    "refreshToken": "keep_this",
                    "expiresAt": 1000000000000
                }}
            }}"#
        )
        .unwrap();

        let new_expires = chrono::Utc::now().timestamp_millis() + 3_600_000;
        write_refreshed_credentials(&path, "new_tok", None, new_expires).unwrap();

        let creds = load_credentials_from(&path).unwrap();
        assert_eq!(creds.access_token.as_deref(), Some("new_tok"));
        // Original refresh token preserved when no new one provided
        assert_eq!(creds.refresh_token.as_deref(), Some("keep_this"));
        assert_eq!(creds.expires_at, Some(new_expires));
    }

    #[test]
    fn test_write_refreshed_credentials_preserves_other_keys() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("creds.json");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(
            f,
            r#"{{
                "claudeAiOauth": {{
                    "accessToken": "old_tok",
                    "expiresAt": 1000000000000
                }},
                "someOtherKey": "preserved"
            }}"#
        )
        .unwrap();

        let new_expires = chrono::Utc::now().timestamp_millis() + 3_600_000;
        write_refreshed_credentials(&path, "new_tok", Some("new_ref"), new_expires).unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        let root: serde_json::Value = serde_json::from_str(&contents).unwrap();
        assert_eq!(root["someOtherKey"], "preserved");
        assert_eq!(root["claudeAiOauth"]["accessToken"], "new_tok");
    }

    #[test]
    fn test_build_refreshed_credentials_json_updates_flat_shape() {
        let json = build_refreshed_credentials_json(
            Some(
                r#"{
                    "accessToken": "old_tok",
                    "refreshToken": "old_ref",
                    "expiresAt": 100
                }"#,
            ),
            "new_tok",
            Some("new_ref"),
            200,
        )
        .unwrap();

        let creds = load_credentials_from_str(&json).unwrap();
        assert_eq!(creds.access_token.as_deref(), Some("new_tok"));
        assert_eq!(creds.refresh_token.as_deref(), Some("new_ref"));
        assert_eq!(creds.expires_at, Some(200));
    }

    #[test]
    fn test_persist_refreshed_credentials_noop_when_no_store() {
        // None store: should succeed without touching the filesystem.
        persist_refreshed_credentials(None, "new_tok", Some("new_ref"), 200).unwrap();
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn parse_account_from_security_output_extracts_quoted_account() {
        let sample = "\
            keychain: \"/Users/test/Library/Keychains/login.keychain-db\"\n\
            version: 512\n\
            class: \"genp\"\n\
            attributes:\n\
                0x00000007 <blob>=\"Claude Code-credentials\"\n\
                0x00000008 <blob>=<NULL>\n\
                \"acct\"<blob>=\"npochaev\"\n\
                \"cdat\"<timedate>=0x20260421\n\
        ";
        assert_eq!(
            parse_account_from_security_output(sample),
            Some("npochaev".to_string())
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn parse_account_from_security_output_handles_null() {
        let sample = "    \"acct\"<blob>=<NULL>\n";
        assert_eq!(parse_account_from_security_output(sample), None);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn parse_account_from_security_output_handles_missing() {
        let sample = "class: \"genp\"\n";
        assert_eq!(parse_account_from_security_output(sample), None);
    }

    #[test]
    fn resolve_auth_prefers_env_override_over_file_credentials() {
        let tmp = TempDir::new().unwrap();
        let config_dir = tmp.path().join(".claude");
        std::fs::create_dir_all(&config_dir).unwrap();
        let path = config_dir.join(".credentials.json");
        std::fs::write(
            &path,
            format!(
                r#"{{
                    "claudeAiOauth": {{
                        "accessToken": "tok_abc",
                        "refreshToken": "ref_xyz",
                        "expiresAt": {},
                        "rateLimitTier": "claude_max"
                    }}
                }}"#,
                chrono::Utc::now().timestamp_millis() + 3_600_000
            ),
        )
        .unwrap();

        let env = vec![
            (
                "CLAUDE_CONFIG_DIR".to_string(),
                config_dir.display().to_string(),
            ),
            ("ANTHROPIC_API_KEY".to_string(), "sk-test".to_string()),
        ];
        let resolved = resolve_auth(&env);

        assert!(resolved.credentials.is_none());
        assert_eq!(
            resolved.health.diagnostic_code.as_deref(),
            Some("env-override")
        );
        assert_eq!(resolved.health.login_method.as_deref(), Some("api-key"));
        assert!(!resolved.health.is_source_compatible);
    }

    #[test]
    fn resolve_auth_detects_api_key_helper_masking_subscription_oauth() {
        let tmp = TempDir::new().unwrap();
        let config_dir = tmp.path().join(".claude");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(
            config_dir.join("settings.json"),
            r#"{"apiKeyHelper":"op read op://example/anthropic"}"#,
        )
        .unwrap();

        let env = vec![(
            "CLAUDE_CONFIG_DIR".to_string(),
            config_dir.display().to_string(),
        )];
        let resolved = resolve_auth(&env);

        assert_eq!(
            resolved.health.failure_reason.as_deref(),
            Some("Claude settings enable apiKeyHelper, which masks subscription OAuth.")
        );
        assert_eq!(
            resolved.health.diagnostic_code.as_deref(),
            Some("env-override")
        );
        assert_eq!(
            resolved.health.credential_backend.as_deref(),
            Some("config")
        );
    }

    #[test]
    fn test_empty_access_token_invalid() {
        let creds = OAuthCredentials {
            access_token: Some("".into()),
            refresh_token: None,
            expires_at: None,
            scopes: None,
            rate_limit_tier: None,
        };
        assert!(!is_token_valid(&creds));
        assert!(get_access_token(&creds).is_none());
    }

    #[test]
    fn test_token_no_expiration_treated_valid() {
        let creds = OAuthCredentials {
            access_token: Some("tok_valid".into()),
            refresh_token: None,
            expires_at: None, // No expiration info
            scopes: None,
            rate_limit_tier: None,
        };
        assert!(is_token_valid(&creds));
        assert_eq!(get_access_token(&creds), Some("tok_valid"));
    }

    #[test]
    fn test_identity_case_variations() {
        // Test various case patterns for plan detection
        for (tier, expected_plan) in [
            ("claude_MAX", Some(Plan::Max)),
            ("Pro", Some(Plan::Pro)),
            ("TEAM_enterprise", Some(Plan::Team)), // "team" matches first
            ("ENTERPRISE", Some(Plan::Enterprise)),
        ] {
            let creds = OAuthCredentials {
                access_token: None,
                refresh_token: None,
                expires_at: None,
                scopes: None,
                rate_limit_tier: Some(tier.into()),
            };
            let identity = get_identity(&creds);
            assert_eq!(identity.plan, expected_plan, "Failed for tier: {}", tier);
        }
    }

    #[test]
    fn test_null_access_token() {
        let creds = OAuthCredentials {
            access_token: None,
            refresh_token: None,
            expires_at: Some(chrono::Utc::now().timestamp_millis() + 3_600_000),
            scopes: None,
            rate_limit_tier: None,
        };
        assert!(!is_token_valid(&creds));
        assert!(get_access_token(&creds).is_none());
    }
}
