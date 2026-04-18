//! OpenCode provider — reads AI session history from OpenCode's SQLite databases.
//!
//! # Backend
//!
//! SQLite-backed. OpenCode stores sessions under a platform-specific data
//! directory. The exact schema is not publicly documented; this provider probes
//! for files ending in `.db` or `.sqlite` and inspects available tables.
//!
//! # Schema probing strategy
//!
//! For each candidate DB file the provider:
//! 1. Opens it read-only via `rusqlite`.
//! 2. Lists all table names via `sqlite_master`.
//! 3. Looks for tables whose names contain "session", "message", or "turn" (case-insensitive).
//! 4. If no recognizable tables are found it logs a `warn!` and returns empty — no panic.
//!
//! # Dedup key
//!
//! `{session_id}:{message_id}` — both columns are used when present.
//! Derived from Codeburn's opencode.ts reference implementation.
//!
//! # Platform paths
//!
//! - macOS: `~/Library/Application Support/opencode/`
//! - Linux / other: `~/.local/share/opencode/` (via `dirs::data_dir()`)
//! - Windows: `%APPDATA%/opencode/` (via `dirs::data_dir()`)

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::Result;
use rusqlite::{Connection, OpenFlags};
use tracing::warn;

use crate::models::Turn;
use crate::pricing;
use crate::scanner::provider::{Provider, SessionSource};

/// Provider slug stored in `turns.provider`.
pub const PROVIDER_OPENCODE: &str = "opencode";

// ---------------------------------------------------------------------------
// Provider struct
// ---------------------------------------------------------------------------

pub struct OpenCodeProvider {
    pub dirs: Vec<PathBuf>,
}

impl OpenCodeProvider {
    /// Construct with the platform-default OpenCode data directories.
    pub fn new() -> Self {
        let dirs = Self::default_dirs();
        Self { dirs }
    }

    fn default_dirs() -> Vec<PathBuf> {
        let mut dirs = Vec::new();

        // Primary: platform data dir / opencode
        if let Some(data_dir) = dirs::data_dir() {
            dirs.push(data_dir.join("opencode"));
        }

        // macOS also has Application Support as an alias — include both so we
        // don't miss installs that pre-date the dirs::data_dir() path.
        #[cfg(target_os = "macos")]
        if let Some(home) = dirs::home_dir() {
            let app_support = home
                .join("Library")
                .join("Application Support")
                .join("opencode");
            if !dirs.contains(&app_support) {
                dirs.push(app_support);
            }
        }

        dirs
    }

    /// Construct with explicit discovery directories (used in tests).
    #[cfg(test)]
    pub fn new_with_dirs(dirs: Vec<PathBuf>) -> Self {
        Self { dirs }
    }
}

impl Default for OpenCodeProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for OpenCodeProvider {
    fn name(&self) -> &'static str {
        PROVIDER_OPENCODE
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
                if path
                    .extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|ext| ext == "db" || ext == "sqlite")
                {
                    sources.push(SessionSource {
                        path: path.to_path_buf(),
                        provider_name: self.name(),
                    });
                }
            }
        }
        Ok(sources)
    }

    fn parse(&self, path: &Path) -> Result<Vec<Turn>> {
        Ok(parse_opencode_db(path))
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Open an OpenCode SQLite DB and extract turns.
///
/// All recoverable errors are logged at `warn!` and produce an empty return.
/// The function never panics.
fn parse_opencode_db(path: &Path) -> Vec<Turn> {
    let conn = match Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(c) => c,
        Err(e) => {
            warn!("opencode: cannot open {}: {}", path.display(), e);
            return Vec::new();
        }
    };

    // Enumerate tables present in this DB.
    let tables = list_tables(&conn);
    if tables.is_empty() {
        warn!("opencode: no tables found in {} — skipping", path.display());
        return Vec::new();
    }

    // Look for any table whose name suggests session/message/turn data.
    let session_table = tables
        .iter()
        .find(|t| {
            let lower = t.to_lowercase();
            lower.contains("session") || lower.contains("message") || lower.contains("turn")
        })
        .cloned();

    let Some(table_name) = session_table else {
        warn!(
            "opencode: no recognizable session/message/turn tables in {} — tables found: {:?}",
            path.display(),
            tables
        );
        return Vec::new();
    };

    // Introspect columns available in the chosen table.
    let columns = list_columns(&conn, &table_name);

    extract_turns_from_table(&conn, path, &table_name, &columns)
}

/// List all table names in the database.
fn list_tables(conn: &Connection) -> Vec<String> {
    let mut stmt = match conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'")
    {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    stmt.query_map([], |row| row.get::<_, String>(0))
        .ok()
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default()
}

/// List column names for a given table.
fn list_columns(conn: &Connection, table_name: &str) -> HashSet<String> {
    let query = format!("PRAGMA table_info({table_name})");
    let mut stmt = match conn.prepare(&query) {
        Ok(s) => s,
        Err(_) => return HashSet::new(),
    };

    stmt.query_map([], |row| row.get::<_, String>(1))
        .ok()
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default()
}

/// Extract Turn records from a probed table using whatever columns are available.
fn extract_turns_from_table(
    conn: &Connection,
    source_path: &Path,
    table_name: &str,
    columns: &HashSet<String>,
) -> Vec<Turn> {
    // Build the SELECT list based on what columns exist.
    let has = |col: &str| columns.contains(col);

    // We need at least something to identify a session and some token data.
    // Candidate session columns (in preference order).
    let session_col = ["session_id", "sessionId", "session", "id"]
        .iter()
        .find(|c| has(c))
        .copied();

    let message_col = ["message_id", "messageId", "id", "request_id", "requestId"]
        .iter()
        .find(|c| has(c))
        .copied();

    let input_col = [
        "input_tokens",
        "inputTokens",
        "prompt_tokens",
        "promptTokens",
    ]
    .iter()
    .find(|c| has(c))
    .copied();

    let output_col = [
        "output_tokens",
        "outputTokens",
        "completion_tokens",
        "completionTokens",
    ]
    .iter()
    .find(|c| has(c))
    .copied();

    let model_col = ["model", "model_id", "modelId"]
        .iter()
        .find(|c| has(c))
        .copied();

    let ts_col = ["timestamp", "created_at", "createdAt", "time", "updated_at"]
        .iter()
        .find(|c| has(c))
        .copied();

    // If there are no token columns at all, this table doesn't carry usage data.
    if input_col.is_none() && output_col.is_none() {
        warn!(
            "opencode: table '{}' in {} has no token count columns — skipping",
            table_name,
            source_path.display()
        );
        return Vec::new();
    }

    // Build a minimal SELECT. Always include rowid as fallback dedup key.
    let mut select_cols = vec!["rowid"];
    if let Some(c) = session_col {
        select_cols.push(c);
    }
    if let Some(c) = message_col {
        // avoid duplicating session_col if they're the same column name
        if !select_cols.contains(&c) {
            select_cols.push(c);
        }
    }
    if let Some(c) = input_col {
        select_cols.push(c);
    }
    if let Some(c) = output_col {
        select_cols.push(c);
    }
    if let Some(c) = model_col {
        select_cols.push(c);
    }
    if let Some(c) = ts_col {
        select_cols.push(c);
    }

    let sql = format!("SELECT {} FROM {}", select_cols.join(", "), table_name);
    let mut stmt = match conn.prepare(&sql) {
        Ok(s) => s,
        Err(e) => {
            warn!(
                "opencode: failed to prepare query for {}: {}",
                source_path.display(),
                e
            );
            return Vec::new();
        }
    };

    // Map column names to positional indices.
    // `find_col` is a named function (not a closure) so clippy's
    // redundant_closure lint is satisfied when used with `and_then`.
    fn find_col(name: &str, cols: &[&str]) -> Option<usize> {
        cols.iter().position(|c| *c == name)
    }
    let rowid_idx = 0usize;
    let session_idx = session_col.and_then(|c| find_col(c, &select_cols));
    let message_idx = message_col
        .and_then(|c| find_col(c, &select_cols))
        .filter(|&i| {
            // Don't double-use the session column as the message column.
            session_idx.map(|si| si != i).unwrap_or(true)
        });
    let input_idx = input_col.and_then(|c| find_col(c, &select_cols));
    let output_idx = output_col.and_then(|c| find_col(c, &select_cols));
    let model_idx = model_col.and_then(|c| find_col(c, &select_cols));
    let ts_idx = ts_col.and_then(|c| find_col(c, &select_cols));

    let db_stem = source_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let source_str = source_path.to_string_lossy().to_string();

    let mut turns = Vec::new();
    let mut seen_dedup: HashSet<String> = HashSet::new();

    // Use query() + manual row loop so we can mutate `turns` and `seen_dedup`
    // directly. query_map() requires a closure that returns a value per row and
    // the resulting MappedRows iterator must be fully consumed — using it with
    // mutable captures of external Vecs is awkward; the manual loop is cleaner.
    let mut rows = match stmt.query([]) {
        Ok(r) => r,
        Err(e) => {
            warn!("opencode: failed to query {}: {}", source_path.display(), e);
            return Vec::new();
        }
    };

    loop {
        let row = match rows.next() {
            Ok(Some(r)) => r,
            Ok(None) => break,
            Err(e) => {
                warn!("opencode: row error in {}: {}", source_path.display(), e);
                break;
            }
        };

        let rowid: i64 = row.get(rowid_idx).unwrap_or(0);

        let session_raw: String = session_idx
            .and_then(|i| row.get::<_, String>(i).ok())
            .unwrap_or_else(|| db_stem.clone());

        let message_raw: String = message_idx
            .and_then(|i| row.get::<_, String>(i).ok())
            .unwrap_or_else(|| rowid.to_string());

        let dedup_key = format!("{session_raw}:{message_raw}");
        if seen_dedup.contains(&dedup_key) {
            continue;
        }
        seen_dedup.insert(dedup_key);

        let input_tokens: i64 = input_idx
            .and_then(|i| row.get::<_, i64>(i).ok())
            .unwrap_or(0);
        let output_tokens: i64 = output_idx
            .and_then(|i| row.get::<_, i64>(i).ok())
            .unwrap_or(0);

        if input_tokens == 0 && output_tokens == 0 {
            continue;
        }

        let model: String = model_idx
            .and_then(|i| row.get::<_, String>(i).ok())
            .unwrap_or_else(|| "unknown".to_string());

        let timestamp: String = ts_idx
            .and_then(|i| row.get::<_, String>(i).ok())
            .unwrap_or_default();

        let session_id = format!("opencode:{session_raw}");
        let estimate = pricing::estimate_cost(&model, input_tokens, output_tokens, 0, 0);

        turns.push(Turn {
            session_id,
            provider: PROVIDER_OPENCODE.to_string(),
            timestamp,
            model,
            input_tokens,
            output_tokens,
            cache_read_tokens: 0,
            cache_creation_tokens: 0,
            reasoning_output_tokens: 0,
            estimated_cost_nanos: estimate.estimated_cost_nanos,
            tool_name: None,
            cwd: String::new(),
            message_id: message_raw,
            service_tier: None,
            inference_geo: None,
            is_subagent: false,
            agent_id: None,
            source_path: source_str.clone(),
            version: None,
            pricing_version: estimate.pricing_version,
            pricing_model: estimate.pricing_model,
            billing_mode: "estimated_local".to_string(),
            cost_confidence: estimate.cost_confidence,
            category: String::new(),
            all_tools: Vec::new(),
            tool_use_ids: Vec::new(),
            tool_inputs: Vec::new(),
        });
    }

    turns
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use tempfile::TempDir;

    fn create_sessions_db(path: &Path) -> Connection {
        let conn = Connection::open(path).unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                message_id TEXT,
                model TEXT,
                input_tokens INTEGER NOT NULL DEFAULT 0,
                output_tokens INTEGER NOT NULL DEFAULT 0,
                timestamp TEXT
            );",
        )
        .unwrap();
        conn
    }

    fn insert_session(
        conn: &Connection,
        id: &str,
        message_id: &str,
        model: &str,
        input: i64,
        output: i64,
    ) {
        conn.execute(
            "INSERT INTO sessions (id, message_id, model, input_tokens, output_tokens, timestamp) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![id, message_id, model, input, output, "2026-04-17T10:00:00Z"],
        )
        .unwrap();
    }

    // -----------------------------------------------------------------------
    // name()
    // -----------------------------------------------------------------------

    #[test]
    fn opencode_provider_name() {
        assert_eq!(OpenCodeProvider::new_with_dirs(vec![]).name(), "opencode");
    }

    // -----------------------------------------------------------------------
    // discover_sessions
    // -----------------------------------------------------------------------

    #[test]
    fn opencode_discover_sessions_finds_db_files() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("opencode.db");
        let _conn = create_sessions_db(&db_path);

        let provider = OpenCodeProvider::new_with_dirs(vec![dir.path().to_path_buf()]);
        let sources = provider.discover_sessions().unwrap();
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].path, db_path);
        assert_eq!(sources[0].provider_name, "opencode");
    }

    #[test]
    fn opencode_discover_sessions_empty_dir() {
        let dir = TempDir::new().unwrap();
        let provider = OpenCodeProvider::new_with_dirs(vec![dir.path().to_path_buf()]);
        let sources = provider.discover_sessions().unwrap();
        assert!(sources.is_empty());
    }

    #[test]
    fn opencode_discover_sessions_nonexistent_dir() {
        let provider =
            OpenCodeProvider::new_with_dirs(vec![PathBuf::from("/nonexistent/opencode/xyz")]);
        let sources = provider.discover_sessions().unwrap();
        assert!(sources.is_empty());
    }

    // -----------------------------------------------------------------------
    // parse: DB with no recognizable tables -> Ok(empty)
    // -----------------------------------------------------------------------

    #[test]
    fn opencode_parse_no_recognizable_tables_returns_empty() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("weird.db");
        let conn = Connection::open(&db_path).unwrap();
        conn.execute_batch("CREATE TABLE unrelated (x INTEGER);")
            .unwrap();
        drop(conn);

        let provider = OpenCodeProvider::new_with_dirs(vec![dir.path().to_path_buf()]);
        let result = provider.parse(&db_path);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    // -----------------------------------------------------------------------
    // parse: seeded sessions table -> turns with correct provider/session_id
    // -----------------------------------------------------------------------

    #[test]
    fn opencode_parse_seeded_db_produces_turns() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("opencode.db");
        let conn = create_sessions_db(&db_path);
        insert_session(&conn, "sess-abc", "msg-001", "gpt-4o", 200, 100);
        drop(conn);

        let provider = OpenCodeProvider::new_with_dirs(vec![dir.path().to_path_buf()]);
        let turns = provider.parse(&db_path).unwrap();
        assert!(
            !turns.is_empty(),
            "expected at least one Turn from seeded DB"
        );
        let t = &turns[0];
        assert_eq!(t.provider, "opencode");
        assert!(
            t.session_id.starts_with("opencode:"),
            "session_id should be prefixed with 'opencode:'"
        );
        assert_eq!(t.input_tokens, 200);
        assert_eq!(t.output_tokens, 100);
    }

    // -----------------------------------------------------------------------
    // parse: invalid SQLite bytes -> Ok(empty), no panic
    // -----------------------------------------------------------------------

    #[test]
    fn opencode_parse_invalid_sqlite_returns_empty() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("bad.db");
        std::fs::write(&db_path, b"this is not sqlite").unwrap();

        let provider = OpenCodeProvider::new_with_dirs(vec![dir.path().to_path_buf()]);
        let result = provider.parse(&db_path);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
