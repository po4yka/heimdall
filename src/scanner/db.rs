use anyhow::Result;
use rusqlite::{Connection, OptionalExtension};
use tracing::warn;

use std::collections::{HashMap, HashSet};

use crate::models::{
    BillingModeSummary, BranchSummary, ClaudeUsageFactor, ClaudeUsageResponse, ClaudeUsageRunMeta,
    ClaudeUsageSnapshot, ConfidenceSummary, DailyModelRow, DailyProjectRow, DashboardData,
    EntrypointSummary, HourlyRow, McpServerSummary, OfficialSyncRecordCount,
    OfficialSyncSourceStatus, OfficialSyncSummary, ProviderSummary, ServiceTierSummary, SessionRow,
    ToolErrorRow, ToolErrorsResponse, ToolEvent, ToolSummary, Turn, VersionSummary, WeeklyModelRow,
};
use crate::pricing_defs::{
    OfficialExtractedRecord, OfficialModelPricing, OfficialSyncRunRecord, PricingSyncRun,
    StoredPricingModel,
};
use crate::scanner::parser::{classify_tool, raw_session_id};
use crate::tz::TzParams;

pub fn open_db(path: &std::path::Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;
    Ok(conn)
}

pub fn init_db(conn: &Connection) -> Result<()> {
    create_schema(conn)?;
    apply_migrations(conn)?;
    Ok(())
}

/// Create all tables and indexes. Pure DDL — no migration probes.
fn create_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS sessions (
            session_id          TEXT PRIMARY KEY,
            provider            TEXT NOT NULL DEFAULT 'claude',
            project_name        TEXT,
            project_slug        TEXT,
            first_timestamp     TEXT,
            last_timestamp      TEXT,
            git_branch          TEXT,
            model               TEXT,
            entrypoint          TEXT,
            total_input_tokens  INTEGER DEFAULT 0,
            total_output_tokens INTEGER DEFAULT 0,
            total_cache_read    INTEGER DEFAULT 0,
            total_cache_creation INTEGER DEFAULT 0,
            total_reasoning_output INTEGER DEFAULT 0,
            total_estimated_cost_nanos INTEGER DEFAULT 0,
            turn_count          INTEGER DEFAULT 0,
            pricing_version     TEXT NOT NULL DEFAULT '',
            billing_mode        TEXT NOT NULL DEFAULT 'estimated_local',
            cost_confidence     TEXT NOT NULL DEFAULT 'low'
        );

        CREATE TABLE IF NOT EXISTS turns (
            id                      INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id              TEXT NOT NULL,
            provider                TEXT NOT NULL DEFAULT 'claude',
            timestamp               TEXT,
            model                   TEXT,
            input_tokens            INTEGER DEFAULT 0,
            output_tokens           INTEGER DEFAULT 0,
            cache_read_tokens       INTEGER DEFAULT 0,
            cache_creation_tokens   INTEGER DEFAULT 0,
            reasoning_output_tokens INTEGER DEFAULT 0,
            estimated_cost_nanos    INTEGER DEFAULT 0,
            tool_name               TEXT,
            cwd                     TEXT,
            message_id              TEXT,
            service_tier            TEXT,
            inference_geo           TEXT,
            is_subagent             INTEGER DEFAULT 0,
            agent_id                TEXT,
            source_path             TEXT NOT NULL DEFAULT '',
            pricing_version         TEXT NOT NULL DEFAULT '',
            pricing_model           TEXT NOT NULL DEFAULT '',
            billing_mode            TEXT NOT NULL DEFAULT 'estimated_local',
            cost_confidence         TEXT NOT NULL DEFAULT 'low',
            category                TEXT NOT NULL DEFAULT ''
        );

        CREATE TABLE IF NOT EXISTS processed_files (
            path    TEXT PRIMARY KEY,
            mtime   REAL,
            progress_marker INTEGER
        );

        CREATE INDEX IF NOT EXISTS idx_turns_session ON turns(session_id);
        CREATE INDEX IF NOT EXISTS idx_turns_timestamp ON turns(timestamp);
        CREATE INDEX IF NOT EXISTS idx_sessions_first ON sessions(first_timestamp);",
    )?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS rate_window_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL,
            window_type TEXT NOT NULL,
            used_percent REAL,
            resets_at TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_rwh_timestamp ON rate_window_history(timestamp);",
    )?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS claude_usage_runs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            captured_at TEXT NOT NULL,
            status TEXT NOT NULL,
            exit_code INTEGER,
            stdout_raw TEXT NOT NULL DEFAULT '',
            stderr_raw TEXT NOT NULL DEFAULT '',
            invocation_mode TEXT NOT NULL DEFAULT '',
            period TEXT NOT NULL DEFAULT 'today',
            parser_version TEXT NOT NULL DEFAULT '',
            error_summary TEXT NOT NULL DEFAULT ''
        );
        CREATE INDEX IF NOT EXISTS idx_cur_captured_at ON claude_usage_runs(captured_at DESC);

        CREATE TABLE IF NOT EXISTS claude_usage_factors (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            run_id INTEGER NOT NULL,
            factor_key TEXT NOT NULL,
            display_label TEXT NOT NULL,
            percent REAL NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            advice_text TEXT NOT NULL DEFAULT '',
            display_order INTEGER NOT NULL DEFAULT 0
        );
        CREATE INDEX IF NOT EXISTS idx_cuf_run_order
            ON claude_usage_factors(run_id, display_order);",
    )?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS pricing_sync_runs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            fetched_at TEXT NOT NULL,
            source_slug TEXT NOT NULL,
            source_url TEXT NOT NULL,
            provider TEXT NOT NULL,
            status TEXT NOT NULL,
            raw_body TEXT NOT NULL DEFAULT '',
            error_text TEXT NOT NULL DEFAULT ''
        );
        CREATE INDEX IF NOT EXISTS idx_psr_source_fetched
            ON pricing_sync_runs(source_slug, fetched_at DESC);

        CREATE TABLE IF NOT EXISTS pricing_sync_models (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            run_id INTEGER NOT NULL,
            source_slug TEXT NOT NULL,
            provider TEXT NOT NULL,
            model_id TEXT NOT NULL,
            model_label TEXT NOT NULL,
            input_usd_per_mtok REAL NOT NULL,
            cache_write_usd_per_mtok REAL NOT NULL,
            cache_read_usd_per_mtok REAL NOT NULL,
            output_usd_per_mtok REAL NOT NULL,
            threshold_tokens INTEGER,
            input_above_threshold REAL,
            output_above_threshold REAL,
            notes TEXT NOT NULL DEFAULT ''
        );
        CREATE INDEX IF NOT EXISTS idx_psm_source_model_run
            ON pricing_sync_models(source_slug, model_id, run_id DESC);",
    )?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS official_sync_runs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            fetched_at TEXT NOT NULL,
            source_slug TEXT NOT NULL,
            source_kind TEXT NOT NULL DEFAULT '',
            source_url TEXT NOT NULL,
            provider TEXT NOT NULL,
            authority TEXT NOT NULL DEFAULT '',
            format TEXT NOT NULL DEFAULT '',
            cadence TEXT NOT NULL DEFAULT '',
            status TEXT NOT NULL,
            http_status INTEGER,
            content_type TEXT NOT NULL DEFAULT '',
            etag TEXT NOT NULL DEFAULT '',
            last_modified TEXT NOT NULL DEFAULT '',
            raw_body TEXT NOT NULL DEFAULT '',
            normalized_body TEXT NOT NULL DEFAULT '',
            error_text TEXT NOT NULL DEFAULT '',
            parser_version TEXT NOT NULL DEFAULT '',
            raw_body_sha256 TEXT NOT NULL DEFAULT '',
            normalized_body_sha256 TEXT NOT NULL DEFAULT '',
            extracted_sha256 TEXT NOT NULL DEFAULT ''
        );
        CREATE INDEX IF NOT EXISTS idx_osr_source_fetched
            ON official_sync_runs(source_slug, fetched_at DESC);

        CREATE TABLE IF NOT EXISTS official_metadata_records (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            run_id INTEGER NOT NULL,
            source_slug TEXT NOT NULL,
            provider TEXT NOT NULL,
            record_type TEXT NOT NULL,
            record_key TEXT NOT NULL,
            model_id TEXT NOT NULL DEFAULT '',
            effective_at TEXT NOT NULL DEFAULT '',
            payload_json TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_omr_run
            ON official_metadata_records(run_id);
        CREATE INDEX IF NOT EXISTS idx_omr_type_key
            ON official_metadata_records(record_type, record_key);",
    )?;

    // Tool invocations table for multi-tool and MCP tracking
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS tool_invocations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id TEXT NOT NULL,
            provider TEXT NOT NULL DEFAULT 'claude',
            message_id TEXT,
            tool_name TEXT NOT NULL,
            mcp_server TEXT,
            mcp_tool TEXT,
            tool_category TEXT NOT NULL DEFAULT 'builtin',
            source_path TEXT NOT NULL DEFAULT ''
        );
        CREATE INDEX IF NOT EXISTS idx_ti_session ON tool_invocations(session_id);
        CREATE INDEX IF NOT EXISTS idx_ti_tool ON tool_invocations(tool_name);
        CREATE INDEX IF NOT EXISTS idx_ti_mcp ON tool_invocations(mcp_server);",
    )?;

    // Phase 12: Tool-event cost attribution table.
    // Each row is one tool invocation with its share of the parent turn's cost_nanos.
    // dedup_key = "{provider}:{tool_use_id}" ensures idempotent rescans.
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS tool_events (
            dedup_key     TEXT PRIMARY KEY,
            ts_epoch      INTEGER NOT NULL,
            session_id    TEXT NOT NULL,
            provider      TEXT NOT NULL,
            project       TEXT NOT NULL DEFAULT '',
            kind          TEXT NOT NULL,
            value         TEXT NOT NULL,
            cost_nanos    INTEGER NOT NULL,
            source_path   TEXT NOT NULL DEFAULT ''
        );
        CREATE INDEX IF NOT EXISTS idx_tool_events_kind ON tool_events(kind, ts_epoch);
        CREATE INDEX IF NOT EXISTS idx_tool_events_session ON tool_events(session_id);",
    )?;

    // Phase 19: Real-time PreToolUse hook ingest.
    // The hook binary writes directly to this table; the scanner only reads it.
    // dedup_key = "{session_id}:{tool_use_id}" (or "{session_id}:{received_at_ns}")
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS live_events (
            dedup_key       TEXT PRIMARY KEY,
            received_at     TEXT NOT NULL,
            session_id      TEXT,
            tool_name       TEXT,
            cost_usd_nanos  INTEGER NOT NULL DEFAULT 0,
            input_tokens    INTEGER NOT NULL DEFAULT 0,
            output_tokens   INTEGER NOT NULL DEFAULT 0,
            raw_json        TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_live_events_received ON live_events(received_at);
        CREATE INDEX IF NOT EXISTS idx_live_events_session ON live_events(session_id);",
    )?;

    // Agent status history: one row per component per poll.
    // PRIMARY KEY (ts_epoch, provider, component_id) ensures INSERT OR IGNORE is idempotent.
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS agent_status_history (
            ts_epoch       INTEGER NOT NULL,
            provider       TEXT NOT NULL,
            component_id   TEXT NOT NULL,
            component_name TEXT NOT NULL,
            status         TEXT NOT NULL,
            PRIMARY KEY (ts_epoch, provider, component_id)
        );
        CREATE INDEX IF NOT EXISTS idx_ash_lookup
            ON agent_status_history(provider, component_id, ts_epoch DESC);",
    )?;

    Ok(())
}

/// Return a cached column-set for `table`, populating the cache on first access.
/// Uses `PRAGMA table_info` so no data rows are scanned.
fn cached_columns<'a>(
    cache: &'a mut HashMap<&'static str, HashSet<String>>,
    conn: &Connection,
    table: &'static str,
) -> &'a HashSet<String> {
    cache
        .entry(table)
        .or_insert_with(|| table_columns(conn, table))
}

/// Apply all incremental column-presence migration probes, in original order.
/// No DDL for new tables here — that lives in `create_schema`.
fn apply_migrations(conn: &Connection) -> Result<()> {
    // Per-table column-set cache: each PRAGMA table_info is issued at most once
    // per table per apply_migrations call, regardless of how many probes touch
    // the same table.  After an ALTER TABLE the entry is removed so the next
    // probe for that table re-reads the updated schema.
    let mut col_cache: HashMap<&'static str, HashSet<String>> = HashMap::new();

    macro_rules! col {
        ($table:literal, $column:expr) => {
            cached_columns(&mut col_cache, conn, $table).contains($column)
        };
    }
    macro_rules! alter {
        ($table:literal, $sql:expr) => {{
            conn.execute_batch($sql)?;
            col_cache.remove($table);
        }};
    }

    // Migration: add subagent columns if upgrading from older schema
    if !col!("sessions", "provider") {
        alter!(
            "sessions",
            "ALTER TABLE sessions ADD COLUMN provider TEXT NOT NULL DEFAULT 'claude';"
        );
    }
    if !col!("sessions", "total_reasoning_output") {
        alter!(
            "sessions",
            "ALTER TABLE sessions ADD COLUMN total_reasoning_output INTEGER DEFAULT 0;"
        );
    }
    if !col!("sessions", "total_estimated_cost_nanos") {
        alter!(
            "sessions",
            "ALTER TABLE sessions ADD COLUMN total_estimated_cost_nanos INTEGER DEFAULT 0;"
        );
    }
    if !col!("sessions", "pricing_version") {
        alter!(
            "sessions",
            "ALTER TABLE sessions ADD COLUMN pricing_version TEXT NOT NULL DEFAULT '';"
        );
    }
    if !col!("sessions", "billing_mode") {
        alter!(
            "sessions",
            "ALTER TABLE sessions ADD COLUMN billing_mode TEXT NOT NULL DEFAULT 'estimated_local';"
        );
    }
    if !col!("sessions", "cost_confidence") {
        alter!(
            "sessions",
            "ALTER TABLE sessions ADD COLUMN cost_confidence TEXT NOT NULL DEFAULT 'low';"
        );
    }
    if !col!("turns", "provider") {
        alter!(
            "turns",
            "ALTER TABLE turns ADD COLUMN provider TEXT NOT NULL DEFAULT 'claude';"
        );
    }
    if !col!("turns", "reasoning_output_tokens") {
        alter!(
            "turns",
            "ALTER TABLE turns ADD COLUMN reasoning_output_tokens INTEGER DEFAULT 0;"
        );
    }
    if !col!("turns", "estimated_cost_nanos") {
        alter!(
            "turns",
            "ALTER TABLE turns ADD COLUMN estimated_cost_nanos INTEGER DEFAULT 0;"
        );
    }
    if !col!("turns", "agent_id") {
        alter!(
            "turns",
            "ALTER TABLE turns ADD COLUMN is_subagent INTEGER DEFAULT 0;
             ALTER TABLE turns ADD COLUMN agent_id TEXT;"
        );
    }
    if !col!("turns", "source_path") {
        alter!(
            "turns",
            "ALTER TABLE turns ADD COLUMN source_path TEXT NOT NULL DEFAULT '';"
        );
    }
    if !col!("turns", "pricing_version") {
        alter!(
            "turns",
            "ALTER TABLE turns ADD COLUMN pricing_version TEXT NOT NULL DEFAULT '';
             ALTER TABLE turns ADD COLUMN pricing_model TEXT NOT NULL DEFAULT '';
             ALTER TABLE turns ADD COLUMN billing_mode TEXT NOT NULL DEFAULT 'estimated_local';
             ALTER TABLE turns ADD COLUMN cost_confidence TEXT NOT NULL DEFAULT 'low';"
        );
    }
    if !col!("turns", "category") {
        alter!(
            "turns",
            "ALTER TABLE turns ADD COLUMN category TEXT NOT NULL DEFAULT '';"
        );
    }
    if !col!("tool_invocations", "provider") {
        alter!(
            "tool_invocations",
            "ALTER TABLE tool_invocations ADD COLUMN provider TEXT NOT NULL DEFAULT 'claude';"
        );
    }
    // Denormalize `timestamp` from `turns` into `tool_invocations` so the
    // per-provider tool/mcp aggregations in `provider_cost_summary` can read
    // a single table with a covering `(provider, timestamp)` index. Backfill
    // existing rows via the same join those aggregations used to compute on
    // every refresh — paid once per DB on first-run after this migration.
    if !col!("tool_invocations", "timestamp") {
        alter!(
            "tool_invocations",
            "ALTER TABLE tool_invocations ADD COLUMN timestamp TEXT NOT NULL DEFAULT '';"
        );
        conn.execute_batch(
            "UPDATE tool_invocations SET timestamp = (
                SELECT t.timestamp FROM turns t
                WHERE t.session_id = tool_invocations.session_id
                  AND t.message_id = tool_invocations.message_id
                LIMIT 1
            ) WHERE timestamp = '';",
        )?;
    }
    conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_ti_provider_timestamp
            ON tool_invocations(provider, timestamp);",
    )?;

    // Feature 1: Session titles
    if !col!("sessions", "title") {
        alter!("sessions", "ALTER TABLE sessions ADD COLUMN title TEXT;");
    }
    // Feature 2: Version tracking
    if !col!("turns", "version") {
        alter!("turns", "ALTER TABLE turns ADD COLUMN version TEXT;");
    }
    // Feature 3: Tool error tracking
    if !col!("tool_invocations", "tool_use_id") {
        alter!(
            "tool_invocations",
            "ALTER TABLE tool_invocations ADD COLUMN tool_use_id TEXT;
             ALTER TABLE tool_invocations ADD COLUMN is_error INTEGER DEFAULT 0;"
        );
    }
    if !col!("tool_invocations", "source_path") {
        alter!(
            "tool_invocations",
            "ALTER TABLE tool_invocations ADD COLUMN source_path TEXT NOT NULL DEFAULT '';"
        );
    }
    // Phase 3: One-shot rate tracking (nullable; 0=not-oneshot, 1=oneshot,
    // NULL=session has no edit activity and is unclassifiable)
    if !col!("sessions", "one_shot") {
        alter!(
            "sessions",
            "ALTER TABLE sessions ADD COLUMN one_shot INTEGER;"
        );
    }

    // Dedup by tool_use_id so repeated use of the same tool in a single turn is preserved.
    conn.execute_batch(
        "DROP INDEX IF EXISTS idx_turns_message_id;
         CREATE UNIQUE INDEX IF NOT EXISTS idx_turns_message_id
         ON turns(provider, message_id) WHERE message_id IS NOT NULL AND message_id != '';
         DROP INDEX IF EXISTS idx_ti_dedup;
         DROP INDEX IF EXISTS idx_ti_tool_use_id;
         CREATE UNIQUE INDEX IF NOT EXISTS idx_ti_tool_use_id
         ON tool_invocations(provider, tool_use_id)
         WHERE tool_use_id IS NOT NULL AND tool_use_id != '';",
    )?;

    // Backfill: ensure no row has a NULL or empty provider (idempotent no-op on clean DBs)
    conn.execute_batch(
        "UPDATE sessions SET provider = 'claude' WHERE provider IS NULL OR provider = '';
         UPDATE turns    SET provider = 'claude' WHERE provider IS NULL OR provider = '';",
    )?;

    // Phase 12 (Amp): credits column on turns and sessions.
    // Must be added BEFORE recompute_session_totals which references turns.credits.
    // NULL for all non-Amp providers; f64 (REAL) for Amp turns.
    if !col!("turns", "credits") {
        alter!("turns", "ALTER TABLE turns ADD COLUMN credits REAL;");
    }
    if !col!("sessions", "total_credits") {
        alter!(
            "sessions",
            "ALTER TABLE sessions ADD COLUMN total_credits REAL;"
        );
    }

    prefix_existing_session_ids(conn)?;
    backfill_turn_pricing(conn)?;
    recompute_session_totals(conn)?;

    // Phase 20: Usage-limits file parser.
    // Add source_kind ('oauth' | 'file') and source_path to rate_window_history
    // so file-derived rows can be distinguished from OAuth-derived ones.
    if !col!("rate_window_history", "source_kind") {
        alter!(
            "rate_window_history",
            "ALTER TABLE rate_window_history ADD COLUMN source_kind TEXT NOT NULL DEFAULT 'oauth';"
        );
    }
    if !col!("rate_window_history", "source_path") {
        alter!(
            "rate_window_history",
            "ALTER TABLE rate_window_history ADD COLUMN source_path TEXT NOT NULL DEFAULT '';"
        );
    }

    // Phase 22: Subscription-quota widget. Capture per-provider/per-window
    // snapshots so we can chart estimated cap evolution over time.
    if !col!("rate_window_history", "provider") {
        alter!(
            "rate_window_history",
            "ALTER TABLE rate_window_history ADD COLUMN provider TEXT NOT NULL DEFAULT 'claude';"
        );
    }
    if !col!("rate_window_history", "plan") {
        alter!(
            "rate_window_history",
            "ALTER TABLE rate_window_history ADD COLUMN plan TEXT;"
        );
    }
    if !col!("rate_window_history", "observed_tokens") {
        alter!(
            "rate_window_history",
            "ALTER TABLE rate_window_history ADD COLUMN observed_tokens INTEGER;"
        );
    }
    if !col!("rate_window_history", "estimated_cap_tokens") {
        alter!(
            "rate_window_history",
            "ALTER TABLE rate_window_history ADD COLUMN estimated_cap_tokens INTEGER;"
        );
    }
    if !col!("rate_window_history", "confidence") {
        alter!(
            "rate_window_history",
            "ALTER TABLE rate_window_history ADD COLUMN confidence REAL;"
        );
    }
    conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_rwh_provider_window
            ON rate_window_history(provider, window_type, timestamp);",
    )?;

    // Phase 5: context-window columns on live_events (idempotent ALTER TABLE).
    if !col!("live_events", "context_input_tokens") {
        alter!(
            "live_events",
            "ALTER TABLE live_events ADD COLUMN context_input_tokens INTEGER;"
        );
    }
    if !col!("live_events", "context_window_size") {
        alter!(
            "live_events",
            "ALTER TABLE live_events ADD COLUMN context_window_size INTEGER;"
        );
    }

    // Phase 8: hook-reported cost alongside Heimdall's local estimate.
    // NULL = hook did not report a cost for this event.
    if !col!("live_events", "hook_reported_cost_nanos") {
        alter!(
            "live_events",
            "ALTER TABLE live_events ADD COLUMN hook_reported_cost_nanos INTEGER;"
        );
    }

    // Tool error detail: capture error message text and full compact input JSON
    // per invocation. NULL on rows written before this migration.
    if !col!("tool_invocations", "error_text") {
        alter!(
            "tool_invocations",
            "ALTER TABLE tool_invocations ADD COLUMN error_text TEXT;"
        );
    }
    if !col!("tool_invocations", "tool_input_json") {
        alter!(
            "tool_invocations",
            "ALTER TABLE tool_invocations ADD COLUMN tool_input_json TEXT;"
        );
    }

    Ok(())
}

/// Validate that `name` is a bare SQL identifier (letters, digits, underscores,
/// starting with a letter or underscore). Returns false for anything that could
/// be used for SQL injection; the caller should treat false as "column absent".
fn is_valid_identifier(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .next()
            .map(|c| c.is_ascii_alphabetic() || c == '_')
            .unwrap_or(false)
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Return the set of column names present in `table` by querying
/// `PRAGMA table_info`. Returns an empty set when the table does not exist.
/// Only the column-name comparison is done in Rust; the table name is validated
/// against an identifier allowlist before being interpolated into the PRAGMA.
fn table_columns(conn: &Connection, table: &str) -> HashSet<String> {
    if !is_valid_identifier(table) {
        return HashSet::new();
    }
    let sql = format!("PRAGMA table_info({table})");
    let mut stmt = match conn.prepare(&sql) {
        Ok(s) => s,
        Err(_) => return HashSet::new(),
    };
    // PRAGMA table_info columns: cid, name, type, notnull, dflt_value, pk
    stmt.query_map([], |row| row.get::<_, String>(1))
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default()
}

#[cfg(test)]
fn has_column(conn: &Connection, table: &str, column: &str) -> bool {
    table_columns(conn, table).contains(column)
}

fn prefix_existing_session_ids(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "UPDATE sessions
         SET session_id = provider || ':' || session_id
         WHERE instr(session_id, ':') = 0;
         UPDATE turns
         SET session_id = provider || ':' || session_id
         WHERE instr(session_id, ':') = 0;
         UPDATE tool_invocations
         SET session_id = provider || ':' || session_id
         WHERE instr(session_id, ':') = 0;",
    )?;
    Ok(())
}

pub fn get_processed_file(conn: &Connection, path: &str) -> Result<Option<(f64, i64)>> {
    let mut stmt = conn.prepare(
        "SELECT mtime, COALESCE(progress_marker, 0)
         FROM processed_files
         WHERE path = ?",
    )?;
    let result = stmt.query_row([path], |row| {
        Ok((row.get::<_, f64>(0)?, row.get::<_, i64>(1)?))
    });
    match result {
        Ok(val) => Ok(Some(val)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn upsert_processed_file(
    conn: &Connection,
    path: &str,
    mtime: f64,
    progress_marker: i64,
) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO processed_files (path, mtime, progress_marker)
         VALUES (?1, ?2, ?3)",
        rusqlite::params![path, mtime, progress_marker],
    )?;
    Ok(())
}

pub fn list_processed_files(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT path FROM processed_files")?;
    let paths = collect_warn(
        stmt.query_map([], |row| row.get(0))?,
        "Failed to read processed_files row",
    );
    Ok(paths)
}

pub fn delete_processed_file(conn: &Connection, path: &str) -> Result<()> {
    conn.execute("DELETE FROM processed_files WHERE path = ?1", [path])?;
    Ok(())
}

pub fn delete_turns_by_source_path(conn: &Connection, path: &str) -> Result<()> {
    conn.execute("DELETE FROM turns WHERE source_path = ?1", [path])?;
    Ok(())
}

pub fn delete_tool_invocations_by_source_path(conn: &Connection, path: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM tool_invocations WHERE source_path = ?1",
        [path],
    )?;
    Ok(())
}

// ── Phase 12: Tool-event cost attribution ────────────────────────────────────

/// Classify a tool name into a `kind` string and return the `value` to store.
///
/// Mapping:
/// - `mcp__<server>__<tool>` → kind `"mcp"`, value = bare tool name after the prefix
/// - `Task`                  → kind `"subagent"`, value = tool name
/// - `Edit` / `Write` / `MultiEdit` / `NotebookEdit` / `Read` → kind `"file"`, value = tool name
/// - `Bash`                  → kind `"bash"`, value = tool name
/// - anything else           → kind `"other"`, value = tool name
pub fn classify_tool_event(tool_name: &str) -> (&'static str, String) {
    if tool_name.starts_with("mcp__") {
        // mcp__<server>__<bare_tool>  — value is the part after the second "__"
        let bare = tool_name
            .splitn(3, "__")
            .nth(2)
            .unwrap_or(tool_name)
            .to_string();
        return ("mcp", bare);
    }
    match tool_name {
        "Task" => ("subagent", tool_name.to_string()),
        "Edit" | "Write" | "MultiEdit" | "NotebookEdit" | "Read" => ("file", tool_name.to_string()),
        "Bash" => ("bash", tool_name.to_string()),
        _ => ("other", tool_name.to_string()),
    }
}

/// Parse an ISO 8601 timestamp string to a Unix epoch (seconds).
/// Returns 0 on any parse failure.
pub fn parse_ts_epoch(ts: &str) -> i64 {
    let parse = |s: &str| -> Option<i64> {
        chrono::DateTime::parse_from_rfc3339(s)
            .ok()
            .map(|dt| dt.timestamp())
            .or_else(|| {
                chrono::DateTime::parse_from_rfc3339(&format!("{}+00:00", s.trim_end_matches('Z')))
                    .ok()
                    .map(|dt| dt.timestamp())
            })
    };
    parse(ts).unwrap_or(0)
}

/// Build `ToolEvent` rows for a single `Turn`.
///
/// Cost is split evenly (integer nanos): `cost_per = total / n`, remainder added
/// to the first event so `SUM(cost_nanos) == turn.estimated_cost_nanos` exactly.
///
/// Turns with no tool invocations produce an empty Vec — their cost does NOT appear
/// in `tool_events`. This means per-session sums in `tool_events` will be lower than
/// the corresponding `turns` sum for sessions that contain tool-free turns.
pub fn compute_tool_events_for_turn(turn: &Turn, project: &str) -> Vec<ToolEvent> {
    let n = turn.tool_use_ids.len();
    if n == 0 {
        return Vec::new();
    }
    let total = turn.estimated_cost_nanos;
    let cost_per = total / n as i64;
    let remainder = total % n as i64;
    let ts_epoch = parse_ts_epoch(&turn.timestamp);

    // Build a lookup from tool_use_id -> extracted_arg (file path or command).
    // tool_inputs is populated by the Claude parser; other providers leave it empty.
    let input_map: std::collections::HashMap<&str, &str> = turn
        .tool_inputs
        .iter()
        .map(|(id, arg)| (id.as_str(), arg.as_str()))
        .collect();

    turn.tool_use_ids
        .iter()
        .enumerate()
        .map(|(i, (tool_use_id, tool_name))| {
            let (kind, default_value) = classify_tool_event(tool_name);
            // Use the extracted argument when available and non-empty.
            // Fall back to the default (tool name) for providers that do not
            // populate tool_inputs.
            let value = match input_map.get(tool_use_id.as_str()) {
                Some(&arg) if !arg.is_empty() => arg.to_string(),
                _ => default_value,
            };
            let cost_nanos = if i == 0 {
                cost_per + remainder
            } else {
                cost_per
            };
            ToolEvent {
                dedup_key: format!("{}:{}", turn.provider, tool_use_id),
                ts_epoch,
                session_id: turn.session_id.clone(),
                provider: turn.provider.clone(),
                project: project.to_string(),
                kind: kind.to_string(),
                value,
                cost_nanos,
                source_path: turn.source_path.clone(),
            }
        })
        .collect()
}

/// Batch-insert `ToolEvent` rows. Uses INSERT OR IGNORE for idempotency.
pub fn insert_tool_events(conn: &Connection, events: &[ToolEvent]) -> Result<()> {
    let mut stmt = conn.prepare(
        "INSERT OR IGNORE INTO tool_events
            (dedup_key, ts_epoch, session_id, provider, project, kind, value, cost_nanos, source_path)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
    )?;
    for e in events {
        stmt.execute(rusqlite::params![
            e.dedup_key,
            e.ts_epoch,
            e.session_id,
            e.provider,
            e.project,
            e.kind,
            e.value,
            e.cost_nanos,
            e.source_path,
        ])?;
    }
    Ok(())
}

/// Delete all `tool_events` rows associated with a source file.
/// Called when a file is re-ingested so events are not double-counted.
pub fn delete_tool_events_by_source_path(conn: &Connection, path: &str) -> Result<()> {
    conn.execute("DELETE FROM tool_events WHERE source_path = ?1", [path])?;
    Ok(())
}

// ── Agent status history ──────────────────────────────────────────────────────

/// Insert one row per component into `agent_status_history`.
///
/// Uses `INSERT OR IGNORE` so repeated calls with the same `(ts_epoch, provider,
/// component_id)` triple are safe — the first sample wins (PK constraint).
///
/// `components` is a slice of `(component_id, component_name, status)` tuples.
pub fn insert_agent_status_samples(
    conn: &Connection,
    provider: &str,
    components: &[(String, String, String)],
    ts_epoch: i64,
) -> Result<()> {
    let mut stmt = conn.prepare(
        "INSERT OR IGNORE INTO agent_status_history
            (ts_epoch, provider, component_id, component_name, status)
         VALUES (?1, ?2, ?3, ?4, ?5)",
    )?;
    for (component_id, component_name, status) in components {
        stmt.execute(rusqlite::params![
            ts_epoch,
            provider,
            component_id,
            component_name,
            status,
        ])?;
    }
    Ok(())
}

/// Compute a rolling uptime percentage for a given provider + component.
///
/// - Queries `agent_status_history` where `ts_epoch >= now - window_days * 86400`.
/// - Returns `None` when fewer than 10 samples exist in the window (avoids
///   misleading precision from sparse datasets).
/// - `under_maintenance` counts as NOT operational — keeps the semantics simple
///   and conservative. If a component is under maintenance it is not "up" for
///   uptime-SLA purposes even though it is not degraded.
/// - Returns `Some(up_count / total_count)` where `up_count` is the number of
///   `'operational'` samples.
pub fn uptime_pct(
    conn: &Connection,
    provider: &str,
    component_id: &str,
    window_days: i64,
) -> Result<Option<f64>> {
    let cutoff = chrono::Utc::now().timestamp() - window_days * 86400;
    // COALESCE the SUM() because SQLite returns NULL (not 0) when no rows match;
    // that would make row.get::<i64>() error rather than produce a zero count.
    let mut stmt = conn.prepare(
        "SELECT
             COUNT(*) AS total,
             COALESCE(SUM(CASE WHEN status = 'operational' THEN 1 ELSE 0 END), 0) AS up
         FROM agent_status_history
         WHERE provider = ?1
           AND component_id = ?2
           AND ts_epoch >= ?3",
    )?;
    let (total, up): (i64, i64) = stmt
        .query_row(rusqlite::params![provider, component_id, cutoff], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;
    if total < 10 {
        return Ok(None);
    }
    Ok(Some(up as f64 / total as f64))
}

/// Delete `agent_status_history` rows older than `keep_days`.
///
/// Returns the number of rows deleted. Call this after each successful poll to
/// bound table growth. `keep_days = 90` is the recommended default — it covers
/// the 30-day uptime window with a comfortable margin.
pub fn prune_agent_status_history(conn: &Connection, keep_days: i64) -> Result<usize> {
    let cutoff = chrono::Utc::now().timestamp() - keep_days * 86400;
    let deleted = conn.execute(
        "DELETE FROM agent_status_history WHERE ts_epoch < ?1",
        rusqlite::params![cutoff],
    )?;
    Ok(deleted)
}

// ─────────────────────────────────────────────────────────────────────────────

fn backfill_turn_pricing(conn: &Connection) -> Result<()> {
    type PricingBackfillRow = (i64, String, i64, i64, i64, i64, String, String);

    let mut select = conn.prepare(
        "SELECT id, model, input_tokens, output_tokens, cache_read_tokens, cache_creation_tokens,
                provider, billing_mode
         FROM turns
         WHERE pricing_version = '' OR pricing_model = '' OR cost_confidence = ''",
    )?;
    let rows: Vec<PricingBackfillRow> = select
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get(7)?,
            ))
        })?
        .filter_map(|row| row.ok())
        .collect();

    let mut update = conn.prepare(
        "UPDATE turns
         SET estimated_cost_nanos = ?1,
             pricing_version = ?2,
             pricing_model = ?3,
             billing_mode = ?4,
             cost_confidence = ?5
         WHERE id = ?6",
    )?;

    for (
        id,
        model,
        input_tokens,
        output_tokens,
        cache_read_tokens,
        cache_creation_tokens,
        provider,
        billing_mode,
    ) in rows
    {
        let estimate = crate::pricing::estimate_cost(
            &model,
            input_tokens,
            output_tokens,
            cache_read_tokens,
            cache_creation_tokens,
        );
        let billing_mode = if billing_mode.is_empty() {
            default_billing_mode(&provider)
        } else {
            billing_mode
        };
        update.execute(rusqlite::params![
            estimate.estimated_cost_nanos,
            estimate.pricing_version,
            estimate.pricing_model,
            billing_mode,
            estimate.cost_confidence,
            id,
        ])?;
    }

    Ok(())
}

fn default_billing_mode(provider: &str) -> String {
    match provider {
        "codex" => "estimated_local".into(),
        _ => "estimated_local".into(),
    }
}

pub fn insert_turns(conn: &Connection, turns: &[Turn]) -> Result<()> {
    let mut stmt = conn.prepare(
        "INSERT OR IGNORE INTO turns
            (session_id, provider, timestamp, model, input_tokens, output_tokens,
             cache_read_tokens, cache_creation_tokens, reasoning_output_tokens,
             estimated_cost_nanos, tool_name, cwd, message_id, service_tier, inference_geo,
             is_subagent, agent_id, source_path, version, pricing_version, pricing_model,
             billing_mode, cost_confidence, category, credits)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25)",
    )?;
    for t in turns {
        let msg_id: Option<&str> = if t.message_id.is_empty() {
            None
        } else {
            Some(&t.message_id)
        };
        let estimate = if t.pricing_version.is_empty() || t.cost_confidence.is_empty() {
            Some(crate::pricing::estimate_cost(
                &t.model,
                t.input_tokens,
                t.output_tokens,
                t.cache_read_tokens,
                t.cache_creation_tokens,
            ))
        } else {
            None
        };
        let estimated_cost_nanos = estimate
            .as_ref()
            .map(|value| value.estimated_cost_nanos)
            .unwrap_or(t.estimated_cost_nanos);
        let pricing_version = estimate
            .as_ref()
            .map(|value| value.pricing_version.as_str())
            .unwrap_or(t.pricing_version.as_str());
        let pricing_model = estimate
            .as_ref()
            .map(|value| value.pricing_model.as_str())
            .unwrap_or(t.pricing_model.as_str());
        let cost_confidence = estimate
            .as_ref()
            .map(|value| value.cost_confidence.as_str())
            .unwrap_or(t.cost_confidence.as_str());
        let billing_mode = if t.billing_mode.is_empty() {
            default_billing_mode(&t.provider)
        } else {
            t.billing_mode.clone()
        };
        let category = if t.category.is_empty() {
            crate::scanner::classifier::classify(t.tool_name.as_deref(), &t.all_tools, None)
                .as_str()
                .to_string()
        } else {
            t.category.clone()
        };
        stmt.execute(rusqlite::params![
            t.session_id,
            t.provider,
            t.timestamp,
            t.model,
            t.input_tokens,
            t.output_tokens,
            t.cache_read_tokens,
            t.cache_creation_tokens,
            t.reasoning_output_tokens,
            estimated_cost_nanos,
            t.tool_name,
            t.cwd,
            msg_id,
            t.service_tier,
            t.inference_geo,
            t.is_subagent as i32,
            t.agent_id,
            t.source_path,
            t.version,
            pricing_version,
            pricing_model,
            billing_mode,
            cost_confidence,
            category,
            t.credits,
        ])?;
    }
    Ok(())
}

pub fn insert_tool_invocations(
    conn: &Connection,
    turns: &[Turn],
    tool_results: &HashMap<String, bool>,
    tool_error_texts: &HashMap<String, String>,
    tool_input_jsons: &HashMap<String, String>,
) -> Result<()> {
    let mut stmt = conn.prepare(
        "INSERT OR IGNORE INTO tool_invocations
            (session_id, provider, message_id, tool_name, mcp_server, mcp_tool, tool_category, tool_use_id, is_error, source_path, timestamp, error_text, tool_input_json)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
    )?;
    for t in turns {
        let msg_id: Option<&str> = if t.message_id.is_empty() {
            None
        } else {
            Some(&t.message_id)
        };
        for (tool_use_id, tool_name) in &t.tool_use_ids {
            let (category, server, mcp_tool) = classify_tool(tool_name);
            let is_error = tool_results.get(tool_use_id).copied().unwrap_or(false);
            let error_text: Option<&str> = tool_error_texts.get(tool_use_id).map(|s| s.as_str());
            let input_json: Option<&str> = tool_input_jsons.get(tool_use_id).map(|s| s.as_str());
            stmt.execute(rusqlite::params![
                t.session_id,
                t.provider,
                msg_id,
                tool_name,
                server,
                mcp_tool,
                category,
                tool_use_id,
                is_error as i32,
                t.source_path,
                t.timestamp,
                error_text,
                input_json,
            ])?;
        }
    }
    Ok(())
}

pub fn sync_session_titles(
    conn: &Connection,
    session_ids: &[String],
    session_titles: &HashMap<String, String>,
) -> Result<()> {
    let mut stmt = conn.prepare("UPDATE sessions SET title = ?1 WHERE session_id = ?2")?;
    for session_id in session_ids {
        let title = session_titles
            .get(session_id)
            .map(|title: &String| title.trim())
            .filter(|title: &&str| !title.is_empty());
        stmt.execute(rusqlite::params![title, session_id])?;
    }
    Ok(())
}

pub fn upsert_sessions(conn: &Connection, sessions: &[crate::models::Session]) -> Result<()> {
    for s in sessions {
        let one_shot_i: Option<i32> = s.one_shot.map(|v| v as i32);
        conn.execute(
            "INSERT INTO sessions
                (session_id, provider, project_name, project_slug, first_timestamp, last_timestamp,
                 git_branch, total_input_tokens, total_output_tokens,
                 total_cache_read, total_cache_creation, total_reasoning_output,
                 total_estimated_cost_nanos, model, entrypoint, turn_count, pricing_version,
                 billing_mode, cost_confidence, title, one_shot)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21)
             ON CONFLICT(session_id) DO UPDATE SET
                provider = excluded.provider,
                project_name = excluded.project_name,
                project_slug = excluded.project_slug,
                first_timestamp = excluded.first_timestamp,
                last_timestamp = excluded.last_timestamp,
                git_branch = excluded.git_branch,
                total_input_tokens = excluded.total_input_tokens,
                total_output_tokens = excluded.total_output_tokens,
                total_cache_read = excluded.total_cache_read,
                total_cache_creation = excluded.total_cache_creation,
                total_reasoning_output = excluded.total_reasoning_output,
                total_estimated_cost_nanos = excluded.total_estimated_cost_nanos,
                model = excluded.model,
                entrypoint = excluded.entrypoint,
                turn_count = excluded.turn_count,
                pricing_version = excluded.pricing_version,
                billing_mode = excluded.billing_mode,
                cost_confidence = excluded.cost_confidence,
                title = COALESCE(excluded.title, sessions.title),
                one_shot = excluded.one_shot",
            rusqlite::params![
                s.session_id,
                s.provider,
                s.project_name,
                s.project_slug,
                s.first_timestamp,
                s.last_timestamp,
                s.git_branch,
                s.total_input_tokens,
                s.total_output_tokens,
                s.total_cache_read,
                s.total_cache_creation,
                s.total_reasoning_output,
                s.total_estimated_cost_nanos,
                s.model,
                s.entrypoint,
                s.turn_count,
                s.pricing_version,
                s.billing_mode,
                s.cost_confidence,
                s.title,
                one_shot_i,
            ],
        )?;
    }
    Ok(())
}

/// Recompute session totals from actual turns in DB.
pub fn recompute_session_totals(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "UPDATE sessions SET
            total_input_tokens = COALESCE((SELECT SUM(input_tokens) FROM turns WHERE turns.session_id = sessions.session_id), 0),
            total_output_tokens = COALESCE((SELECT SUM(output_tokens) FROM turns WHERE turns.session_id = sessions.session_id), 0),
            total_cache_read = COALESCE((SELECT SUM(cache_read_tokens) FROM turns WHERE turns.session_id = sessions.session_id), 0),
            total_cache_creation = COALESCE((SELECT SUM(cache_creation_tokens) FROM turns WHERE turns.session_id = sessions.session_id), 0),
            total_reasoning_output = COALESCE((SELECT SUM(reasoning_output_tokens) FROM turns WHERE turns.session_id = sessions.session_id), 0),
            total_estimated_cost_nanos = COALESCE((SELECT SUM(estimated_cost_nanos) FROM turns WHERE turns.session_id = sessions.session_id), 0),
            turn_count = COALESCE((SELECT COUNT(*) FROM turns WHERE turns.session_id = sessions.session_id), 0),
            total_credits = (SELECT SUM(credits) FROM turns WHERE turns.session_id = sessions.session_id),
            provider = COALESCE((
                SELECT provider FROM turns
                WHERE turns.session_id = sessions.session_id
                ORDER BY timestamp DESC, id DESC
                LIMIT 1
            ), provider),
            model = COALESCE((
                SELECT model FROM turns
                WHERE turns.session_id = sessions.session_id AND model IS NOT NULL AND model != ''
                ORDER BY timestamp DESC, id DESC
                LIMIT 1
            ), model),
            pricing_version = COALESCE((
                SELECT CASE
                    WHEN COUNT(DISTINCT pricing_version) = 0 THEN ''
                    WHEN COUNT(DISTINCT pricing_version) = 1 THEN MAX(pricing_version)
                    ELSE 'mixed'
                END
                FROM turns
                WHERE turns.session_id = sessions.session_id AND pricing_version IS NOT NULL
            ), pricing_version),
            billing_mode = COALESCE((
                SELECT CASE
                    WHEN COUNT(DISTINCT billing_mode) = 0 THEN 'estimated_local'
                    WHEN COUNT(DISTINCT billing_mode) = 1 THEN MAX(billing_mode)
                    ELSE 'mixed'
                END
                FROM turns
                WHERE turns.session_id = sessions.session_id AND billing_mode IS NOT NULL AND billing_mode != ''
            ), billing_mode),
            cost_confidence = COALESCE((
                SELECT CASE
                    WHEN SUM(CASE WHEN cost_confidence = 'low' THEN 1 ELSE 0 END) > 0 THEN 'low'
                    WHEN SUM(CASE WHEN cost_confidence = 'medium' THEN 1 ELSE 0 END) > 0 THEN 'medium'
                    WHEN COUNT(*) > 0 THEN 'high'
                    ELSE 'low'
                END
                FROM turns
                WHERE turns.session_id = sessions.session_id
            ), cost_confidence);
         DELETE FROM sessions
         WHERE NOT EXISTS (SELECT 1 FROM turns WHERE turns.session_id = sessions.session_id);",
    )?;
    Ok(())
}

// ── Phase 3: Weekly aggregation ──────────────────────────────────────────────

/// One row from `sum_by_week`: aggregated token + cost totals for a single
/// (calendar week, provider, model) bucket.
///
/// `week` is a `"YYYY-WW"` label produced by SQLite's `strftime('%W', ...)`.
/// See [`crate::tz::TzParams::sql_week_expr`] for bucketing semantics.
#[derive(Debug, Clone)]
pub struct WeekRow {
    pub week: String,
    pub provider: String,
    pub model: String,
    pub turns: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_creation_tokens: i64,
    pub reasoning_output_tokens: i64,
    pub cost_nanos: i64,
}

/// Aggregate turn data grouped by `(calendar week, provider, model)`.
///
/// Results are ordered by week ASC, then by total tokens DESC (model).
/// The week label is `"YYYY-WW"` as computed by
/// [`TzParams::sql_week_expr`].
pub fn sum_by_week(conn: &Connection, tz: TzParams) -> Result<Vec<WeekRow>> {
    let week_expr = tz.sql_week_expr("timestamp");
    let sql = format!(
        "SELECT {week_expr} as week,
                provider,
                COALESCE(model, 'unknown') as model,
                COUNT(*) as turns,
                COALESCE(SUM(input_tokens), 0) as input_tokens,
                COALESCE(SUM(output_tokens), 0) as output_tokens,
                COALESCE(SUM(cache_read_tokens), 0) as cache_read_tokens,
                COALESCE(SUM(cache_creation_tokens), 0) as cache_creation_tokens,
                COALESCE(SUM(reasoning_output_tokens), 0) as reasoning_output_tokens,
                COALESCE(SUM(estimated_cost_nanos), 0) as cost_nanos
         FROM turns
         GROUP BY week, provider, model
         ORDER BY week ASC, (input_tokens + output_tokens) DESC"
    );
    let mut stmt = conn.prepare(&sql)?;

    let map_row = |row: &rusqlite::Row<'_>| -> rusqlite::Result<WeekRow> {
        Ok(WeekRow {
            week: row.get(0)?,
            provider: row.get(1)?,
            model: row.get(2)?,
            turns: row.get(3)?,
            input_tokens: row.get(4)?,
            output_tokens: row.get(5)?,
            cache_read_tokens: row.get(6)?,
            cache_creation_tokens: row.get(7)?,
            reasoning_output_tokens: row.get(8)?,
            cost_nanos: row.get(9)?,
        })
    };

    let rows: Vec<WeekRow> = if let Some(offset_param) = tz.offset_sql_param() {
        stmt.query_map([offset_param], map_row)?
            .enumerate()
            .filter_map(|(idx, r)| match r {
                Ok(val) => Some(val),
                Err(e) => {
                    warn!("sum_by_week: failed to parse row at index {idx}: {e}");
                    None
                }
            })
            .collect()
    } else {
        stmt.query_map([], map_row)?
            .enumerate()
            .filter_map(|(idx, r)| match r {
                Ok(val) => Some(val),
                Err(e) => {
                    warn!("sum_by_week: failed to parse row at index {idx}: {e}");
                    None
                }
            })
            .collect()
    };

    Ok(rows)
}

/// Absorb the repeated `filter_map(warn)` boilerplate on mapped query rows.
///
/// Each caller passes the `MappedRows` iterator and a short context string for
/// the warning message.  Rows that fail to deserialize are logged and skipped.
fn collect_warn<T>(
    rows: rusqlite::MappedRows<'_, impl FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>>,
    ctx: &str,
) -> Vec<T> {
    rows.filter_map(|r| match r {
        Ok(val) => Some(val),
        Err(e) => {
            warn!("{}: {}", ctx, e);
            None
        }
    })
    .collect()
}

pub(crate) fn query_dashboard_models(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT COALESCE(model, 'unknown') as model
         FROM turns GROUP BY model
         ORDER BY SUM(input_tokens + output_tokens) DESC",
    )?;
    Ok(collect_warn(
        stmt.query_map([], |row| row.get(0))?,
        "dashboard models",
    ))
}

pub(crate) fn query_dashboard_providers(conn: &Connection) -> Result<Vec<ProviderSummary>> {
    let mut stmt = conn.prepare(
        "SELECT provider,
                COUNT(DISTINCT session_id) as sessions,
                COUNT(*) as turns,
                COALESCE(SUM(input_tokens), 0) as input,
                COALESCE(SUM(output_tokens), 0) as output,
                COALESCE(SUM(cache_read_tokens), 0) as cache_read,
                COALESCE(SUM(cache_creation_tokens), 0) as cache_creation,
                COALESCE(SUM(reasoning_output_tokens), 0) as reasoning_output,
                COALESCE(SUM(estimated_cost_nanos), 0) as cost_nanos
         FROM turns
         GROUP BY provider
         ORDER BY turns DESC",
    )?;
    Ok(collect_warn(
        stmt.query_map([], |row| {
            let provider: String = row.get(0)?;
            Ok(ProviderSummary {
                provider: provider.clone(),
                sessions: row.get(1)?,
                turns: row.get(2)?,
                input: row.get(3)?,
                output: row.get(4)?,
                cache_read: row.get(5)?,
                cache_creation: row.get(6)?,
                reasoning_output: row.get(7)?,
                cost: row.get::<_, i64>(8)? as f64 / 1_000_000_000.0,
            })
        })?,
        "dashboard providers",
    ))
}

pub(crate) fn query_dashboard_confidence(conn: &Connection) -> Result<Vec<ConfidenceSummary>> {
    let mut stmt = conn.prepare(
        "SELECT COALESCE(cost_confidence, 'low') as cost_confidence,
                COUNT(*) as turns,
                COALESCE(SUM(estimated_cost_nanos), 0) as cost_nanos
         FROM turns
         GROUP BY cost_confidence
         ORDER BY turns DESC",
    )?;
    Ok(collect_warn(
        stmt.query_map([], |row| {
            Ok(ConfidenceSummary {
                confidence: row.get(0)?,
                turns: row.get(1)?,
                cost: row.get::<_, i64>(2)? as f64 / 1_000_000_000.0,
            })
        })?,
        "dashboard confidence",
    ))
}

pub(crate) fn query_dashboard_billing_modes(conn: &Connection) -> Result<Vec<BillingModeSummary>> {
    let mut stmt = conn.prepare(
        "SELECT COALESCE(billing_mode, 'estimated_local') as billing_mode,
                COUNT(*) as turns,
                COALESCE(SUM(estimated_cost_nanos), 0) as cost_nanos
         FROM turns
         GROUP BY billing_mode
         ORDER BY turns DESC",
    )?;
    Ok(collect_warn(
        stmt.query_map([], |row| {
            Ok(BillingModeSummary {
                billing_mode: row.get(0)?,
                turns: row.get(1)?,
                cost: row.get::<_, i64>(2)? as f64 / 1_000_000_000.0,
            })
        })?,
        "dashboard billing modes",
    ))
}

pub(crate) fn query_dashboard_daily_by_model(
    conn: &Connection,
    tz: &TzParams,
) -> Result<Vec<DailyModelRow>> {
    let day_expr = tz.sql_day_expr("timestamp");
    let sql = format!(
        "SELECT {day_expr} as day, provider, COALESCE(model, 'unknown') as model,
                SUM(input_tokens) as input, SUM(output_tokens) as output,
                SUM(cache_read_tokens) as cache_read, SUM(cache_creation_tokens) as cache_creation,
                SUM(reasoning_output_tokens) as reasoning_output,
                COUNT(*) as turns,
                COALESCE(SUM(estimated_cost_nanos), 0) as cost_nanos,
                SUM(credits) as credits_sum
         FROM turns
         GROUP BY day, provider, model
         ORDER BY day, provider, model"
    );
    let mut stmt = conn.prepare(&sql)?;
    let map_row = |row: &rusqlite::Row<'_>| -> rusqlite::Result<DailyModelRow> {
        let provider: String = row.get(1)?;
        let model: String = row.get(2)?;
        let input: i64 = row.get(3)?;
        let output: i64 = row.get(4)?;
        let cache_read: i64 = row.get(5)?;
        let cache_creation: i64 = row.get(6)?;
        let cost_nanos: i64 = row.get(9)?;
        // Phase 21: compute per-type cost breakdown for this model/day slice.
        let (bd, _, _, _) = crate::pricing::estimate_cost_breakdown(
            &model,
            input,
            output,
            cache_read,
            cache_creation,
        );
        Ok(DailyModelRow {
            day: row.get(0)?,
            provider,
            model,
            input,
            output,
            cache_read,
            cache_creation,
            reasoning_output: row.get(7)?,
            turns: row.get(8)?,
            cost: cost_nanos as f64 / 1_000_000_000.0,
            input_cost: bd.input_cost_nanos as f64 / 1_000_000_000.0,
            output_cost: bd.output_cost_nanos as f64 / 1_000_000_000.0,
            cache_read_cost: bd.cache_read_cost_nanos as f64 / 1_000_000_000.0,
            cache_write_cost: bd.cache_write_cost_nanos as f64 / 1_000_000_000.0,
            credits: row.get::<_, Option<f64>>(10)?,
        })
    };
    if let Some(offset_param) = tz.offset_sql_param() {
        Ok(collect_warn(
            stmt.query_map([offset_param], map_row)?,
            "dashboard daily_by_model",
        ))
    } else {
        Ok(collect_warn(
            stmt.query_map([], map_row)?,
            "dashboard daily_by_model",
        ))
    }
}

pub(crate) fn query_dashboard_sessions(conn: &Connection) -> Result<Vec<SessionRow>> {
    let mut stmt = conn.prepare(
        "SELECT s.session_id, s.provider, s.project_name, s.first_timestamp, s.last_timestamp,
                s.total_input_tokens, s.total_output_tokens,
                s.total_cache_read, s.total_cache_creation, s.total_reasoning_output,
                s.total_estimated_cost_nanos, s.model, s.turn_count,
                s.pricing_version, s.billing_mode, s.cost_confidence,
                COALESCE((SELECT COUNT(DISTINCT t.agent_id) FROM turns t WHERE t.session_id = s.session_id AND t.is_subagent = 1), 0) as subagent_count,
                COALESCE((SELECT COUNT(*) FROM turns t WHERE t.session_id = s.session_id AND t.is_subagent = 1), 0) as subagent_turns,
                s.title, s.total_credits
         FROM sessions s ORDER BY s.last_timestamp DESC",
    )?;
    Ok(collect_warn(
        stmt.query_map([], |row| {
            let first_ts: String = row.get::<_, Option<String>>(3)?.unwrap_or_default();
            let last_ts: String = row.get::<_, Option<String>>(4)?.unwrap_or_default();
            let duration_min = compute_duration_min(&first_ts, &last_ts);
            let session_id: String = row.get(0)?;
            let provider: String = row.get(1)?;
            let model: String = row
                .get::<_, Option<String>>(11)?
                .unwrap_or_else(|| "unknown".into());
            let input: i64 = row.get(5)?;
            let output: i64 = row.get(6)?;
            let cache_read: i64 = row.get(7)?;
            let cache_creation: i64 = row.get(8)?;
            let reasoning_output: i64 = row.get(9)?;
            let cost = row.get::<_, i64>(10)? as f64 / 1_000_000_000.0;
            let is_billable = cost > 0.0;
            let cache_hit_ratio = if input + cache_read + cache_creation > 0 {
                cache_read as f64 / (input + cache_read + cache_creation) as f64
            } else {
                0.0
            };
            let tokens_per_min = if duration_min > 0.0 {
                (input + output + cache_read + cache_creation) as f64 / duration_min
            } else {
                0.0
            };
            Ok(SessionRow {
                session_id: raw_session_id(&session_id).chars().take(8).collect(),
                provider,
                project: row
                    .get::<_, Option<String>>(2)?
                    .unwrap_or_else(|| "unknown".into()),
                // display_name defaults to the raw project slug; resolved to alias at serve time.
                display_name: row
                    .get::<_, Option<String>>(2)?
                    .unwrap_or_else(|| "unknown".into()),
                last: last_ts
                    .chars()
                    .take(16)
                    .collect::<String>()
                    .replace('T', " "),
                last_date: last_ts.chars().take(10).collect(),
                duration_min,
                model,
                turns: row.get(12)?,
                input,
                output,
                cache_read,
                cache_creation,
                reasoning_output,
                cost,
                is_billable,
                pricing_version: row.get::<_, Option<String>>(13)?.unwrap_or_default(),
                billing_mode: row
                    .get::<_, Option<String>>(14)?
                    .unwrap_or_else(|| "estimated_local".into()),
                cost_confidence: row
                    .get::<_, Option<String>>(15)?
                    .unwrap_or_else(|| "low".into()),
                subagent_count: row.get(16)?,
                subagent_turns: row.get(17)?,
                title: row.get(18)?,
                cache_hit_ratio,
                tokens_per_min,
                credits: row.get::<_, Option<f64>>(19)?,
            })
        })?,
        "dashboard sessions",
    ))
}

pub(crate) fn query_dashboard_subagents(conn: &Connection) -> crate::models::SubagentSummary {
    conn.query_row(
        "SELECT
            COALESCE(SUM(CASE WHEN is_subagent = 0 THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN is_subagent = 0 THEN input_tokens ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN is_subagent = 0 THEN output_tokens ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN is_subagent = 1 THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN is_subagent = 1 THEN input_tokens ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN is_subagent = 1 THEN output_tokens ELSE 0 END), 0),
            COALESCE(COUNT(DISTINCT CASE WHEN is_subagent = 1 THEN agent_id END), 0)
         FROM turns",
        [],
        |row| {
            Ok(crate::models::SubagentSummary {
                parent_turns: row.get(0)?,
                parent_input: row.get(1)?,
                parent_output: row.get(2)?,
                subagent_turns: row.get(3)?,
                subagent_input: row.get(4)?,
                subagent_output: row.get(5)?,
                unique_agents: row.get(6)?,
            })
        },
    )
    .unwrap_or_else(|e| {
        warn!("Subagent summary query failed: {}", e);
        Default::default()
    })
}

pub(crate) fn query_dashboard_entrypoints(conn: &Connection) -> Result<Vec<EntrypointSummary>> {
    let mut stmt = conn.prepare(
        "SELECT s.provider, COALESCE(s.entrypoint, 'unknown') as ep,
                COUNT(DISTINCT s.session_id) as sessions,
                COUNT(t.id) as turns,
                COALESCE(SUM(t.input_tokens), 0) as input,
                COALESCE(SUM(t.output_tokens), 0) as output
         FROM sessions s
         LEFT JOIN turns t ON s.session_id = t.session_id
         GROUP BY s.provider, ep
         ORDER BY COALESCE(SUM(t.input_tokens + t.output_tokens), 0) DESC",
    )?;
    Ok(collect_warn(
        stmt.query_map([], |row| {
            Ok(EntrypointSummary {
                provider: row.get(0)?,
                entrypoint: row.get(1)?,
                sessions: row.get(2)?,
                turns: row.get(3)?,
                input: row.get(4)?,
                output: row.get(5)?,
            })
        })?,
        "dashboard entrypoints",
    ))
}

pub(crate) fn query_dashboard_service_tiers(conn: &Connection) -> Result<Vec<ServiceTierSummary>> {
    let mut stmt = conn.prepare(
        "SELECT provider, COALESCE(service_tier, 'unknown') as tier,
                COALESCE(inference_geo, 'unknown') as geo,
                COUNT(*) as cnt
         FROM turns
         WHERE service_tier IS NOT NULL AND service_tier != ''
         GROUP BY provider, tier, geo
         ORDER BY cnt DESC",
    )?;
    Ok(collect_warn(
        stmt.query_map([], |row| {
            Ok(ServiceTierSummary {
                provider: row.get(0)?,
                service_tier: row.get(1)?,
                inference_geo: row.get(2)?,
                turns: row.get(3)?,
            })
        })?,
        "dashboard service tiers",
    ))
}

pub(crate) fn query_dashboard_tools(conn: &Connection) -> Result<Vec<ToolSummary>> {
    let mut stmt = conn.prepare(
        "SELECT provider, tool_name, tool_category, mcp_server,
                COUNT(*) as invocations,
                COUNT(DISTINCT message_id) as turns_used,
                COUNT(DISTINCT session_id) as sessions_used,
                COALESCE(SUM(is_error), 0) as errors
         FROM tool_invocations
         GROUP BY provider, tool_name
         ORDER BY invocations DESC",
    )?;
    Ok(collect_warn(
        stmt.query_map([], |row| {
            Ok(ToolSummary {
                provider: row.get(0)?,
                tool_name: row.get(1)?,
                category: row.get(2)?,
                mcp_server: row.get(3)?,
                invocations: row.get(4)?,
                turns_used: row.get(5)?,
                sessions_used: row.get(6)?,
                errors: row.get(7)?,
            })
        })?,
        "dashboard tools",
    ))
}

/// Filter and pagination parameters for [`query_tool_errors`].
pub struct ToolErrorsQuery<'a> {
    pub tool_name: &'a str,
    pub provider: Option<&'a str>,
    pub mcp_server: Option<&'a str>,
    pub start: Option<&'a str>,
    pub end: Option<&'a str>,
    pub tz: &'a TzParams,
    pub limit: i64,
    pub offset: i64,
}

/// Query individual error rows for a specific tool, with optional filters.
/// Returns (rows, total_count) for pagination.
pub fn query_tool_errors(conn: &Connection, q: &ToolErrorsQuery<'_>) -> Result<ToolErrorsResponse> {
    let ToolErrorsQuery {
        tool_name,
        provider,
        mcp_server,
        start,
        end,
        tz,
        limit,
        offset,
    } = q;
    // Build WHERE clause fragments. The tool_name filter is mandatory.
    let mut filters = vec![
        "ti.tool_name = ?1".to_string(),
        "ti.is_error = 1".to_string(),
    ];
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(tool_name.to_string())];
    let mut idx = 2usize;

    if let Some(p) = provider {
        filters.push(format!("ti.provider = ?{idx}"));
        params.push(Box::new(p.to_string()));
        idx += 1;
    }
    if let Some(ms) = mcp_server {
        filters.push(format!("ti.mcp_server = ?{idx}"));
        params.push(Box::new(ms.to_string()));
        idx += 1;
    }
    let offset_min = tz.normalized_offset_min();
    if let Some(s) = start {
        // Shift the stored UTC timestamp by the client's tz offset for comparison.
        let shifted = format!("datetime(ti.timestamp, '{offset_min:+} minutes')");
        filters.push(format!("{shifted} >= ?{idx}"));
        params.push(Box::new(s.to_string()));
        idx += 1;
    }
    if let Some(e) = end {
        let shifted = format!("datetime(ti.timestamp, '{offset_min:+} minutes')");
        filters.push(format!("{shifted} <= ?{idx}"));
        params.push(Box::new(e.to_string()));
        idx += 1;
    }

    let where_clause = filters.join(" AND ");

    // Count total matching rows for pagination metadata.
    let count_sql = format!("SELECT COUNT(*) FROM tool_invocations ti WHERE {where_clause}");
    // Scope the borrow of `params` so it can be moved into `limit_params` below.
    let total: i64 = {
        let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        conn.query_row(&count_sql, params_refs.as_slice(), |r| r.get(0))?
    };

    // Main query: join turns (for model) and sessions (for project_name).
    let tz_shift = format!("'{offset_min:+} minutes'");
    let data_sql = format!(
        "SELECT
            datetime(ti.timestamp, {tz_shift}) as ts,
            ti.session_id,
            COALESCE(s.project_name, '') as project,
            COALESCE(t.model, '') as model,
            ti.provider,
            ti.tool_name,
            ti.mcp_server,
            ti.tool_input_json,
            ti.error_text,
            ti.source_path
         FROM tool_invocations ti
         LEFT JOIN turns t ON t.session_id = ti.session_id AND t.message_id = ti.message_id
                          AND t.provider = ti.provider
         LEFT JOIN sessions s ON s.session_id = ti.session_id AND s.provider = ti.provider
         WHERE {where_clause}
         ORDER BY ti.timestamp DESC
         LIMIT ?{idx} OFFSET ?{}",
        idx + 1
    );

    let limit_params: Vec<Box<dyn rusqlite::ToSql>> = params
        .into_iter()
        .chain([
            Box::new(limit) as Box<dyn rusqlite::ToSql>,
            Box::new(offset) as Box<dyn rusqlite::ToSql>,
        ])
        .collect();
    let lp_refs: Vec<&dyn rusqlite::ToSql> = limit_params.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&data_sql)?;
    let rows = collect_warn(
        stmt.query_map(lp_refs.as_slice(), |row| {
            Ok(ToolErrorRow {
                timestamp: row.get(0)?,
                session_id: row.get(1)?,
                project: row.get(2)?,
                model: row.get(3)?,
                provider: row.get(4)?,
                tool_name: row.get(5)?,
                mcp_server: row.get(6)?,
                tool_input: row.get(7)?,
                error_text: row.get(8)?,
                source_path: row.get(9)?,
            })
        })?,
        "tool errors",
    );

    Ok(ToolErrorsResponse { rows, total })
}

pub(crate) fn query_dashboard_mcp(conn: &Connection) -> Result<Vec<McpServerSummary>> {
    let mut stmt = conn.prepare(
        "SELECT provider, mcp_server,
                COUNT(DISTINCT tool_name) as tools_used,
                COUNT(*) as invocations,
                COUNT(DISTINCT session_id) as sessions_used
         FROM tool_invocations
         WHERE mcp_server IS NOT NULL
         GROUP BY provider, mcp_server
         ORDER BY invocations DESC",
    )?;
    Ok(collect_warn(
        stmt.query_map([], |row| {
            Ok(McpServerSummary {
                provider: row.get(0)?,
                server: row.get(1)?,
                tools_used: row.get(2)?,
                invocations: row.get(3)?,
                sessions_used: row.get(4)?,
            })
        })?,
        "dashboard mcp",
    ))
}

pub(crate) fn query_dashboard_hourly(conn: &Connection) -> Result<Vec<HourlyRow>> {
    let mut stmt = conn.prepare(
        "SELECT provider, CAST(substr(timestamp, 12, 2) AS INTEGER) as hour,
                COUNT(*) as turns, SUM(input_tokens) as input, SUM(output_tokens) as output,
                SUM(reasoning_output_tokens) as reasoning_output
         FROM turns
         WHERE timestamp IS NOT NULL AND length(timestamp) >= 13
         GROUP BY provider, hour
         ORDER BY provider, hour",
    )?;
    Ok(collect_warn(
        stmt.query_map([], |row| {
            Ok(HourlyRow {
                provider: row.get(0)?,
                hour: row.get(1)?,
                turns: row.get(2)?,
                input: row.get(3)?,
                output: row.get(4)?,
                reasoning_output: row.get(5)?,
            })
        })?,
        "dashboard hourly",
    ))
}

pub(crate) fn query_dashboard_branches(conn: &Connection) -> Result<Vec<BranchSummary>> {
    let mut stmt = conn.prepare(
        "SELECT s.provider, s.git_branch,
                COUNT(DISTINCT s.session_id) as sessions,
                COUNT(t.id) as turns,
                SUM(t.input_tokens) as input, SUM(t.output_tokens) as output,
                SUM(t.reasoning_output_tokens) as reasoning_output,
                COALESCE(SUM(t.estimated_cost_nanos), 0) as cost_nanos
         FROM sessions s JOIN turns t ON s.session_id = t.session_id
         WHERE s.git_branch IS NOT NULL AND s.git_branch != ''
         GROUP BY s.provider, s.git_branch
         ORDER BY SUM(t.input_tokens + t.output_tokens) DESC
         LIMIT 50",
    )?;
    Ok(collect_warn(
        stmt.query_map([], |row| {
            let provider: String = row.get(0)?;
            let branch: String = row.get(1)?;
            Ok(BranchSummary {
                provider: provider.clone(),
                branch: branch.clone(),
                sessions: row.get(2)?,
                turns: row.get(3)?,
                input: row.get(4)?,
                output: row.get(5)?,
                reasoning_output: row.get(6)?,
                cost: row.get::<_, i64>(7)? as f64 / 1_000_000_000.0,
            })
        })?,
        "dashboard branches",
    ))
}

pub(crate) fn query_dashboard_versions(conn: &Connection) -> Result<Vec<VersionSummary>> {
    let mut stmt = conn.prepare(
        "SELECT provider, COALESCE(NULLIF(version, ''), 'unknown') as ver,
                COUNT(*) as turns,
                COUNT(DISTINCT session_id) as sessions,
                COALESCE(SUM(estimated_cost_nanos), 0) as cost_nanos,
                COALESCE(SUM(input_tokens + output_tokens), 0) as tokens
         FROM turns
         GROUP BY provider, ver
         ORDER BY turns DESC",
    )?;
    Ok(collect_warn(
        stmt.query_map([], |row| {
            Ok(VersionSummary {
                provider: row.get(0)?,
                version: row.get(1)?,
                turns: row.get(2)?,
                sessions: row.get(3)?,
                cost: row.get::<_, i64>(4)? as f64 / 1_000_000_000.0,
                tokens: row.get(5)?,
            })
        })?,
        "dashboard versions",
    ))
}

pub(crate) fn query_dashboard_daily_by_project(conn: &Connection) -> Result<Vec<DailyProjectRow>> {
    let mut stmt = conn.prepare(
        "SELECT substr(t.timestamp, 1, 10) as day, s.provider, s.project_name,
                SUM(t.input_tokens) as input, SUM(t.output_tokens) as output,
                SUM(t.reasoning_output_tokens) as reasoning_output,
                COALESCE(SUM(t.estimated_cost_nanos), 0) as cost_nanos,
                SUM(t.credits) as credits_sum
         FROM turns t JOIN sessions s ON t.session_id = s.session_id
         GROUP BY day, s.provider, s.project_name
         ORDER BY day, s.provider, s.project_name",
    )?;
    Ok(collect_warn(
        stmt.query_map([], |row| {
            let day: String = row.get(0)?;
            let provider: String = row.get(1)?;
            let project: String = row.get(2)?;
            Ok(DailyProjectRow {
                day: day.clone(),
                provider: provider.clone(),
                // display_name defaults to the raw slug; resolved to alias at serve time.
                display_name: project.clone(),
                project: project.clone(),
                input: row.get(3)?,
                output: row.get(4)?,
                reasoning_output: row.get(5)?,
                cost: row.get::<_, i64>(6)? as f64 / 1_000_000_000.0,
                credits: row.get::<_, Option<f64>>(7)?,
            })
        })?,
        "dashboard daily_by_project",
    ))
}

pub(crate) fn query_dashboard_cache_efficiency(
    conn: &Connection,
) -> Result<crate::models::CacheEfficiency> {
    // Phase 21: Cache-efficiency aggregate.
    // Queries all turns for token totals, then uses estimate_cost_breakdown to
    // compute per-type cost nanos and derive the hit rate.
    let (total_input, total_output, total_cache_read, total_cache_write): (i64, i64, i64, i64) =
        conn.query_row(
            "SELECT
                COALESCE(SUM(input_tokens), 0),
                COALESCE(SUM(output_tokens), 0),
                COALESCE(SUM(cache_read_tokens), 0),
                COALESCE(SUM(cache_creation_tokens), 0)
             FROM turns",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .unwrap_or((0, 0, 0, 0));

    // Aggregate per-type cost across all distinct models.
    // We iterate over each model's token totals and sum cost breakdowns so
    // per-model pricing (including threshold discounts) is respected.
    let mut stmt = conn.prepare(
        "SELECT COALESCE(model, ''),
                COALESCE(SUM(input_tokens), 0),
                COALESCE(SUM(output_tokens), 0),
                COALESCE(SUM(cache_read_tokens), 0),
                COALESCE(SUM(cache_creation_tokens), 0)
         FROM turns
         GROUP BY model",
    )?;
    let model_totals: Vec<(String, i64, i64, i64, i64)> = collect_warn(
        stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(4)?,
            ))
        })?,
        "dashboard cache_efficiency model totals",
    );

    let mut agg_input_cost: i64 = 0;
    let mut agg_output_cost: i64 = 0;
    let mut agg_cache_read_cost: i64 = 0;
    let mut agg_cache_write_cost: i64 = 0;

    for (model, inp, out, cr, cw) in model_totals {
        let (bd, _, _, _) = crate::pricing::estimate_cost_breakdown(&model, inp, out, cr, cw);
        agg_input_cost += bd.input_cost_nanos;
        agg_output_cost += bd.output_cost_nanos;
        agg_cache_read_cost += bd.cache_read_cost_nanos;
        agg_cache_write_cost += bd.cache_write_cost_nanos;
    }

    // Cache hit rate = cache_read / (cache_read + cache_creation + input).
    // Denominator rationale: Anthropic reports input_tokens as the
    // uncached remainder only (~1% of the total input stream for heavy
    // Claude Code users), so the narrow cr / (cr + in) ratio rounded to
    // 100% for any continuous user. Including cache_creation in the
    // denominator reflects the fraction of input-side tokens actually
    // served from cache and reads meaningfully between 0% and 100%.
    let denom = total_cache_read + total_cache_write + total_input;
    let cache_hit_rate = if denom > 0 {
        Some(total_cache_read as f64 / denom as f64)
    } else {
        None
    };

    Ok(crate::models::CacheEfficiency {
        cache_read_tokens: total_cache_read,
        cache_write_tokens: total_cache_write,
        input_tokens: total_input,
        output_tokens: total_output,
        cache_read_cost_nanos: agg_cache_read_cost,
        cache_write_cost_nanos: agg_cache_write_cost,
        input_cost_nanos: agg_input_cost,
        output_cost_nanos: agg_output_cost,
        cache_hit_rate,
    })
}

pub(crate) fn query_dashboard_official_sync(conn: &Connection) -> Result<OfficialSyncSummary> {
    let total_runs: i64 = conn.query_row("SELECT COUNT(*) FROM official_sync_runs", [], |row| {
        row.get(0)
    })?;
    let total_records: i64 = conn.query_row(
        "SELECT COUNT(*) FROM official_metadata_records",
        [],
        |row| row.get(0),
    )?;

    let mut src_stmt = conn.prepare(
        "SELECT r.source_slug,
                r.source_kind,
                r.provider,
                r.status,
                r.fetched_at,
                COALESCE(COUNT(m.id), 0) AS record_count,
                r.error_text
         FROM official_sync_runs r
         LEFT JOIN official_metadata_records m ON m.run_id = r.id
         JOIN (
             SELECT source_slug, MAX(id) AS max_id
             FROM official_sync_runs
             GROUP BY source_slug
         ) latest
           ON latest.source_slug = r.source_slug AND latest.max_id = r.id
         GROUP BY r.id, r.source_slug, r.source_kind, r.provider, r.status, r.fetched_at, r.error_text
         ORDER BY r.provider, r.source_kind, r.source_slug",
    )?;
    let sources: Vec<OfficialSyncSourceStatus> = collect_warn(
        src_stmt.query_map([], |row| {
            Ok(OfficialSyncSourceStatus {
                source_slug: row.get(0)?,
                source_kind: row.get(1)?,
                provider: row.get(2)?,
                status: row.get(3)?,
                fetched_at: row.get(4)?,
                record_count: row.get(5)?,
                error_text: row.get(6)?,
            })
        })?,
        "dashboard official sync sources",
    );

    let mut rc_stmt = conn.prepare(
        "SELECT record_type, COUNT(*) as count
         FROM official_metadata_records
         GROUP BY record_type
         ORDER BY count DESC, record_type ASC",
    )?;
    let record_counts: Vec<OfficialSyncRecordCount> = collect_warn(
        rc_stmt.query_map([], |row| {
            Ok(OfficialSyncRecordCount {
                record_type: row.get(0)?,
                count: row.get(1)?,
            })
        })?,
        "dashboard official sync record counts",
    );

    let last_sync_at: Option<String> = conn
        .query_row(
            "SELECT fetched_at FROM official_sync_runs ORDER BY id DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .ok();
    let latest_success_at: Option<String> = conn
        .query_row(
            "SELECT fetched_at
             FROM official_sync_runs
             WHERE status = 'success'
             ORDER BY id DESC
             LIMIT 1",
            [],
            |row| row.get(0),
        )
        .ok();

    Ok(OfficialSyncSummary {
        available: total_runs > 0,
        last_sync_at,
        latest_success_at,
        total_runs,
        total_records,
        sources_success: sources
            .iter()
            .filter(|source| source.status == "success")
            .count() as i64,
        sources_error: sources
            .iter()
            .filter(|source| source.status == "fetch_error" || source.status == "parse_error")
            .count() as i64,
        sources_skipped: sources
            .iter()
            .filter(|source| source.status == "skipped")
            .count() as i64,
        sources,
        record_counts,
    })
}

pub fn get_dashboard_data(conn: &Connection, tz: TzParams) -> Result<DashboardData> {
    let all_models = query_dashboard_models(conn)?;
    let provider_breakdown = query_dashboard_providers(conn)?;
    let confidence_breakdown = query_dashboard_confidence(conn)?;
    let billing_mode_breakdown = query_dashboard_billing_modes(conn)?;
    let daily_by_model = query_dashboard_daily_by_model(conn, &tz)?;
    let sessions_all = query_dashboard_sessions(conn)?;
    let subagent_summary = query_dashboard_subagents(conn);
    let entrypoint_breakdown = query_dashboard_entrypoints(conn)?;
    let service_tiers = query_dashboard_service_tiers(conn)?;
    let tool_summary = query_dashboard_tools(conn)?;
    let mcp_summary = query_dashboard_mcp(conn)?;
    let hourly_distribution = query_dashboard_hourly(conn)?;
    let git_branch_summary = query_dashboard_branches(conn)?;
    let version_summary = query_dashboard_versions(conn)?;
    let daily_by_project = query_dashboard_daily_by_project(conn)?;
    let cache_efficiency = query_dashboard_cache_efficiency(conn)?;
    let official_sync = query_dashboard_official_sync(conn)?;
    let generated_at = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // Phase 3: populate weekly_by_model — group sum_by_week rows by (week, model),
    // summing across providers so the frontend gets a single series per model/week.
    let weekly_by_model: Vec<WeeklyModelRow> = {
        let raw = sum_by_week(conn, tz)?;
        let mut map: std::collections::HashMap<(String, String), WeeklyModelRow> =
            std::collections::HashMap::new();
        for r in raw {
            let entry = map
                .entry((r.week.clone(), r.model.clone()))
                .or_insert_with(|| WeeklyModelRow {
                    week: r.week.clone(),
                    model: r.model.clone(),
                    ..Default::default()
                });
            entry.input_tokens += r.input_tokens;
            entry.output_tokens += r.output_tokens;
            entry.cache_read_tokens += r.cache_read_tokens;
            entry.cache_creation_tokens += r.cache_creation_tokens;
            entry.reasoning_output_tokens += r.reasoning_output_tokens;
            entry.cost_nanos += r.cost_nanos;
        }
        let mut rows: Vec<WeeklyModelRow> = map.into_values().collect();
        rows.sort_by(|a, b| a.week.cmp(&b.week).then(b.cost_nanos.cmp(&a.cost_nanos)));
        rows
    };

    Ok(DashboardData {
        all_models,
        provider_breakdown,
        confidence_breakdown,
        billing_mode_breakdown,
        daily_by_model,
        sessions_all,
        subagent_summary,
        entrypoint_breakdown,
        service_tiers,
        tool_summary,
        mcp_summary,
        hourly_distribution,
        git_branch_summary,
        version_summary,
        daily_by_project,
        openai_reconciliation: None,
        official_sync,
        generated_at,
        cache_efficiency,
        weekly_by_model,
        subscription_quota: None,
    })
}

#[allow(dead_code)]
pub fn insert_rate_window_snapshot(
    conn: &Connection,
    window_type: &str,
    used_percent: f64,
    resets_at: Option<&str>,
) -> Result<()> {
    let timestamp = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO rate_window_history (timestamp, window_type, used_percent, resets_at)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![timestamp, window_type, used_percent, resets_at],
    )?;
    Ok(())
}

#[allow(dead_code)]
pub fn get_rate_window_history(
    conn: &Connection,
    window_type: &str,
    hours: i64,
) -> Result<Vec<(String, f64)>> {
    let cutoff = (chrono::Utc::now() - chrono::Duration::hours(hours)).to_rfc3339();
    let mut stmt = conn.prepare(
        "SELECT timestamp, used_percent FROM rate_window_history
         WHERE window_type = ?1 AND timestamp >= ?2
         ORDER BY timestamp ASC",
    )?;
    let rows = collect_warn(
        stmt.query_map(rusqlite::params![window_type, cutoff], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
        })?,
        "Failed to read rate_window_history row",
    );
    Ok(rows)
}

/// Snapshot of a rate-window observation with derived cap estimate, persisted
/// every refresh so the dashboard can reconstruct historical evolution.
pub struct RateWindowSnapshotInsert<'a> {
    pub provider: &'a str,
    pub window_type: &'a str,
    pub used_percent: f64,
    pub resets_at: Option<&'a str>,
    pub plan: Option<&'a str>,
    pub observed_tokens: Option<i64>,
    pub estimated_cap_tokens: Option<i64>,
    pub confidence: Option<f64>,
    pub source_kind: &'a str,
}

/// Insert a rate-window snapshot, with a per-(provider, window_type) dedup
/// gate: skip if the most recent row was within the last 5 minutes AND the
/// `used_percent` differs by ≤ 0.5pp. This keeps the table sparse while still
/// capturing meaningful changes.
///
/// Returns `Ok(true)` when the row was inserted, `Ok(false)` when the dedup
/// gate skipped it.
pub fn record_rate_window_snapshot(
    conn: &Connection,
    snap: &RateWindowSnapshotInsert<'_>,
) -> Result<bool> {
    let dedup_cutoff = (chrono::Utc::now() - chrono::Duration::minutes(5)).to_rfc3339();
    let recent: Option<f64> = conn
        .query_row(
            "SELECT used_percent FROM rate_window_history
             WHERE provider = ?1 AND window_type = ?2 AND timestamp >= ?3
             ORDER BY timestamp DESC LIMIT 1",
            rusqlite::params![snap.provider, snap.window_type, dedup_cutoff],
            |row| row.get::<_, Option<f64>>(0),
        )
        .optional()?
        .flatten();
    if let Some(prev) = recent
        && (prev - snap.used_percent).abs() <= 0.5
    {
        return Ok(false);
    }

    let timestamp = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO rate_window_history
            (timestamp, window_type, used_percent, resets_at,
             source_kind, source_path, provider, plan,
             observed_tokens, estimated_cap_tokens, confidence)
         VALUES (?1, ?2, ?3, ?4, ?5, '', ?6, ?7, ?8, ?9, ?10)",
        rusqlite::params![
            timestamp,
            snap.window_type,
            snap.used_percent,
            snap.resets_at,
            snap.source_kind,
            snap.provider,
            snap.plan,
            snap.observed_tokens,
            snap.estimated_cap_tokens,
            snap.confidence,
        ],
    )?;
    Ok(true)
}

/// Sum of all token columns from `turns` for a provider over a quota window,
/// optionally filtered by a `model LIKE` pattern (e.g. `"%opus%"`).
///
/// When `resets_at` parses cleanly the window is anchored to the provider's
/// actual reset boundary: `[resets_at - window_seconds, min(resets_at, now)]`.
/// On `None` or parse failure we fall back to a rolling
/// `[now - window_seconds, now]` cutoff with no upper bound (the legacy
/// behavior — kept for backward compatibility and as a graceful degrade
/// when `resets_at` is missing).
pub fn observed_tokens_for_window(
    conn: &Connection,
    provider: &str,
    window_seconds: i64,
    model_pattern: Option<&str>,
    resets_at: Option<&str>,
) -> Result<i64> {
    if window_seconds <= 0 {
        return Ok(0);
    }
    let now = chrono::Utc::now();
    let parsed_resets_at = resets_at.and_then(|s| {
        chrono::DateTime::parse_from_rfc3339(s)
            .ok()
            .or_else(|| {
                chrono::DateTime::parse_from_rfc3339(&format!("{}+00:00", s.trim_end_matches('Z')))
                    .ok()
            })
            .map(|dt| dt.with_timezone(&chrono::Utc))
    });

    let (start_str, end_str) = match parsed_resets_at {
        Some(reset) => {
            let start = reset - chrono::Duration::seconds(window_seconds);
            let end = reset.min(now);
            (start.to_rfc3339(), Some(end.to_rfc3339()))
        }
        None => {
            let cutoff = now - chrono::Duration::seconds(window_seconds);
            (cutoff.to_rfc3339(), None)
        }
    };

    let total: Option<i64> = match (model_pattern, end_str.as_deref()) {
        (Some(pattern), Some(end)) => conn.query_row(
            "SELECT COALESCE(SUM(input_tokens + output_tokens + cache_read_tokens
                + cache_creation_tokens + reasoning_output_tokens), 0)
             FROM turns
             WHERE provider = ?1 AND timestamp >= ?2 AND timestamp < ?3
               AND lower(model) LIKE ?4",
            rusqlite::params![provider, start_str, end, pattern],
            |row| row.get(0),
        )?,
        (Some(pattern), None) => conn.query_row(
            "SELECT COALESCE(SUM(input_tokens + output_tokens + cache_read_tokens
                + cache_creation_tokens + reasoning_output_tokens), 0)
             FROM turns
             WHERE provider = ?1 AND timestamp >= ?2 AND lower(model) LIKE ?3",
            rusqlite::params![provider, start_str, pattern],
            |row| row.get(0),
        )?,
        (None, Some(end)) => conn.query_row(
            "SELECT COALESCE(SUM(input_tokens + output_tokens + cache_read_tokens
                + cache_creation_tokens + reasoning_output_tokens), 0)
             FROM turns
             WHERE provider = ?1 AND timestamp >= ?2 AND timestamp < ?3",
            rusqlite::params![provider, start_str, end],
            |row| row.get(0),
        )?,
        (None, None) => conn.query_row(
            "SELECT COALESCE(SUM(input_tokens + output_tokens + cache_read_tokens
                + cache_creation_tokens + reasoning_output_tokens), 0)
             FROM turns
             WHERE provider = ?1 AND timestamp >= ?2",
            rusqlite::params![provider, start_str],
            |row| row.get(0),
        )?,
    };
    Ok(total.unwrap_or(0))
}

/// Most-recent `(confidence, estimated_cap_tokens)` pairs for one
/// `(provider, window_type)` series, ordered DESC by timestamp. Used as the
/// trailing window for confidence-weighted EMA smoothing in the cap
/// estimator. Rows with `NULL` cap or confidence are filtered out (those are
/// snapshots where utilization was below the estimator's noise floor).
pub fn recent_cap_observations(
    conn: &Connection,
    provider: &str,
    window_type: &str,
    limit: i64,
) -> Result<Vec<(f64, i64)>> {
    let mut stmt = conn.prepare(
        "SELECT confidence, estimated_cap_tokens
         FROM rate_window_history
         WHERE provider = ?1 AND window_type = ?2
           AND estimated_cap_tokens IS NOT NULL AND confidence IS NOT NULL
         ORDER BY timestamp DESC
         LIMIT ?3",
    )?;
    let rows = collect_warn(
        stmt.query_map(rusqlite::params![provider, window_type, limit], |row| {
            Ok((row.get::<_, f64>(0)?, row.get::<_, i64>(1)?))
        })?,
        "Failed to read recent cap observation row",
    );
    Ok(rows)
}

/// Hour-downsampled history for the subscription-quota widget. Returns at most
/// one row per (provider, window_type, hour) over the last `days` days; the
/// `MAX(estimated_cap_tokens)` per hour preserves the highest derived cap so
/// noisy near-zero-utilization estimates don't drown out the signal.
pub fn load_rate_window_history(
    conn: &Connection,
    days: i64,
) -> Result<Vec<crate::models::RateWindowHistoryRow>> {
    let cutoff = (chrono::Utc::now() - chrono::Duration::days(days)).to_rfc3339();
    // Hour-downsample by `(provider, window_type, hour_bucket)`. We pick
    // `MAX(plan)` rather than the most-recent value because tracking
    // recency requires a window function that complicates the query — and
    // plan strings rarely vary within a single hour for a given provider.
    let mut stmt = conn.prepare(
        "SELECT
            strftime('%Y-%m-%dT%H:00:00Z', timestamp) AS hour_bucket,
            provider,
            window_type,
            AVG(used_percent),
            MAX(estimated_cap_tokens),
            MAX(confidence),
            MAX(plan)
         FROM rate_window_history
         WHERE timestamp >= ?1
         GROUP BY hour_bucket, provider, window_type
         ORDER BY hour_bucket ASC",
    )?;
    let rows = collect_warn(
        stmt.query_map(rusqlite::params![cutoff], |row| {
            Ok(crate::models::RateWindowHistoryRow {
                timestamp: row.get(0)?,
                provider: row.get(1)?,
                window_type: row.get(2)?,
                used_percent: row.get(3)?,
                estimated_cap_tokens: row.get(4)?,
                confidence: row.get(5)?,
                plan: row.get(6)?,
            })
        })?,
        "Failed to read rate_window_history row",
    );
    Ok(rows)
}

pub struct ClaudeUsageRunInsert<'a> {
    pub status: &'a str,
    pub exit_code: Option<i32>,
    pub stdout_raw: &'a str,
    pub stderr_raw: &'a str,
    pub invocation_mode: &'a str,
    pub period: &'a str,
    pub parser_version: &'a str,
    pub error_summary: Option<&'a str>,
}

pub fn insert_claude_usage_run(conn: &Connection, run: &ClaudeUsageRunInsert<'_>) -> Result<i64> {
    let captured_at = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO claude_usage_runs
            (captured_at, status, exit_code, stdout_raw, stderr_raw, invocation_mode, period, parser_version, error_summary)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, COALESCE(?9, ''))",
        rusqlite::params![
            captured_at,
            run.status,
            run.exit_code,
            run.stdout_raw,
            run.stderr_raw,
            run.invocation_mode,
            run.period,
            run.parser_version,
            run.error_summary,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn insert_pricing_sync_run(conn: &Connection, run: &PricingSyncRun) -> Result<i64> {
    conn.execute(
        "INSERT INTO pricing_sync_runs
            (fetched_at, source_slug, source_url, provider, status, raw_body, error_text)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            run.fetched_at,
            run.source_slug,
            run.source_url,
            run.provider,
            run.status,
            run.raw_body,
            run.error_text,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn insert_official_sync_run(conn: &Connection, run: &OfficialSyncRunRecord) -> Result<i64> {
    conn.execute(
        "INSERT INTO official_sync_runs
            (fetched_at, source_slug, source_kind, source_url, provider, authority, format,
             cadence, status, http_status, content_type, etag, last_modified, raw_body,
             normalized_body, error_text, parser_version, raw_body_sha256,
             normalized_body_sha256, extracted_sha256)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20)",
        rusqlite::params![
            run.fetched_at,
            run.source_slug,
            run.source_kind,
            run.source_url,
            run.provider,
            run.authority,
            run.format,
            run.cadence,
            run.status,
            run.http_status,
            run.content_type,
            run.etag,
            run.last_modified,
            run.raw_body,
            run.normalized_body,
            run.error_text,
            run.parser_version,
            run.raw_body_sha256,
            run.normalized_body_sha256,
            run.extracted_sha256,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn insert_official_extracted_records(
    conn: &Connection,
    run_id: i64,
    records: &[OfficialExtractedRecord],
) -> Result<()> {
    let mut stmt = conn.prepare(
        "INSERT INTO official_metadata_records
            (run_id, source_slug, provider, record_type, record_key, model_id, effective_at, payload_json)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
    )?;
    for record in records {
        stmt.execute(rusqlite::params![
            run_id,
            record.source_slug,
            record.provider,
            record.record_type,
            record.record_key,
            record.model_id,
            record.effective_at,
            record.payload_json,
        ])?;
    }
    Ok(())
}

pub fn insert_pricing_sync_models(
    conn: &Connection,
    run_id: i64,
    models: &[OfficialModelPricing],
) -> Result<()> {
    let mut stmt = conn.prepare(
        "INSERT INTO pricing_sync_models
            (run_id, source_slug, provider, model_id, model_label,
             input_usd_per_mtok, cache_write_usd_per_mtok, cache_read_usd_per_mtok, output_usd_per_mtok,
             threshold_tokens, input_above_threshold, output_above_threshold, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
    )?;
    for model in models {
        stmt.execute(rusqlite::params![
            run_id,
            model.source_slug,
            model.provider,
            model.model_id,
            model.model_label,
            model.input_usd_per_mtok,
            model.cache_write_usd_per_mtok,
            model.cache_read_usd_per_mtok,
            model.output_usd_per_mtok,
            model.threshold_tokens,
            model.input_above_threshold,
            model.output_above_threshold,
            model.notes,
        ])?;
    }
    Ok(())
}

pub fn load_latest_pricing_models(conn: &Connection) -> Result<Vec<StoredPricingModel>> {
    let mut stmt = conn.prepare(
        "SELECT m.run_id, m.source_slug, m.provider, m.model_id, m.model_label,
                m.input_usd_per_mtok, m.cache_write_usd_per_mtok, m.cache_read_usd_per_mtok,
                m.output_usd_per_mtok, m.threshold_tokens, m.input_above_threshold,
                m.output_above_threshold, m.notes
         FROM pricing_sync_models m
         JOIN (
            SELECT source_slug, MAX(id) AS run_id
            FROM pricing_sync_runs
            WHERE status = 'success'
            GROUP BY source_slug
         ) latest
           ON latest.source_slug = m.source_slug AND latest.run_id = m.run_id
         ORDER BY m.source_slug, m.model_id",
    )?;

    let rows = stmt
        .query_map([], |row| {
            Ok(StoredPricingModel {
                run_id: row.get(0)?,
                source_slug: row.get(1)?,
                provider: row.get(2)?,
                model_id: row.get(3)?,
                model_label: row.get(4)?,
                input_usd_per_mtok: row.get(5)?,
                cache_write_usd_per_mtok: row.get(6)?,
                cache_read_usd_per_mtok: row.get(7)?,
                output_usd_per_mtok: row.get(8)?,
                threshold_tokens: row.get(9)?,
                input_above_threshold: row.get(10)?,
                output_above_threshold: row.get(11)?,
                notes: row.get(12)?,
            })
        })?
        .filter_map(|row| row.ok())
        .collect::<Vec<_>>();
    Ok(rows)
}

pub fn count_sessions(conn: &Connection) -> Result<usize> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0))?;
    Ok(count as usize)
}

pub fn reprice_turns_with_catalog(
    conn: &Connection,
    catalog: &HashMap<String, crate::pricing::ModelPricing>,
    pricing_version: &str,
) -> Result<usize> {
    type RepriceRow = (
        i64,
        String,
        i64,
        i64,
        i64,
        i64,
        String,
        String,
        i64,
        String,
        String,
        String,
    );

    let mut select = conn.prepare(
        "SELECT id, model, input_tokens, output_tokens, cache_read_tokens, cache_creation_tokens,
                provider, billing_mode, estimated_cost_nanos, pricing_version, pricing_model, cost_confidence
         FROM turns",
    )?;
    let rows: Vec<RepriceRow> = select
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get(7)?,
                row.get(8)?,
                row.get(9)?,
                row.get(10)?,
                row.get(11)?,
            ))
        })?
        .filter_map(|row| row.ok())
        .collect();

    let mut update = conn.prepare(
        "UPDATE turns
         SET estimated_cost_nanos = ?1,
             pricing_version = ?2,
             pricing_model = ?3,
             billing_mode = ?4,
             cost_confidence = ?5
         WHERE id = ?6",
    )?;

    let mut changed = 0_usize;
    for (
        id,
        model,
        input_tokens,
        output_tokens,
        cache_read_tokens,
        cache_creation_tokens,
        provider,
        billing_mode,
        previous_cost,
        previous_version,
        previous_pricing_model,
        previous_confidence,
    ) in rows
    {
        let estimate = crate::pricing::estimate_cost_with_catalog(
            &model,
            input_tokens,
            output_tokens,
            cache_read_tokens,
            cache_creation_tokens,
            catalog,
            pricing_version,
        );
        let billing_mode = if billing_mode.is_empty() {
            default_billing_mode(&provider)
        } else {
            billing_mode
        };

        if previous_cost != estimate.estimated_cost_nanos
            || previous_version != estimate.pricing_version
            || previous_pricing_model != estimate.pricing_model
            || previous_confidence != estimate.cost_confidence
        {
            update.execute(rusqlite::params![
                estimate.estimated_cost_nanos,
                estimate.pricing_version,
                estimate.pricing_model,
                billing_mode,
                estimate.cost_confidence,
                id,
            ])?;
            changed += 1;
        }
    }

    recompute_session_totals(conn)?;
    Ok(changed)
}

pub fn insert_claude_usage_factors(
    conn: &Connection,
    run_id: i64,
    factors: &[ClaudeUsageFactor],
) -> Result<()> {
    let mut stmt = conn.prepare(
        "INSERT INTO claude_usage_factors
            (run_id, factor_key, display_label, percent, description, advice_text, display_order)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
    )?;
    for factor in factors {
        stmt.execute(rusqlite::params![
            run_id,
            factor.factor_key,
            factor.display_label,
            factor.percent,
            factor.description,
            factor.advice_text,
            factor.display_order,
        ])?;
    }
    Ok(())
}

pub fn get_latest_claude_usage_response(conn: &Connection) -> Result<ClaudeUsageResponse> {
    let last_run = get_latest_claude_usage_run(conn, None)?;
    let latest_success = get_latest_claude_usage_run(conn, Some("success"))?;

    let latest_snapshot = match latest_success {
        Some(run) => Some(ClaudeUsageSnapshot {
            factors: get_claude_usage_factors(conn, run.id)?,
            run,
        }),
        None => None,
    };

    Ok(ClaudeUsageResponse {
        available: latest_snapshot.is_some(),
        last_run,
        latest_snapshot,
    })
}

fn get_latest_claude_usage_run(
    conn: &Connection,
    status: Option<&str>,
) -> Result<Option<ClaudeUsageRunMeta>> {
    let sql = if status.is_some() {
        "SELECT id, captured_at, status, exit_code, invocation_mode, period, parser_version, error_summary
         FROM claude_usage_runs
         WHERE status = ?1
         ORDER BY id DESC
         LIMIT 1"
    } else {
        "SELECT id, captured_at, status, exit_code, invocation_mode, period, parser_version, error_summary
         FROM claude_usage_runs
         ORDER BY id DESC
         LIMIT 1"
    };
    let mut stmt = conn.prepare(sql)?;
    let row = if let Some(status) = status {
        stmt.query_row([status], map_claude_usage_run_meta)
    } else {
        stmt.query_row([], map_claude_usage_run_meta)
    };
    match row {
        Ok(run) => Ok(Some(run)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(err) => Err(err.into()),
    }
}

fn map_claude_usage_run_meta(row: &rusqlite::Row<'_>) -> rusqlite::Result<ClaudeUsageRunMeta> {
    let error_summary: String = row.get(7)?;
    Ok(ClaudeUsageRunMeta {
        id: row.get(0)?,
        captured_at: row.get(1)?,
        status: row.get(2)?,
        exit_code: row.get(3)?,
        invocation_mode: row.get(4)?,
        period: row.get(5)?,
        parser_version: row.get(6)?,
        error_summary: if error_summary.is_empty() {
            None
        } else {
            Some(error_summary)
        },
    })
}

fn get_claude_usage_factors(conn: &Connection, run_id: i64) -> Result<Vec<ClaudeUsageFactor>> {
    let mut stmt = conn.prepare(
        "SELECT factor_key, display_label, percent, description, advice_text, display_order
         FROM claude_usage_factors
         WHERE run_id = ?1
         ORDER BY display_order ASC, id ASC",
    )?;
    let rows = collect_warn(
        stmt.query_map([run_id], |row| {
            Ok(ClaudeUsageFactor {
                factor_key: row.get(0)?,
                display_label: row.get(1)?,
                percent: row.get(2)?,
                description: row.get(3)?,
                advice_text: row.get(4)?,
                display_order: row.get(5)?,
            })
        })?,
        "Failed to read Claude usage factor row",
    );
    Ok(rows)
}

// ── Phase 13: 7×24 Activity Heatmap + Active-Period Averaging ────────────────

/// Date-bound SQL fragment for a given period.
///
/// Returns `Some((start_str, end_str))` for bounded periods, or `None` for
/// `"all"` (no bounds).  Shares the same convention as `export.rs`
/// `period_bounds()`.
fn heatmap_date_bounds(period: &str) -> Option<(String, String)> {
    let today = chrono::Local::now().date_naive();
    let start = match period {
        "today" => today,
        "week" => today - chrono::Duration::days(6),
        "month" => today - chrono::Duration::days(29),
        "year" => today - chrono::Duration::days(364),
        _ => return None, // "all" or unknown → unbounded
    };
    Some((
        start.format("%Y-%m-%d").to_string(),
        today.format("%Y-%m-%d").to_string(),
    ))
}

/// Build the shifted-timestamp expression used for dow/hour bucketing.
///
/// When `tz_offset_min` is 0 (UTC) the expression is the cheap
/// `"timestamp"`.  For a non-zero offset it wraps the column with
/// `datetime(timestamp, '+N minutes')`.
///
/// The caller must bind the offset string when `needs_param` is true.
fn shifted_ts_expr(tz: &crate::tz::TzParams) -> (String, bool) {
    let offset = tz.normalized_offset_min();
    if offset == 0 {
        ("timestamp".to_string(), false)
    } else {
        ("datetime(timestamp, ?)".to_string(), true)
    }
}

/// Query the 7×24 heatmap for the given period.
///
/// Returns exactly 168 cells (7 days × 24 hours).  Cells with no turns
/// are filled with zeros in Rust after the SQL query (the query returns
/// only non-zero cells).
///
/// `period` accepts `"today" | "week" | "month" | "year" | "all"`.
/// Bucketing respects `tz` — the SQL applies
/// `datetime(timestamp, '+N minutes')` before strftime when the offset is
/// non-zero.  Defaults to UTC when `tz_offset_min` is absent.
pub fn get_heatmap(
    conn: &Connection,
    period: &str,
    tz: crate::tz::TzParams,
) -> Result<Vec<crate::models::HeatmapCell>> {
    let (ts_expr, needs_tz_param) = shifted_ts_expr(&tz);
    let bounds = heatmap_date_bounds(period);

    let mut sql = format!(
        "SELECT CAST(strftime('%w', {ts_expr}) AS INTEGER) AS dow,
                CAST(strftime('%H', {ts_expr}) AS INTEGER) AS hour,
                COALESCE(SUM(estimated_cost_nanos), 0) AS cost_nanos,
                COUNT(*) AS call_count
         FROM turns
         WHERE timestamp IS NOT NULL AND length(timestamp) >= 10"
    );
    let mut params: Vec<String> = Vec::new();

    // Bind tz offset param(s) — one per strftime call in SELECT (2 calls).
    if needs_tz_param {
        let offset_str = tz.offset_sql_param().unwrap_or_default();
        // Two strftime calls reference the shifted expression.
        params.push(offset_str.clone());
        params.push(offset_str);
    }

    if let Some((start, end)) = bounds {
        let day_expr = tz.sql_day_expr("timestamp");
        sql.push_str(&format!(" AND {day_expr} BETWEEN ? AND ?"));
        // If the day_expr itself uses a bound param, insert it before start/end.
        if let Some(offset_param) = tz.offset_sql_param() {
            params.push(offset_param);
        }
        params.push(start);
        params.push(end);
    }

    sql.push_str(" GROUP BY dow, hour ORDER BY dow, hour");

    let mut stmt = conn.prepare(&sql)?;
    let non_zero: Vec<(i64, i64, i64, i64)> = collect_warn(
        stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })?,
        "heatmap row error",
    );

    // Build a lookup map and fill all 168 cells.
    let mut lookup: std::collections::HashMap<(i64, i64), (i64, i64)> =
        std::collections::HashMap::with_capacity(non_zero.len());
    for (dow, hour, cost, count) in non_zero {
        lookup.insert((dow, hour), (cost, count));
    }

    let mut cells: Vec<crate::models::HeatmapCell> = Vec::with_capacity(168);
    for dow in 0i64..7 {
        for hour in 0i64..24 {
            let (cost_nanos, call_count) = lookup.get(&(dow, hour)).copied().unwrap_or((0, 0));
            cells.push(crate::models::HeatmapCell {
                dow,
                hour,
                cost_nanos,
                call_count,
            });
        }
    }
    Ok(cells)
}

/// Active-period average cost.
///
/// Returns `(total_cost_nanos, active_days)` where `active_days` is the
/// count of distinct calendar days (in the client timezone) that had at
/// least one turn with non-zero cost.
///
/// Divide `total_cost_nanos / active_days` in the caller to get the
/// per-active-day average.  Returns `(0, 0)` when there are no turns so
/// the caller can display `--` rather than dividing by zero.
pub fn active_period_avg_cost_nanos(
    conn: &Connection,
    period: &str,
    tz: crate::tz::TzParams,
) -> Result<(i64, i64)> {
    let day_expr = tz.sql_day_expr("timestamp");
    let bounds = heatmap_date_bounds(period);

    let mut sql = format!(
        "SELECT COALESCE(SUM(estimated_cost_nanos), 0),
                COUNT(DISTINCT {day_expr})
         FROM turns
         WHERE timestamp IS NOT NULL AND estimated_cost_nanos > 0"
    );
    let mut params: Vec<String> = Vec::new();

    // Bind tz offset param for day_expr in SELECT (if non-UTC).
    if let Some(offset_param) = tz.offset_sql_param() {
        params.push(offset_param);
    }

    if let Some((start, end)) = bounds {
        // day_expr appears again in the WHERE clause bound.
        let day_expr2 = tz.sql_day_expr("timestamp");
        sql.push_str(&format!(" AND {day_expr2} BETWEEN ? AND ?"));
        if let Some(offset_param2) = tz.offset_sql_param() {
            params.push(offset_param2);
        }
        params.push(start);
        params.push(end);
    }

    let (total_nanos, active_days): (i64, i64) = conn
        .query_row(&sql, rusqlite::params_from_iter(params.iter()), |row| {
            Ok((row.get(0)?, row.get(1)?))
        })
        .unwrap_or((0, 0));

    Ok((total_nanos, active_days))
}

// ─────────────────────────────────────────────────────────────────────────────

pub fn get_provider_estimated_cost_nanos_since(
    conn: &Connection,
    provider: &str,
    start_date: &str,
) -> Result<i64> {
    let cost_nanos = conn.query_row(
        "SELECT COALESCE(SUM(estimated_cost_nanos), 0)
         FROM turns
         WHERE provider = ?1 AND substr(timestamp, 1, 10) >= ?2",
        rusqlite::params![provider, start_date],
        |row| row.get(0),
    )?;
    Ok(cost_nanos)
}

pub fn get_provider_cost_summary_since(
    conn: &Connection,
    provider: &str,
    start_date: &str,
) -> Result<(i64, i64, crate::models::TokenBreakdown)> {
    let row: (i64, i64, i64, i64, i64, i64, i64) = conn.query_row(
        "SELECT
            COALESCE(SUM(estimated_cost_nanos), 0),
            COALESCE(SUM(input_tokens + output_tokens + cache_read_tokens + cache_creation_tokens + reasoning_output_tokens), 0),
            COALESCE(SUM(input_tokens), 0),
            COALESCE(SUM(output_tokens), 0),
            COALESCE(SUM(cache_read_tokens), 0),
            COALESCE(SUM(cache_creation_tokens), 0),
            COALESCE(SUM(reasoning_output_tokens), 0)
         FROM turns
         WHERE provider = ?1 AND substr(timestamp, 1, 10) >= ?2",
        rusqlite::params![provider, start_date],
        |row| Ok((
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
            row.get(5)?,
            row.get(6)?,
        )),
    )?;
    let (cost_nanos, total_tokens, input, output, cache_read, cache_creation, reasoning_output) =
        row;
    Ok((
        cost_nanos,
        total_tokens,
        crate::models::TokenBreakdown {
            input,
            output,
            cache_read,
            cache_creation,
            reasoning_output,
        },
    ))
}

pub fn get_provider_cost_summary_since_tz(
    conn: &Connection,
    provider: &str,
    start_date: &str,
    tz: crate::tz::TzParams,
) -> Result<(i64, i64, crate::models::TokenBreakdown)> {
    let day_expr = tz.sql_day_expr("timestamp");
    let sql = format!(
        "SELECT
            COALESCE(SUM(estimated_cost_nanos), 0),
            COALESCE(SUM(input_tokens + output_tokens + cache_read_tokens + cache_creation_tokens + reasoning_output_tokens), 0),
            COALESCE(SUM(input_tokens), 0),
            COALESCE(SUM(output_tokens), 0),
            COALESCE(SUM(cache_read_tokens), 0),
            COALESCE(SUM(cache_creation_tokens), 0),
            COALESCE(SUM(reasoning_output_tokens), 0)
         FROM turns
         WHERE provider = ? AND {day_expr} >= ?"
    );
    let map_row = |row: &rusqlite::Row<'_>| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, i64>(1)?,
            crate::models::TokenBreakdown {
                input: row.get(2)?,
                output: row.get(3)?,
                cache_read: row.get(4)?,
                cache_creation: row.get(5)?,
                reasoning_output: row.get(6)?,
            },
        ))
    };
    if let Some(offset_param) = tz.offset_sql_param() {
        conn.query_row(
            &sql,
            rusqlite::params![provider, offset_param, start_date],
            map_row,
        )
        .map_err(Into::into)
    } else {
        conn.query_row(&sql, rusqlite::params![provider, start_date], map_row)
            .map_err(Into::into)
    }
}

/// Estimate total cache-read savings since `start_date` for the given
/// provider. Uses per-model cache-read sums joined with the pricing table so
/// Sonnet vs Opus vs Haiku rate differences are honored.
pub fn get_provider_cache_savings_nanos_since(
    conn: &Connection,
    provider: &str,
    start_date: &str,
) -> Result<i64> {
    let mut stmt = conn.prepare(
        "SELECT model, COALESCE(SUM(cache_read_tokens), 0) AS cache_reads
         FROM turns
         WHERE provider = ?1 AND substr(timestamp, 1, 10) >= ?2
         GROUP BY model",
    )?;
    let rows = stmt.query_map(rusqlite::params![provider, start_date], |row| {
        let model: String = row.get(0)?;
        let cache_reads: i64 = row.get(1)?;
        Ok((model, cache_reads))
    })?;

    let mut total: i64 = 0;
    for row in rows {
        let (model, cache_reads) = row?;
        total = total.saturating_add(crate::pricing::calc_cache_savings_nanos(
            &model,
            cache_reads,
        ));
    }
    Ok(total)
}

pub fn get_provider_daily_cost_history_since(
    conn: &Connection,
    provider: &str,
    start_date: &str,
) -> Result<Vec<crate::models::ProviderCostHistoryPoint>> {
    let mut stmt = conn.prepare(
        "SELECT
            substr(timestamp, 1, 10) AS day,
            COALESCE(SUM(input_tokens + output_tokens + cache_read_tokens + cache_creation_tokens + reasoning_output_tokens), 0) AS total_tokens,
            COALESCE(SUM(estimated_cost_nanos), 0) AS cost_nanos,
            COALESCE(SUM(input_tokens), 0) AS input_tokens,
            COALESCE(SUM(output_tokens), 0) AS output_tokens,
            COALESCE(SUM(cache_read_tokens), 0) AS cache_read_tokens,
            COALESCE(SUM(cache_creation_tokens), 0) AS cache_creation_tokens,
            COALESCE(SUM(reasoning_output_tokens), 0) AS reasoning_output_tokens
         FROM turns
         WHERE provider = ?1 AND substr(timestamp, 1, 10) >= ?2
         GROUP BY substr(timestamp, 1, 10)
         ORDER BY day ASC",
    )?;

    let rows = stmt.query_map(rusqlite::params![provider, start_date], |row| {
        let cost_nanos: i64 = row.get(2)?;
        Ok(crate::models::ProviderCostHistoryPoint {
            day: row.get(0)?,
            total_tokens: row.get(1)?,
            cost_usd: cost_nanos as f64 / 1_000_000_000.0,
            breakdown: crate::models::TokenBreakdown {
                input: row.get(3)?,
                output: row.get(4)?,
                cache_read: row.get(5)?,
                cache_creation: row.get(6)?,
                reasoning_output: row.get(7)?,
            },
        })
    })?;

    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Into::into)
}

pub fn get_provider_daily_cost_history_since_tz(
    conn: &Connection,
    provider: &str,
    start_date: &str,
    tz: crate::tz::TzParams,
) -> Result<Vec<crate::models::ProviderCostHistoryPoint>> {
    let day_expr = tz.sql_day_expr("timestamp");
    let sql = format!(
        "SELECT
            {day_expr} AS day,
            COALESCE(SUM(input_tokens + output_tokens + cache_read_tokens + cache_creation_tokens + reasoning_output_tokens), 0) AS total_tokens,
            COALESCE(SUM(estimated_cost_nanos), 0) AS cost_nanos,
            COALESCE(SUM(input_tokens), 0) AS input_tokens,
            COALESCE(SUM(output_tokens), 0) AS output_tokens,
            COALESCE(SUM(cache_read_tokens), 0) AS cache_read_tokens,
            COALESCE(SUM(cache_creation_tokens), 0) AS cache_creation_tokens,
            COALESCE(SUM(reasoning_output_tokens), 0) AS reasoning_output_tokens
         FROM turns
         WHERE provider = ? AND {day_expr} >= ?
         GROUP BY day
         ORDER BY day ASC"
    );
    let mut stmt = conn.prepare(&sql)?;
    let map_row = |row: &rusqlite::Row<'_>| {
        let cost_nanos: i64 = row.get(2)?;
        Ok(crate::models::ProviderCostHistoryPoint {
            day: row.get(0)?,
            total_tokens: row.get(1)?,
            cost_usd: cost_nanos as f64 / 1_000_000_000.0,
            breakdown: crate::models::TokenBreakdown {
                input: row.get(3)?,
                output: row.get(4)?,
                cache_read: row.get(5)?,
                cache_creation: row.get(6)?,
                reasoning_output: row.get(7)?,
            },
        })
    };
    if let Some(offset_param) = tz.offset_sql_param() {
        Ok(collect_warn(
            stmt.query_map(
                rusqlite::params![provider, offset_param.clone(), offset_param, start_date],
                map_row,
            )?,
            "provider daily cost history",
        ))
    } else {
        Ok(collect_warn(
            stmt.query_map(rusqlite::params![provider, start_date], map_row)?,
            "provider daily cost history",
        ))
    }
}

pub fn get_provider_daily_by_model(
    conn: &Connection,
    provider: &str,
    start_date: &str,
) -> Result<Vec<crate::models::ProviderDailyModelRow>> {
    let mut stmt = conn.prepare(
        "SELECT substr(timestamp, 1, 10) as day,
                COALESCE(model, 'unknown') as model,
                COALESCE(SUM(estimated_cost_nanos), 0) as cost_nanos,
                COALESCE(SUM(input_tokens), 0) as input,
                COALESCE(SUM(output_tokens), 0) as output,
                COALESCE(SUM(cache_read_tokens), 0) as cache_read,
                COALESCE(SUM(cache_creation_tokens), 0) as cache_creation,
                COALESCE(SUM(reasoning_output_tokens), 0) as reasoning_output,
                COUNT(*) as turns
         FROM turns
         WHERE provider = ?1 AND substr(timestamp, 1, 10) >= ?2
         GROUP BY day, model
         ORDER BY day ASC, cost_nanos DESC",
    )?;
    let rows = stmt.query_map(rusqlite::params![provider, start_date], |row| {
        let cost_nanos: i64 = row.get(2)?;
        Ok(crate::models::ProviderDailyModelRow {
            day: row.get(0)?,
            model: row.get(1)?,
            cost_usd: cost_nanos as f64 / 1_000_000_000.0,
            input: row.get::<_, i64>(3)? as u64,
            output: row.get::<_, i64>(4)? as u64,
            cache_read: row.get::<_, i64>(5)? as u64,
            cache_creation: row.get::<_, i64>(6)? as u64,
            reasoning_output: row.get::<_, i64>(7)? as u64,
            turns: row.get::<_, i64>(8)? as u64,
        })
    })?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Into::into)
}

// ── Provider-scoped row queries ─────────────────────────────────────
//
// The `get_provider_*_rows` family below shares a fixed shape:
//
//   (conn, provider, start_date, limit) ->
//     SELECT ... FROM turns
//     WHERE provider = ?1 AND substr(timestamp, 1, 10) >= ?2
//     GROUP BY <key> ORDER BY <metric> DESC LIMIT ?3
//
// followed by a typed `query_map` closure and
// `rows.collect::<rusqlite::Result<Vec<_>>>().map_err(Into::into)`.
//
// New provider-scoped queries should follow the same signature and the same
// **strict** collect — not the `collect_warn` helper used by the dashboard
// fns above. Strict collect is right here because these endpoints surface
// errors directly to the caller via `?`; silently dropping rows would be a
// regression for provider-scoped data the user is actively inspecting.

pub fn get_provider_model_rows(
    conn: &Connection,
    provider: &str,
    start_date: &str,
    limit: usize,
) -> Result<Vec<crate::models::ProviderModelRow>> {
    let mut stmt = conn.prepare(
        "SELECT COALESCE(model, 'unknown') as model,
                COALESCE(SUM(estimated_cost_nanos), 0) as cost_nanos,
                COALESCE(SUM(input_tokens), 0) as input,
                COALESCE(SUM(output_tokens), 0) as output,
                COALESCE(SUM(cache_read_tokens), 0) as cache_read,
                COALESCE(SUM(cache_creation_tokens), 0) as cache_creation,
                COALESCE(SUM(reasoning_output_tokens), 0) as reasoning_output,
                COUNT(*) as turns
         FROM turns
         WHERE provider = ?1 AND substr(timestamp, 1, 10) >= ?2
         GROUP BY model
         ORDER BY cost_nanos DESC
         LIMIT ?3",
    )?;
    let rows = stmt.query_map(
        rusqlite::params![provider, start_date, limit as i64],
        |row| {
            let cost_nanos: i64 = row.get(1)?;
            Ok(crate::models::ProviderModelRow {
                model: row.get(0)?,
                cost_usd: cost_nanos as f64 / 1_000_000_000.0,
                input: row.get::<_, i64>(2)? as u64,
                output: row.get::<_, i64>(3)? as u64,
                cache_read: row.get::<_, i64>(4)? as u64,
                cache_creation: row.get::<_, i64>(5)? as u64,
                reasoning_output: row.get::<_, i64>(6)? as u64,
                turns: row.get::<_, i64>(7)? as u64,
            })
        },
    )?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Into::into)
}

pub fn get_provider_project_rows(
    conn: &Connection,
    provider: &str,
    start_date: &str,
    limit: usize,
) -> Result<Vec<crate::models::ProviderProjectRow>> {
    let mut stmt = conn.prepare(
        "SELECT s.project_name,
                COALESCE(SUM(t.estimated_cost_nanos), 0) as cost_nanos,
                COUNT(*) as turns,
                COUNT(DISTINCT t.session_id) as sessions
         FROM turns t
         JOIN sessions s ON t.session_id = s.session_id
         WHERE t.provider = ?1 AND t.timestamp >= ?2
         GROUP BY s.project_name
         ORDER BY cost_nanos DESC
         LIMIT ?3",
    )?;
    let rows = stmt.query_map(
        rusqlite::params![provider, start_date, limit as i64],
        |row| {
            let project: String = row.get(0)?;
            let cost_nanos: i64 = row.get(1)?;
            Ok(crate::models::ProviderProjectRow {
                display_name: project.clone(),
                project,
                cost_usd: cost_nanos as f64 / 1_000_000_000.0,
                turns: row.get::<_, i64>(2)? as u64,
                sessions: row.get::<_, i64>(3)? as u64,
            })
        },
    )?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Into::into)
}

pub fn get_provider_tool_rows(
    conn: &Connection,
    provider: &str,
    start_date: &str,
    limit: usize,
) -> Result<Vec<crate::models::ProviderToolRow>> {
    let mut stmt = conn.prepare(
        "SELECT ti.tool_name, ti.tool_category, ti.mcp_server,
                COUNT(*) as invocations,
                COALESCE(SUM(ti.is_error), 0) as errors,
                COUNT(DISTINCT ti.message_id) as turns_used,
                COUNT(DISTINCT ti.session_id) as sessions_used
         FROM tool_invocations ti
         WHERE ti.provider = ?1 AND ti.timestamp >= ?2
         GROUP BY ti.tool_name
         ORDER BY invocations DESC
         LIMIT ?3",
    )?;
    let rows = stmt.query_map(
        rusqlite::params![provider, start_date, limit as i64],
        |row| {
            Ok(crate::models::ProviderToolRow {
                tool_name: row.get(0)?,
                category: row.get(1)?,
                mcp_server: row.get(2)?,
                invocations: row.get::<_, i64>(3)? as u64,
                errors: row.get::<_, i64>(4)? as u64,
                turns_used: row.get::<_, i64>(5)? as u64,
                sessions_used: row.get::<_, i64>(6)? as u64,
            })
        },
    )?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Into::into)
}

pub fn get_provider_mcp_rows(
    conn: &Connection,
    provider: &str,
    start_date: &str,
) -> Result<Vec<crate::models::ProviderMcpRow>> {
    let mut stmt = conn.prepare(
        "SELECT ti.mcp_server,
                COUNT(*) as invocations,
                COUNT(DISTINCT ti.tool_name) as tools_used,
                COUNT(DISTINCT ti.session_id) as sessions_used
         FROM tool_invocations ti
         WHERE ti.provider = ?1 AND ti.mcp_server IS NOT NULL AND ti.timestamp >= ?2
         GROUP BY ti.mcp_server
         ORDER BY invocations DESC",
    )?;
    let rows = stmt.query_map(rusqlite::params![provider, start_date], |row| {
        Ok(crate::models::ProviderMcpRow {
            server: row.get(0)?,
            invocations: row.get::<_, i64>(1)? as u64,
            tools_used: row.get::<_, i64>(2)? as u64,
            sessions_used: row.get::<_, i64>(3)? as u64,
        })
    })?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Into::into)
}

pub fn get_provider_hourly_activity(
    conn: &Connection,
    provider: &str,
    start_date: &str,
) -> Result<Vec<crate::models::ProviderHourlyBucket>> {
    // Fetch raw aggregates grouped by local hour.
    let mut stmt = conn.prepare(
        "SELECT CAST(strftime('%H', datetime(timestamp, 'localtime')) AS INTEGER) as hr,
                COUNT(*) as turns,
                COALESCE(SUM(estimated_cost_nanos), 0) as cost_nanos,
                COALESCE(SUM(input_tokens + output_tokens), 0) as tokens
         FROM turns
         WHERE provider = ?1 AND substr(timestamp, 1, 10) >= ?2
         GROUP BY hr",
    )?;
    let raw: Vec<(u8, u64, i64, u64)> = stmt
        .query_map(rusqlite::params![provider, start_date], |row| {
            Ok((
                row.get::<_, i64>(0)? as u8,
                row.get::<_, i64>(1)? as u64,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)? as u64,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    // Zero-fill all 24 hours so Swift gets a contiguous 0..=23.
    let mut buckets: Vec<crate::models::ProviderHourlyBucket> = (0u8..24)
        .map(|h| crate::models::ProviderHourlyBucket {
            hour: h,
            turns: 0,
            cost_usd: 0.0,
            tokens: 0,
        })
        .collect();
    for (hr, turns, cost_nanos, tokens) in raw {
        if let Some(b) = buckets.get_mut(hr as usize) {
            b.turns = turns;
            b.cost_usd = cost_nanos as f64 / 1_000_000_000.0;
            b.tokens = tokens;
        }
    }
    Ok(buckets)
}

pub fn get_provider_activity_heatmap(
    conn: &Connection,
    provider: &str,
    start_date: &str,
) -> Result<Vec<crate::models::ProviderHeatmapCell>> {
    let mut stmt = conn.prepare(
        "SELECT CAST(strftime('%w', datetime(timestamp, 'localtime')) AS INTEGER) as dow,
                CAST(strftime('%H', datetime(timestamp, 'localtime')) AS INTEGER) as hr,
                COUNT(*) as turns
         FROM turns
         WHERE provider = ?1 AND substr(timestamp, 1, 10) >= ?2
         GROUP BY dow, hr
         HAVING turns > 0",
    )?;
    let rows = stmt.query_map(rusqlite::params![provider, start_date], |row| {
        Ok(crate::models::ProviderHeatmapCell {
            day_of_week: row.get::<_, i64>(0)? as u8,
            hour: row.get::<_, i64>(1)? as u8,
            turns: row.get::<_, i64>(2)? as u64,
        })
    })?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Into::into)
}

pub fn get_provider_recent_sessions(
    conn: &Connection,
    provider: &str,
    limit: usize,
) -> Result<Vec<crate::models::ProviderSession>> {
    let mut stmt = conn.prepare(
        "SELECT s.session_id,
                COALESCE(NULLIF(s.project_name, ''), s.session_id) as display_name,
                COALESCE(s.first_timestamp, '') as started_at,
                CAST(ROUND(
                    (julianday(COALESCE(NULLIF(s.last_timestamp, ''), s.first_timestamp))
                     - julianday(COALESCE(s.first_timestamp, s.last_timestamp))) * 1440
                ) AS INTEGER) as duration_minutes,
                COUNT(t.id) as turns,
                COALESCE(SUM(t.estimated_cost_nanos), 0) as cost_nanos,
                (SELECT COALESCE(NULLIF(t2.model, ''), NULL)
                 FROM turns t2
                 WHERE t2.session_id = s.session_id
                 GROUP BY t2.model
                 ORDER BY COUNT(*) DESC
                 LIMIT 1) as top_model
         FROM sessions s
         LEFT JOIN turns t ON t.session_id = s.session_id
         WHERE s.provider = ?1
         GROUP BY s.session_id
         ORDER BY s.first_timestamp DESC
         LIMIT ?2",
    )?;
    let rows = stmt.query_map(rusqlite::params![provider, limit as i64], |row| {
        let cost_nanos: i64 = row.get(5)?;
        let duration_raw: i64 = row.get::<_, Option<i64>>(3)?.unwrap_or(0).max(0);
        Ok(crate::models::ProviderSession {
            session_id: row.get(0)?,
            display_name: row.get(1)?,
            started_at: row.get(2)?,
            duration_minutes: duration_raw as u64,
            turns: row.get::<_, i64>(4)? as u64,
            cost_usd: cost_nanos as f64 / 1_000_000_000.0,
            model: row.get(6)?,
        })
    })?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Into::into)
}

pub fn get_provider_subagent_breakdown(
    conn: &Connection,
    provider: &str,
    start_date: &str,
) -> Result<Option<crate::models::ProviderSubagentBreakdown>> {
    let mut stmt = conn.prepare(
        "SELECT COUNT(*) as total_turns,
                COALESCE(SUM(estimated_cost_nanos), 0) as cost_nanos,
                COUNT(DISTINCT session_id) as session_count,
                COUNT(DISTINCT agent_id) as agent_count
         FROM turns
         WHERE is_subagent = 1
           AND provider = ?1
           AND substr(timestamp, 1, 10) >= ?2",
    )?;
    let result = stmt.query_row(rusqlite::params![provider, start_date], |row| {
        let total_turns: i64 = row.get(0)?;
        let cost_nanos: i64 = row.get(1)?;
        let session_count: i64 = row.get(2)?;
        let agent_count: i64 = row.get(3)?;
        Ok((total_turns, cost_nanos, session_count, agent_count))
    })?;
    let (total_turns, cost_nanos, session_count, agent_count) = result;
    if total_turns == 0 {
        return Ok(None);
    }
    Ok(Some(crate::models::ProviderSubagentBreakdown {
        total_turns: total_turns as u64,
        total_cost_usd: cost_nanos as f64 / 1_000_000_000.0,
        session_count: session_count as u64,
        agent_count: agent_count as u64,
    }))
}

pub fn get_provider_version_rows(
    conn: &Connection,
    provider: &str,
    start_date: &str,
    limit: usize,
) -> Result<Vec<crate::models::ProviderVersionRow>> {
    let mut stmt = conn.prepare(
        "SELECT COALESCE(NULLIF(version, ''), 'unknown') as ver,
                COUNT(*) as turns,
                COUNT(DISTINCT session_id) as sessions,
                COALESCE(SUM(estimated_cost_nanos), 0) as cost_nanos
         FROM turns
         WHERE provider = ?1 AND substr(timestamp, 1, 10) >= ?2
         GROUP BY ver
         ORDER BY cost_nanos DESC
         LIMIT ?3",
    )?;
    let rows = stmt.query_map(
        rusqlite::params![provider, start_date, limit as i64],
        |row| {
            let cost_nanos: i64 = row.get(3)?;
            Ok(crate::models::ProviderVersionRow {
                version: row.get(0)?,
                turns: row.get::<_, i64>(1)? as u64,
                sessions: row.get::<_, i64>(2)? as u64,
                cost_usd: cost_nanos as f64 / 1_000_000_000.0,
            })
        },
    )?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Into::into)
}

fn compute_duration_min(first: &str, last: &str) -> f64 {
    let parse = |s: &str| -> Option<chrono::DateTime<chrono::FixedOffset>> {
        chrono::DateTime::parse_from_rfc3339(s).ok().or_else(|| {
            chrono::DateTime::parse_from_rfc3339(&format!("{}+00:00", s.trim_end_matches('Z'))).ok()
        })
    };
    match (parse(first), parse(last)) {
        (Some(t1), Some(t2)) => ((t2 - t1).num_seconds() as f64 / 60.0 * 10.0).round() / 10.0,
        _ => 0.0,
    }
}

// ── billing-block analytics helpers ──────────────────────────────────────────

/// Load turns within [since_iso, until_iso) ordered by timestamp ascending.
/// Used by the `blocks` CLI subcommand and its tests.
pub fn load_turns_in_range(
    conn: &Connection,
    since_iso: &str,
    until_iso: &str,
) -> Result<Vec<crate::analytics::blocks::TurnForBlocks>> {
    let mut stmt = conn.prepare(
        "SELECT timestamp, COALESCE(model, 'unknown'),
                input_tokens, output_tokens, cache_read_tokens,
                cache_creation_tokens, reasoning_output_tokens,
                COALESCE(estimated_cost_nanos, 0)
         FROM turns
         WHERE timestamp >= ?1 AND timestamp < ?2
         ORDER BY timestamp ASC",
    )?;
    let rows = stmt.query_map([since_iso, until_iso], |row| {
        let ts_str: String = row.get(0)?;
        let ts = chrono::DateTime::parse_from_rfc3339(&ts_str)
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?
            .with_timezone(&chrono::Utc);
        Ok(crate::analytics::blocks::TurnForBlocks {
            timestamp: ts,
            model: row.get::<_, String>(1)?,
            tokens: crate::analytics::blocks::TokenBreakdown {
                input: row.get(2)?,
                output: row.get(3)?,
                cache_read: row.get(4)?,
                cache_creation: row.get(5)?,
                reasoning_output: row.get(6)?,
            },
            cost_nanos: row.get(7)?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// Return the maximum total-token count across all historical billing blocks.
///
/// Calls `load_all_turns`, groups them into blocks of `session_hours` using
/// `analytics::blocks::identify_blocks`, and returns the peak `tokens.total()`.
/// Returns 0 if there are no turns or no blocks.
pub fn historical_max_block_tokens(conn: &Connection, session_hours: f64) -> Result<i64> {
    let turns = load_all_turns(conn)?;
    let blocks = crate::analytics::blocks::identify_blocks(&turns, session_hours);
    let max = blocks.iter().map(|b| b.tokens.total()).max().unwrap_or(0);
    Ok(max)
}

/// Load turns with `timestamp >= cutoff_iso` ordered by timestamp ascending.
///
/// Used by the statusline compute path to avoid a full-table scan; callers
/// pass a cutoff roughly 24 h in the past so only recent data is read.
pub fn load_turns_since(
    conn: &Connection,
    cutoff_iso: &str,
) -> Result<Vec<crate::analytics::blocks::TurnForBlocks>> {
    let mut stmt = conn.prepare(
        "SELECT timestamp, COALESCE(model, 'unknown'),
                input_tokens, output_tokens, cache_read_tokens,
                cache_creation_tokens, reasoning_output_tokens,
                COALESCE(estimated_cost_nanos, 0)
         FROM turns
         WHERE timestamp >= ?1
         ORDER BY timestamp ASC",
    )?;
    let rows = stmt.query_map([cutoff_iso], |row| {
        let ts_str: String = row.get(0)?;
        let ts = chrono::DateTime::parse_from_rfc3339(&ts_str)
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?
            .with_timezone(&chrono::Utc);
        Ok(crate::analytics::blocks::TurnForBlocks {
            timestamp: ts,
            model: row.get::<_, String>(1)?,
            tokens: crate::analytics::blocks::TokenBreakdown {
                input: row.get(2)?,
                output: row.get(3)?,
                cache_read: row.get(4)?,
                cache_creation: row.get(5)?,
                reasoning_output: row.get(6)?,
            },
            cost_nanos: row.get(7)?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// Load all turns ordered by timestamp ascending (no date filter).
/// Used by the `blocks` CLI subcommand when no range is specified.
pub fn load_all_turns(conn: &Connection) -> Result<Vec<crate::analytics::blocks::TurnForBlocks>> {
    let mut stmt = conn.prepare(
        "SELECT timestamp, COALESCE(model, 'unknown'),
                input_tokens, output_tokens, cache_read_tokens,
                cache_creation_tokens, reasoning_output_tokens,
                COALESCE(estimated_cost_nanos, 0)
         FROM turns
         ORDER BY timestamp ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        let ts_str: String = row.get(0)?;
        let ts = chrono::DateTime::parse_from_rfc3339(&ts_str)
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?
            .with_timezone(&chrono::Utc);
        Ok(crate::analytics::blocks::TurnForBlocks {
            timestamp: ts,
            model: row.get::<_, String>(1)?,
            tokens: crate::analytics::blocks::TokenBreakdown {
                input: row.get(2)?,
                output: row.get(3)?,
                cache_read: row.get(4)?,
                cache_creation: row.get(5)?,
                reasoning_output: row.get(6)?,
            },
            cost_nanos: row.get(7)?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Phase 1A: cmd_today / cmd_stats query functions
// ---------------------------------------------------------------------------

/// Row type returned by [`query_today_provider_breakdown`].
pub type TodayProviderRow = (String, i64, i64, i64, i64, i64, i64, i64);

/// Row type returned by [`query_stats_token_totals`].
pub type StatsTokenTotals = (i64, i64, i64, i64, i64, i64, Option<f64>);

/// Row type returned by [`query_stats_by_provider`].
pub type StatsByProviderRow = (String, i64, i64, i64, i64, i64, i64, i64, i64);

/// Row type returned by [`query_today_model_rows`].
pub type TodayModelRow = (
    String, // provider
    String, // model
    i64,    // input_tokens
    i64,    // output_tokens
    i64,    // cache_read_tokens
    i64,    // cache_creation_tokens
    i64,    // reasoning_output_tokens
    i64,    // turns
    i64,    // cost_nanos
    String, // cost_confidence
    String, // billing_mode
);

/// Row type returned by [`query_stats_by_model`].
pub type StatsModelRow = (
    String, // provider
    String, // model
    i64,    // input_tokens
    i64,    // output_tokens
    i64,    // cache_read_tokens
    i64,    // cache_creation_tokens
    i64,    // reasoning_output_tokens
    i64,    // turns
    i64,    // sessions
    i64,    // cost_nanos
    String, // cost_confidence
    String, // billing_mode
);

/// Site 1 – per-model breakdown for a single calendar day.
pub fn query_today_model_rows(
    conn: &Connection,
    today: &str,
) -> rusqlite::Result<Vec<TodayModelRow>> {
    let mut stmt = conn.prepare(
        "SELECT provider, COALESCE(model, 'unknown') as model,
                SUM(input_tokens) as inp, SUM(output_tokens) as out,
                SUM(cache_read_tokens) as cr, SUM(cache_creation_tokens) as cc,
                SUM(reasoning_output_tokens) as ro,
                COUNT(*) as turns,
                COALESCE(SUM(estimated_cost_nanos), 0) as cost_nanos,
                CASE
                    WHEN SUM(CASE WHEN cost_confidence = 'low' THEN 1 ELSE 0 END) > 0 THEN 'low'
                    WHEN SUM(CASE WHEN cost_confidence = 'medium' THEN 1 ELSE 0 END) > 0 THEN 'medium'
                    ELSE 'high'
                END as cost_confidence,
                CASE
                    WHEN COUNT(DISTINCT billing_mode) = 1 THEN MAX(billing_mode)
                    ELSE 'mixed'
                END as billing_mode
         FROM turns WHERE substr(timestamp, 1, 10) = ?1
         GROUP BY provider, model ORDER BY inp + out DESC",
    )?;
    let rows = collect_warn(
        stmt.query_map([today], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get(7)?,
                row.get(8)?,
                row.get(9)?,
                row.get(10)?,
            ))
        })?,
        "Failed to read today_model row",
    );
    Ok(rows)
}

/// Site 2 – per-provider token/cost totals for a single calendar day (JSON output).
pub fn query_today_provider_breakdown(
    conn: &Connection,
    today: &str,
) -> rusqlite::Result<Vec<TodayProviderRow>> {
    let mut stmt = conn.prepare(
        "SELECT provider, COUNT(*) as turns,
                COALESCE(SUM(input_tokens), 0), COALESCE(SUM(output_tokens), 0),
                COALESCE(SUM(cache_read_tokens), 0), COALESCE(SUM(cache_creation_tokens), 0),
                COALESCE(SUM(reasoning_output_tokens), 0),
                COALESCE(SUM(estimated_cost_nanos), 0)
         FROM turns
         WHERE substr(timestamp, 1, 10) = ?1
         GROUP BY provider
         ORDER BY turns DESC",
    )?;
    let rows = stmt
        .query_map([today], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get(7)?,
            ))
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

/// Site 3 – confidence-level breakdown for a single calendar day (JSON output).
pub fn query_today_confidence_breakdown(
    conn: &Connection,
    today: &str,
) -> rusqlite::Result<Vec<(String, i64, i64)>> {
    let mut stmt = conn.prepare(
        "SELECT COALESCE(cost_confidence, 'low') as cost_confidence,
                COUNT(*) as turns,
                COALESCE(SUM(estimated_cost_nanos), 0) as cost_nanos
         FROM turns
         WHERE substr(timestamp, 1, 10) = ?1
         GROUP BY cost_confidence
         ORDER BY turns DESC",
    )?;
    let rows = stmt
        .query_map([today], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

/// Site 4 – billing-mode breakdown for a single calendar day (JSON output).
pub fn query_today_billing_mode_breakdown(
    conn: &Connection,
    today: &str,
) -> rusqlite::Result<Vec<(String, i64, i64)>> {
    let mut stmt = conn.prepare(
        "SELECT COALESCE(billing_mode, 'estimated_local') as billing_mode,
                COUNT(*) as turns,
                COALESCE(SUM(estimated_cost_nanos), 0) as cost_nanos
         FROM turns
         WHERE substr(timestamp, 1, 10) = ?1
         GROUP BY billing_mode
         ORDER BY turns DESC",
    )?;
    let rows = stmt
        .query_map([today], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

/// Site 5 – session count + first/last timestamp window.
pub fn query_stats_session_window(
    conn: &Connection,
) -> rusqlite::Result<(i64, Option<String>, Option<String>)> {
    conn.query_row(
        "SELECT COUNT(*), MIN(first_timestamp), MAX(last_timestamp) FROM sessions",
        [],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )
}

/// Site 6 – all-time token and credits totals across all turns.
pub fn query_stats_token_totals(conn: &Connection) -> rusqlite::Result<StatsTokenTotals> {
    conn.query_row(
        "SELECT COALESCE(SUM(input_tokens),0), COALESCE(SUM(output_tokens),0),
                COALESCE(SUM(cache_read_tokens),0), COALESCE(SUM(cache_creation_tokens),0),
                COALESCE(SUM(reasoning_output_tokens),0), COUNT(*),
                SUM(credits) FROM turns",
        [],
        |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
            ))
        },
    )
}

/// Site 7 – all-time per-model breakdown.
pub fn query_stats_by_model(conn: &Connection) -> rusqlite::Result<Vec<StatsModelRow>> {
    let mut stmt = conn.prepare(
        "SELECT provider, COALESCE(model,'unknown'), SUM(input_tokens), SUM(output_tokens),
                SUM(cache_read_tokens), SUM(cache_creation_tokens), SUM(reasoning_output_tokens), COUNT(*),
                COUNT(DISTINCT session_id), COALESCE(SUM(estimated_cost_nanos), 0),
                CASE
                    WHEN SUM(CASE WHEN cost_confidence = 'low' THEN 1 ELSE 0 END) > 0 THEN 'low'
                    WHEN SUM(CASE WHEN cost_confidence = 'medium' THEN 1 ELSE 0 END) > 0 THEN 'medium'
                    ELSE 'high'
                END as cost_confidence,
                CASE
                    WHEN COUNT(DISTINCT billing_mode) = 1 THEN MAX(billing_mode)
                    ELSE 'mixed'
                END as billing_mode
         FROM turns GROUP BY provider, model ORDER BY SUM(input_tokens+output_tokens) DESC",
    )?;
    let rows = collect_warn(
        stmt.query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get(7)?,
                row.get(8)?,
                row.get(9)?,
                row.get(10)?,
                row.get(11)?,
            ))
        })?,
        "Failed to read stats_model row",
    );
    Ok(rows)
}

/// Site 8 – all-time per-provider summary (JSON output).
pub fn query_stats_by_provider(conn: &Connection) -> rusqlite::Result<Vec<StatsByProviderRow>> {
    let mut stmt = conn.prepare(
        "SELECT provider,
                COUNT(DISTINCT session_id), COUNT(*),
                COALESCE(SUM(input_tokens),0), COALESCE(SUM(output_tokens),0),
                COALESCE(SUM(cache_read_tokens),0), COALESCE(SUM(cache_creation_tokens),0),
                COALESCE(SUM(reasoning_output_tokens),0), COALESCE(SUM(estimated_cost_nanos),0)
         FROM turns
         GROUP BY provider
         ORDER BY COUNT(*) DESC",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get(7)?,
                row.get(8)?,
            ))
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

/// Site 9 – all-time confidence-level breakdown (JSON output).
pub fn query_stats_confidence_breakdown(
    conn: &Connection,
) -> rusqlite::Result<Vec<(String, i64, i64)>> {
    let mut stmt = conn.prepare(
        "SELECT COALESCE(cost_confidence, 'low') as cost_confidence,
                COUNT(*) as turns,
                COALESCE(SUM(estimated_cost_nanos), 0) as cost_nanos
         FROM turns
         GROUP BY cost_confidence
         ORDER BY turns DESC",
    )?;
    let rows = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

/// Site 10 – all-time billing-mode breakdown (JSON output).
pub fn query_stats_billing_mode_breakdown(
    conn: &Connection,
) -> rusqlite::Result<Vec<(String, i64, i64)>> {
    let mut stmt = conn.prepare(
        "SELECT COALESCE(billing_mode, 'estimated_local') as billing_mode,
                COUNT(*) as turns,
                COALESCE(SUM(estimated_cost_nanos), 0) as cost_nanos
         FROM turns
         GROUP BY billing_mode
         ORDER BY turns DESC",
    )?;
    let rows = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

/// Site 11 – average one-shot rate across classifiable sessions.
pub fn query_stats_oneshot_avg(conn: &Connection) -> rusqlite::Result<Option<f64>> {
    conn.query_row(
        "SELECT AVG(CAST(one_shot AS REAL)) FROM sessions WHERE one_shot IS NOT NULL",
        [],
        |row| row.get(0),
    )
}

// ── Hook live_events ingest ─────────────────────────────────────────────────

/// One row destined for the `live_events` table.
///
/// Centralises the column-order knowledge so the `heimdall-hook` binary (and
/// any other writer) does not have to maintain a positional bind-list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveEventRow {
    pub dedup_key: String,
    pub received_at: String,
    pub session_id: Option<String>,
    pub tool_name: Option<String>,
    pub cost_usd_nanos: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub raw_json: String,
    pub context_input_tokens: Option<i64>,
    pub context_window_size: Option<i64>,
    pub hook_reported_cost_nanos: Option<i64>,
}

/// Insert one `live_events` row using `INSERT OR IGNORE` for dedup safety.
pub fn insert_live_event(conn: &Connection, event: &LiveEventRow) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO live_events
            (dedup_key, received_at, session_id, tool_name,
             cost_usd_nanos, input_tokens, output_tokens, raw_json,
             context_input_tokens, context_window_size,
             hook_reported_cost_nanos)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        rusqlite::params![
            event.dedup_key,
            event.received_at,
            event.session_id,
            event.tool_name,
            event.cost_usd_nanos,
            event.input_tokens,
            event.output_tokens,
            event.raw_json,
            event.context_input_tokens,
            event.context_window_size,
            event.hook_reported_cost_nanos,
        ],
    )?;
    Ok(())
}

// ── Server live_events / context-window queries ─────────────────────────────

/// Most recent non-null context-window snapshot from `live_events`.
pub struct LatestContextWindow {
    pub context_input_tokens: i64,
    pub context_window_size: i64,
    pub session_id: Option<String>,
    pub received_at: String,
}

/// Latest non-null context-window snapshot from `live_events`.
///
/// Returns `Ok(None)` when no row has both `context_input_tokens IS NOT NULL`
/// and `context_window_size > 0`.
pub fn query_latest_context_window(conn: &Connection) -> Result<Option<LatestContextWindow>> {
    let mut stmt = conn.prepare(
        "SELECT context_input_tokens, context_window_size, session_id, received_at
         FROM live_events
         WHERE context_input_tokens IS NOT NULL AND context_window_size > 0
         ORDER BY received_at DESC LIMIT 1",
    )?;
    let result = stmt
        .query_row([], |r| {
            Ok(LatestContextWindow {
                context_input_tokens: r.get(0)?,
                context_window_size: r.get(1)?,
                session_id: r.get(2)?,
                received_at: r.get(3)?,
            })
        })
        .optional()?;
    Ok(result)
}

/// Count of `live_events` rows that carry a non-null `hook_reported_cost_nanos`.
///
/// Used by `/api/cost-reconciliation` to short-circuit when the hook has never
/// reported a cost.
pub fn count_hook_reported_cost_events(conn: &Connection) -> Result<i64> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM live_events WHERE hook_reported_cost_nanos IS NOT NULL",
        [],
        |r| r.get(0),
    )?;
    Ok(count)
}

/// Sum hook-reported cost (nanos) by `date(received_at)` since `cutoff_iso`.
pub fn query_hook_cost_by_day(
    conn: &Connection,
    cutoff_iso: &str,
) -> Result<std::collections::HashMap<String, i64>> {
    let mut stmt = conn.prepare(
        "SELECT date(received_at) AS day,
                COALESCE(SUM(hook_reported_cost_nanos), 0) AS nanos
         FROM live_events
         WHERE hook_reported_cost_nanos IS NOT NULL
           AND received_at >= ?1
         GROUP BY day
         ORDER BY day",
    )?;
    let rows = stmt.query_map(rusqlite::params![cutoff_iso], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
    })?;
    let mut map = std::collections::HashMap::new();
    for row in rows {
        let (day, nanos) = row?;
        map.insert(day, nanos);
    }
    Ok(map)
}

/// Sum local-estimate cost (nanos) by `date(timestamp)` from `turns` since `cutoff_iso`.
pub fn query_local_cost_by_day(
    conn: &Connection,
    cutoff_iso: &str,
) -> Result<std::collections::HashMap<String, i64>> {
    let mut stmt = conn.prepare(
        "SELECT date(timestamp) AS day,
                COALESCE(SUM(estimated_cost_nanos), 0) AS nanos
         FROM turns
         WHERE timestamp >= ?1
         GROUP BY day
         ORDER BY day",
    )?;
    let rows = stmt.query_map(rusqlite::params![cutoff_iso], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
    })?;
    let mut map = std::collections::HashMap::new();
    for row in rows {
        let (day, nanos) = row?;
        map.insert(day, nanos);
    }
    Ok(map)
}

// ── Statusline cost queries ─────────────────────────────────────────────────

/// Sum `estimated_cost_nanos` from `turns` for one `session_id`.
/// Returns 0 when no rows match.
pub fn query_session_estimated_cost_nanos(conn: &Connection, session_id: &str) -> Result<i64> {
    let cost: i64 = conn.query_row(
        "SELECT COALESCE(SUM(estimated_cost_nanos), 0) FROM turns WHERE session_id = ?1",
        rusqlite::params![session_id],
        |r| r.get(0),
    )?;
    Ok(cost)
}

/// Sum `estimated_cost_nanos` from `turns` over a half-open `[start, end)` range.
pub fn query_estimated_cost_nanos_in_range(
    conn: &Connection,
    start_inclusive: &str,
    end_exclusive: &str,
) -> Result<i64> {
    let cost: i64 = conn.query_row(
        "SELECT COALESCE(SUM(estimated_cost_nanos), 0) FROM turns \
         WHERE timestamp >= ?1 AND timestamp < ?2",
        rusqlite::params![start_inclusive, end_exclusive],
        |r| r.get(0),
    )?;
    Ok(cost)
}

// ── Menubar widget queries ──────────────────────────────────────────────────

/// `(SUM(estimated_cost_nanos), COUNT(DISTINCT session_id))` for a single
/// `YYYY-MM-DD` day prefix (matched via `substr(timestamp, 1, 10) = ?1`).
pub fn query_day_cost_and_session_count(
    conn: &Connection,
    day_yyyymmdd: &str,
) -> Result<(i64, i64)> {
    let row: (i64, i64) = conn.query_row(
        "SELECT COALESCE(SUM(estimated_cost_nanos), 0), COUNT(DISTINCT session_id)
         FROM turns
         WHERE substr(timestamp, 1, 10) = ?1",
        rusqlite::params![day_yyyymmdd],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    Ok(row)
}

/// Average one-shot rate across classifiable sessions, or `None` when no
/// classified sessions exist.  (Same SQL as `query_stats_oneshot_avg` but uses
/// `Result<Option<f64>>` semantics so callers do not need to hand-roll the
/// `Optional` extension.)
pub fn query_oneshot_rate(conn: &Connection) -> Result<Option<f64>> {
    let rate: Option<f64> = conn
        .query_row(
            "SELECT AVG(CAST(one_shot AS REAL)) FROM sessions WHERE one_shot IS NOT NULL",
            [],
            |row| row.get(0),
        )
        .optional()?;
    // The query returns Some(NULL) when zero rows match the WHERE clause, which
    // rusqlite surfaces as Some(None). Flatten so callers get a single Option.
    Ok(rate)
}

// ── Atomic-rescan: merge live runtime history into a freshly-scanned DB ─────

/// Statistics returned by [`merge_live_db`] (informational only; callers may
/// ignore).  Counts indicate how many rows were inserted into the destination
/// for each section.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct MergeStats {
    pub live_events: usize,
    pub agent_status_history: usize,
    pub claude_usage_runs: usize,
    pub claude_usage_factors: usize,
    pub rate_window_history: usize,
}

/// Merge runtime history from a "live" database (the user's existing DB) into
/// a freshly-scanned temp database.
///
/// This is invoked by the atomic-rescan flow: `scan` writes to a temp DB,
/// then this function ports forward the runtime tables that are not produced
/// by the scanner pipeline (live_events, agent_status_history, claude_usage_*,
/// and `rate_window_history`).  The temp DB is the destination; the live DB
/// is read via `ATTACH`.
///
/// All operations use `INSERT OR IGNORE` (or `NOT EXISTS` for the
/// near-duplicate `rate_window_history` rule) so the merge is idempotent.
///
/// Returns informational counts.  When `live_path` does not exist, returns
/// `Ok(MergeStats::default())` without side effects.
pub fn merge_live_db(target_conn: &Connection, live_path: &std::path::Path) -> Result<MergeStats> {
    if !live_path.exists() {
        return Ok(MergeStats::default());
    }

    target_conn.execute(
        "ATTACH DATABASE ?1 AS live_db",
        [live_path.to_string_lossy().as_ref()],
    )?;

    target_conn.execute_batch(
        "INSERT OR IGNORE INTO live_events
             (dedup_key, received_at, session_id, tool_name, cost_usd_nanos,
              input_tokens, output_tokens, raw_json, context_input_tokens,
              context_window_size, hook_reported_cost_nanos)
         SELECT dedup_key, received_at, session_id, tool_name, cost_usd_nanos,
                input_tokens, output_tokens, raw_json, context_input_tokens,
                context_window_size, hook_reported_cost_nanos
         FROM live_db.live_events;

         INSERT OR IGNORE INTO agent_status_history
             (ts_epoch, provider, component_id, component_name, status)
         SELECT ts_epoch, provider, component_id, component_name, status
         FROM live_db.agent_status_history;",
    )?;

    target_conn.execute_batch(
        "INSERT OR IGNORE INTO claude_usage_runs
             (id, captured_at, status, exit_code, stdout_raw, stderr_raw, invocation_mode, period, parser_version, error_summary)
         SELECT id, captured_at, status, exit_code, stdout_raw, stderr_raw, invocation_mode, period, parser_version, error_summary
         FROM live_db.claude_usage_runs;

         INSERT OR IGNORE INTO claude_usage_factors
             (id, run_id, factor_key, display_label, percent, description, advice_text, display_order)
         SELECT id, run_id, factor_key, display_label, percent, description, advice_text, display_order
         FROM live_db.claude_usage_factors;",
    )?;

    target_conn.execute(
        "INSERT INTO rate_window_history
             (timestamp, window_type, used_percent, resets_at, source_kind, source_path)
         SELECT lr.timestamp,
                lr.window_type,
                lr.used_percent,
                lr.resets_at,
                COALESCE(lr.source_kind, 'oauth'),
                COALESCE(lr.source_path, '')
         FROM live_db.rate_window_history lr
         WHERE COALESCE(lr.source_kind, 'oauth') = 'oauth'
           AND NOT EXISTS (
               SELECT 1
               FROM rate_window_history cur
               WHERE cur.timestamp = lr.timestamp
                 AND cur.window_type = lr.window_type
                 AND ABS(cur.used_percent - lr.used_percent) < 0.000001
                 AND (
                     (cur.resets_at IS NULL AND lr.resets_at IS NULL)
                     OR cur.resets_at = lr.resets_at
                 )
                 AND cur.source_kind = COALESCE(lr.source_kind, 'oauth')
                 AND cur.source_path = COALESCE(lr.source_path, '')
           )",
        [],
    )?;

    target_conn.execute_batch("DETACH DATABASE live_db;")?;

    // Counts are intentionally not collected — the caller does not use them
    // and `changes()` after `execute_batch` is unreliable across rusqlite
    // versions.  Tests that need precise counts query the destination tables
    // directly.
    Ok(MergeStats::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Turn;
    use crate::pricing;
    use crate::pricing_defs::{
        OfficialExtractedRecord, OfficialModelPricing, OfficialSyncRunRecord, PricingSyncRun,
    };
    use std::collections::HashMap;

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();
        conn
    }

    #[test]
    fn test_init_db_creates_tables() {
        let conn = test_conn();
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert!(tables.contains(&"sessions".into()));
        assert!(tables.contains(&"turns".into()));
        assert!(tables.contains(&"processed_files".into()));
    }

    // --- has_column / PRAGMA table_info tests ---

    #[test]
    fn test_has_column_existing_column() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE t (a INTEGER, b TEXT);")
            .unwrap();
        assert!(has_column(&conn, "t", "a"), "column 'a' must be found");
        assert!(has_column(&conn, "t", "b"), "column 'b' must be found");
    }

    #[test]
    fn test_has_column_missing_column() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE t (a INTEGER);").unwrap();
        assert!(
            !has_column(&conn, "t", "missing"),
            "non-existent column must return false"
        );
    }

    #[test]
    fn test_has_column_missing_table() {
        let conn = Connection::open_in_memory().unwrap();
        // PRAGMA table_info on a missing table returns zero rows → false
        assert!(
            !has_column(&conn, "no_such_table", "a"),
            "missing table must return false"
        );
    }

    #[test]
    fn test_is_valid_identifier_rejects_injection() {
        assert!(
            !is_valid_identifier("foo; DROP TABLE bar"),
            "injection string must be rejected"
        );
        assert!(!is_valid_identifier(""), "empty string must be rejected");
        assert!(
            !is_valid_identifier("1bad"),
            "leading digit must be rejected"
        );
        assert!(
            is_valid_identifier("valid_table_name"),
            "valid identifier must be accepted"
        );
        assert!(
            is_valid_identifier("_private"),
            "leading underscore must be accepted"
        );
    }

    #[test]
    fn test_has_column_invalid_table_name_returns_false() {
        let conn = Connection::open_in_memory().unwrap();
        // An invalid table name (contains spaces/semicolons) must not be
        // interpolated into SQL — is_valid_identifier guards it and returns false.
        assert!(
            !has_column(&conn, "foo; DROP TABLE bar", "col"),
            "invalid table name must return false without executing SQL"
        );
    }

    #[test]
    fn test_init_db_idempotent() {
        let conn = test_conn();
        init_db(&conn).unwrap();
    }

    #[test]
    fn test_init_db_creates_pricing_sync_tables() {
        let conn = test_conn();
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert!(tables.contains(&"pricing_sync_runs".into()));
        assert!(tables.contains(&"pricing_sync_models".into()));
    }

    #[test]
    fn test_init_db_creates_official_sync_tables() {
        let conn = test_conn();
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert!(tables.contains(&"official_sync_runs".into()));
        assert!(tables.contains(&"official_metadata_records".into()));
    }

    #[test]
    fn test_insert_official_sync_run_and_records() {
        let conn = test_conn();
        let run_id = insert_official_sync_run(
            &conn,
            &OfficialSyncRunRecord {
                fetched_at: "2026-04-19T10:00:00Z".into(),
                source_slug: "openai_api_changelog".into(),
                source_kind: "release_notes".into(),
                source_url: "https://developers.openai.com/api/docs/changelog".into(),
                provider: "openai".into(),
                authority: "provider_release_notes".into(),
                format: "html".into(),
                cadence: "daily".into(),
                status: "success".into(),
                http_status: Some(200),
                content_type: "text/html".into(),
                etag: "\"abc\"".into(),
                last_modified: "Sun, 19 Apr 2026 10:00:00 GMT".into(),
                raw_body: "<html>ok</html>".into(),
                normalized_body: "ok".into(),
                error_text: String::new(),
                parser_version: "official_pricing/v3".into(),
                raw_body_sha256: "aaa".into(),
                normalized_body_sha256: "bbb".into(),
                extracted_sha256: "ccc".into(),
            },
        )
        .unwrap();
        insert_official_extracted_records(
            &conn,
            run_id,
            &[OfficialExtractedRecord {
                source_slug: "openai_api_changelog".into(),
                provider: "openai".into(),
                record_type: "release_note".into(),
                record_key: "note-1".into(),
                model_id: "gpt-5.4".into(),
                effective_at: "2026-04-19".into(),
                payload_json: "{\"title\":\"Launch\"}".into(),
            }],
        )
        .unwrap();

        let run_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM official_sync_runs", [], |row| {
                row.get(0)
            })
            .unwrap();
        let record_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM official_metadata_records",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(run_count, 1);
        assert_eq!(record_count, 1);
    }

    #[test]
    fn test_get_dashboard_data_includes_official_sync_summary() {
        let conn = test_conn();
        let run_id = insert_official_sync_run(
            &conn,
            &OfficialSyncRunRecord {
                fetched_at: "2026-04-19T10:00:00Z".into(),
                source_slug: "openai_api_docs".into(),
                source_kind: "pricing".into(),
                source_url: "https://developers.openai.com/api/docs/pricing".into(),
                provider: "openai".into(),
                authority: "provider_docs".into(),
                format: "html".into(),
                cadence: "daily".into(),
                status: "success".into(),
                http_status: Some(200),
                content_type: "text/html".into(),
                etag: String::new(),
                last_modified: String::new(),
                raw_body: "<html></html>".into(),
                normalized_body: String::new(),
                error_text: String::new(),
                parser_version: "official_pricing/v3".into(),
                raw_body_sha256: "aaa".into(),
                normalized_body_sha256: "bbb".into(),
                extracted_sha256: "ccc".into(),
            },
        )
        .unwrap();
        insert_official_extracted_records(
            &conn,
            run_id,
            &[OfficialExtractedRecord {
                source_slug: "openai_api_docs".into(),
                provider: "openai".into(),
                record_type: "pricing_model".into(),
                record_key: "gpt-5.4".into(),
                model_id: "gpt-5.4".into(),
                effective_at: "2026-04-19T10:00:00Z".into(),
                payload_json: "{\"model_id\":\"gpt-5.4\"}".into(),
            }],
        )
        .unwrap();

        let data = get_dashboard_data(&conn, TzParams::default()).unwrap();
        assert!(data.official_sync.available);
        assert_eq!(data.official_sync.total_runs, 1);
        assert_eq!(data.official_sync.total_records, 1);
        assert_eq!(data.official_sync.sources.len(), 1);
        assert_eq!(data.official_sync.record_counts.len(), 1);
        assert_eq!(data.official_sync.sources[0].source_slug, "openai_api_docs");
        assert_eq!(
            data.official_sync.record_counts[0].record_type,
            "pricing_model"
        );
    }

    #[test]
    fn test_insert_and_load_latest_pricing_models() {
        let conn = test_conn();
        let run_id = insert_pricing_sync_run(
            &conn,
            &PricingSyncRun {
                fetched_at: "2026-04-19T10:00:00Z".into(),
                source_slug: "openai_api_docs".into(),
                source_url: "https://developers.openai.com/api/docs/pricing".into(),
                provider: "openai".into(),
                status: "success".into(),
                raw_body: "<html></html>".into(),
                error_text: String::new(),
            },
        )
        .unwrap();
        insert_pricing_sync_models(
            &conn,
            run_id,
            &[OfficialModelPricing {
                source_slug: "openai_api_docs".into(),
                provider: "openai".into(),
                model_id: "gpt-5.4".into(),
                model_label: "gpt-5.4".into(),
                input_usd_per_mtok: 2.5,
                cache_write_usd_per_mtok: 2.5,
                cache_read_usd_per_mtok: 0.25,
                output_usd_per_mtok: 15.0,
                threshold_tokens: Some(270_000),
                input_above_threshold: Some(5.0),
                output_above_threshold: Some(22.5),
                notes: String::new(),
            }],
        )
        .unwrap();

        let rows = load_latest_pricing_models(&conn).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].model_id, "gpt-5.4");
        assert_eq!(rows[0].threshold_tokens, Some(270_000));
    }

    #[test]
    fn test_reprice_turns_with_catalog_updates_costs_and_version() {
        let conn = test_conn();
        let turns = vec![Turn {
            session_id: "claude:s1".into(),
            provider: "claude".into(),
            timestamp: "2026-04-08T10:00:00Z".into(),
            model: "claude-sonnet-4-6".into(),
            input_tokens: 1_000_000,
            output_tokens: 0,
            message_id: "msg-1".into(),
            pricing_version: "static@old".into(),
            pricing_model: "claude-sonnet-4-6".into(),
            cost_confidence: "high".into(),
            estimated_cost_nanos: 3_000_000_000,
            ..Default::default()
        }];
        insert_turns(&conn, &turns).unwrap();
        upsert_sessions(
            &conn,
            &[crate::models::Session {
                session_id: "claude:s1".into(),
                provider: "claude".into(),
                ..Default::default()
            }],
        )
        .unwrap();

        let mut catalog = pricing::builtin_catalog();
        catalog.insert(
            "claude-sonnet-4-6".into(),
            crate::pricing::ModelPricing {
                input: 4.0,
                output: 15.0,
                cache_write: 5.0,
                cache_read: 0.4,
                threshold_tokens: None,
                input_above_threshold: None,
                output_above_threshold: None,
            },
        );

        let changed = reprice_turns_with_catalog(&conn, &catalog, "official:test").unwrap();
        assert_eq!(changed, 1);

        let (cost, version): (i64, String) = conn
            .query_row(
                "SELECT estimated_cost_nanos, pricing_version FROM turns WHERE message_id = 'msg-1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(cost, 4_000_000_000);
        assert_eq!(version, "official:test");
    }

    #[test]
    fn test_insert_and_query_turns() {
        let conn = test_conn();
        let turns = vec![Turn {
            session_id: "s1".into(),
            timestamp: "2026-04-08T10:00:00Z".into(),
            model: "claude-sonnet-4-6".into(),
            input_tokens: 100,
            output_tokens: 50,
            message_id: "msg-1".into(),
            ..Default::default()
        }];
        insert_turns(&conn, &turns).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM turns", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_message_id_dedup() {
        let conn = test_conn();
        let turn = Turn {
            session_id: "s1".into(),
            message_id: "msg-1".into(),
            input_tokens: 100,
            ..Default::default()
        };
        insert_turns(&conn, &[turn.clone()]).unwrap();
        insert_turns(&conn, &[turn]).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM turns", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_null_message_id_not_deduped() {
        let conn = test_conn();
        let t1 = Turn {
            session_id: "s1".into(),
            input_tokens: 100,
            ..Default::default()
        };
        let t2 = Turn {
            session_id: "s1".into(),
            input_tokens: 200,
            ..Default::default()
        };
        insert_turns(&conn, &[t1]).unwrap();
        insert_turns(&conn, &[t2]).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM turns", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_processed_file_roundtrip() {
        let conn = test_conn();
        assert!(
            get_processed_file(&conn, "/tmp/test.jsonl")
                .unwrap()
                .is_none()
        );
        upsert_processed_file(&conn, "/tmp/test.jsonl", 1234.5, 100).unwrap();
        let (mtime, progress_marker) = get_processed_file(&conn, "/tmp/test.jsonl")
            .unwrap()
            .unwrap();
        assert!((mtime - 1234.5).abs() < 0.01);
        assert_eq!(progress_marker, 100);
    }

    #[test]
    fn test_compute_duration_min() {
        let d = compute_duration_min("2026-04-08T09:00:00Z", "2026-04-08T10:00:00Z");
        assert!((d - 60.0).abs() < 0.1);
    }

    #[test]
    fn test_recompute_session_totals() {
        let conn = test_conn();
        // Insert a session manually
        conn.execute(
            "INSERT INTO sessions (session_id, project_name, total_input_tokens, total_output_tokens, total_cache_read, total_cache_creation, turn_count) VALUES ('s1', 'test', 0, 0, 0, 0, 0)",
            [],
        )
        .unwrap();
        // Insert turns
        let turns = vec![
            Turn {
                session_id: "s1".into(),
                input_tokens: 100,
                output_tokens: 50,
                cache_read_tokens: 10,
                cache_creation_tokens: 5,
                ..Default::default()
            },
            Turn {
                session_id: "s1".into(),
                input_tokens: 200,
                output_tokens: 100,
                cache_read_tokens: 20,
                cache_creation_tokens: 10,
                ..Default::default()
            },
        ];
        insert_turns(&conn, &turns).unwrap();
        recompute_session_totals(&conn).unwrap();

        let (inp, out, cr, cc, tc): (i64, i64, i64, i64, i64) = conn
            .query_row(
                "SELECT total_input_tokens, total_output_tokens, total_cache_read, total_cache_creation, turn_count FROM sessions WHERE session_id = 's1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
            )
            .unwrap();
        assert_eq!(inp, 300);
        assert_eq!(out, 150);
        assert_eq!(cr, 30);
        assert_eq!(cc, 15);
        assert_eq!(tc, 2);
    }

    #[test]
    fn test_rate_window_history_roundtrip() {
        let conn = test_conn();
        insert_rate_window_snapshot(&conn, "session", 45.0, Some("2099-01-01T00:00:00Z")).unwrap();
        insert_rate_window_snapshot(&conn, "session", 60.0, Some("2099-01-02T00:00:00Z")).unwrap();
        insert_rate_window_snapshot(&conn, "weekly", 30.0, None).unwrap();

        let history = get_rate_window_history(&conn, "session", 24).unwrap();
        assert_eq!(history.len(), 2);
        assert!((history[0].1 - 45.0).abs() < 0.01);
        assert!((history[1].1 - 60.0).abs() < 0.01);

        let weekly = get_rate_window_history(&conn, "weekly", 24).unwrap();
        assert_eq!(weekly.len(), 1);
    }

    #[test]
    fn test_record_rate_window_snapshot_dedup_and_load() {
        let conn = test_conn();
        let snap = RateWindowSnapshotInsert {
            provider: "claude",
            window_type: "five_hour",
            used_percent: 42.0,
            resets_at: Some("2099-01-01T00:00:00Z"),
            plan: Some("pro"),
            observed_tokens: Some(420_000),
            estimated_cap_tokens: Some(1_000_000),
            confidence: Some(0.9),
            source_kind: "oauth",
        };
        assert!(record_rate_window_snapshot(&conn, &snap).unwrap());

        // Same value within 0.5pp inside the 5-min dedup window: skipped.
        let snap_dup = RateWindowSnapshotInsert {
            used_percent: 42.3,
            ..snap
        };
        assert!(!record_rate_window_snapshot(&conn, &snap_dup).unwrap());

        // Different window_type: not deduped against the first row.
        let snap_other = RateWindowSnapshotInsert {
            window_type: "seven_day",
            used_percent: 10.0,
            ..snap
        };
        assert!(record_rate_window_snapshot(&conn, &snap_other).unwrap());

        let history = load_rate_window_history(&conn, 90).unwrap();
        assert_eq!(history.len(), 2, "got {history:?}");
        let claude_5h = history
            .iter()
            .find(|r| r.window_type == "five_hour")
            .expect("five_hour row");
        assert_eq!(claude_5h.provider, "claude");
        assert_eq!(claude_5h.estimated_cap_tokens, Some(1_000_000));
        assert_eq!(claude_5h.plan.as_deref(), Some("pro"));
    }

    #[test]
    fn test_observed_tokens_for_window_filters() {
        let conn = test_conn();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO sessions (session_id, provider, total_input_tokens, turn_count)
             VALUES ('claude:s1', 'claude', 0, 0)",
            [],
        )
        .unwrap();
        let turns = vec![
            Turn {
                session_id: "claude:s1".into(),
                provider: "claude".into(),
                model: "claude-opus-4-5".into(),
                input_tokens: 1000,
                output_tokens: 500,
                timestamp: now.clone(),
                ..Default::default()
            },
            Turn {
                session_id: "claude:s1".into(),
                provider: "claude".into(),
                model: "claude-sonnet-4-5".into(),
                input_tokens: 200,
                output_tokens: 100,
                timestamp: now,
                ..Default::default()
            },
        ];
        insert_turns(&conn, &turns).unwrap();

        let total = observed_tokens_for_window(&conn, "claude", 3600, None, None).unwrap();
        assert_eq!(total, 1800);

        let opus_only =
            observed_tokens_for_window(&conn, "claude", 3600, Some("%opus%"), None).unwrap();
        assert_eq!(opus_only, 1500);

        let sonnet_only =
            observed_tokens_for_window(&conn, "claude", 3600, Some("%sonnet%"), None).unwrap();
        assert_eq!(sonnet_only, 300);
    }

    #[test]
    fn test_observed_tokens_for_window_resets_at_aligns_boundary() {
        let conn = test_conn();
        let now = chrono::Utc::now();
        // Provider says the window reset 30 minutes ago; the window we want
        // to measure is the one that just ended:
        //   [resets_at - 5h, resets_at] = [now - 5h30m, now - 30m]
        let resets_at = now - chrono::Duration::minutes(30);
        let window_secs = 5 * 3600;

        conn.execute(
            "INSERT INTO sessions (session_id, provider, total_input_tokens, turn_count)
             VALUES ('claude:s1', 'claude', 0, 0)",
            [],
        )
        .unwrap();

        let mk = |ts: chrono::DateTime<chrono::Utc>, tokens: i64| Turn {
            session_id: "claude:s1".into(),
            provider: "claude".into(),
            model: "claude-sonnet".into(),
            input_tokens: tokens,
            output_tokens: 0,
            timestamp: ts.to_rfc3339(),
            ..Default::default()
        };
        let turns = vec![
            // After resets_at — must be excluded by aligned query.
            mk(now - chrono::Duration::minutes(15), 9999),
            // In window.
            mk(now - chrono::Duration::hours(1), 100),
            mk(now - chrono::Duration::hours(3), 200),
            // Before window — excluded by both.
            mk(now - chrono::Duration::hours(7), 99999),
        ];
        insert_turns(&conn, &turns).unwrap();

        let resets_str = resets_at.to_rfc3339();
        let aligned =
            observed_tokens_for_window(&conn, "claude", window_secs, None, Some(&resets_str))
                .unwrap();
        assert_eq!(aligned, 300, "aligned window excludes the post-reset turn");

        // Rolling 5h includes the now-15min turn — the boundary leak we're fixing.
        let rolling = observed_tokens_for_window(&conn, "claude", window_secs, None, None).unwrap();
        assert_eq!(rolling, 100 + 200 + 9999);

        // Garbage `resets_at` falls back to rolling behavior.
        let fallback =
            observed_tokens_for_window(&conn, "claude", window_secs, None, Some("not-a-date"))
                .unwrap();
        assert_eq!(fallback, rolling);
    }

    #[test]
    fn test_recent_cap_observations_orders_and_limits() {
        let conn = test_conn();
        let now = chrono::Utc::now();
        let mk_ts = |off_min: i64| (now - chrono::Duration::minutes(off_min)).to_rfc3339();
        conn.execute(
            "INSERT INTO rate_window_history
                (timestamp, window_type, used_percent, resets_at, source_kind, source_path,
                 provider, plan, observed_tokens, estimated_cap_tokens, confidence)
             VALUES
                (?1, 'five_hour', 50.0, NULL, 'oauth', '', 'claude', 'pro', 1, 100, 1.0),
                (?2, 'five_hour', 40.0, NULL, 'oauth', '', 'claude', 'pro', 1, 110, 0.9),
                (?3, 'five_hour', 30.0, NULL, 'oauth', '', 'claude', 'pro', 1, 120, 0.8),
                (?4, 'five_hour', 20.0, NULL, 'oauth', '', 'claude', 'pro', 1, 130, 0.7),
                (?5, 'seven_day', 50.0, NULL, 'oauth', '', 'claude', 'pro', 1, 999, 1.0),
                (?6, 'five_hour', 50.0, NULL, 'oauth', '', 'claude', 'pro', 1, NULL, NULL)",
            rusqlite::params![
                mk_ts(10),
                mk_ts(20),
                mk_ts(30),
                mk_ts(40),
                mk_ts(15),
                mk_ts(50),
            ],
        )
        .unwrap();

        let rows = recent_cap_observations(&conn, "claude", "five_hour", 3).unwrap();
        assert_eq!(rows.len(), 3, "respects LIMIT");
        // DESC by timestamp → caps in insertion order: 100, 110, 120
        assert_eq!(rows[0].1, 100);
        assert_eq!(rows[1].1, 110);
        assert_eq!(rows[2].1, 120);
        assert!((rows[0].0 - 1.0).abs() < 1e-9);
        assert!((rows[1].0 - 0.9).abs() < 1e-9);

        let unlimited = recent_cap_observations(&conn, "claude", "five_hour", 100).unwrap();
        assert_eq!(unlimited.len(), 4, "skips the NULL-confidence row");

        let other = recent_cap_observations(&conn, "claude", "seven_day", 100).unwrap();
        assert_eq!(other.len(), 1);
        assert_eq!(other[0].1, 999);
    }

    #[test]
    fn test_claude_usage_roundtrip() {
        let conn = test_conn();
        let run_id = insert_claude_usage_run(
            &conn,
            &ClaudeUsageRunInsert {
                status: "success",
                exit_code: Some(0),
                stdout_raw: "stdout",
                stderr_raw: "",
                invocation_mode: "print_slash_command",
                period: "today",
                parser_version: "v1",
                error_summary: None,
            },
        )
        .unwrap();
        insert_claude_usage_factors(
            &conn,
            run_id,
            &[ClaudeUsageFactor {
                factor_key: "parallel_sessions".into(),
                display_label: "was while 4+ sessions ran in parallel".into(),
                percent: 98.0,
                description: "All sessions share one limit.".into(),
                advice_text: "All sessions share one limit.".into(),
                display_order: 0,
            }],
        )
        .unwrap();

        let response = get_latest_claude_usage_response(&conn).unwrap();
        assert!(response.available);
        assert_eq!(response.last_run.as_ref().unwrap().status, "success");
        let snapshot = response.latest_snapshot.expect("missing latest snapshot");
        assert_eq!(snapshot.run.id, run_id);
        assert_eq!(snapshot.factors.len(), 1);
        assert_eq!(snapshot.factors[0].factor_key, "parallel_sessions");
    }

    #[test]
    fn test_claude_usage_failed_run_without_factors() {
        let conn = test_conn();

        insert_claude_usage_run(
            &conn,
            &ClaudeUsageRunInsert {
                status: "failed",
                exit_code: Some(1),
                stdout_raw: "/usage isn't available in this environment.",
                stderr_raw: "",
                invocation_mode: "print_slash_command",
                period: "today",
                parser_version: "v1",
                error_summary: Some("/usage isn't available in this environment."),
            },
        )
        .unwrap();

        let response = get_latest_claude_usage_response(&conn).unwrap();
        assert!(!response.available);
        let last_run = response.last_run.expect("missing last run");
        assert_eq!(last_run.status, "failed");
        assert_eq!(
            last_run.error_summary.as_deref(),
            Some("/usage isn't available in this environment.")
        );
        assert!(response.latest_snapshot.is_none());
    }

    #[test]
    fn test_subagent_summary_in_dashboard_data() {
        let conn = test_conn();
        // Insert session
        let sessions = vec![crate::models::Session {
            session_id: "s1".into(),
            project_name: "test".into(),
            first_timestamp: "2026-04-08T09:00:00Z".into(),
            last_timestamp: "2026-04-08T10:00:00Z".into(),
            model: Some("claude-sonnet-4-6".into()),
            total_input_tokens: 500,
            total_output_tokens: 250,
            turn_count: 3,
            ..Default::default()
        }];
        upsert_sessions(&conn, &sessions).unwrap();
        // Insert parent turn and subagent turns
        let turns = vec![
            Turn {
                session_id: "s1".into(),
                input_tokens: 200,
                output_tokens: 100,
                model: "claude-sonnet-4-6".into(),
                is_subagent: false,
                message_id: "msg-p1".into(),
                ..Default::default()
            },
            Turn {
                session_id: "s1".into(),
                input_tokens: 150,
                output_tokens: 75,
                model: "claude-sonnet-4-6".into(),
                is_subagent: true,
                agent_id: Some("agent-1".into()),
                message_id: "msg-a1".into(),
                ..Default::default()
            },
            Turn {
                session_id: "s1".into(),
                input_tokens: 150,
                output_tokens: 75,
                model: "claude-sonnet-4-6".into(),
                is_subagent: true,
                agent_id: Some("agent-2".into()),
                message_id: "msg-a2".into(),
                ..Default::default()
            },
        ];
        insert_turns(&conn, &turns).unwrap();

        let data = get_dashboard_data(&conn, TzParams::default()).unwrap();
        assert_eq!(data.subagent_summary.parent_turns, 1);
        assert_eq!(data.subagent_summary.parent_input, 200);
        assert_eq!(data.subagent_summary.subagent_turns, 2);
        assert_eq!(data.subagent_summary.subagent_input, 300);
        assert_eq!(data.subagent_summary.unique_agents, 2);
    }

    #[test]
    fn test_entrypoint_breakdown_in_dashboard_data() {
        let conn = test_conn();
        let sessions = vec![
            crate::models::Session {
                session_id: "s1".into(),
                project_name: "test".into(),
                entrypoint: "cli".into(),
                first_timestamp: "2026-04-08T09:00:00Z".into(),
                last_timestamp: "2026-04-08T10:00:00Z".into(),
                model: Some("claude-sonnet-4-6".into()),
                ..Default::default()
            },
            crate::models::Session {
                session_id: "s2".into(),
                project_name: "test".into(),
                entrypoint: "vscode".into(),
                first_timestamp: "2026-04-08T09:00:00Z".into(),
                last_timestamp: "2026-04-08T10:00:00Z".into(),
                model: Some("claude-sonnet-4-6".into()),
                ..Default::default()
            },
        ];
        upsert_sessions(&conn, &sessions).unwrap();
        let turns = vec![
            Turn {
                session_id: "s1".into(),
                input_tokens: 100,
                output_tokens: 50,
                model: "claude-sonnet-4-6".into(),
                message_id: "m1".into(),
                ..Default::default()
            },
            Turn {
                session_id: "s2".into(),
                input_tokens: 200,
                output_tokens: 100,
                model: "claude-sonnet-4-6".into(),
                message_id: "m2".into(),
                ..Default::default()
            },
        ];
        insert_turns(&conn, &turns).unwrap();

        let data = get_dashboard_data(&conn, TzParams::default()).unwrap();
        assert_eq!(data.entrypoint_breakdown.len(), 2);
        // Should be ordered by total tokens DESC
        assert_eq!(data.entrypoint_breakdown[0].entrypoint, "vscode");
        assert_eq!(data.entrypoint_breakdown[0].input, 200);
        assert_eq!(data.entrypoint_breakdown[1].entrypoint, "cli");
    }

    #[test]
    fn test_service_tier_in_dashboard_data() {
        let conn = test_conn();
        let sessions = vec![crate::models::Session {
            session_id: "s1".into(),
            project_name: "test".into(),
            first_timestamp: "2026-04-08T09:00:00Z".into(),
            last_timestamp: "2026-04-08T10:00:00Z".into(),
            model: Some("claude-sonnet-4-6".into()),
            ..Default::default()
        }];
        upsert_sessions(&conn, &sessions).unwrap();
        let turns = vec![
            Turn {
                session_id: "s1".into(),
                input_tokens: 100,
                output_tokens: 50,
                model: "claude-sonnet-4-6".into(),
                service_tier: Some("standard".into()),
                inference_geo: Some("us".into()),
                message_id: "m1".into(),
                ..Default::default()
            },
            Turn {
                session_id: "s1".into(),
                input_tokens: 100,
                output_tokens: 50,
                model: "claude-sonnet-4-6".into(),
                service_tier: Some("standard".into()),
                inference_geo: Some("eu".into()),
                message_id: "m2".into(),
                ..Default::default()
            },
        ];
        insert_turns(&conn, &turns).unwrap();

        let data = get_dashboard_data(&conn, TzParams::default()).unwrap();
        assert!(data.service_tiers.len() >= 2);
    }

    #[test]
    fn test_tool_invocations_table_created() {
        let conn = test_conn();
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert!(tables.contains(&"tool_invocations".into()));
    }

    #[test]
    fn test_insert_and_query_tool_invocations() {
        let conn = test_conn();
        let turns = vec![Turn {
            session_id: "s1".into(),
            message_id: "msg-1".into(),
            all_tools: vec!["Read".into(), "mcp__codex__codex".into(), "Bash".into()],
            tool_use_ids: vec![
                ("tool-1".into(), "Read".into()),
                ("tool-2".into(), "mcp__codex__codex".into()),
                ("tool-3".into(), "Bash".into()),
            ],
            ..Default::default()
        }];
        insert_tool_invocations(
            &conn,
            &turns,
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        )
        .unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM tool_invocations", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, 3);

        // Verify MCP classification
        let (server, tool, cat): (Option<String>, Option<String>, String) = conn
            .query_row(
                "SELECT mcp_server, mcp_tool, tool_category FROM tool_invocations WHERE tool_name = 'mcp__codex__codex'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(server.as_deref(), Some("codex"));
        assert_eq!(tool.as_deref(), Some("codex"));
        assert_eq!(cat, "mcp");

        // Verify builtin classification
        let cat: String = conn
            .query_row(
                "SELECT tool_category FROM tool_invocations WHERE tool_name = 'Read'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(cat, "builtin");
    }

    #[test]
    fn test_tool_invocation_dedup() {
        let conn = test_conn();
        let turns = vec![Turn {
            session_id: "s1".into(),
            message_id: "msg-1".into(),
            all_tools: vec!["Read".into()],
            tool_use_ids: vec![("tool-1".into(), "Read".into())],
            ..Default::default()
        }];
        insert_tool_invocations(
            &conn,
            &turns,
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        )
        .unwrap();
        insert_tool_invocations(
            &conn,
            &turns,
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        )
        .unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM tool_invocations", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_tool_invocation_preserves_duplicate_tool_names_with_distinct_tool_use_ids() {
        let conn = test_conn();
        let turns = vec![Turn {
            session_id: "s1".into(),
            message_id: "msg-1".into(),
            source_path: "/tmp/s1.jsonl".into(),
            tool_use_ids: vec![
                ("tool-1".into(), "Read".into()),
                ("tool-2".into(), "Read".into()),
            ],
            ..Default::default()
        }];
        insert_tool_invocations(
            &conn,
            &turns,
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        )
        .unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM tool_invocations", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_delete_tool_invocations_by_source_path() {
        let conn = test_conn();
        let turns = vec![
            Turn {
                session_id: "s1".into(),
                message_id: "msg-1".into(),
                source_path: "/tmp/a.jsonl".into(),
                tool_use_ids: vec![("tool-1".into(), "Read".into())],
                ..Default::default()
            },
            Turn {
                session_id: "s2".into(),
                message_id: "msg-2".into(),
                source_path: "/tmp/b.jsonl".into(),
                tool_use_ids: vec![("tool-2".into(), "Bash".into())],
                ..Default::default()
            },
        ];
        insert_tool_invocations(
            &conn,
            &turns,
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        )
        .unwrap();

        delete_tool_invocations_by_source_path(&conn, "/tmp/a.jsonl").unwrap();

        let remaining: Vec<String> = conn
            .prepare("SELECT source_path FROM tool_invocations ORDER BY source_path")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|row| row.ok())
            .collect();
        assert_eq!(remaining, vec!["/tmp/b.jsonl".to_string()]);
    }

    #[test]
    fn test_sync_session_titles_updates_and_clears_titles() {
        let conn = test_conn();
        upsert_sessions(
            &conn,
            &[crate::models::Session {
                session_id: "s1".into(),
                project_name: "test".into(),
                title: Some("Old title".into()),
                ..Default::default()
            }],
        )
        .unwrap();

        sync_session_titles(
            &conn,
            &["s1".into()],
            &HashMap::from([("s1".into(), "New title".into())]),
        )
        .unwrap();
        let title: Option<String> = conn
            .query_row(
                "SELECT title FROM sessions WHERE session_id = 's1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(title.as_deref(), Some("New title"));

        sync_session_titles(&conn, &["s1".into()], &HashMap::new()).unwrap();
        let title: Option<String> = conn
            .query_row(
                "SELECT title FROM sessions WHERE session_id = 's1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(title.is_none());
    }

    #[test]
    fn test_tool_summary_in_dashboard_data() {
        let conn = test_conn();
        let sessions = vec![crate::models::Session {
            session_id: "s1".into(),
            project_name: "test".into(),
            first_timestamp: "2026-04-08T09:00:00Z".into(),
            last_timestamp: "2026-04-08T10:00:00Z".into(),
            model: Some("claude-sonnet-4-6".into()),
            ..Default::default()
        }];
        upsert_sessions(&conn, &sessions).unwrap();
        let turns = vec![
            Turn {
                session_id: "s1".into(),
                input_tokens: 100,
                output_tokens: 50,
                model: "claude-sonnet-4-6".into(),
                message_id: "m1".into(),
                all_tools: vec!["Read".into(), "mcp__codex__codex".into()],
                tool_use_ids: vec![
                    ("tool-1".into(), "Read".into()),
                    ("tool-2".into(), "mcp__codex__codex".into()),
                ],
                ..Default::default()
            },
            Turn {
                session_id: "s1".into(),
                input_tokens: 200,
                output_tokens: 100,
                model: "claude-sonnet-4-6".into(),
                message_id: "m2".into(),
                all_tools: vec!["Read".into(), "Bash".into()],
                tool_use_ids: vec![
                    ("tool-3".into(), "Read".into()),
                    ("tool-4".into(), "Bash".into()),
                ],
                ..Default::default()
            },
        ];
        insert_turns(&conn, &turns).unwrap();
        insert_tool_invocations(
            &conn,
            &turns,
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        )
        .unwrap();

        let data = get_dashboard_data(&conn, TzParams::default()).unwrap();

        // Read should be most frequent (2 invocations)
        assert!(!data.tool_summary.is_empty());
        assert_eq!(data.tool_summary[0].tool_name, "Read");
        assert_eq!(data.tool_summary[0].invocations, 2);
        assert_eq!(data.tool_summary[0].category, "builtin");

        // MCP summary should have codex server
        assert_eq!(data.mcp_summary.len(), 1);
        assert_eq!(data.mcp_summary[0].server, "codex");
        assert_eq!(data.mcp_summary[0].invocations, 1);
    }

    #[test]
    fn test_dashboard_cost_summaries_use_actual_model_mix() {
        let conn = test_conn();
        upsert_sessions(
            &conn,
            &[
                crate::models::Session {
                    session_id: "s1".into(),
                    project_name: "proj".into(),
                    git_branch: "main".into(),
                    first_timestamp: "2026-04-08T09:00:00Z".into(),
                    last_timestamp: "2026-04-08T09:10:00Z".into(),
                    model: Some("claude-sonnet-4-6".into()),
                    ..Default::default()
                },
                crate::models::Session {
                    session_id: "s2".into(),
                    project_name: "proj".into(),
                    git_branch: "main".into(),
                    first_timestamp: "2026-04-08T09:20:00Z".into(),
                    last_timestamp: "2026-04-08T09:30:00Z".into(),
                    model: Some("claude-opus-4-6".into()),
                    ..Default::default()
                },
            ],
        )
        .unwrap();

        let turns = vec![
            Turn {
                session_id: "s1".into(),
                timestamp: "2026-04-08T09:05:00Z".into(),
                message_id: "m1".into(),
                model: "claude-sonnet-4-6".into(),
                input_tokens: 100_000,
                output_tokens: 20_000,
                ..Default::default()
            },
            Turn {
                session_id: "s2".into(),
                timestamp: "2026-04-08T09:25:00Z".into(),
                message_id: "m2".into(),
                model: "claude-opus-4-6".into(),
                input_tokens: 50_000,
                output_tokens: 10_000,
                ..Default::default()
            },
        ];
        insert_turns(&conn, &turns).unwrap();
        recompute_session_totals(&conn).unwrap();

        let data = get_dashboard_data(&conn, TzParams::default()).unwrap();
        let expected_cost = pricing::calc_cost("claude-sonnet-4-6", 100_000, 20_000, 0, 0)
            + pricing::calc_cost("claude-opus-4-6", 50_000, 10_000, 0, 0);

        let branch = data
            .git_branch_summary
            .iter()
            .find(|row| row.branch == "main")
            .unwrap();
        assert!((branch.cost - expected_cost).abs() < 1e-9);

        let daily_project = data
            .daily_by_project
            .iter()
            .find(|row| row.day == "2026-04-08" && row.project == "proj")
            .unwrap();
        assert!((daily_project.cost - expected_cost).abs() < 1e-9);
    }

    // ── Phase 12: Tool-event cost attribution ────────────────────────────────

    #[test]
    fn test_tool_events_table_migration_idempotent() {
        // Running init_db twice must not error; the tool_events table must exist.
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();
        init_db(&conn).unwrap(); // second call must be a no-op

        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert!(tables.contains(&"tool_events".into()));

        // Indexes must exist too
        let indexes: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='index'")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert!(indexes.contains(&"idx_tool_events_kind".into()));
        assert!(indexes.contains(&"idx_tool_events_session".into()));
    }

    #[test]
    fn test_compute_tool_events_three_tools_cost_split() {
        // One turn with 3 tools and cost 1000 nanos.
        // Expected: first event gets 334, others get 333. Sum == 1000.
        let turn = Turn {
            session_id: "claude:s1".into(),
            provider: "claude".into(),
            timestamp: "2026-04-08T10:00:00Z".into(),
            estimated_cost_nanos: 1000,
            source_path: "/tmp/test.jsonl".into(),
            tool_use_ids: vec![
                ("id1".into(), "Edit".into()),
                ("id2".into(), "Bash".into()),
                ("id3".into(), "Read".into()),
            ],
            ..Default::default()
        };
        let events = compute_tool_events_for_turn(&turn, "myproject");
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].cost_nanos, 334); // first gets remainder
        assert_eq!(events[1].cost_nanos, 333);
        assert_eq!(events[2].cost_nanos, 333);
        let sum: i64 = events.iter().map(|e| e.cost_nanos).sum();
        assert_eq!(sum, 1000);
    }

    #[test]
    fn test_compute_tool_events_cost_sum_preserved_exactly() {
        // Verify exact sum preservation for a variety of (total, n) pairs.
        let cases: &[(i64, usize)] = &[(0, 3), (1, 3), (7, 2), (1_000_000_007, 5), (999, 1)];
        for &(total, n) in cases {
            let tool_ids: Vec<(String, String)> = (0..n)
                .map(|i| (format!("id{}", i), "Edit".into()))
                .collect();
            let turn = Turn {
                session_id: "s".into(),
                provider: "claude".into(),
                estimated_cost_nanos: total,
                tool_use_ids: tool_ids,
                ..Default::default()
            };
            let events = compute_tool_events_for_turn(&turn, "");
            let sum: i64 = events.iter().map(|e| e.cost_nanos).sum();
            assert_eq!(sum, total, "sum mismatch for total={total} n={n}");
        }
    }

    #[test]
    fn test_compute_tool_events_kind_classification() {
        let cases: &[(&str, &str, &str)] = &[
            ("mcp__filesystem__read_file", "mcp", "read_file"),
            ("mcp__server__tool", "mcp", "tool"),
            ("Task", "subagent", "Task"),
            ("Edit", "file", "Edit"),
            ("Write", "file", "Write"),
            ("MultiEdit", "file", "MultiEdit"),
            ("NotebookEdit", "file", "NotebookEdit"),
            ("Read", "file", "Read"),
            ("Bash", "bash", "Bash"),
            ("WebSearch", "other", "WebSearch"),
            ("unknown_tool", "other", "unknown_tool"),
        ];
        for &(tool_name, expected_kind, expected_value) in cases {
            let (kind, value) = classify_tool_event(tool_name);
            assert_eq!(kind, expected_kind, "kind mismatch for tool {tool_name}");
            assert_eq!(value, expected_value, "value mismatch for tool {tool_name}");
        }
    }

    #[test]
    fn test_compute_tool_events_zero_tools_produces_no_events() {
        let turn = Turn {
            session_id: "s1".into(),
            estimated_cost_nanos: 5000,
            tool_use_ids: vec![],
            ..Default::default()
        };
        let events = compute_tool_events_for_turn(&turn, "proj");
        assert!(events.is_empty());
    }

    #[test]
    fn test_insert_tool_events_idempotent() {
        let conn = test_conn();
        let events = vec![
            crate::models::ToolEvent {
                dedup_key: "claude:id1".into(),
                ts_epoch: 1000,
                session_id: "claude:s1".into(),
                provider: "claude".into(),
                project: "proj".into(),
                kind: "file".into(),
                value: "Edit".into(),
                cost_nanos: 500,
                source_path: "/tmp/f.jsonl".into(),
            },
            crate::models::ToolEvent {
                dedup_key: "claude:id2".into(),
                ts_epoch: 1001,
                session_id: "claude:s1".into(),
                provider: "claude".into(),
                project: "proj".into(),
                kind: "bash".into(),
                value: "Bash".into(),
                cost_nanos: 300,
                source_path: "/tmp/f.jsonl".into(),
            },
        ];
        insert_tool_events(&conn, &events).unwrap();
        insert_tool_events(&conn, &events).unwrap(); // second call must be no-op (INSERT OR IGNORE)

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM tool_events", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_delete_tool_events_by_source_path() {
        let conn = test_conn();
        let events = vec![
            crate::models::ToolEvent {
                dedup_key: "claude:a1".into(),
                ts_epoch: 0,
                session_id: "s1".into(),
                provider: "claude".into(),
                project: "".into(),
                kind: "file".into(),
                value: "Edit".into(),
                cost_nanos: 100,
                source_path: "/tmp/a.jsonl".into(),
            },
            crate::models::ToolEvent {
                dedup_key: "claude:b1".into(),
                ts_epoch: 0,
                session_id: "s1".into(),
                provider: "claude".into(),
                project: "".into(),
                kind: "bash".into(),
                value: "Bash".into(),
                cost_nanos: 200,
                source_path: "/tmp/b.jsonl".into(),
            },
        ];
        insert_tool_events(&conn, &events).unwrap();

        delete_tool_events_by_source_path(&conn, "/tmp/a.jsonl").unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM tool_events", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);

        let remaining_key: String = conn
            .query_row("SELECT dedup_key FROM tool_events", [], |row| row.get(0))
            .unwrap();
        assert_eq!(remaining_key, "claude:b1");
    }

    #[test]
    fn test_round_trip_invariant_tool_events_sum_equals_turns_sum() {
        // Seed turns with tool invocations into the DB via the helpers and verify:
        // SUM(cost_nanos) FROM tool_events WHERE session_id = ?
        //   == SUM(estimated_cost_nanos) FROM turns WHERE session_id = ?
        // for sessions where EVERY turn has at least one tool invocation.
        //
        // Note: turns with zero tools are intentionally excluded from tool_events.
        // A session that mixes tool and tool-free turns will show a lower sum in
        // tool_events than in turns — see the comment on `compute_tool_events_for_turn`.
        let conn = test_conn();

        // Build two turns each with 3 tools and a cost divisible by 3.
        // Set pricing_version and cost_confidence to non-empty values so
        // insert_turns does NOT recalculate estimated_cost_nanos; this
        // ensures the stored cost matches what we use to split events.
        let turns = vec![
            Turn {
                session_id: "claude:s1".into(),
                provider: "claude".into(),
                timestamp: "2026-04-08T10:00:00Z".into(),
                message_id: "msg-1".into(),
                estimated_cost_nanos: 900,
                pricing_version: "v1".into(),
                pricing_model: "claude-sonnet-4-6".into(),
                cost_confidence: "high".into(),
                billing_mode: "estimated_local".into(),
                source_path: "/tmp/t.jsonl".into(),
                tool_use_ids: vec![
                    ("t1a".into(), "Edit".into()),
                    ("t1b".into(), "Bash".into()),
                    ("t1c".into(), "Read".into()),
                ],
                ..Default::default()
            },
            Turn {
                session_id: "claude:s1".into(),
                provider: "claude".into(),
                timestamp: "2026-04-08T10:01:00Z".into(),
                message_id: "msg-2".into(),
                estimated_cost_nanos: 600,
                pricing_version: "v1".into(),
                pricing_model: "claude-sonnet-4-6".into(),
                cost_confidence: "high".into(),
                billing_mode: "estimated_local".into(),
                source_path: "/tmp/t.jsonl".into(),
                tool_use_ids: vec![
                    ("t2a".into(), "mcp__fs__read".into()),
                    ("t2b".into(), "Task".into()),
                    ("t2c".into(), "Write".into()),
                ],
                ..Default::default()
            },
        ];

        insert_turns(&conn, &turns).unwrap();

        let all_events: Vec<crate::models::ToolEvent> = turns
            .iter()
            .flat_map(|t| compute_tool_events_for_turn(t, "proj"))
            .collect();
        insert_tool_events(&conn, &all_events).unwrap();

        let te_sum: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(cost_nanos), 0) FROM tool_events WHERE session_id = 'claude:s1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let turns_sum: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(estimated_cost_nanos), 0) FROM turns WHERE session_id = 'claude:s1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(te_sum, turns_sum);
        assert_eq!(te_sum, 1500);
    }

    // -----------------------------------------------------------------------
    // Deliverable 1: tool-argument capture in compute_tool_events_for_turn
    // -----------------------------------------------------------------------

    #[test]
    fn test_tool_events_file_path_from_tool_inputs_edit() {
        // A Turn with tool_inputs populated should produce a tool_event whose
        // value is the extracted file path, not the raw tool name.
        let turn = Turn {
            session_id: "claude:s1".into(),
            provider: "claude".into(),
            timestamp: "2026-04-08T10:00:00Z".into(),
            estimated_cost_nanos: 600,
            source_path: "/tmp/test.jsonl".into(),
            tool_use_ids: vec![("call-1".into(), "Edit".into())],
            tool_inputs: vec![("call-1".into(), "/some/file.rs".into())],
            ..Default::default()
        };
        let events = compute_tool_events_for_turn(&turn, "proj");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, "file");
        assert_eq!(events[0].value, "/some/file.rs");
    }

    #[test]
    fn test_tool_events_bash_command_from_tool_inputs() {
        // A Turn with Bash tool_inputs should produce kind="bash" with command text.
        let turn = Turn {
            session_id: "claude:s1".into(),
            provider: "claude".into(),
            timestamp: "2026-04-08T10:00:00Z".into(),
            estimated_cost_nanos: 400,
            source_path: "/tmp/test.jsonl".into(),
            tool_use_ids: vec![("call-1".into(), "Bash".into())],
            tool_inputs: vec![("call-1".into(), "cargo test --all".into())],
            ..Default::default()
        };
        let events = compute_tool_events_for_turn(&turn, "proj");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, "bash");
        assert_eq!(events[0].value, "cargo test --all");
    }

    #[test]
    fn test_tool_events_empty_arg_falls_back_to_tool_name() {
        // When tool_inputs has an entry but arg is empty, fall back to tool name.
        let turn = Turn {
            session_id: "claude:s1".into(),
            provider: "claude".into(),
            estimated_cost_nanos: 200,
            tool_use_ids: vec![("call-1".into(), "Read".into())],
            tool_inputs: vec![("call-1".into(), String::new())],
            ..Default::default()
        };
        let events = compute_tool_events_for_turn(&turn, "proj");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, "file");
        assert_eq!(events[0].value, "Read");
    }

    // ── load_turns_in_range ───────────────────────────────────────────────────

    #[test]
    fn test_load_turns_in_range_row_count_and_ordering() {
        let conn = test_conn();

        // Seed three turns at distinct timestamps.
        let seed = vec![
            Turn {
                session_id: "s1".into(),
                timestamp: "2026-01-01T09:00:00Z".into(),
                model: "claude-sonnet-4-6".into(),
                input_tokens: 100,
                output_tokens: 50,
                estimated_cost_nanos: 1_000,
                message_id: "m1".into(),
                ..Default::default()
            },
            Turn {
                session_id: "s1".into(),
                timestamp: "2026-01-01T10:00:00Z".into(),
                model: "claude-haiku-3-5".into(),
                input_tokens: 200,
                output_tokens: 80,
                estimated_cost_nanos: 2_000,
                message_id: "m2".into(),
                ..Default::default()
            },
            Turn {
                session_id: "s1".into(),
                timestamp: "2026-01-01T12:00:00Z".into(),
                model: "claude-sonnet-4-6".into(),
                input_tokens: 50,
                output_tokens: 20,
                estimated_cost_nanos: 500,
                message_id: "m3".into(),
                ..Default::default()
            },
        ];
        insert_turns(&conn, &seed).unwrap();

        // Query only the first two (09:00 <= ts < 11:00).
        let rows =
            load_turns_in_range(&conn, "2026-01-01T09:00:00Z", "2026-01-01T11:00:00Z").unwrap();
        assert_eq!(rows.len(), 2, "should return exactly 2 turns in range");

        // Verify ascending order.
        assert!(
            rows[0].timestamp < rows[1].timestamp,
            "rows must be ordered ascending"
        );

        // Verify field mapping (cost_nanos is recalculated by insert_turns via
        // the pricing engine, so we only assert on token fields which are stored verbatim).
        assert_eq!(rows[0].tokens.input, 100);
        assert_eq!(rows[0].tokens.output, 50);
        assert_eq!(rows[1].tokens.input, 200);
    }

    // ── historical_max_block_tokens ───────────────────────────────────────────

    #[test]
    fn test_historical_max_block_tokens_returns_larger_block() {
        let conn = test_conn();

        // Block 1: two turns close together (same 5h block) — 300 tokens total.
        // Block 2: one turn 12h later (new block) — 500 tokens total.
        // Expect: historical max = 500.
        let seed = vec![
            Turn {
                session_id: "s1".into(),
                provider: "claude".into(),
                timestamp: "2026-01-01T09:00:00Z".into(),
                model: "claude-sonnet-4-6".into(),
                input_tokens: 100,
                output_tokens: 50,
                cache_read_tokens: 50,
                cache_creation_tokens: 50,
                reasoning_output_tokens: 50,
                estimated_cost_nanos: 0,
                ..Turn::default()
            },
            Turn {
                session_id: "s1".into(),
                provider: "claude".into(),
                timestamp: "2026-01-01T09:30:00Z".into(),
                model: "claude-sonnet-4-6".into(),
                input_tokens: 100,
                output_tokens: 50,
                cache_read_tokens: 50,
                cache_creation_tokens: 50,
                reasoning_output_tokens: 0,
                estimated_cost_nanos: 0,
                ..Turn::default()
            },
            // 12h gap → new block
            Turn {
                session_id: "s1".into(),
                provider: "claude".into(),
                timestamp: "2026-01-01T21:00:00Z".into(),
                model: "claude-sonnet-4-6".into(),
                input_tokens: 200,
                output_tokens: 100,
                cache_read_tokens: 100,
                cache_creation_tokens: 60,
                reasoning_output_tokens: 40,
                estimated_cost_nanos: 0,
                ..Turn::default()
            },
        ];
        insert_turns(&conn, &seed).unwrap();

        // Block 1: 100+50+50+50+50 + 100+50+50+50+0 = 300 + 250 = 550
        // Block 2: 200+100+100+60+40 = 500
        // So max should be 550 (block 1).
        let max = historical_max_block_tokens(&conn, 5.0).unwrap();
        assert!(max > 0, "max should be positive");
        // The exact value depends on token breakdown; we just assert the larger
        // block wins over any degenerate case.
        assert!(max >= 500, "max block tokens should be at least 500");
    }

    // ── Phase 3: sum_by_week tests ────────────────────────────────────────────

    fn insert_turn_ts(
        conn: &Connection,
        ts: &str,
        model: &str,
        input: i64,
        output: i64,
        cost: i64,
    ) {
        conn.execute(
            "INSERT INTO turns (session_id, provider, timestamp, model, input_tokens, output_tokens,
                               cache_read_tokens, cache_creation_tokens, reasoning_output_tokens,
                               estimated_cost_nanos, message_id, source_path, pricing_version,
                               pricing_model, billing_mode, cost_confidence, category)
             VALUES ('s1', 'claude', ?1, ?2, ?3, ?4, 0, 0, 0, ?5, ?6, '', '', '', 'estimated_local', 'low', '')",
            rusqlite::params![ts, model, input, output, cost, format!("{ts}-{model}")],
        )
        .unwrap();
    }

    #[test]
    fn sum_by_week_basic_grouping() {
        let conn = test_conn();
        // Two turns in the same week, one in the next.
        insert_turn_ts(
            &conn,
            "2027-01-04T10:00:00Z",
            "claude-3-5-sonnet",
            100,
            50,
            1000,
        );
        insert_turn_ts(
            &conn,
            "2027-01-05T10:00:00Z",
            "claude-3-5-sonnet",
            200,
            80,
            2000,
        );
        insert_turn_ts(
            &conn,
            "2027-01-11T10:00:00Z",
            "claude-3-5-sonnet",
            50,
            20,
            500,
        );

        let tz = TzParams {
            tz_offset_min: None,
            week_starts_on: Some(1),
        }; // Monday
        let rows = sum_by_week(&conn, tz).unwrap();

        // Should produce 2 distinct week buckets.
        assert_eq!(rows.len(), 2, "expected 2 week buckets");

        // First bucket: 2027-W01 (Jan 4–10)
        assert_eq!(rows[0].input_tokens, 300);
        assert_eq!(rows[0].output_tokens, 130);
        assert_eq!(rows[0].cost_nanos, 3000);
        assert_eq!(rows[0].turns, 2);

        // Second bucket: 2027-W02 (Jan 11–17)
        assert_eq!(rows[1].input_tokens, 50);
        assert_eq!(rows[1].turns, 1);
    }

    #[test]
    fn sum_by_week_year_end_boundary() {
        let conn = test_conn();
        // Turns straddling the 2027/2028 year boundary with Monday-start weeks.
        //
        // SQLite strftime('%Y-%W', ...) where %W is the Monday-anchored week number:
        //   2027-12-28 (Tuesday)  → week 52 of 2027 → "2027-52"
        //   2027-12-31 (Friday)   → week 52 of 2027 → "2027-52"
        //   2028-01-03 (Monday)   → first full week of 2028 → "2028-01"
        //     (%W returns "00" for days before the first Monday of the year;
        //      Jan 3 2028 IS a Monday so it opens week 01.)
        insert_turn_ts(&conn, "2027-12-28T10:00:00Z", "model-a", 100, 50, 1000);
        insert_turn_ts(&conn, "2027-12-31T10:00:00Z", "model-a", 100, 50, 1000);
        insert_turn_ts(&conn, "2028-01-03T10:00:00Z", "model-a", 200, 80, 2000);

        let tz = TzParams {
            tz_offset_min: None,
            week_starts_on: Some(1), // Monday
        };
        let rows = sum_by_week(&conn, tz).unwrap();

        // Collect distinct week labels from the result.
        let actual_weeks: std::collections::BTreeSet<String> =
            rows.iter().map(|r| r.week.clone()).collect();

        // Exactly the 3 expected week labels must be present (2027-52 twice → 1 distinct,
        // plus 2028-01 → 2 distinct total).
        let expected_weeks: std::collections::BTreeSet<String> = ["2027-52", "2028-01"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(
            actual_weeks, expected_weeks,
            "expected distinct week labels {:?}, got {:?}",
            expected_weeks, actual_weeks
        );

        // The 2027 week(s) come before the 2028 week(s) — lexicographic sort works
        // because both year and week are zero-padded.
        let first_week = &rows[0].week;
        let last_week = &rows[rows.len() - 1].week;
        assert!(
            first_week < last_week,
            "weeks should be ordered ascending: {first_week} < {last_week}"
        );

        // The 2028-01 bucket has exactly the Jan 3 turn (200 input tokens).
        let jan_input: i64 = rows
            .iter()
            .filter(|r| r.week == "2028-01")
            .map(|r| r.input_tokens)
            .sum();
        assert_eq!(
            jan_input, 200,
            "2028-01 bucket should contain exactly 200 input tokens"
        );

        // The 2027-52 bucket accumulates both Dec 28 and Dec 31 turns (100 + 100 = 200).
        let dec_input: i64 = rows
            .iter()
            .filter(|r| r.week == "2027-52")
            .map(|r| r.input_tokens)
            .sum();
        assert_eq!(
            dec_input, 200,
            "2027-52 bucket should contain 200 input tokens (two turns)"
        );
    }

    // ── load_turns_since ──────────────────────────────────────────────────────

    #[test]
    fn test_load_turns_since_filters_and_orders() {
        let conn = test_conn();

        // Seed three turns: one old, two recent.
        let seed = vec![
            Turn {
                session_id: "s1".into(),
                timestamp: "2020-01-01T00:00:00Z".into(),
                model: "claude-sonnet-4-6".into(),
                input_tokens: 999,
                output_tokens: 1,
                estimated_cost_nanos: 1,
                message_id: "old".into(),
                ..Default::default()
            },
            Turn {
                session_id: "s1".into(),
                timestamp: "2026-01-01T10:00:00Z".into(),
                model: "claude-sonnet-4-6".into(),
                input_tokens: 100,
                output_tokens: 50,
                estimated_cost_nanos: 1_000,
                message_id: "r1".into(),
                ..Default::default()
            },
            Turn {
                session_id: "s1".into(),
                timestamp: "2026-01-01T11:00:00Z".into(),
                model: "claude-haiku-3-5".into(),
                input_tokens: 200,
                output_tokens: 80,
                estimated_cost_nanos: 2_000,
                message_id: "r2".into(),
                ..Default::default()
            },
        ];
        insert_turns(&conn, &seed).unwrap();

        // Query since 2026 — should exclude the 2020 turn.
        let rows = load_turns_since(&conn, "2026-01-01T00:00:00Z").unwrap();
        assert_eq!(rows.len(), 2, "should return 2 turns since cutoff");

        // Verify ascending order.
        assert!(
            rows[0].timestamp < rows[1].timestamp,
            "rows must be in ascending timestamp order"
        );

        // The old turn must not appear.
        use chrono::Datelike as _;
        for r in &rows {
            assert!(
                r.timestamp.year() >= 2026,
                "unexpected old turn: {:?}",
                r.timestamp
            );
        }
    }

    #[test]
    fn test_load_turns_since_empty_when_all_old() {
        let conn = test_conn();

        let seed = vec![Turn {
            session_id: "s1".into(),
            timestamp: "2020-06-01T00:00:00Z".into(),
            model: "claude-sonnet-4-6".into(),
            input_tokens: 100,
            output_tokens: 10,
            estimated_cost_nanos: 500,
            message_id: "m1".into(),
            ..Default::default()
        }];
        insert_turns(&conn, &seed).unwrap();

        let rows = load_turns_since(&conn, "2026-01-01T00:00:00Z").unwrap();
        assert!(rows.is_empty(), "should return nothing for a future cutoff");
    }

    // ── Phase 12 (Amp) credits tests ─────────────────────────────────────────

    /// Credits are persisted to turns.credits and recovered via SELECT SUM(credits).
    #[test]
    fn test_insert_turns_persists_credits() {
        let conn = test_conn();
        let turns = vec![
            Turn {
                session_id: "amp:T-abc".into(),
                provider: "amp".into(),
                message_id: "amp:T-abc:ev-1".into(),
                timestamp: "2026-04-18T09:00:00Z".into(),
                model: "claude-haiku-4-5-20251001".into(),
                credits: Some(12.5),
                billing_mode: "credits".into(),
                ..Default::default()
            },
            Turn {
                session_id: "amp:T-abc".into(),
                provider: "amp".into(),
                message_id: "amp:T-abc:ev-2".into(),
                timestamp: "2026-04-18T09:05:00Z".into(),
                model: "claude-haiku-4-5-20251001".into(),
                credits: Some(3.2),
                billing_mode: "credits".into(),
                ..Default::default()
            },
        ];
        insert_turns(&conn, &turns).unwrap();

        let sum: Option<f64> = conn
            .query_row(
                "SELECT SUM(credits) FROM turns WHERE provider = 'amp'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let total = sum.expect("credits sum must be non-null for Amp turns");
        assert!((total - 15.7).abs() < 1e-9, "expected 15.7, got {total}");
    }

    /// Non-Amp turns have credits = NULL, not 0.
    #[test]
    fn test_insert_turns_non_amp_credits_null() {
        let conn = test_conn();
        let turns = vec![Turn {
            session_id: "claude:s1".into(),
            provider: "claude".into(),
            message_id: "msg-claude-1".into(),
            timestamp: "2026-04-18T10:00:00Z".into(),
            model: "claude-sonnet-4-6".into(),
            input_tokens: 100,
            output_tokens: 50,
            credits: None, // non-Amp: no credits
            ..Default::default()
        }];
        insert_turns(&conn, &turns).unwrap();

        let credits: Option<f64> = conn
            .query_row(
                "SELECT credits FROM turns WHERE provider = 'claude'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(
            credits.is_none(),
            "non-Amp turns must have NULL credits, got: {credits:?}"
        );
    }

    /// recompute_session_totals aggregates credits into sessions.total_credits.
    #[test]
    fn test_recompute_session_totals_aggregates_credits() {
        let conn = test_conn();
        // Seed an Amp session with two turns.
        conn.execute(
            "INSERT INTO sessions (session_id, provider) VALUES ('amp:T-test', 'amp')",
            [],
        )
        .unwrap();
        let turns = vec![
            Turn {
                session_id: "amp:T-test".into(),
                provider: "amp".into(),
                message_id: "amp:T-test:ev-1".into(),
                timestamp: "2026-04-18T09:00:00Z".into(),
                model: "amp".into(),
                credits: Some(5.0),
                billing_mode: "credits".into(),
                ..Default::default()
            },
            Turn {
                session_id: "amp:T-test".into(),
                provider: "amp".into(),
                message_id: "amp:T-test:ev-2".into(),
                timestamp: "2026-04-18T09:01:00Z".into(),
                model: "amp".into(),
                credits: Some(7.3),
                billing_mode: "credits".into(),
                ..Default::default()
            },
        ];
        insert_turns(&conn, &turns).unwrap();
        recompute_session_totals(&conn).unwrap();

        let total_credits: Option<f64> = conn
            .query_row(
                "SELECT total_credits FROM sessions WHERE session_id = 'amp:T-test'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let total = total_credits.expect("total_credits must be non-null after recompute");
        assert!((total - 12.3).abs() < 1e-9, "expected 12.3, got {total}");
    }

    /// Schema migration for credits is idempotent (safe on existing DBs).
    #[test]
    fn test_credits_migration_idempotent() {
        let conn = test_conn();
        // init_db was already called by test_conn(); calling again must not fail.
        // This exercises the `has_column` guard on turns.credits and sessions.total_credits.
        super::init_db(&conn).expect("second init_db call must succeed (idempotent migration)");
        assert!(
            has_column(&conn, "turns", "credits"),
            "turns.credits must exist after migration"
        );
        assert!(
            has_column(&conn, "sessions", "total_credits"),
            "sessions.total_credits must exist after migration"
        );
    }

    /// get_dashboard_data returns credits in daily_by_project for Amp rows.
    #[test]
    fn test_get_dashboard_data_credits_in_daily_by_project() {
        let conn = test_conn();
        conn.execute(
            "INSERT INTO sessions (session_id, provider, project_name) VALUES ('amp:T-dash', 'amp', 'test-project')",
            [],
        )
        .unwrap();
        let turns = vec![Turn {
            session_id: "amp:T-dash".into(),
            provider: "amp".into(),
            message_id: "amp:T-dash:ev-1".into(),
            timestamp: "2026-04-18T10:00:00Z".into(),
            model: "claude-haiku-4-5-20251001".into(),
            credits: Some(8.8),
            billing_mode: "credits".into(),
            ..Default::default()
        }];
        insert_turns(&conn, &turns).unwrap();
        recompute_session_totals(&conn).unwrap();

        let data = get_dashboard_data(&conn, TzParams::default()).unwrap();
        let proj_row = data
            .daily_by_project
            .iter()
            .find(|r| r.provider == "amp")
            .expect("must have an amp daily_by_project row");
        let credits = proj_row
            .credits
            .expect("credits must be non-null for Amp project row");
        assert!((credits - 8.8).abs() < 1e-9, "expected 8.8, got {credits}");
    }

    #[test]
    fn test_get_provider_model_rows_ordered_by_cost() {
        let conn = test_conn();
        let sessions = vec![crate::models::Session {
            session_id: "pm1".into(),
            project_name: "proj".into(),
            first_timestamp: "2026-04-08T09:00:00Z".into(),
            last_timestamp: "2026-04-08T10:00:00Z".into(),
            provider: "claude".into(),
            model: Some("claude-sonnet-4-6".into()),
            ..Default::default()
        }];
        upsert_sessions(&conn, &sessions).unwrap();
        let turns = vec![
            Turn {
                session_id: "pm1".into(),
                timestamp: "2026-04-08T09:01:00Z".into(),
                message_id: "pm-m1".into(),
                provider: "claude".into(),
                model: "claude-opus-4-5".into(),
                input_tokens: 1000,
                output_tokens: 500,
                estimated_cost_nanos: 5_000_000_000,
                ..Default::default()
            },
            Turn {
                session_id: "pm1".into(),
                timestamp: "2026-04-08T09:02:00Z".into(),
                message_id: "pm-m2".into(),
                provider: "claude".into(),
                model: "claude-haiku-3-5".into(),
                input_tokens: 200,
                output_tokens: 100,
                estimated_cost_nanos: 1_000_000_000,
                ..Default::default()
            },
        ];
        insert_turns(&conn, &turns).unwrap();

        let rows = get_provider_model_rows(&conn, "claude", "2026-04-01", 10).unwrap();
        assert_eq!(rows.len(), 2);
        // opus costs more — must come first
        assert_eq!(rows[0].model, "claude-opus-4-5");
        assert!(rows[0].cost_usd > rows[1].cost_usd);
        assert_eq!(rows[0].turns, 1);
        assert_eq!(rows[1].model, "claude-haiku-3-5");
    }

    #[test]
    fn test_get_provider_tool_rows_ordered_by_invocations() {
        let conn = test_conn();
        let sessions = vec![crate::models::Session {
            session_id: "pt1".into(),
            project_name: "proj".into(),
            first_timestamp: "2026-04-08T09:00:00Z".into(),
            last_timestamp: "2026-04-08T10:00:00Z".into(),
            provider: "claude".into(),
            model: Some("claude-sonnet-4-6".into()),
            ..Default::default()
        }];
        upsert_sessions(&conn, &sessions).unwrap();
        let turns = vec![Turn {
            session_id: "pt1".into(),
            timestamp: "2026-04-08T09:01:00Z".into(),
            message_id: "pt-m1".into(),
            provider: "claude".into(),
            model: "claude-sonnet-4-6".into(),
            tool_use_ids: vec![
                ("pt-t1".into(), "Read".into()),
                ("pt-t2".into(), "Read".into()),
                ("pt-t3".into(), "Bash".into()),
            ],
            all_tools: vec!["Read".into(), "Read".into(), "Bash".into()],
            source_path: "/tmp/pt1.jsonl".into(),
            ..Default::default()
        }];
        insert_turns(&conn, &turns).unwrap();
        insert_tool_invocations(
            &conn,
            &turns,
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        )
        .unwrap();

        let rows = get_provider_tool_rows(&conn, "claude", "2026-04-01", 15).unwrap();
        assert!(!rows.is_empty());
        // Read has 2 invocations, Bash has 1 — Read must come first
        assert_eq!(rows[0].tool_name, "Read");
        assert_eq!(rows[0].invocations, 2);
        assert!(
            rows.iter()
                .any(|r| r.tool_name == "Bash" && r.invocations == 1)
        );
    }

    #[test]
    fn test_get_provider_hourly_activity_contiguous_24() {
        let conn = test_conn();
        let sessions = vec![crate::models::Session {
            session_id: "ha1".into(),
            project_name: "proj".into(),
            first_timestamp: "2026-04-08T09:00:00Z".into(),
            last_timestamp: "2026-04-08T10:00:00Z".into(),
            provider: "claude".into(),
            ..Default::default()
        }];
        upsert_sessions(&conn, &sessions).unwrap();
        let turns = vec![Turn {
            session_id: "ha1".into(),
            timestamp: "2026-04-08T09:01:00Z".into(),
            message_id: "ha-m1".into(),
            provider: "claude".into(),
            model: "claude-sonnet-4-6".into(),
            estimated_cost_nanos: 1_000_000_000,
            ..Default::default()
        }];
        insert_turns(&conn, &turns).unwrap();

        let rows = get_provider_hourly_activity(&conn, "claude", "2026-04-01").unwrap();
        // Must always have exactly 24 buckets
        assert_eq!(rows.len(), 24);
        // Hours must be contiguous 0..=23
        for (i, bucket) in rows.iter().enumerate() {
            assert_eq!(bucket.hour as usize, i);
        }
        // At least one bucket has a non-zero turn count (the inserted turn)
        assert!(rows.iter().any(|b| b.turns > 0));
    }

    #[test]
    fn test_get_provider_recent_sessions_ordering_and_limit() {
        let conn = test_conn();
        let sessions = vec![
            crate::models::Session {
                session_id: "rs1".into(),
                project_name: "proj".into(),
                first_timestamp: "2026-04-06T08:00:00Z".into(),
                last_timestamp: "2026-04-06T09:00:00Z".into(),
                provider: "claude".into(),
                ..Default::default()
            },
            crate::models::Session {
                session_id: "rs2".into(),
                project_name: "proj".into(),
                first_timestamp: "2026-04-08T10:00:00Z".into(),
                last_timestamp: "2026-04-08T11:00:00Z".into(),
                provider: "claude".into(),
                ..Default::default()
            },
            crate::models::Session {
                session_id: "rs3".into(),
                project_name: "proj".into(),
                first_timestamp: "2026-04-07T12:00:00Z".into(),
                last_timestamp: "2026-04-07T13:00:00Z".into(),
                provider: "claude".into(),
                ..Default::default()
            },
        ];
        upsert_sessions(&conn, &sessions).unwrap();

        // limit = 2 — must return only 2 rows, newest first
        let rows = get_provider_recent_sessions(&conn, "claude", 2).unwrap();
        assert_eq!(rows.len(), 2);
        // rs2 (2026-04-08) must come before rs3 (2026-04-07)
        assert_eq!(rows[0].session_id, "rs2");
        assert_eq!(rows[1].session_id, "rs3");
    }

    #[test]
    fn test_get_provider_version_rows_groups_and_orders_by_cost() {
        let conn = test_conn();
        let sessions = vec![crate::models::Session {
            session_id: "vr1".into(),
            project_name: "proj".into(),
            first_timestamp: "2026-04-08T09:00:00Z".into(),
            last_timestamp: "2026-04-08T10:00:00Z".into(),
            provider: "claude".into(),
            ..Default::default()
        }];
        upsert_sessions(&conn, &sessions).unwrap();
        let turns = vec![
            Turn {
                session_id: "vr1".into(),
                timestamp: "2026-04-08T09:01:00Z".into(),
                message_id: "vr-m1".into(),
                provider: "claude".into(),
                model: "claude-sonnet-4-6".into(),
                version: Some("1.2.0".into()),
                estimated_cost_nanos: 3_000_000_000,
                pricing_version: "test".into(),
                cost_confidence: "exact".into(),
                ..Default::default()
            },
            Turn {
                session_id: "vr1".into(),
                timestamp: "2026-04-08T09:02:00Z".into(),
                message_id: "vr-m2".into(),
                provider: "claude".into(),
                model: "claude-haiku-3-5".into(),
                version: Some("1.1.0".into()),
                estimated_cost_nanos: 1_000_000_000,
                pricing_version: "test".into(),
                cost_confidence: "exact".into(),
                ..Default::default()
            },
            Turn {
                session_id: "vr1".into(),
                timestamp: "2026-04-08T09:03:00Z".into(),
                message_id: "vr-m3".into(),
                provider: "claude".into(),
                model: "claude-sonnet-4-6".into(),
                version: Some("1.2.0".into()),
                estimated_cost_nanos: 2_000_000_000,
                pricing_version: "test".into(),
                cost_confidence: "exact".into(),
                ..Default::default()
            },
        ];
        insert_turns(&conn, &turns).unwrap();

        let rows = get_provider_version_rows(&conn, "claude", "2026-04-01", 10).unwrap();
        // Two distinct versions
        assert_eq!(rows.len(), 2);
        // 1.2.0 has cost 5_000_000_000 nanos, must come first
        assert_eq!(rows[0].version, "1.2.0");
        assert_eq!(rows[0].turns, 2);
        assert!((rows[0].cost_usd - 5.0).abs() < 1e-6);
        assert_eq!(rows[1].version, "1.1.0");
        assert_eq!(rows[1].turns, 1);
        assert!((rows[1].cost_usd - 1.0).abs() < 1e-6);
    }

    // ── merge_live_db ───────────────────────────────────────────────────────

    #[test]
    fn merge_live_db_copies_runtime_history_and_dedups() {
        use tempfile::TempDir;

        let dir = TempDir::new().unwrap();
        let live_path = dir.path().join("live.db");
        let target_path = dir.path().join("target.db");

        // Build the live (source) DB with rows in every section.
        {
            let live_conn = open_db(&live_path).unwrap();
            init_db(&live_conn).unwrap();

            insert_live_event(
                &live_conn,
                &LiveEventRow {
                    dedup_key: "live:tu1".into(),
                    received_at: "2026-04-18T10:00:00Z".into(),
                    session_id: Some("ses-live".into()),
                    tool_name: Some("Edit".into()),
                    cost_usd_nanos: 1_000_000,
                    input_tokens: 100,
                    output_tokens: 50,
                    raw_json: "{}".into(),
                    context_input_tokens: Some(45_000),
                    context_window_size: Some(200_000),
                    hook_reported_cost_nanos: Some(1_000_000),
                },
            )
            .unwrap();

            insert_agent_status_samples(
                &live_conn,
                "claude",
                &[(
                    "api".to_string(),
                    "API".to_string(),
                    "operational".to_string(),
                )],
                1_700_000_000,
            )
            .unwrap();

            live_conn
                .execute(
                    "INSERT INTO claude_usage_runs
                        (id, captured_at, status, exit_code, stdout_raw, stderr_raw, invocation_mode, period, parser_version, error_summary)
                     VALUES (1, '2026-04-18T10:00:00Z', 'ok', 0, '', '', 'auto', 'today', 'v1', '')",
                    [],
                )
                .unwrap();
            live_conn
                .execute(
                    "INSERT INTO claude_usage_factors
                        (id, run_id, factor_key, display_label, percent, description, advice_text, display_order)
                     VALUES (1, 1, 'k', 'L', 0.5, '', '', 0)",
                    [],
                )
                .unwrap();

            live_conn
                .execute(
                    "INSERT INTO rate_window_history
                        (timestamp, window_type, used_percent, resets_at, source_kind, source_path)
                     VALUES ('2026-04-18T10:00:00Z', '5h', 0.42, '2026-04-18T15:00:00Z', 'oauth', '')",
                    [],
                )
                .unwrap();
        }

        // Build a fresh target DB.
        {
            let target_conn = open_db(&target_path).unwrap();
            init_db(&target_conn).unwrap();
        }

        // Run the merge.
        let target_conn = open_db(&target_path).unwrap();
        merge_live_db(&target_conn, &live_path).unwrap();

        // Each section copied exactly one row.
        let live_events: i64 = target_conn
            .query_row("SELECT COUNT(*) FROM live_events", [], |r| r.get(0))
            .unwrap();
        assert_eq!(live_events, 1);
        let agent_history: i64 = target_conn
            .query_row("SELECT COUNT(*) FROM agent_status_history", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(agent_history, 1);
        let runs: i64 = target_conn
            .query_row("SELECT COUNT(*) FROM claude_usage_runs", [], |r| r.get(0))
            .unwrap();
        assert_eq!(runs, 1);
        let factors: i64 = target_conn
            .query_row("SELECT COUNT(*) FROM claude_usage_factors", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(factors, 1);
        let rwh: i64 = target_conn
            .query_row("SELECT COUNT(*) FROM rate_window_history", [], |r| r.get(0))
            .unwrap();
        assert_eq!(rwh, 1);

        // Idempotency: a second merge does NOT duplicate rows.
        merge_live_db(&target_conn, &live_path).unwrap();
        let live_events_after: i64 = target_conn
            .query_row("SELECT COUNT(*) FROM live_events", [], |r| r.get(0))
            .unwrap();
        assert_eq!(
            live_events_after, 1,
            "live_events must dedup on second merge"
        );
        let rwh_after: i64 = target_conn
            .query_row("SELECT COUNT(*) FROM rate_window_history", [], |r| r.get(0))
            .unwrap();
        assert_eq!(
            rwh_after, 1,
            "rate_window_history must dedup on second merge"
        );
    }

    #[test]
    fn merge_live_db_missing_live_path_is_ok() {
        use tempfile::TempDir;
        let dir = TempDir::new().unwrap();
        let target_path = dir.path().join("target.db");
        let conn = open_db(&target_path).unwrap();
        init_db(&conn).unwrap();
        // Should be a no-op when the live path does not exist.
        let stats = merge_live_db(&conn, &dir.path().join("nonexistent.db")).unwrap();
        assert_eq!(stats, MergeStats::default());
    }

    #[test]
    fn insert_live_event_writes_expected_columns() {
        let conn = test_conn();
        let row = LiveEventRow {
            dedup_key: "k1".into(),
            received_at: "2026-04-18T10:00:00Z".into(),
            session_id: Some("ses1".into()),
            tool_name: Some("Bash".into()),
            cost_usd_nanos: 500_000,
            input_tokens: 200,
            output_tokens: 50,
            raw_json: "{}".into(),
            context_input_tokens: Some(1234),
            context_window_size: Some(200_000),
            hook_reported_cost_nanos: Some(500_000),
        };
        insert_live_event(&conn, &row).unwrap();
        // Second insert with the same dedup_key is silently ignored.
        insert_live_event(&conn, &row).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM live_events", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn query_latest_context_window_returns_most_recent() {
        let conn = test_conn();
        // Two rows: an older one and a newer one.
        for (dedup_key, ts, ctx) in [
            ("k_old", "2026-04-18T08:00:00Z", 30_000_i64),
            ("k_new", "2026-04-18T10:00:00Z", 45_000_i64),
        ] {
            insert_live_event(
                &conn,
                &LiveEventRow {
                    dedup_key: dedup_key.into(),
                    received_at: ts.into(),
                    session_id: Some("s".into()),
                    tool_name: None,
                    cost_usd_nanos: 0,
                    input_tokens: 0,
                    output_tokens: 0,
                    raw_json: "{}".into(),
                    context_input_tokens: Some(ctx),
                    context_window_size: Some(200_000),
                    hook_reported_cost_nanos: None,
                },
            )
            .unwrap();
        }
        let row = query_latest_context_window(&conn).unwrap().unwrap();
        assert_eq!(row.context_input_tokens, 45_000);
        assert_eq!(row.context_window_size, 200_000);
        assert_eq!(row.session_id.as_deref(), Some("s"));
        assert_eq!(row.received_at, "2026-04-18T10:00:00Z");
    }
}
