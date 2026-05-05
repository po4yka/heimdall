use anyhow::{Context, Result};
use chrono::Utc;
use semver::Version;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

const DEFAULT_INTERVAL_S: u64 = 6 * 60 * 60; // 6h
const FIRST_POLL_DELAY_S: u64 = 5;
const REQUEST_TIMEOUT_S: u64 = 8;
const ERROR_BACKOFF_S: u64 = 60;
const DEFAULT_REPO: &str = "po4yka/heimdall";
const USER_AGENT: &str = concat!("heimdall/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VersionInfo {
    pub current: String,
    pub latest: Option<String>,
    pub latest_url: Option<String>,
    pub latest_name: Option<String>,
    pub published_at: Option<String>,
    pub last_checked_at: Option<String>,
    pub next_check_at: Option<String>,
    pub last_error: Option<String>,
    pub update_available: bool,
}

impl Default for VersionInfo {
    fn default() -> Self {
        Self {
            current: env!("CARGO_PKG_VERSION").to_string(),
            latest: None,
            latest_url: None,
            latest_name: None,
            published_at: None,
            last_checked_at: None,
            next_check_at: None,
            last_error: None,
            update_available: false,
        }
    }
}

pub type VersionCache = Arc<RwLock<VersionInfo>>;

pub fn new_cache() -> VersionCache {
    Arc::new(RwLock::new(VersionInfo::default()))
}

pub fn spawn_version_check_loop(cache: VersionCache) -> JoinHandle<()> {
    tokio::spawn(async move {
        if std::env::var("HEIMDALL_VERSION_CHECK_DISABLED").is_ok() {
            info!("version check disabled via HEIMDALL_VERSION_CHECK_DISABLED");
            return;
        }
        let interval_s = interval_secs();
        let repo =
            std::env::var("HEIMDALL_GITHUB_REPO").unwrap_or_else(|_| DEFAULT_REPO.to_string());
        let url = format!("https://api.github.com/repos/{repo}/releases/latest");

        tokio::time::sleep(Duration::from_secs(FIRST_POLL_DELAY_S)).await;
        loop {
            match poll_once(&cache, &url, interval_s).await {
                Ok(()) => {
                    debug!(interval_s, "version check OK; sleeping {interval_s}s");
                    tokio::time::sleep(Duration::from_secs(interval_s)).await;
                }
                Err(e) => {
                    warn!("version check failed: {e:#}");
                    let mut w = cache.write().await;
                    w.last_error = Some(format!("{e:#}"));
                    w.last_checked_at = Some(Utc::now().to_rfc3339());
                    w.next_check_at = Some(
                        (Utc::now() + chrono::Duration::seconds(ERROR_BACKOFF_S as i64))
                            .to_rfc3339(),
                    );
                    drop(w);
                    tokio::time::sleep(Duration::from_secs(ERROR_BACKOFF_S)).await;
                }
            }
        }
    })
}

fn interval_secs() -> u64 {
    std::env::var("HEIMDALL_VERSION_CHECK_INTERVAL_S")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(DEFAULT_INTERVAL_S)
}

async fn poll_once(cache: &VersionCache, url: &str, interval_s: u64) -> Result<()> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_S))
        .user_agent(USER_AGENT)
        .build()
        .context("build reqwest client")?;

    let resp = client
        .get(url)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .context("send request")?;

    let status = resp.status();
    if status == reqwest::StatusCode::NOT_FOUND {
        // 404 from the releases endpoint means the repo has no releases (or
        // doesn't exist). That's a stable state, not a transient failure —
        // record it on the cache and let the loop sleep the regular interval
        // instead of warn-spamming every 60s.
        let now = Utc::now();
        let mut w = cache.write().await;
        w.last_checked_at = Some(now.to_rfc3339());
        w.next_check_at = Some((now + chrono::Duration::seconds(interval_s as i64)).to_rfc3339());
        w.last_error = Some("no releases published for this repo".to_string());
        w.latest = None;
        w.latest_url = None;
        w.latest_name = None;
        w.published_at = None;
        w.update_available = false;
        debug!("version check: GitHub returned 404 (no releases); next check in {interval_s}s");
        return Ok(());
    }
    if !status.is_success() {
        anyhow::bail!("GitHub Releases API returned {status}");
    }

    #[derive(serde::Deserialize)]
    struct GhRelease {
        tag_name: String,
        name: Option<String>,
        html_url: Option<String>,
        published_at: Option<String>,
    }

    let release: GhRelease = resp.json().await.context("parse GitHub release JSON")?;

    let tag = release.tag_name.trim_start_matches('v').to_string();
    let current = env!("CARGO_PKG_VERSION").to_string();
    let update_available = match (Version::parse(&current), Version::parse(&tag)) {
        (Ok(c), Ok(l)) => l > c,
        _ => false,
    };

    let now = Utc::now();
    let mut w = cache.write().await;
    w.current = current;
    w.latest = Some(tag);
    w.latest_url = release.html_url;
    w.latest_name = release.name;
    w.published_at = release.published_at;
    w.last_checked_at = Some(now.to_rfc3339());
    w.next_check_at = Some((now + chrono::Duration::seconds(interval_s as i64)).to_rfc3339());
    w.last_error = None;
    w.update_available = update_available;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read as _, Write as _};
    use std::net::TcpListener;

    #[tokio::test]
    async fn version_cache_default_has_current() {
        let cache = new_cache();
        let info = cache.read().await;
        assert!(!info.current.is_empty());
        assert!(!info.update_available);
        assert!(info.latest.is_none());
    }

    #[tokio::test]
    async fn poll_once_parses_release_json() {
        // Spin up a minimal TCP server returning a canned GitHub releases response.
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("local_addr");
        let url = format!("http://{addr}/repos/test/test/releases/latest");

        let server_handle = tokio::task::spawn_blocking(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            // Read request (discard).
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let body = r#"{"tag_name":"v9.9.9","name":"Test Release","html_url":"https://example.com/release","published_at":"2099-01-01T00:00:00Z"}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).expect("write");
        });

        let cache = new_cache();
        poll_once(&cache, &url, 21600).await.expect("poll_once");
        server_handle.await.expect("server");

        let info = cache.read().await;
        assert_eq!(info.latest.as_deref(), Some("9.9.9"));
        assert!(info.update_available, "9.9.9 > 0.1.0");
        assert!(info.last_error.is_none());
        assert!(info.last_checked_at.is_some());
        assert_eq!(
            info.latest_url.as_deref(),
            Some("https://example.com/release")
        );
    }

    #[tokio::test]
    async fn poll_once_treats_404_as_no_releases() {
        // 404 means the repo has no releases (or doesn't exist). poll_once
        // must succeed and record that fact on the cache so the loop sleeps
        // the regular interval instead of warn-spamming every 60s.
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("local_addr");
        let url = format!("http://{addr}/repos/test/test/releases/latest");

        let server_handle = tokio::task::spawn_blocking(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let response = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
            stream.write_all(response.as_bytes()).expect("write");
        });

        let cache = new_cache();
        let result = poll_once(&cache, &url, 21600).await;
        server_handle.await.expect("server");
        assert!(result.is_ok(), "404 must be treated as a successful poll");
        let info = cache.read().await;
        assert_eq!(
            info.last_error.as_deref(),
            Some("no releases published for this repo")
        );
        assert!(info.latest.is_none());
        assert!(!info.update_available);
        assert!(info.last_checked_at.is_some());
        assert!(info.next_check_at.is_some());
    }

    #[tokio::test]
    async fn poll_once_errors_on_non_404_failure() {
        // 5xx is still treated as a transient failure — the loop should warn
        // and back off briefly to retry.
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("local_addr");
        let url = format!("http://{addr}/repos/test/test/releases/latest");

        let server_handle = tokio::task::spawn_blocking(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let response = "HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\n\r\n";
            stream.write_all(response.as_bytes()).expect("write");
        });

        let cache = new_cache();
        let result = poll_once(&cache, &url, 21600).await;
        server_handle.await.expect("server");
        assert!(result.is_err(), "503 must surface as a transient error");
    }
}
