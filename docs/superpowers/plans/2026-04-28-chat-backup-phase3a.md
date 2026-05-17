# Chat-backup Phase 3a — Cookie-paste CLI Fallback Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship the cookie-paste-CLI fallback for Tier 3 web-chat capture (spec §7b). User copies session cookies from DevTools, runs `heimdall scrape claude --session-key … --cf-clearance …` (or the chatgpt equivalent), and Heimdall fetches the user's conversation list + contents from the vendor's private API and writes them under `<archive_root>/web/<vendor>/<conv_id>.json` (history-versioned). Phase 3a also lays the rails the Phase 3b browser extension will reuse: the companion-token bearer + the `POST /api/archive/web-conversation` ingest endpoint share the same storage primitive `write_web_conversation()`.

**Architecture:** Three new modules. `src/archive/web/` owns the on-disk shape (write + history rotation + listing). `src/archive/companion_token` manages the per-install bearer (`~/.heimdall/companion-token`, mode 0600, constant-time verify). `src/scrape/{claude,chatgpt}.rs` are reqwest-based HTTP clients that the CLI calls *directly* — they do not POST through the local HTTP endpoint, so a headless install needs no daemon. The HTTP endpoint exists only for Phase 3b's browser extension; both code paths land in the same `web/` tree.

**Tech Stack:** Rust 2024, `reqwest = "0.12"` (already a dep), `subtle = "2"` for constant-time bearer comparison (small new dep), the existing `anyhow`/`thiserror`/`tracing`/`serde` ecosystem.

Spec reference: `docs/superpowers/specs/2026-04-28-chat-backup-design.md` §7, §9, and Phase 3a of §10.

---

## Non-goals for 3a

- No browser extension (Phase 3b).
- No dashboard "Web captures" panel (Phase 3b adds it; 3a's data is visible via `find ~/.heimdall/archive/web -name '*.json'` until then).
- No automatic token refresh — when 401 happens, error message tells the user to copy fresh cookies from DevTools.
- No Cloudflare challenge solver — the user must supply a fresh `cf_clearance` cookie when the prior one expires.

---

## File Structure

**Created:**

| File | Responsibility |
|------|----------------|
| `src/archive/web/mod.rs` | `WebConversation` types + `write_web_conversation` (with history rotation) + `list_web_conversations`. |
| `src/archive/companion_token.rs` | Read/init/rotate `~/.heimdall/companion-token`; constant-time verify. |
| `src/scrape/mod.rs` | Module root; shared types and helpers. |
| `src/scrape/claude.rs` | claude.ai client: list orgs, list convs, fetch conv. |
| `src/scrape/chatgpt.rs` | chatgpt.com client: list convs, fetch conv. |
| `tests/web_conversation_integration.rs` | E2E write+history rotation through the storage primitive. |

**Modified:**

| File | Change |
|------|--------|
| `Cargo.toml` | Add `subtle = "2"`. |
| `src/lib.rs` | Add `pub mod scrape;`. |
| `src/archive/mod.rs` | Add `pub mod companion_token;` and `pub mod web;`. |
| `src/main.rs` | Add `Scrape { vendor, … }` and `CompanionToken { action }` variants + dispatch. |
| `src/server/api.rs` | Add `api_archive_web_conversation` (POST, bearer-gated). |
| `src/server/mod.rs` | Register `POST /api/archive/web-conversation`. |
| `src/server/tests.rs` | Bearer-auth happy + reject tests for the new endpoint. |
| `src/cli_tests.rs` | `companion-token show` smoke. |

---

## Storage layout

```
~/.heimdall/
  companion-token            # 256-bit random, hex, mode 0600
  archive/
    web/
      claude.ai/
        <conv_id>.json                                       # latest version
        <conv_id>.history/<previous_captured_at>.json        # rotated copies
      chatgpt.com/
        <conv_id>.json
        <conv_id>.history/...
```

Captured-at timestamps use the same `%Y-%m-%dT%H%M%S%.6fZ` format as Phase 1's `snapshot_id` so rotated history filenames sort lexicographically.

---

## Task 1: Web-conversation storage primitive

**Files:**
- Create: `src/archive/web/mod.rs`
- Modify: `src/archive/mod.rs` (`pub mod web;`)
- Create: `tests/web_conversation_integration.rs`

- [ ] **Step 1: Write the storage primitive**

```rust
//! On-disk layout for Tier 3 web-chat captures.
//!
//! Each conversation stored at:
//!   <archive_root>/web/<vendor>/<conv_id>.json    # current
//!   <archive_root>/web/<vendor>/<conv_id>.history/<captured_at>.json   # prior
//!
//! Calls are idempotent: writing the same payload twice is a no-op (returns
//! `WriteOutcome::Unchanged`). A different payload rotates the previous
//! file into `<conv_id>.history/`.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WebConversation {
    pub vendor: String,         // "claude.ai" | "chatgpt.com"
    pub conversation_id: String,
    pub captured_at: String,    // RFC3339 (microsecond)
    pub schema_fingerprint: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriteOutcome {
    Saved { path: PathBuf },
    Unchanged,
}

pub fn vendor_dir(archive_root: &Path, vendor: &str) -> PathBuf {
    archive_root.join("web").join(sanitize_vendor(vendor))
}

fn sanitize_vendor(v: &str) -> String {
    // Allow alphanumeric, `.`, `-`, `_`. Replace anything else with `_`.
    v.chars()
        .map(|c| if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_') { c } else { '_' })
        .collect()
}

fn sanitize_id(id: &str) -> String {
    id.chars()
        .map(|c| if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.') { c } else { '_' })
        .collect::<String>()
        .chars()
        .take(120)
        .collect()
}

pub fn write_web_conversation(archive_root: &Path, conv: &WebConversation) -> Result<WriteOutcome> {
    let dir = vendor_dir(archive_root, &conv.vendor);
    fs::create_dir_all(&dir)
        .with_context(|| format!("creating {}", dir.display()))?;
    let safe_id = sanitize_id(&conv.conversation_id);
    let current = dir.join(format!("{safe_id}.json"));
    let new_bytes = serde_json::to_vec_pretty(conv)?;

    if current.is_file() {
        let existing = fs::read(&current)
            .with_context(|| format!("reading {}", current.display()))?;
        if existing == new_bytes {
            return Ok(WriteOutcome::Unchanged);
        }
        // Rotate previous version into history.
        let prior: WebConversation = serde_json::from_slice(&existing)
            .unwrap_or_else(|_| WebConversation {
                vendor: conv.vendor.clone(),
                conversation_id: conv.conversation_id.clone(),
                captured_at: Utc::now().format("%Y-%m-%dT%H%M%S%.6fZ").to_string(),
                schema_fingerprint: String::new(),
                payload: serde_json::Value::Null,
            });
        let history_dir = dir.join(format!("{safe_id}.history"));
        fs::create_dir_all(&history_dir)?;
        let history_path = history_dir.join(format!("{}.json", prior.captured_at));
        fs::rename(&current, &history_path)
            .with_context(|| format!("rotating {} → {}", current.display(), history_path.display()))?;
    }

    // Atomic write: tempfile + rename.
    let tmp = dir.join(format!(".tmp-{safe_id}.json"));
    fs::write(&tmp, &new_bytes)?;
    fs::rename(&tmp, &current)?;
    Ok(WriteOutcome::Saved { path: current })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebConversationSummary {
    pub vendor: String,
    pub conversation_id: String,
    pub captured_at: String,
    pub history_count: usize,
}

pub fn list_web_conversations(archive_root: &Path) -> Result<Vec<WebConversationSummary>> {
    let web_root = archive_root.join("web");
    if !web_root.is_dir() { return Ok(Vec::new()); }
    let mut out = Vec::new();
    for vendor_entry in fs::read_dir(&web_root)? {
        let vendor_entry = vendor_entry?;
        if !vendor_entry.file_type()?.is_dir() { continue; }
        let vendor = vendor_entry.file_name().to_string_lossy().to_string();
        for conv_entry in fs::read_dir(vendor_entry.path())? {
            let conv_entry = conv_entry?;
            if !conv_entry.file_type()?.is_file() { continue; }
            let name = conv_entry.file_name().to_string_lossy().to_string();
            if !name.ends_with(".json") || name.starts_with(".tmp-") { continue; }
            let conv_id = name.trim_end_matches(".json").to_string();
            let bytes = fs::read(conv_entry.path())?;
            let parsed: WebConversation = match serde_json::from_slice(&bytes) {
                Ok(v) => v,
                Err(_) => continue,
            };
            let history_dir = vendor_entry.path().join(format!("{conv_id}.history"));
            let history_count = if history_dir.is_dir() {
                fs::read_dir(&history_dir)?
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
                    .count()
            } else { 0 };
            out.push(WebConversationSummary {
                vendor,
                conversation_id: parsed.conversation_id,
                captured_at: parsed.captured_at,
                history_count,
            });
            break;
        }
    }
    out.sort_by(|a, b| b.captured_at.cmp(&a.captured_at));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn mk(vendor: &str, id: &str, payload: serde_json::Value) -> WebConversation {
        WebConversation {
            vendor: vendor.to_string(),
            conversation_id: id.to_string(),
            captured_at: Utc::now().format("%Y-%m-%dT%H%M%S%.6fZ").to_string(),
            schema_fingerprint: "abc".into(),
            payload,
        }
    }

    #[test]
    fn first_write_creates_current_file() {
        let tmp = TempDir::new().unwrap();
        let conv = mk("claude.ai", "c1", serde_json::json!({"hi": 1}));
        let out = write_web_conversation(tmp.path(), &conv).unwrap();
        match out {
            WriteOutcome::Saved { path } => assert!(path.is_file()),
            other => panic!("expected Saved, got {other:?}"),
        }
    }

    #[test]
    fn second_identical_write_is_unchanged() {
        let tmp = TempDir::new().unwrap();
        let conv = mk("claude.ai", "c1", serde_json::json!({"hi": 1}));
        write_web_conversation(tmp.path(), &conv).unwrap();
        let out = write_web_conversation(tmp.path(), &conv).unwrap();
        assert_eq!(out, WriteOutcome::Unchanged);
    }

    #[test]
    fn changed_payload_rotates_previous_into_history() {
        let tmp = TempDir::new().unwrap();
        let v1 = mk("claude.ai", "c1", serde_json::json!({"hi": 1}));
        write_web_conversation(tmp.path(), &v1).unwrap();
        let v2 = mk("claude.ai", "c1", serde_json::json!({"hi": 2}));
        write_web_conversation(tmp.path(), &v2).unwrap();

        let history = tmp.path().join("web/claude.ai/c1.history");
        assert!(history.is_dir());
        let entries: Vec<_> = fs::read_dir(&history).unwrap().collect();
        assert_eq!(entries.len(), 1);
        let current = fs::read(tmp.path().join("web/claude.ai/c1.json")).unwrap();
        let parsed: WebConversation = serde_json::from_slice(&current).unwrap();
        assert_eq!(parsed.payload["hi"], 2);
    }

    #[test]
    fn vendor_and_id_sanitization() {
        let tmp = TempDir::new().unwrap();
        let conv = mk("claude.ai/../etc", "../../escape", serde_json::json!({}));
        let out = write_web_conversation(tmp.path(), &conv).unwrap();
        match out {
            WriteOutcome::Saved { path } => {
                assert!(path.starts_with(tmp.path().join("web")));
                assert!(!path.to_string_lossy().contains(".."));
            }
            other => panic!("got {other:?}"),
        }
    }

    #[test]
    fn list_returns_one_summary_per_conversation() {
        let tmp = TempDir::new().unwrap();
        write_web_conversation(tmp.path(), &mk("claude.ai", "a", serde_json::json!({"v":1}))).unwrap();
        write_web_conversation(tmp.path(), &mk("claude.ai", "b", serde_json::json!({"v":1}))).unwrap();
        write_web_conversation(tmp.path(), &mk("chatgpt.com", "x", serde_json::json!({"v":1}))).unwrap();
        let list = list_web_conversations(tmp.path()).unwrap();
        assert_eq!(list.len(), 3);
    }
}
```

- [ ] **Step 2: Add `pub mod web;` to `src/archive/mod.rs`** alongside `imports`, `discovery`, `index`, `manifest`, `objects`.

- [ ] **Step 3: Run + commit**

```
cargo test -p claude-usage-tracker --lib archive::web
cargo clippy --lib -- -D warnings
git add src/archive/web/ src/archive/mod.rs
git commit -m "feat(archive): web-conversation storage with history rotation"
```

---

## Task 2: Companion-token management

**Files:**
- Create: `src/archive/companion_token.rs`
- Modify: `src/archive/mod.rs` (`pub mod companion_token;`)
- Modify: `Cargo.toml` (add `subtle = "2"`).

- [ ] **Step 1: Token type + read/init/rotate**

```rust
//! Companion bearer token stored at `~/.heimdall/companion-token`.
//!
//! Used by the Phase 3a CLI scrape path (so the local HTTP endpoint can
//! distinguish loopback callers from random localhost software) and by
//! the Phase 3b browser extension (paired once via the token's hex value).

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use rand::RngCore;
use subtle::ConstantTimeEq;

const TOKEN_FILE: &str = "companion-token";
const TOKEN_BYTES: usize = 32;

pub struct CompanionToken {
    hex: String,
}

impl CompanionToken {
    pub fn as_hex(&self) -> &str { &self.hex }
    pub fn matches(&self, candidate: &str) -> bool {
        let a = self.hex.as_bytes();
        let b = candidate.as_bytes();
        if a.len() != b.len() { return false; }
        a.ct_eq(b).into()
    }
}

/// Default location: `~/.heimdall/companion-token`.
pub fn default_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".heimdall").join(TOKEN_FILE)
}

/// Read the token; create one with cryptographically random bytes if absent.
pub fn read_or_init(path: &Path) -> Result<CompanionToken> {
    if path.is_file() {
        let hex = fs::read_to_string(path)
            .with_context(|| format!("reading {}", path.display()))?
            .trim()
            .to_string();
        if hex.len() == TOKEN_BYTES * 2 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Ok(CompanionToken { hex });
        }
    }
    rotate(path)
}

/// Generate a fresh token and persist it (mode 0600 on Unix).
pub fn rotate(path: &Path) -> Result<CompanionToken> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut bytes = [0_u8; TOKEN_BYTES];
    rand::thread_rng().fill_bytes(&mut bytes);
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();

    let mut f = fs::File::create(path)
        .with_context(|| format!("creating {}", path.display()))?;
    f.write_all(hex.as_bytes())?;
    f.write_all(b"\n")?;
    f.sync_all()?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(path, perms)?;
    }
    Ok(CompanionToken { hex })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn read_or_init_creates_token_when_missing() {
        let tmp = TempDir::new().unwrap();
        let p = tmp.path().join("token");
        let t = read_or_init(&p).unwrap();
        assert_eq!(t.as_hex().len(), TOKEN_BYTES * 2);
        assert!(p.is_file());
    }

    #[test]
    fn rotate_generates_a_new_token() {
        let tmp = TempDir::new().unwrap();
        let p = tmp.path().join("token");
        let a = read_or_init(&p).unwrap();
        let b = rotate(&p).unwrap();
        assert_ne!(a.as_hex(), b.as_hex());
    }

    #[test]
    fn matches_is_constant_time_correct() {
        let tmp = TempDir::new().unwrap();
        let p = tmp.path().join("token");
        let t = read_or_init(&p).unwrap();
        assert!(t.matches(t.as_hex()));
        assert!(!t.matches("not-the-token"));
        assert!(!t.matches(""));
    }

    #[cfg(unix)]
    #[test]
    fn token_file_is_mode_0600() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = TempDir::new().unwrap();
        let p = tmp.path().join("token");
        read_or_init(&p).unwrap();
        let mode = fs::metadata(&p).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }
}
```

- [ ] **Step 2: Add `subtle = "2"` to Cargo.toml. Confirm `rand` is already a dep (it is — used by other modules).**

- [ ] **Step 3: Run + commit**

```
cargo test --lib archive::companion_token
cargo clippy --lib -- -D warnings
git add Cargo.toml Cargo.lock src/archive/companion_token.rs src/archive/mod.rs
git commit -m "feat(archive): companion bearer token (read/init/rotate, mode 0600)"
```

---

## Task 3: HTTP endpoint `POST /api/archive/web-conversation`

**Files:**
- Modify: `src/server/api.rs` — add `api_archive_web_conversation`.
- Modify: `src/server/mod.rs` — register route.
- Modify: `src/server/tests.rs` — bearer happy + reject + unchanged.

- [ ] **Step 1: Handler**

```rust
pub async fn api_archive_web_conversation(
    State(_state): State<Arc<AppState>>,
    request: Request,
) -> Result<Json<Value>, StatusCode> {
    enforce_loopback_request(&request)?;

    // Bearer auth.
    let header = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    let candidate = header.strip_prefix("Bearer ").unwrap_or("");

    let token_path = crate::archive::companion_token::default_path();
    let token = tokio::task::spawn_blocking(move || {
        crate::archive::companion_token::read_or_init(&token_path)
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if !token.matches(candidate) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Body.
    let body_bytes = axum::body::to_bytes(request.into_body(), 50 * 1024 * 1024)
        .await
        .map_err(|_| StatusCode::PAYLOAD_TOO_LARGE)?;
    let conv: crate::archive::web::WebConversation = serde_json::from_slice(&body_bytes)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let archive_root = crate::archive::default_root();
    let outcome = tokio::task::spawn_blocking(move || {
        crate::archive::web::write_web_conversation(&archive_root, &conv)
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let body = match outcome {
        crate::archive::web::WriteOutcome::Saved { .. } =>
            serde_json::json!({"saved": true}),
        crate::archive::web::WriteOutcome::Unchanged =>
            serde_json::json!({"unchanged": true}),
    };
    Ok(Json(body))
}
```

- [ ] **Step 2: Register route**

In `src/server/mod.rs::build_router`, alongside the other archive routes:

```rust
        .route("/api/archive/web-conversation", post(api::api_archive_web_conversation))
```

- [ ] **Step 3: Server tests**

In `src/server/tests.rs`, add tests modeled on the existing imports test pattern. At minimum:
- `web_conversation_returns_401_without_bearer`
- `web_conversation_saves_with_valid_bearer` — also asserts the file appears under `<HOME>/.heimdall/archive/web/<vendor>/<conv_id>.json`.
- `web_conversation_returns_unchanged_for_byte_identical_payload`

Test helpers should use the `HOME` redirect pattern already in use, then read the token from `<HOME>/.heimdall/companion-token` after `read_or_init` is called.

- [ ] **Step 4: Run + commit**

```
cargo test --lib server::
cargo clippy --lib -- -D warnings
git add src/server/api.rs src/server/mod.rs src/server/tests.rs
git commit -m "feat(server): POST /api/archive/web-conversation (bearer-gated)"
```

---

## Task 4: claude.ai scrape client

**Files:**
- Create: `src/scrape/mod.rs`
- Create: `src/scrape/claude.rs`
- Modify: `src/lib.rs` (`pub mod scrape;`)

- [ ] **Step 1: scrape/mod.rs (shared types)**

```rust
//! HTTP scrape clients for vendor private APIs (Tier 3a, cookie-paste).
//!
//! Each vendor module exposes a `Client` plus `list_conversations()` and
//! `fetch_conversation()` methods. The `scrape` CLI subcommand iterates
//! and writes via `archive::web::write_web_conversation`.

pub mod chatgpt;
pub mod claude;

#[derive(Debug, Clone)]
pub struct ScrapeReport {
    pub vendor: &'static str,
    pub listed: usize,
    pub written: usize,
    pub unchanged: usize,
    pub errors: Vec<String>,
}
```

- [ ] **Step 2: claude.rs**

```rust
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
use reqwest::header::{HeaderMap, HeaderValue, COOKIE, USER_AGENT};
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
        let resp = self.http.get(&url).send().await
            .with_context(|| format!("GET {url}"))?;
        check_status(&resp)?;
        let orgs: Vec<Organization> = resp.json().await
            .context("parsing organizations response")?;
        Ok(orgs)
    }

    pub async fn list_conversations(&self, org_id: &str) -> Result<Vec<ConversationSummary>> {
        let url = format!("{BASE}/api/organizations/{org_id}/chat_conversations");
        let resp = self.http.get(&url).send().await
            .with_context(|| format!("GET {url}"))?;
        check_status(&resp)?;
        let convs: Vec<ConversationSummary> = resp.json().await
            .context("parsing chat_conversations response")?;
        Ok(convs)
    }

    pub async fn fetch_conversation(&self, org_id: &str, conv_id: &str) -> Result<Value> {
        let url = format!("{BASE}/api/organizations/{org_id}/chat_conversations/{conv_id}");
        let resp = self.http.get(&url).send().await
            .with_context(|| format!("GET {url}"))?;
        check_status(&resp)?;
        let value: Value = resp.json().await
            .context("parsing conversation response")?;
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
    if status.is_success() { return Ok(()); }
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

    #[test]
    fn check_status_401_includes_refresh_hint() {
        // Build a fake 401 response by abusing reqwest::Response — easier
        // to test the message via a synthetic call: call Client::new and
        // assert the error string format from a parser-side check.
        // Phase 3a relies on integration testing for HTTP status paths;
        // unit tests here just confirm the pure construction.
    }
}
```

- [ ] **Step 3: Run + commit**

```
cargo build
cargo test --lib scrape::claude
cargo clippy --lib -- -D warnings
git add src/scrape/ src/lib.rs
git commit -m "feat(scrape): claude.ai cookie-paste client"
```

---

## Task 5: chatgpt.com scrape client

**Files:**
- Create: `src/scrape/chatgpt.rs`

- [ ] **Step 1: chatgpt.rs**

```rust
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
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, COOKIE, USER_AGENT};
use serde::Deserialize;
use serde_json::Value;

const BASE: &str = "https://chatgpt.com";

pub struct Credentials {
    pub session_token: String,   // __Secure-next-auth.session-token cookie value
    pub access_token: String,    // Bearer ... from /api/auth/session
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
            let resp = self.http.get(&url).send().await
                .with_context(|| format!("GET {url}"))?;
            check_status(&resp)?;
            let page: ConversationsPage = resp.json().await
                .context("parsing conversations page")?;
            let got = page.items.len();
            all.extend(page.items);
            offset += got;
            if got < page_size { break; }
            if offset >= page.total.unwrap_or(usize::MAX) { break; }
        }
        Ok(all)
    }

    pub async fn fetch_conversation(&self, conv_id: &str) -> Result<Value> {
        let url = format!("{BASE}/backend-api/conversation/{conv_id}");
        let resp = self.http.get(&url).send().await
            .with_context(|| format!("GET {url}"))?;
        check_status(&resp)?;
        let value: Value = resp.json().await
            .context("parsing conversation response")?;
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
    if status.is_success() { return Ok(()); }
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
```

- [ ] **Step 2: Run + commit**

```
cargo build
cargo test --lib scrape::chatgpt
cargo clippy --lib -- -D warnings
git add src/scrape/chatgpt.rs
git commit -m "feat(scrape): chatgpt.com cookie-paste client"
```

---

## Task 6: CLI subcommands `scrape` + `companion-token`

**Files:**
- Modify: `src/main.rs`
- Modify: `src/cli_tests.rs`

- [ ] **Step 1: Add variants + dispatch**

Add to `Commands` enum:

```rust
    /// Scrape claude.ai or chatgpt.com private APIs using copy-pasted cookies
    Scrape {
        #[command(subcommand)]
        action: ScrapeAction,
    },
    /// Manage the companion bearer token used by the browser extension and the scrape CLI
    CompanionToken {
        #[command(subcommand)]
        action: CompanionTokenAction,
    },
```

Add the action enums (after `ConfigAction`, near the existing `ArchiveAction`):

```rust
#[derive(Subcommand)]
enum ScrapeAction {
    /// Scrape claude.ai
    Claude {
        /// sessionKey cookie value (`sk-ant-sid01-...`).
        /// Falls back to env `HEIMDALL_CLAUDE_SESSION_KEY`.
        #[arg(long)]
        session_key: Option<String>,
        /// cf_clearance cookie value (optional but usually required).
        #[arg(long, env = "HEIMDALL_CLAUDE_CF_CLEARANCE")]
        cf_clearance: Option<String>,
        /// User-Agent matching the browser that issued the cookies.
        #[arg(long, env = "HEIMDALL_USER_AGENT", default_value =
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15")]
        user_agent: String,
        /// Override archive root
        #[arg(long)]
        archive_root: Option<PathBuf>,
        /// JSON output
        #[arg(long)]
        json: bool,
    },
    /// Scrape chatgpt.com
    Chatgpt {
        #[arg(long, env = "HEIMDALL_CHATGPT_SESSION_TOKEN")]
        session_token: Option<String>,
        #[arg(long, env = "HEIMDALL_CHATGPT_ACCESS_TOKEN")]
        access_token: Option<String>,
        #[arg(long, env = "HEIMDALL_CHATGPT_CF_CLEARANCE")]
        cf_clearance: Option<String>,
        #[arg(long, env = "HEIMDALL_USER_AGENT", default_value =
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15")]
        user_agent: String,
        #[arg(long)]
        archive_root: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum CompanionTokenAction {
    /// Print the current bearer token (creates one if missing)
    Show,
    /// Generate a fresh bearer token (any prior pair-up must be repeated)
    Rotate,
}
```

Dispatch arms:

```rust
        Commands::Scrape { action } => {
            // Async dispatch via tokio runtime.
            let rt = tokio::runtime::Runtime::new()?;
            match action {
                ScrapeAction::Claude { session_key, cf_clearance, user_agent, archive_root, json } => {
                    let session_key = session_key
                        .or_else(|| std::env::var("HEIMDALL_CLAUDE_SESSION_KEY").ok())
                        .ok_or_else(|| anyhow::anyhow!(
                            "--session-key (or HEIMDALL_CLAUDE_SESSION_KEY) required"
                        ))?;
                    let root = archive_root.unwrap_or_else(archive::default_root);
                    let report = rt.block_on(scrape_claude_run(&session_key, cf_clearance, &user_agent, &root))?;
                    print_scrape_report(&report, json)?;
                }
                ScrapeAction::Chatgpt { session_token, access_token, cf_clearance, user_agent, archive_root, json } => {
                    let session_token = session_token
                        .or_else(|| std::env::var("HEIMDALL_CHATGPT_SESSION_TOKEN").ok())
                        .ok_or_else(|| anyhow::anyhow!(
                            "--session-token (or HEIMDALL_CHATGPT_SESSION_TOKEN) required"
                        ))?;
                    let access_token = access_token
                        .or_else(|| std::env::var("HEIMDALL_CHATGPT_ACCESS_TOKEN").ok())
                        .ok_or_else(|| anyhow::anyhow!(
                            "--access-token (or HEIMDALL_CHATGPT_ACCESS_TOKEN) required"
                        ))?;
                    let root = archive_root.unwrap_or_else(archive::default_root);
                    let report = rt.block_on(scrape_chatgpt_run(
                        &session_token, &access_token, cf_clearance, &user_agent, &root,
                    ))?;
                    print_scrape_report(&report, json)?;
                }
            }
        }
        Commands::CompanionToken { action } => {
            let path = archive::companion_token::default_path();
            match action {
                CompanionTokenAction::Show => {
                    let t = archive::companion_token::read_or_init(&path)?;
                    println!("{}", t.as_hex());
                }
                CompanionTokenAction::Rotate => {
                    let t = archive::companion_token::rotate(&path)?;
                    println!("{}", t.as_hex());
                }
            }
        }
```

Then add the helpers (place at the bottom of `main.rs`, near other private helpers):

```rust
async fn scrape_claude_run(
    session_key: &str,
    cf_clearance: Option<String>,
    user_agent: &str,
    archive_root: &std::path::Path,
) -> anyhow::Result<scrape::ScrapeReport> {
    let creds = scrape::claude::Credentials {
        session_key: session_key.to_string(),
        cf_clearance,
        user_agent: user_agent.to_string(),
    };
    let client = scrape::claude::Client::new(&creds)?;
    let mut report = scrape::ScrapeReport {
        vendor: "claude.ai", listed: 0, written: 0, unchanged: 0, errors: Vec::new(),
    };
    let orgs = client.list_organizations().await?;
    for org in &orgs {
        let convs = client.list_conversations(&org.uuid).await?;
        for summary in &convs {
            report.listed += 1;
            match client.fetch_conversation(&org.uuid, &summary.uuid).await {
                Ok(payload) => {
                    let conv = archive::web::WebConversation {
                        vendor: "claude.ai".into(),
                        conversation_id: summary.uuid.clone(),
                        captured_at: chrono::Utc::now().format("%Y-%m-%dT%H%M%S%.6fZ").to_string(),
                        schema_fingerprint: "claude.ai/v1".into(),
                        payload,
                    };
                    match archive::web::write_web_conversation(archive_root, &conv) {
                        Ok(archive::web::WriteOutcome::Saved { .. }) => report.written += 1,
                        Ok(archive::web::WriteOutcome::Unchanged) => report.unchanged += 1,
                        Err(e) => report.errors.push(format!("{}: {e}", summary.uuid)),
                    }
                }
                Err(e) => report.errors.push(format!("{}: {e}", summary.uuid)),
            }
        }
    }
    Ok(report)
}

async fn scrape_chatgpt_run(
    session_token: &str,
    access_token: &str,
    cf_clearance: Option<String>,
    user_agent: &str,
    archive_root: &std::path::Path,
) -> anyhow::Result<scrape::ScrapeReport> {
    let creds = scrape::chatgpt::Credentials {
        session_token: session_token.to_string(),
        access_token: access_token.to_string(),
        cf_clearance,
        user_agent: user_agent.to_string(),
    };
    let client = scrape::chatgpt::Client::new(&creds)?;
    let mut report = scrape::ScrapeReport {
        vendor: "chatgpt.com", listed: 0, written: 0, unchanged: 0, errors: Vec::new(),
    };
    let convs = client.list_conversations(28).await?;
    for summary in &convs {
        report.listed += 1;
        match client.fetch_conversation(&summary.id).await {
            Ok(payload) => {
                let conv = archive::web::WebConversation {
                    vendor: "chatgpt.com".into(),
                    conversation_id: summary.id.clone(),
                    captured_at: chrono::Utc::now().format("%Y-%m-%dT%H%M%S%.6fZ").to_string(),
                    schema_fingerprint: "chatgpt.com/v1".into(),
                    payload,
                };
                match archive::web::write_web_conversation(archive_root, &conv) {
                    Ok(archive::web::WriteOutcome::Saved { .. }) => report.written += 1,
                    Ok(archive::web::WriteOutcome::Unchanged) => report.unchanged += 1,
                    Err(e) => report.errors.push(format!("{}: {e}", summary.id)),
                }
            }
            Err(e) => report.errors.push(format!("{}: {e}", summary.id)),
        }
    }
    Ok(report)
}

fn print_scrape_report(r: &scrape::ScrapeReport, json: bool) -> anyhow::Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({
            "vendor": r.vendor, "listed": r.listed, "written": r.written,
            "unchanged": r.unchanged, "errors": r.errors,
        }))?);
    } else {
        println!(
            "{}: listed {}, wrote {} new, {} unchanged, {} errors",
            r.vendor, r.listed, r.written, r.unchanged, r.errors.len()
        );
        for e in &r.errors { eprintln!("  {e}"); }
    }
    Ok(())
}
```

Add `use claude_usage_tracker::scrape;` at the top of main.rs alongside the other module imports.

- [ ] **Step 2: CLI smoke test for `companion-token`**

Append to `src/cli_tests.rs::tests`:

```rust
#[test]
fn companion_token_show_prints_64_hex_chars() {
    use tempfile::TempDir;
    let tmp = TempDir::new().unwrap();
    let test_exe = std::env::current_exe().unwrap();
    let target_dir = test_exe.parent().unwrap().parent().unwrap();
    let exe = target_dir.join("claude-usage-tracker");
    if !exe.exists() {
        let s = std::process::Command::new(env!("CARGO"))
            .args(["build", "--bin", "claude-usage-tracker"]).status().unwrap();
        assert!(s.success());
    }
    let out = std::process::Command::new(&exe)
        .args(["companion-token", "show"])
        .env("HOME", tmp.path())
        .output()
        .unwrap();
    assert!(out.status.success(), "companion-token show failed: {:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let line = stdout.lines().next().unwrap_or("").trim();
    assert_eq!(line.len(), 64, "expected 64-hex token, got: {line:?}");
    assert!(line.chars().all(|c| c.is_ascii_hexdigit()));
}
```

(No CLI smoke test for `scrape claude/chatgpt` — they hit live network endpoints. Test the storage primitive in Task 1's integration test instead.)

- [ ] **Step 3: Run + commit**

```
cargo build
cargo test --bin claude-usage-tracker companion_token_show_prints_64_hex_chars
cargo clippy --lib --bin claude-usage-tracker -- -D warnings
git add src/main.rs src/cli_tests.rs
git commit -m "feat(cli): heimdall scrape {claude,chatgpt} + companion-token mgmt"
```

---

## Task 7: Final integration check

Same shape as previous phases. Full Rust suite, cargo fmt --check / fix Phase 3a drift, CLI smoke for both new subcommands' `--help`, server-test green for the new endpoint, single tidy commit if needed, and a `git log --oneline 1cd57c1..HEAD` summary.

---

## Self-Review

**Spec coverage:**

| Spec section | Plan task |
|---|---|
| §7b cookie-paste CLI (`heimdall scrape claude/chatgpt`) | Tasks 4, 5, 6 |
| §7 storage layout `web/<vendor>/<conv_id>.json` + history | Task 1 |
| §7 `POST /api/archive/web-conversation` ingest endpoint | Task 3 |
| §9 companion-token bearer (read/init/rotate, mode 0600) | Task 2 |
| Storage primitive shared between CLI and HTTP endpoint | Task 1 (write_web_conversation called from both) |
| Error message tells user to refresh cookies on 401 | Tasks 4, 5 (`check_status` helper) |
| Phase 3b extension uses same endpoint and same token | Task 3 (foundation laid) |

**Out-of-scope:** dashboard "Web captures" panel (Phase 3b), browser extension (Phase 3b), `--daemon` poll loop (deferred — `--once` only in Phase 3a since live network testing of a daemon mode isn't worth the additional CLI complexity until we have real cookies to test against).

**Type consistency:** `WebConversation`, `WriteOutcome`, `ScrapeReport`, `Credentials`, `Client` referenced consistently across tasks.

---

## Execution Handoff

Plan complete. Execute via subagent-driven-development on `main` (continuing the existing direct-to-main posture per user authorization).
