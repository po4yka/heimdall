//! claude.ai private API client.
//!
//! Endpoints (all under `https://claude.ai`):
//!   GET /api/organizations
//!   GET /api/organizations/{org_id}/chat_conversations
//!   GET /api/organizations/{org_id}/chat_conversations/{conv_id}
//!
//! Auth: `sessionKey` cookie (`sk-ant-sid01...`) plus a fresh `cf_clearance`
//! cookie bound to the same User-Agent the user's browser had when the
//! cookie was issued. We do not solve Cloudflare challenges; we surface a
//! clear error when the cookies expire.

use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::header::{COOKIE, HeaderMap, HeaderValue, USER_AGENT};
use serde::Deserialize;
use serde_json::Value;

const BASE: &str = "https://claude.ai";

pub struct Credentials {
    pub session_key: String,
    pub cf_clearance: Option<String>,
    pub user_agent: String,
}

pub struct Client {
    http: reqwest::Client,
}

impl Client {
    pub fn new(creds: &Credentials) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_str(&creds.user_agent)?);
        let mut cookie = format!("sessionKey={}", creds.session_key);
        if let Some(cf) = &creds.cf_clearance {
            cookie.push_str(&format!("; cf_clearance={cf}"));
        }
        headers.insert(COOKIE, HeaderValue::from_str(&cookie)?);

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(30))
            .build()?;
        Ok(Self { http })
    }

    pub async fn list_organizations(&self) -> Result<Vec<Organization>> {
        let url = format!("{BASE}/api/organizations");
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .with_context(|| format!("GET {url}"))?;
        check_status(&resp)?;
        let orgs: Vec<Organization> = resp
            .json()
            .await
            .context("parsing organizations response")?;
        Ok(orgs)
    }

    pub async fn list_conversations(&self, org_id: &str) -> Result<Vec<ConversationSummary>> {
        let url = format!("{BASE}/api/organizations/{org_id}/chat_conversations");
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .with_context(|| format!("GET {url}"))?;
        check_status(&resp)?;
        let convs: Vec<ConversationSummary> = resp
            .json()
            .await
            .context("parsing chat_conversations response")?;
        Ok(convs)
    }

    pub async fn fetch_conversation(&self, org_id: &str, conv_id: &str) -> Result<Value> {
        let url = format!("{BASE}/api/organizations/{org_id}/chat_conversations/{conv_id}");
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .with_context(|| format!("GET {url}"))?;
        check_status(&resp)?;
        let value: Value = resp.json().await.context("parsing conversation response")?;
        Ok(value)
    }
}

#[derive(Debug, Deserialize)]
pub struct Organization {
    pub uuid: String,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ConversationSummary {
    pub uuid: String,
    pub name: Option<String>,
    pub updated_at: Option<String>,
}

fn check_status(resp: &reqwest::Response) -> Result<()> {
    let status = resp.status();
    if status.is_success() {
        return Ok(());
    }
    if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
        anyhow::bail!(
            "claude.ai returned {status} — sessionKey/cf_clearance likely expired. \
             Re-copy from your browser DevTools and retry."
        );
    }
    anyhow::bail!("claude.ai returned {status}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_builds_with_minimal_credentials() {
        let creds = Credentials {
            session_key: "sk-ant-sid01-test".into(),
            cf_clearance: None,
            user_agent: "Mozilla/5.0 (test)".into(),
        };
        assert!(Client::new(&creds).is_ok());
    }

    #[test]
    fn client_includes_cf_clearance_when_present() {
        let creds = Credentials {
            session_key: "sk-ant-sid01-test".into(),
            cf_clearance: Some("cf-abc".into()),
            user_agent: "Mozilla/5.0".into(),
        };
        assert!(Client::new(&creds).is_ok());
    }
}
