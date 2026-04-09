use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::Value;
use tracing::{debug, info, warn};

use super::models::{CredentialsFile, Identity, OAuthCredentials, Plan};

const REFRESH_ENDPOINT: &str = "https://platform.claude.com/v1/oauth/token";
const CLIENT_ID: &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e";
const REFRESH_TIMEOUT: Duration = Duration::from_secs(30);

fn credentials_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
        .join(".credentials.json")
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
/// On success, writes updated credentials to disk and returns the new access token.
/// Returns None on any failure (network, invalid_grant, I/O).
pub async fn refresh_token(creds: &OAuthCredentials) -> Option<String> {
    refresh_token_to(creds, &credentials_path()).await
}

pub async fn refresh_token_to(creds: &OAuthCredentials, creds_path: &Path) -> Option<String> {
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

    if let Err(e) = write_refreshed_credentials(
        creds_path,
        &new_access_token,
        new_refresh_token.as_deref(),
        new_expires_at,
    ) {
        warn!("Failed to write refreshed credentials: {}", e);
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
    let mut root: Value =
        serde_json::from_str(&contents).unwrap_or_else(|_| Value::Object(Default::default()));

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

    let serialized = serde_json::to_string_pretty(&root).map_err(std::io::Error::other)?;
    std::fs::write(path, serialized)
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
