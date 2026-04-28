//! chatgpt.com private API client.
//!
//! Endpoints (all under `https://chatgpt.com`):
//!   GET /backend-api/conversations?offset=0&limit=28&order=updated
//!   GET /backend-api/conversation/{conv_id}
//!
//! Auth: Bearer access-token from `/api/auth/session` plus the
//! `__Secure-next-auth.session-token` cookie. Cloudflare cookies
//! (`cf_clearance`) and User-Agent matching the issuing browser are
//! also required.

use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::header::{AUTHORIZATION, COOKIE, HeaderMap, HeaderValue, USER_AGENT};
use serde::Deserialize;
use serde_json::Value;

const BASE: &str = "https://chatgpt.com";

pub struct Credentials {
    pub session_token: String, // __Secure-next-auth.session-token cookie value
    pub access_token: String,  // Bearer ... from /api/auth/session
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
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", creds.access_token))?,
        );
        let mut cookie = format!("__Secure-next-auth.session-token={}", creds.session_token);
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

    pub async fn list_conversations(&self, page_size: usize) -> Result<Vec<ConversationItem>> {
        let mut all = Vec::new();
        let mut offset = 0usize;
        loop {
            let url = format!(
                "{BASE}/backend-api/conversations?offset={offset}&limit={page_size}&order=updated"
            );
            let resp = self
                .http
                .get(&url)
                .send()
                .await
                .with_context(|| format!("GET {url}"))?;
            check_status(&resp)?;
            let page: ConversationsPage =
                resp.json().await.context("parsing conversations page")?;
            let got = page.items.len();
            all.extend(page.items);
            offset += got;
            if got < page_size {
                break;
            }
            if offset >= page.total.unwrap_or(usize::MAX) {
                break;
            }
        }
        Ok(all)
    }

    pub async fn fetch_conversation(&self, conv_id: &str) -> Result<Value> {
        let url = format!("{BASE}/backend-api/conversation/{conv_id}");
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
pub struct ConversationsPage {
    pub items: Vec<ConversationItem>,
    pub total: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct ConversationItem {
    pub id: String,
    pub title: Option<String>,
    pub update_time: Option<f64>,
}

fn check_status(resp: &reqwest::Response) -> Result<()> {
    let status = resp.status();
    if status.is_success() {
        return Ok(());
    }
    if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
        anyhow::bail!(
            "chatgpt.com returned {status} — session-token/access-token/cf_clearance \
             likely expired. Re-copy from your browser DevTools and retry."
        );
    }
    anyhow::bail!("chatgpt.com returned {status}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_builds_with_minimal_credentials() {
        let creds = Credentials {
            session_token: "sess".into(),
            access_token: "tok".into(),
            cf_clearance: None,
            user_agent: "Mozilla/5.0".into(),
        };
        assert!(Client::new(&creds).is_ok());
    }
}
