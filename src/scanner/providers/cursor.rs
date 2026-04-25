//! Cursor IDE provider — reads AI chat history from Cursor's SQLite workspace
//! state databases (`state.vscdb`).
//!
//! # Schema probing
//!
//! Cursor's `state.vscdb` is a SQLite database with a single key-value table:
//!
//! ```sql
//! CREATE TABLE ItemTable (key TEXT PRIMARY KEY, value TEXT);
//! ```
//!
//! AI/chat data is stored as JSON blobs under specific string keys. The primary
//! key observed in practice is:
//!
//!   `workbench.panel.aichat.view.aichat.chatdata`
//!
//! The JSON blob shape (as observed from Codeburn's cursor.ts reference) is:
//!
//! ```json
//! {
//!   "tabs": [
//!     {
//!       "tabId": "<uuid>",
//!       "chatTitle": "...",
//!       "bubbles": [
//!         {
//!           "type": "user" | "ai",
//!           "text": "...",
//!           "modelType": "claude-3-5-sonnet",
//!           "requestId": "<uuid>",
//!           "timingInfo": { "clientStartTime": 1700000000000 },
//!           "tokenCount": {
//!             "promptTokens": 100,
//!             "generationTokens": 50
//!           }
//!         }
//!       ]
//!     }
//!   ]
//! }
//! ```
//!
//! Only `"ai"` bubbles with non-zero token counts become `Turn` records.
//! Missing fields are treated as zero / absent — never panic.
//!
//! # Cache invalidation
//!
//! A sidecar JSON file is written to `~/.cache/heimdall/cursor/<workspace-hash>.json`
//! after every successful parse. On subsequent scans, if the source DB's
//! mtime + size match the cached entry, parsing is skipped entirely.
//!
//! # SQLite-backed provider
//!
//! Unlike JSONL-backed providers (Claude, Codex, Xcode), this provider opens
//! the source file as a SQLite database in read-only mode. See `cursor_cache`
//! for the cache helper module. This is an example of a SQLite-backed provider.

use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use anyhow::Result;
use rusqlite::{Connection, OpenFlags};
use tracing::warn;

use crate::models::Turn;
use crate::pricing;
use crate::scanner::parser::{ParseResult, empty_parse_result, parse_provider_turns_result};
use crate::scanner::provider::{Provider, SessionSource};
use crate::scanner::providers::cursor_cache::{
    CursorCacheEntry, is_cache_fresh, load_cache, save_cache,
};

/// Key in `ItemTable` that holds Cursor AI chat JSON.
const CHAT_DATA_KEY: &str = "workbench.panel.aichat.view.aichat.chatdata";

/// Provider slug stored in `turns.provider`.
pub const PROVIDER_CURSOR: &str = "cursor";

// ---------------------------------------------------------------------------
// Provider struct
// ---------------------------------------------------------------------------

pub struct CursorProvider {
    pub dirs: Vec<PathBuf>,
    /// Directory for sidecar cache files. Defaults to `~/.cache/heimdall/cursor`.
    cache_dir: PathBuf,
}

impl CursorProvider {
    /// Construct with the platform-default Cursor workspace storage paths.
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| home.join(".cache"))
            .join("heimdall")
            .join("cursor");

        #[cfg(target_os = "macos")]
        let workspace_dir = home
            .join("Library")
            .join("Application Support")
            .join("Cursor")
            .join("User")
            .join("workspaceStorage");

        #[cfg(target_os = "linux")]
        let workspace_dir = home
            .join(".config")
            .join("Cursor")
            .join("User")
            .join("workspaceStorage");

        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        let workspace_dir = home.join(".cursor").join("workspaceStorage");

        Self {
            dirs: vec![workspace_dir],
            cache_dir,
        }
    }

    /// Construct with explicit discovery directories (used in tests).
    #[cfg(test)]
    pub fn new_with_dirs(dirs: Vec<PathBuf>) -> Self {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("heimdall")
            .join("cursor");
        Self { dirs, cache_dir }
    }

    /// Construct with explicit discovery directories AND explicit cache dir (used in tests).
    #[cfg(test)]
    pub fn new_with_dirs_and_cache(dirs: Vec<PathBuf>, cache_dir: PathBuf) -> Self {
        Self { dirs, cache_dir }
    }
}

impl Default for CursorProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for CursorProvider {
    fn name(&self) -> &'static str {
        PROVIDER_CURSOR
    }

    fn discover_sessions(&self) -> Result<Vec<SessionSource>> {
        let mut sources = Vec::new();
        for dir in &self.dirs {
            if !dir.exists() {
                continue;
            }
            for entry in walkdir::WalkDir::new(dir)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                if path.file_name().is_some_and(|n| n == "state.vscdb") {
                    sources.push(SessionSource {
                        path: path.to_path_buf(),
                    });
                }
            }
        }
        Ok(sources)
    }

    fn parse(&self, path: &Path) -> Result<Vec<Turn>> {
        let workspace_hash = workspace_hash_from_path(path);
        let cache_path = self.cache_dir.join(format!("{workspace_hash}.json"));

        // Check cache freshness before opening the DB.
        if let (Some(entry), Ok(meta)) = (load_cache(&cache_path), std::fs::metadata(path)) {
            let mtime = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs_f64())
                .unwrap_or(0.0);
            let size = meta.len();
            if is_cache_fresh(&entry, mtime, size) {
                // Return empty vec — the caller knows from the DB that this
                // workspace's turns were already inserted; returning empty
                // means "nothing new to process" from a re-parse perspective.
                // In practice the full scan pipeline uses incremental logic
                // separately; this cache avoids redundant SQLite opens.
                return Ok(Vec::new());
            }
        }

        let turns = parse_cursor_db(path, &workspace_hash);

        // Update cache on successful (possibly-empty) parse.
        let mtime = std::fs::metadata(path)
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);
        let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        let entry = CursorCacheEntry {
            source_mtime: mtime,
            source_size: size,
            parsed_turns_count: turns.len(),
            parsed_at: chrono::Utc::now().to_rfc3339(),
        };
        if let Err(e) = save_cache(&cache_path, &entry) {
            warn!(
                "cursor: failed to write cache {}: {}",
                cache_path.display(),
                e
            );
        }

        Ok(turns)
    }

    fn parse_source(&self, path: &Path, _skip_lines: i64) -> ParseResult {
        match self.parse(path) {
            Ok(turns) => parse_provider_turns_result(self.name(), turns, path, None),
            Err(e) => {
                warn!(
                    "cursor: provider parse failed for {}: {}",
                    path.display(),
                    e
                );
                empty_parse_result()
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Extract the workspace hash from the path.
/// `state.vscdb` lives at `…/workspaceStorage/<hash>/state.vscdb`.
/// Returns the hash directory name, or a best-effort fallback.
fn workspace_hash_from_path(path: &Path) -> String {
    path.parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string()
}

/// Open the SQLite DB in read-only mode and parse known AI-chat keys.
///
/// All recoverable errors are logged at `warn!` and result in an empty return.
fn parse_cursor_db(path: &Path, workspace_hash: &str) -> Vec<Turn> {
    let conn = match Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(c) => c,
        Err(e) => {
            warn!("cursor: cannot open {}: {}", path.display(), e);
            return Vec::new();
        }
    };

    // Probe whether ItemTable exists.
    let table_exists: bool = conn
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='ItemTable'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map(|n| n > 0)
        .unwrap_or(false);

    if !table_exists {
        warn!("cursor: ItemTable not found in {}", path.display());
        return Vec::new();
    }

    let mut turns = Vec::new();
    let source_path = path.to_string_lossy().to_string();

    // Query each known chat-data key.
    for key in [CHAT_DATA_KEY] {
        let value: Option<String> = conn
            .query_row("SELECT value FROM ItemTable WHERE key = ?1", [key], |row| {
                row.get(0)
            })
            .ok();

        let Some(json_str) = value else {
            continue;
        };

        let blob: serde_json::Value = match serde_json::from_str(&json_str) {
            Ok(v) => v,
            Err(e) => {
                warn!("cursor: failed to parse JSON for key {key}: {e}");
                continue;
            }
        };

        parse_chat_blob(&blob, workspace_hash, &source_path, &mut turns);
    }

    turns
}

/// Convert a Cursor AI chat JSON blob into `Turn` records.
///
/// The structure is:
/// ```json
/// { "tabs": [ { "tabId": "...", "bubbles": [ { "type": "ai", ... } ] } ] }
/// ```
fn parse_chat_blob(
    blob: &serde_json::Value,
    workspace_hash: &str,
    source_path: &str,
    turns: &mut Vec<Turn>,
) {
    let tabs = match blob.get("tabs").and_then(|v| v.as_array()) {
        Some(t) => t,
        None => return,
    };

    let session_id = format!("cursor:{workspace_hash}");

    for tab in tabs {
        let tab_id = tab
            .get("tabId")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let bubbles = match tab.get("bubbles").and_then(|v| v.as_array()) {
            Some(b) => b,
            None => continue,
        };

        for (bubble_idx, bubble) in bubbles.iter().enumerate() {
            let bubble_type = bubble.get("type").and_then(|v| v.as_str()).unwrap_or("");
            if bubble_type != "ai" {
                continue;
            }

            let model = bubble
                .get("modelType")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            let (input_tokens, output_tokens) = parse_token_count(bubble);
            if input_tokens == 0 && output_tokens == 0 {
                continue;
            }

            let timestamp = parse_timestamp(bubble);

            let request_id = bubble
                .get("requestId")
                .and_then(|v| v.as_str())
                .map(String::from)
                .unwrap_or_else(|| format!("{tab_id}:{bubble_idx}"));

            let estimate = pricing::estimate_cost(&model, input_tokens, output_tokens, 0, 0);

            turns.push(Turn {
                session_id: session_id.clone(),
                provider: PROVIDER_CURSOR.to_string(),
                timestamp,
                model: model.clone(),
                input_tokens,
                output_tokens,
                cache_read_tokens: 0,
                cache_creation_tokens: 0,
                reasoning_output_tokens: 0,
                estimated_cost_nanos: estimate.estimated_cost_nanos,
                tool_name: None,
                cwd: String::new(),
                message_id: request_id,
                service_tier: None,
                inference_geo: None,
                is_subagent: false,
                agent_id: None,
                source_path: source_path.to_string(),
                version: None,
                pricing_version: estimate.pricing_version,
                pricing_model: estimate.pricing_model,
                billing_mode: "estimated_local".to_string(),
                cost_confidence: estimate.cost_confidence,
                category: String::new(),
                all_tools: Vec::new(),
                tool_use_ids: Vec::new(),
                tool_inputs: Vec::new(),
                credits: None,
            });
        }
    }
}

fn parse_token_count(bubble: &serde_json::Value) -> (i64, i64) {
    let tc = match bubble.get("tokenCount").and_then(|v| v.as_object()) {
        Some(t) => t,
        None => return (0, 0),
    };
    let input = tc.get("promptTokens").and_then(|v| v.as_i64()).unwrap_or(0);
    let output = tc
        .get("generationTokens")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    (input, output)
}

fn parse_timestamp(bubble: &serde_json::Value) -> String {
    bubble
        .get("timingInfo")
        .and_then(|v| v.get("clientStartTime"))
        .and_then(|v| v.as_i64())
        .and_then(|ms| {
            chrono::DateTime::from_timestamp(ms / 1000, ((ms % 1000) * 1_000_000) as u32)
        })
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use tempfile::TempDir;

    // -----------------------------------------------------------------------
    // Helper: create a minimal state.vscdb with an ItemTable
    // -----------------------------------------------------------------------
    fn create_state_db(path: &Path) -> Connection {
        let conn = Connection::open(path).unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS ItemTable (key TEXT PRIMARY KEY, value TEXT);",
        )
        .unwrap();
        conn
    }

    fn insert_item(conn: &Connection, key: &str, value: &str) {
        conn.execute(
            "INSERT OR REPLACE INTO ItemTable (key, value) VALUES (?1, ?2)",
            [key, value],
        )
        .unwrap();
    }

    fn minimal_chat_json(
        tab_id: &str,
        model: &str,
        input_tokens: u64,
        output_tokens: u64,
    ) -> serde_json::Value {
        serde_json::json!({
            "tabs": [{
                "tabId": tab_id,
                "chatTitle": "Test chat",
                "bubbles": [
                    {
                        "type": "user",
                        "text": "Hello",
                    },
                    {
                        "type": "ai",
                        "modelType": model,
                        "requestId": "req-001",
                        "timingInfo": { "clientStartTime": 1_700_000_000_000i64 },
                        "tokenCount": {
                            "promptTokens": input_tokens,
                            "generationTokens": output_tokens,
                        }
                    }
                ]
            }]
        })
    }

    // -----------------------------------------------------------------------
    // Provider::name
    // -----------------------------------------------------------------------

    #[test]
    fn cursor_provider_name() {
        assert_eq!(CursorProvider::new_with_dirs(vec![]).name(), "cursor");
    }

    // -----------------------------------------------------------------------
    // Platform path test
    // -----------------------------------------------------------------------

    #[test]
    fn cursor_new_resolves_platform_path() {
        let provider = CursorProvider::new();
        assert_eq!(provider.dirs.len(), 1);
        let dir_str = provider.dirs[0].to_string_lossy();
        // Path must contain "Cursor" regardless of OS.
        assert!(
            dir_str.contains("Cursor"),
            "Expected 'Cursor' in path, got: {dir_str}"
        );
    }

    // -----------------------------------------------------------------------
    // discover_sessions
    // -----------------------------------------------------------------------

    #[test]
    fn discover_sessions_finds_state_vscdb() {
        let dir = TempDir::new().unwrap();
        // Mimic workspaceStorage/<hash>/state.vscdb
        let hash_dir = dir.path().join("abc123deadbeef");
        std::fs::create_dir_all(&hash_dir).unwrap();
        let db_path = hash_dir.join("state.vscdb");
        // Create a minimal SQLite file so the path exists.
        let _conn = create_state_db(&db_path);

        let provider = CursorProvider::new_with_dirs(vec![dir.path().to_path_buf()]);
        let sources = provider.discover_sessions().unwrap();
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].path, db_path);
        assert_eq!(sources[0].path, db_path);
    }

    #[test]
    fn discover_sessions_skips_non_vscdb_files() {
        let dir = TempDir::new().unwrap();
        let hash_dir = dir.path().join("deadbeef");
        std::fs::create_dir_all(&hash_dir).unwrap();
        std::fs::write(hash_dir.join("state.json"), b"{}").unwrap();
        std::fs::write(hash_dir.join("something.db"), b"").unwrap();

        let provider = CursorProvider::new_with_dirs(vec![dir.path().to_path_buf()]);
        let sources = provider.discover_sessions().unwrap();
        assert!(sources.is_empty());
    }

    #[test]
    fn discover_sessions_nonexistent_dir_returns_empty() {
        let provider = CursorProvider::new_with_dirs(vec![PathBuf::from("/nonexistent/path/xyz")]);
        let sources = provider.discover_sessions().unwrap();
        assert!(sources.is_empty());
    }

    // -----------------------------------------------------------------------
    // parse: empty DB (no AI chat keys)
    // -----------------------------------------------------------------------

    #[test]
    fn parse_empty_db_returns_empty() {
        let dir = TempDir::new().unwrap();
        let hash_dir = dir.path().join("hash001");
        std::fs::create_dir_all(&hash_dir).unwrap();
        let db_path = hash_dir.join("state.vscdb");
        let _conn = create_state_db(&db_path);

        let cache_dir = TempDir::new().unwrap();
        let provider = CursorProvider::new_with_dirs_and_cache(
            vec![dir.path().to_path_buf()],
            cache_dir.path().to_path_buf(),
        );
        let turns = provider.parse(&db_path).unwrap();
        assert!(turns.is_empty());
    }

    // -----------------------------------------------------------------------
    // parse: DB with realistic chat JSON produces Turns
    // -----------------------------------------------------------------------

    #[test]
    fn parse_db_with_chat_data_produces_turns() {
        let dir = TempDir::new().unwrap();
        let hash_dir = dir.path().join("cafebabe1234");
        std::fs::create_dir_all(&hash_dir).unwrap();
        let db_path = hash_dir.join("state.vscdb");
        let conn = create_state_db(&db_path);

        let chat_json = minimal_chat_json("tab-1", "claude-3-5-sonnet", 100, 50);
        insert_item(&conn, CHAT_DATA_KEY, &chat_json.to_string());
        drop(conn); // close so read-only open works

        let cache_dir = TempDir::new().unwrap();
        let provider = CursorProvider::new_with_dirs_and_cache(
            vec![dir.path().to_path_buf()],
            cache_dir.path().to_path_buf(),
        );
        let turns = provider.parse(&db_path).unwrap();
        assert_eq!(turns.len(), 1, "expected exactly one AI turn");
        assert_eq!(turns[0].provider, "cursor");
        assert!(
            turns[0].session_id.starts_with("cursor:"),
            "session_id should be prefixed with 'cursor:'"
        );
        assert_eq!(turns[0].input_tokens, 100);
        assert_eq!(turns[0].output_tokens, 50);
    }

    // -----------------------------------------------------------------------
    // parse: user bubbles are not included
    // -----------------------------------------------------------------------

    #[test]
    fn parse_skips_user_bubbles() {
        let dir = TempDir::new().unwrap();
        let hash_dir = dir.path().join("beefcafe");
        std::fs::create_dir_all(&hash_dir).unwrap();
        let db_path = hash_dir.join("state.vscdb");
        let conn = create_state_db(&db_path);

        let json = serde_json::json!({
            "tabs": [{"tabId": "t1", "bubbles": [
                {"type": "user", "text": "hi"},
                {"type": "user", "text": "hi again"},
            ]}]
        });
        insert_item(&conn, CHAT_DATA_KEY, &json.to_string());
        drop(conn);

        let cache_dir = TempDir::new().unwrap();
        let provider = CursorProvider::new_with_dirs_and_cache(
            vec![dir.path().to_path_buf()],
            cache_dir.path().to_path_buf(),
        );
        let turns = provider.parse(&db_path).unwrap();
        assert!(turns.is_empty());
    }

    // -----------------------------------------------------------------------
    // parse: invalid SQLite file returns empty (not an error)
    // -----------------------------------------------------------------------

    #[test]
    fn parse_invalid_sqlite_returns_empty() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("bad.vscdb");
        std::fs::write(&db_path, b"this is not sqlite").unwrap();

        let cache_dir = TempDir::new().unwrap();
        let provider = CursorProvider::new_with_dirs_and_cache(
            vec![dir.path().to_path_buf()],
            cache_dir.path().to_path_buf(),
        );
        let result = provider.parse(&db_path);
        // Must not propagate an error; returns Ok(empty).
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    // -----------------------------------------------------------------------
    // parse: missing ItemTable returns empty
    // -----------------------------------------------------------------------

    #[test]
    fn parse_db_without_item_table_returns_empty() {
        let dir = TempDir::new().unwrap();
        let hash_dir = dir.path().join("nohash");
        std::fs::create_dir_all(&hash_dir).unwrap();
        let db_path = hash_dir.join("state.vscdb");
        // Create SQLite but WITHOUT ItemTable.
        let conn = Connection::open(&db_path).unwrap();
        conn.execute_batch("CREATE TABLE OtherTable (id INTEGER);")
            .unwrap();
        drop(conn);

        let cache_dir = TempDir::new().unwrap();
        let provider = CursorProvider::new_with_dirs_and_cache(
            vec![dir.path().to_path_buf()],
            cache_dir.path().to_path_buf(),
        );
        let turns = provider.parse(&db_path).unwrap();
        assert!(turns.is_empty());
    }

    // -----------------------------------------------------------------------
    // parse: broken JSON blob under chat key is skipped gracefully
    // -----------------------------------------------------------------------

    #[test]
    fn parse_malformed_json_blob_returns_empty() {
        let dir = TempDir::new().unwrap();
        let hash_dir = dir.path().join("malfhash");
        std::fs::create_dir_all(&hash_dir).unwrap();
        let db_path = hash_dir.join("state.vscdb");
        let conn = create_state_db(&db_path);
        insert_item(&conn, CHAT_DATA_KEY, "not valid json{{{{");
        drop(conn);

        let cache_dir = TempDir::new().unwrap();
        let provider = CursorProvider::new_with_dirs_and_cache(
            vec![dir.path().to_path_buf()],
            cache_dir.path().to_path_buf(),
        );
        let turns = provider.parse(&db_path).unwrap();
        assert!(turns.is_empty());
    }

    // -----------------------------------------------------------------------
    // workspace_hash_from_path helper
    // -----------------------------------------------------------------------

    #[test]
    fn workspace_hash_extracted_from_path() {
        let path =
            PathBuf::from("/home/user/.config/Cursor/User/workspaceStorage/abc123/state.vscdb");
        assert_eq!(workspace_hash_from_path(&path), "abc123");
    }
}
