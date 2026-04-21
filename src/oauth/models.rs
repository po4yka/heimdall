use serde::{Deserialize, Serialize};

// ── Credential file types ──────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CredentialsFile {
    #[serde(rename = "claudeAiOauth")]
    pub claude_ai_oauth: Option<OAuthCredentials>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct OAuthCredentials {
    #[serde(rename = "accessToken")]
    pub access_token: Option<String>,
    #[serde(rename = "refreshToken")]
    pub refresh_token: Option<String>,
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<i64>, // milliseconds since epoch
    pub scopes: Option<Vec<String>>,
    #[serde(rename = "rateLimitTier")]
    pub rate_limit_tier: Option<String>,
}

// ── Plan detection ─────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Plan {
    Max,
    Pro,
    Team,
    Enterprise,
}

impl Plan {
    pub fn from_tier(tier: &str) -> Option<Self> {
        let t = tier.to_lowercase();
        if t.contains("max") {
            Some(Plan::Max)
        } else if t.contains("pro") {
            Some(Plan::Pro)
        } else if t.contains("team") {
            Some(Plan::Team)
        } else if t.contains("enterprise") {
            Some(Plan::Enterprise)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Identity {
    pub plan: Option<Plan>,
    pub rate_limit_tier: Option<String>,
}

// ── OAuth API response types ───────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct OAuthUsageResponse {
    pub five_hour: Option<UsageWindow>,
    pub seven_day: Option<UsageWindow>,
    pub seven_day_oauth_apps: Option<UsageWindow>,
    pub seven_day_opus: Option<UsageWindow>,
    pub seven_day_sonnet: Option<UsageWindow>,
    pub iguana_necktie: Option<UsageWindow>,
    pub extra_usage: Option<ExtraUsage>,
}

#[derive(Debug, Deserialize)]
pub struct UsageWindow {
    pub utilization: Option<f64>,
    pub resets_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ExtraUsage {
    pub is_enabled: Option<bool>,
    pub monthly_limit: Option<f64>,
    pub used_credits: Option<f64>,
    pub utilization: Option<f64>,
    pub currency: Option<String>,
}

// ── Serialized response for our API ────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct UsageWindowsResponse {
    pub available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<WindowInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weekly: Option<WindowInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weekly_opus: Option<WindowInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weekly_sonnet: Option<WindowInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget: Option<BudgetInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity: Option<Identity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WindowInfo {
    pub used_percent: f64,
    pub resets_at: Option<String>,
    pub resets_in_minutes: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BudgetInfo {
    pub used: f64,
    pub limit: f64,
    pub currency: String,
    pub utilization: f64,
}

impl UsageWindowsResponse {
    pub fn unavailable() -> Self {
        Self {
            available: false,
            session: None,
            weekly: None,
            weekly_opus: None,
            weekly_sonnet: None,
            budget: None,
            identity: None,
            error: None,
        }
    }

    pub fn with_error(msg: String) -> Self {
        Self {
            available: false,
            error: Some(msg),
            ..Self::unavailable()
        }
    }
}

impl WindowInfo {
    pub fn from_usage_window(w: &UsageWindow) -> Self {
        // Anthropic's /api/oauth/usage returns `utilization` as a 0-100
        // percentage, not a 0-1 fraction. Live API confirms values like 9.0
        // and 86.0. If we see a value <= 1.0 treat it as a legacy fraction
        // for safety so older API responses don't regress to 0.
        let raw = w.utilization.unwrap_or(0.0);
        let used_percent = if raw > 0.0 && raw <= 1.0 { raw * 100.0 } else { raw };
        let resets_in_minutes = w.resets_at.as_ref().and_then(|ts| {
            let reset = chrono::DateTime::parse_from_rfc3339(ts).ok()?;
            let now = chrono::Utc::now();
            let diff = reset.signed_duration_since(now);
            Some(diff.num_minutes().max(0))
        });
        Self {
            used_percent,
            resets_at: w.resets_at.clone(),
            resets_in_minutes,
        }
    }
}

impl BudgetInfo {
    pub fn from_extra_usage(e: &ExtraUsage) -> Option<Self> {
        if !e.is_enabled.unwrap_or(false) {
            return None;
        }
        let raw = e.utilization.unwrap_or(0.0);
        let utilization = if raw > 0.0 && raw <= 1.0 { raw * 100.0 } else { raw };
        Some(Self {
            used: e.used_credits.unwrap_or(0.0),
            limit: e.monthly_limit.unwrap_or(0.0),
            currency: e.currency.clone().unwrap_or_else(|| "USD".into()),
            utilization,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_from_tier() {
        assert_eq!(Plan::from_tier("claude_max"), Some(Plan::Max));
        assert_eq!(Plan::from_tier("claude_pro"), Some(Plan::Pro));
        assert_eq!(Plan::from_tier("Claude_Team"), Some(Plan::Team));
        assert_eq!(Plan::from_tier("enterprise"), Some(Plan::Enterprise));
        assert_eq!(Plan::from_tier("unknown"), None);
        assert_eq!(Plan::from_tier(""), None);
    }

    #[test]
    fn test_window_info_from_usage_window_percent_integer() {
        // Anthropic's live API returns utilization as a percentage 0-100.
        let w = UsageWindow {
            utilization: Some(86.0),
            resets_at: Some("2099-01-01T00:00:00Z".into()),
        };
        let info = WindowInfo::from_usage_window(&w);
        assert!((info.used_percent - 86.0).abs() < 0.01);
    }

    #[test]
    fn test_window_info_from_usage_window_legacy_fraction() {
        // Legacy fixtures used 0-1 fractions; we still treat those as percent.
        let w = UsageWindow {
            utilization: Some(0.45),
            resets_at: Some("2099-01-01T00:00:00Z".into()),
        };
        let info = WindowInfo::from_usage_window(&w);
        assert!((info.used_percent - 45.0).abs() < 0.01);
        assert!(info.resets_in_minutes.unwrap() > 0);
    }

    #[test]
    fn test_window_info_missing_fields() {
        let w = UsageWindow {
            utilization: None,
            resets_at: None,
        };
        let info = WindowInfo::from_usage_window(&w);
        assert!((info.used_percent - 0.0).abs() < 0.01);
        assert!(info.resets_in_minutes.is_none());
    }

    #[test]
    fn test_budget_info_disabled() {
        let e = ExtraUsage {
            is_enabled: Some(false),
            monthly_limit: Some(100.0),
            used_credits: Some(50.0),
            utilization: Some(0.5),
            currency: Some("USD".into()),
        };
        assert!(BudgetInfo::from_extra_usage(&e).is_none());
    }

    #[test]
    fn test_budget_info_enabled() {
        let e = ExtraUsage {
            is_enabled: Some(true),
            monthly_limit: Some(100.0),
            used_credits: Some(45.5),
            utilization: Some(0.455),
            currency: Some("USD".into()),
        };
        let b = BudgetInfo::from_extra_usage(&e).unwrap();
        assert!((b.used - 45.5).abs() < 0.01);
        assert!((b.limit - 100.0).abs() < 0.01);
        assert!((b.utilization - 45.5).abs() < 0.01);
    }

    #[test]
    fn test_parse_oauth_response() {
        let json = r#"{
            "five_hour": { "utilization": 0.45, "resets_at": "2026-04-09T15:00:00Z" },
            "seven_day": { "utilization": 0.6, "resets_at": "2026-04-16T00:00:00Z" },
            "extra_usage": {
                "is_enabled": true,
                "monthly_limit": 100.0,
                "used_credits": 45.5,
                "utilization": 0.455,
                "currency": "USD"
            }
        }"#;
        let resp: OAuthUsageResponse = serde_json::from_str(json).unwrap();
        assert!((resp.five_hour.unwrap().utilization.unwrap() - 0.45).abs() < 0.01);
        assert!(resp.extra_usage.unwrap().is_enabled.unwrap());
    }

    #[test]
    fn test_parse_credentials_file() {
        let json = r#"{
            "claudeAiOauth": {
                "accessToken": "tok_123",
                "refreshToken": "ref_456",
                "expiresAt": 1712688000000,
                "scopes": ["user:profile"],
                "rateLimitTier": "claude_pro"
            }
        }"#;
        let creds: CredentialsFile = serde_json::from_str(json).unwrap();
        let oauth = creds.claude_ai_oauth.unwrap();
        assert_eq!(oauth.access_token.unwrap(), "tok_123");
        assert_eq!(oauth.rate_limit_tier.unwrap(), "claude_pro");
    }

    #[test]
    fn test_window_info_past_reset() {
        let w = UsageWindow {
            utilization: Some(1.0),
            resets_at: Some("2020-01-01T00:00:00Z".into()), // far past
        };
        let info = WindowInfo::from_usage_window(&w);
        assert_eq!(info.resets_in_minutes, Some(0)); // clamped to 0
    }

    #[test]
    fn test_window_info_invalid_timestamp() {
        let w = UsageWindow {
            utilization: Some(0.5),
            resets_at: Some("not-a-date".into()),
        };
        let info = WindowInfo::from_usage_window(&w);
        assert!(info.resets_in_minutes.is_none());
    }

    #[test]
    fn test_budget_none_is_enabled() {
        let e = ExtraUsage {
            is_enabled: None, // None treated as false
            monthly_limit: Some(100.0),
            used_credits: Some(50.0),
            utilization: Some(0.5),
            currency: Some("USD".into()),
        };
        assert!(BudgetInfo::from_extra_usage(&e).is_none());
    }

    #[test]
    fn test_unavailable_response() {
        let r = UsageWindowsResponse::unavailable();
        assert!(!r.available);
        assert!(r.session.is_none());
        assert!(r.weekly.is_none());
        assert!(r.budget.is_none());
        assert!(r.identity.is_none());
        assert!(r.error.is_none());
    }
}
