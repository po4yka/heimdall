use anyhow::Result;
use rusqlite::Connection;
use tracing::warn;

use std::collections::HashMap;

use crate::models::{
    BillingModeSummary, BranchSummary, ConfidenceSummary, DailyModelRow, DailyProjectRow,
    DashboardData, EntrypointSummary, HourlyRow, McpServerSummary, ProviderSummary,
    ServiceTierSummary, SessionRow, ToolEvent, ToolSummary, Turn, VersionSummary,
};
use crate::scanner::parser::{classify_tool, raw_session_id};
use crate::tz::TzParams;

pub fn open_db(path: &std::path::Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;
    Ok(conn)
}

pub fn init_db(conn: &Connection) -> Result<()> {
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
            lines   INTEGER
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

    // Migration: add subagent columns if upgrading from older schema
    if !has_column(conn, "sessions", "provider") {
        conn.execute_batch(
            "ALTER TABLE sessions ADD COLUMN provider TEXT NOT NULL DEFAULT 'claude';",
        )?;
    }
    if !has_column(conn, "sessions", "total_reasoning_output") {
        conn.execute_batch(
            "ALTER TABLE sessions ADD COLUMN total_reasoning_output INTEGER DEFAULT 0;",
        )?;
    }
    if !has_column(conn, "sessions", "total_estimated_cost_nanos") {
        conn.execute_batch(
            "ALTER TABLE sessions ADD COLUMN total_estimated_cost_nanos INTEGER DEFAULT 0;",
        )?;
    }
    if !has_column(conn, "sessions", "pricing_version") {
        conn.execute_batch(
            "ALTER TABLE sessions ADD COLUMN pricing_version TEXT NOT NULL DEFAULT '';",
        )?;
    }
    if !has_column(conn, "sessions", "billing_mode") {
        conn.execute_batch(
            "ALTER TABLE sessions ADD COLUMN billing_mode TEXT NOT NULL DEFAULT 'estimated_local';",
        )?;
    }
    if !has_column(conn, "sessions", "cost_confidence") {
        conn.execute_batch(
            "ALTER TABLE sessions ADD COLUMN cost_confidence TEXT NOT NULL DEFAULT 'low';",
        )?;
    }
    if !has_column(conn, "turns", "provider") {
        conn.execute_batch(
            "ALTER TABLE turns ADD COLUMN provider TEXT NOT NULL DEFAULT 'claude';",
        )?;
    }
    if !has_column(conn, "turns", "reasoning_output_tokens") {
        conn.execute_batch(
            "ALTER TABLE turns ADD COLUMN reasoning_output_tokens INTEGER DEFAULT 0;",
        )?;
    }
    if !has_column(conn, "turns", "estimated_cost_nanos") {
        conn.execute_batch("ALTER TABLE turns ADD COLUMN estimated_cost_nanos INTEGER DEFAULT 0;")?;
    }
    if !has_column(conn, "turns", "agent_id") {
        conn.execute_batch(
            "ALTER TABLE turns ADD COLUMN is_subagent INTEGER DEFAULT 0;
             ALTER TABLE turns ADD COLUMN agent_id TEXT;",
        )?;
    }
    if !has_column(conn, "turns", "source_path") {
        conn.execute_batch("ALTER TABLE turns ADD COLUMN source_path TEXT NOT NULL DEFAULT '';")?;
    }
    if !has_column(conn, "turns", "pricing_version") {
        conn.execute_batch(
            "ALTER TABLE turns ADD COLUMN pricing_version TEXT NOT NULL DEFAULT '';
             ALTER TABLE turns ADD COLUMN pricing_model TEXT NOT NULL DEFAULT '';
             ALTER TABLE turns ADD COLUMN billing_mode TEXT NOT NULL DEFAULT 'estimated_local';
             ALTER TABLE turns ADD COLUMN cost_confidence TEXT NOT NULL DEFAULT 'low';",
        )?;
    }
    if !has_column(conn, "turns", "category") {
        conn.execute_batch("ALTER TABLE turns ADD COLUMN category TEXT NOT NULL DEFAULT '';")?;
    }

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
    if !has_column(conn, "tool_invocations", "provider") {
        conn.execute_batch(
            "ALTER TABLE tool_invocations ADD COLUMN provider TEXT NOT NULL DEFAULT 'claude';",
        )?;
    }

    // Feature 1: Session titles
    if !has_column(conn, "sessions", "title") {
        conn.execute_batch("ALTER TABLE sessions ADD COLUMN title TEXT;")?;
    }
    // Feature 2: Version tracking
    if !has_column(conn, "turns", "version") {
        conn.execute_batch("ALTER TABLE turns ADD COLUMN version TEXT;")?;
    }
    // Feature 3: Tool error tracking
    if !has_column(conn, "tool_invocations", "tool_use_id") {
        conn.execute_batch(
            "ALTER TABLE tool_invocations ADD COLUMN tool_use_id TEXT;
             ALTER TABLE tool_invocations ADD COLUMN is_error INTEGER DEFAULT 0;",
        )?;
    }
    if !has_column(conn, "tool_invocations", "source_path") {
        conn.execute_batch(
            "ALTER TABLE tool_invocations ADD COLUMN source_path TEXT NOT NULL DEFAULT '';",
        )?;
    }
    // Phase 3: One-shot rate tracking (nullable; 0=not-oneshot, 1=oneshot,
    // NULL=session has no edit activity and is unclassifiable)
    if !has_column(conn, "sessions", "one_shot") {
        conn.execute_batch("ALTER TABLE sessions ADD COLUMN one_shot INTEGER;")?;
    }

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

    prefix_existing_session_ids(conn)?;
    backfill_turn_pricing(conn)?;
    recompute_session_totals(conn)?;

    // Phase 20: Usage-limits file parser.
    // Add source_kind ('oauth' | 'file') and source_path to rate_window_history
    // so file-derived rows can be distinguished from OAuth-derived ones.
    if !has_column(conn, "rate_window_history", "source_kind") {
        conn.execute_batch(
            "ALTER TABLE rate_window_history ADD COLUMN source_kind TEXT NOT NULL DEFAULT 'oauth';",
        )?;
    }
    if !has_column(conn, "rate_window_history", "source_path") {
        conn.execute_batch(
            "ALTER TABLE rate_window_history ADD COLUMN source_path TEXT NOT NULL DEFAULT '';",
        )?;
    }

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

    Ok(())
}

fn has_column(conn: &Connection, table: &str, column: &str) -> bool {
    conn.prepare(&format!("SELECT {column} FROM {table} LIMIT 1"))
        .is_ok()
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
    let mut stmt = conn.prepare("SELECT mtime, lines FROM processed_files WHERE path = ?")?;
    let result = stmt.query_row([path], |row| {
        Ok((row.get::<_, f64>(0)?, row.get::<_, i64>(1)?))
    });
    match result {
        Ok(val) => Ok(Some(val)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn upsert_processed_file(conn: &Connection, path: &str, mtime: f64, lines: i64) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO processed_files (path, mtime, lines) VALUES (?1, ?2, ?3)",
        rusqlite::params![path, mtime, lines],
    )?;
    Ok(())
}

pub fn list_processed_files(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT path FROM processed_files")?;
    let paths = stmt
        .query_map([], |row| row.get(0))?
        .filter_map(|r| match r {
            Ok(val) => Some(val),
            Err(e) => {
                warn!("Failed to read row: {}", e);
                None
            }
        })
        .collect();
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

    turn.tool_use_ids
        .iter()
        .enumerate()
        .map(|(i, (tool_use_id, tool_name))| {
            let (kind, value) = classify_tool_event(tool_name);
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

/// Aggregate `cost_nanos` by `kind`, sorted descending by total cost.
/// Returns `Vec<(kind, total_cost_nanos)>`.
#[allow(dead_code)]
pub fn tool_event_cost_by_kind(conn: &Connection) -> Result<Vec<(String, i64)>> {
    let mut stmt = conn.prepare(
        "SELECT kind, COALESCE(SUM(cost_nanos), 0) as total
         FROM tool_events
         GROUP BY kind
         ORDER BY total DESC",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?
        .filter_map(|r| match r {
            Ok(val) => Some(val),
            Err(e) => {
                warn!("tool_event_cost_by_kind row error: {}", e);
                None
            }
        })
        .collect();
    Ok(rows)
}

/// Drilldown: aggregate `cost_nanos` by `value` for a specific `kind`.
/// Returns up to `limit` rows sorted descending by total cost.
/// Returns `Vec<(value, total_cost_nanos)>`.
#[allow(dead_code)]
pub fn tool_event_cost_by_value(
    conn: &Connection,
    kind: &str,
    limit: usize,
) -> Result<Vec<(String, i64)>> {
    let mut stmt = conn.prepare(
        "SELECT value, COALESCE(SUM(cost_nanos), 0) as total
         FROM tool_events
         WHERE kind = ?1
         GROUP BY value
         ORDER BY total DESC
         LIMIT ?2",
    )?;
    let rows = stmt
        .query_map(rusqlite::params![kind, limit as i64], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?
        .filter_map(|r| match r {
            Ok(val) => Some(val),
            Err(e) => {
                warn!("tool_event_cost_by_value row error: {}", e);
                None
            }
        })
        .collect();
    Ok(rows)
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
             billing_mode, cost_confidence, category)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24)",
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
        ])?;
    }
    Ok(())
}

#[allow(dead_code)]
pub fn insert_tool_invocations(
    conn: &Connection,
    turns: &[Turn],
    tool_results: &HashMap<String, bool>,
) -> Result<()> {
    let mut stmt = conn.prepare(
        "INSERT OR IGNORE INTO tool_invocations
            (session_id, provider, message_id, tool_name, mcp_server, mcp_tool, tool_category, tool_use_id, is_error, source_path)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
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
            ])?;
        }
    }
    Ok(())
}

#[allow(dead_code)]
pub fn delete_tool_invocations_by_session(conn: &Connection, session_ids: &[String]) -> Result<()> {
    for sid in session_ids {
        conn.execute("DELETE FROM tool_invocations WHERE session_id = ?1", [sid])?;
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

pub fn get_dashboard_data(conn: &Connection, tz: TzParams) -> Result<DashboardData> {
    let mut stmt = conn.prepare(
        "SELECT COALESCE(model, 'unknown') as model
         FROM turns GROUP BY model
         ORDER BY SUM(input_tokens + output_tokens) DESC",
    )?;
    let all_models: Vec<String> = stmt
        .query_map([], |row| row.get(0))?
        .filter_map(|r| match r {
            Ok(val) => Some(val),
            Err(e) => {
                warn!("Failed to read row: {}", e);
                None
            }
        })
        .collect();

    let provider_breakdown: Vec<ProviderSummary> = {
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
        })?
        .filter_map(|r| match r {
            Ok(val) => Some(val),
            Err(e) => {
                warn!("Failed to read provider summary row: {}", e);
                None
            }
        })
        .collect()
    };

    let confidence_breakdown: Vec<ConfidenceSummary> = {
        let mut stmt = conn.prepare(
            "SELECT COALESCE(cost_confidence, 'low') as cost_confidence,
                    COUNT(*) as turns,
                    COALESCE(SUM(estimated_cost_nanos), 0) as cost_nanos
             FROM turns
             GROUP BY cost_confidence
             ORDER BY turns DESC",
        )?;
        stmt.query_map([], |row| {
            Ok(ConfidenceSummary {
                confidence: row.get(0)?,
                turns: row.get(1)?,
                cost: row.get::<_, i64>(2)? as f64 / 1_000_000_000.0,
            })
        })?
        .filter_map(|r| r.ok())
        .collect()
    };

    let billing_mode_breakdown: Vec<BillingModeSummary> = {
        let mut stmt = conn.prepare(
            "SELECT COALESCE(billing_mode, 'estimated_local') as billing_mode,
                    COUNT(*) as turns,
                    COALESCE(SUM(estimated_cost_nanos), 0) as cost_nanos
             FROM turns
             GROUP BY billing_mode
             ORDER BY turns DESC",
        )?;
        stmt.query_map([], |row| {
            Ok(BillingModeSummary {
                billing_mode: row.get(0)?,
                turns: row.get(1)?,
                cost: row.get::<_, i64>(2)? as f64 / 1_000_000_000.0,
            })
        })?
        .filter_map(|r| r.ok())
        .collect()
    };

    let daily_by_model: Vec<DailyModelRow> = {
        let day_expr = tz.sql_day_expr("timestamp");
        let sql = format!(
            "SELECT {day_expr} as day, provider, COALESCE(model, 'unknown') as model,
                    SUM(input_tokens) as input, SUM(output_tokens) as output,
                    SUM(cache_read_tokens) as cache_read, SUM(cache_creation_tokens) as cache_creation,
                    SUM(reasoning_output_tokens) as reasoning_output,
                    COUNT(*) as turns,
                    COALESCE(SUM(estimated_cost_nanos), 0) as cost_nanos
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
                cost: row.get::<_, i64>(9)? as f64 / 1_000_000_000.0,
            })
        };
        let collect_rows = |mapped: rusqlite::MappedRows<'_, _>| -> Vec<DailyModelRow> {
            mapped
                .filter_map(|r| match r {
                    Ok(val) => Some(val),
                    Err(e) => {
                        warn!("Failed to read row: {}", e);
                        None
                    }
                })
                .collect()
        };
        let rows: Vec<DailyModelRow> = if let Some(offset_param) = tz.offset_sql_param() {
            collect_rows(stmt.query_map([offset_param], map_row)?)
        } else {
            collect_rows(stmt.query_map([], map_row)?)
        };
        rows
    };

    let sessions_all: Vec<SessionRow> = {
        let mut stmt = conn.prepare(
            "SELECT s.session_id, s.provider, s.project_name, s.first_timestamp, s.last_timestamp,
                    s.total_input_tokens, s.total_output_tokens,
                    s.total_cache_read, s.total_cache_creation, s.total_reasoning_output,
                    s.total_estimated_cost_nanos, s.model, s.turn_count,
                    s.pricing_version, s.billing_mode, s.cost_confidence,
                    COALESCE((SELECT COUNT(DISTINCT t.agent_id) FROM turns t WHERE t.session_id = s.session_id AND t.is_subagent = 1), 0) as subagent_count,
                    COALESCE((SELECT COUNT(*) FROM turns t WHERE t.session_id = s.session_id AND t.is_subagent = 1), 0) as subagent_turns,
                    s.title
             FROM sessions s ORDER BY s.last_timestamp DESC",
        )?;
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
            })
        })?
        .filter_map(|r| match r {
            Ok(val) => Some(val),
            Err(e) => {
                warn!("Failed to read row: {}", e);
                None
            }
        })
        .collect()
    };

    let subagent_summary: crate::models::SubagentSummary = conn
        .query_row(
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
        });

    let entrypoint_breakdown: Vec<EntrypointSummary> = {
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
        stmt.query_map([], |row| {
            Ok(EntrypointSummary {
                provider: row.get(0)?,
                entrypoint: row.get(1)?,
                sessions: row.get(2)?,
                turns: row.get(3)?,
                input: row.get(4)?,
                output: row.get(5)?,
            })
        })?
        .filter_map(|r| match r {
            Ok(val) => Some(val),
            Err(e) => {
                warn!("Failed to read row: {}", e);
                None
            }
        })
        .collect()
    };

    let service_tiers: Vec<ServiceTierSummary> = {
        let mut stmt = conn.prepare(
            "SELECT provider, COALESCE(service_tier, 'unknown') as tier,
                    COALESCE(inference_geo, 'unknown') as geo,
                    COUNT(*) as cnt
             FROM turns
             WHERE service_tier IS NOT NULL AND service_tier != ''
             GROUP BY provider, tier, geo
             ORDER BY cnt DESC",
        )?;
        stmt.query_map([], |row| {
            Ok(ServiceTierSummary {
                provider: row.get(0)?,
                service_tier: row.get(1)?,
                inference_geo: row.get(2)?,
                turns: row.get(3)?,
            })
        })?
        .filter_map(|r| match r {
            Ok(val) => Some(val),
            Err(e) => {
                warn!("Failed to read row: {}", e);
                None
            }
        })
        .collect()
    };

    let tool_summary: Vec<ToolSummary> = {
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
        })?
        .filter_map(|r| match r {
            Ok(val) => Some(val),
            Err(e) => {
                warn!("Failed to read tool summary row: {}", e);
                None
            }
        })
        .collect()
    };

    let mcp_summary: Vec<McpServerSummary> = {
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
        stmt.query_map([], |row| {
            Ok(McpServerSummary {
                provider: row.get(0)?,
                server: row.get(1)?,
                tools_used: row.get(2)?,
                invocations: row.get(3)?,
                sessions_used: row.get(4)?,
            })
        })?
        .filter_map(|r| match r {
            Ok(val) => Some(val),
            Err(e) => {
                warn!("Failed to read MCP summary row: {}", e);
                None
            }
        })
        .collect()
    };

    let hourly_distribution: Vec<HourlyRow> = {
        let mut stmt = conn.prepare(
            "SELECT provider, CAST(substr(timestamp, 12, 2) AS INTEGER) as hour,
                    COUNT(*) as turns, SUM(input_tokens) as input, SUM(output_tokens) as output,
                    SUM(reasoning_output_tokens) as reasoning_output
             FROM turns
             WHERE timestamp IS NOT NULL AND length(timestamp) >= 13
             GROUP BY provider, hour
             ORDER BY provider, hour",
        )?;
        stmt.query_map([], |row| {
            Ok(HourlyRow {
                provider: row.get(0)?,
                hour: row.get(1)?,
                turns: row.get(2)?,
                input: row.get(3)?,
                output: row.get(4)?,
                reasoning_output: row.get(5)?,
            })
        })?
        .filter_map(|r| match r {
            Ok(val) => Some(val),
            Err(e) => {
                warn!("Failed to read hourly row: {}", e);
                None
            }
        })
        .collect()
    };

    let git_branch_summary: Vec<BranchSummary> = {
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
        })?
        .filter_map(|r| match r {
            Ok(val) => Some(val),
            Err(e) => {
                warn!("Failed to read branch summary row: {}", e);
                None
            }
        })
        .collect()
    };

    let version_summary: Vec<VersionSummary> = {
        let mut stmt = conn.prepare(
            "SELECT provider, COALESCE(version, 'unknown') as ver,
                    COUNT(*) as turns,
                    COUNT(DISTINCT session_id) as sessions
             FROM turns
             WHERE version IS NOT NULL AND version != ''
             GROUP BY provider, ver
             ORDER BY turns DESC",
        )?;
        stmt.query_map([], |row| {
            Ok(VersionSummary {
                provider: row.get(0)?,
                version: row.get(1)?,
                turns: row.get(2)?,
                sessions: row.get(3)?,
            })
        })?
        .filter_map(|r| match r {
            Ok(val) => Some(val),
            Err(e) => {
                warn!("Failed to read version summary row: {}", e);
                None
            }
        })
        .collect()
    };

    let daily_by_project: Vec<DailyProjectRow> = {
        let mut stmt = conn.prepare(
            "SELECT substr(t.timestamp, 1, 10) as day, s.provider, s.project_name,
                    SUM(t.input_tokens) as input, SUM(t.output_tokens) as output,
                    SUM(t.reasoning_output_tokens) as reasoning_output,
                    COALESCE(SUM(t.estimated_cost_nanos), 0) as cost_nanos
             FROM turns t JOIN sessions s ON t.session_id = s.session_id
             GROUP BY day, s.provider, s.project_name
             ORDER BY day, s.provider, s.project_name",
        )?;
        stmt.query_map([], |row| {
            let day: String = row.get(0)?;
            let provider: String = row.get(1)?;
            let project: String = row.get(2)?;
            Ok(DailyProjectRow {
                day: day.clone(),
                provider: provider.clone(),
                project: project.clone(),
                input: row.get(3)?,
                output: row.get(4)?,
                reasoning_output: row.get(5)?,
                cost: row.get::<_, i64>(6)? as f64 / 1_000_000_000.0,
            })
        })?
        .filter_map(|r| match r {
            Ok(val) => Some(val),
            Err(e) => {
                warn!("Failed to read daily project row: {}", e);
                None
            }
        })
        .collect()
    };

    let generated_at = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

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
        generated_at,
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
    let rows = stmt
        .query_map(rusqlite::params![window_type, cutoff], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
        })?
        .filter_map(|r| match r {
            Ok(val) => Some(val),
            Err(e) => {
                warn!("Failed to read row: {}", e);
                None
            }
        })
        .collect();
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
    let non_zero: Vec<(i64, i64, i64, i64)> = stmt
        .query_map(rusqlite::params_from_iter(params.iter()), |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })?
        .filter_map(|r| match r {
            Ok(v) => Some(v),
            Err(e) => {
                warn!("heatmap row error: {e}");
                None
            }
        })
        .collect();

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Turn;
    use crate::pricing;
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

    #[test]
    fn test_init_db_idempotent() {
        let conn = test_conn();
        init_db(&conn).unwrap();
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
        let (mtime, lines) = get_processed_file(&conn, "/tmp/test.jsonl")
            .unwrap()
            .unwrap();
        assert!((mtime - 1234.5).abs() < 0.01);
        assert_eq!(lines, 100);
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
        insert_tool_invocations(&conn, &turns, &HashMap::new()).unwrap();
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
        insert_tool_invocations(&conn, &turns, &HashMap::new()).unwrap();
        insert_tool_invocations(&conn, &turns, &HashMap::new()).unwrap();
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
        insert_tool_invocations(&conn, &turns, &HashMap::new()).unwrap();

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
        insert_tool_invocations(&conn, &turns, &HashMap::new()).unwrap();

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
        insert_tool_invocations(&conn, &turns, &HashMap::new()).unwrap();

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
    fn test_tool_event_cost_by_kind_aggregation() {
        let conn = test_conn();

        // Insert a fixture that exercises the aggregation.
        let events = vec![
            crate::models::ToolEvent {
                dedup_key: "c:1".into(),
                ts_epoch: 1,
                session_id: "s1".into(),
                provider: "claude".into(),
                project: "p".into(),
                kind: "file".into(),
                value: "Edit".into(),
                cost_nanos: 400,
                source_path: "/tmp/x.jsonl".into(),
            },
            crate::models::ToolEvent {
                dedup_key: "c:2".into(),
                ts_epoch: 2,
                session_id: "s1".into(),
                provider: "claude".into(),
                project: "p".into(),
                kind: "file".into(),
                value: "Read".into(),
                cost_nanos: 200,
                source_path: "/tmp/x.jsonl".into(),
            },
            crate::models::ToolEvent {
                dedup_key: "c:3".into(),
                ts_epoch: 3,
                session_id: "s1".into(),
                provider: "claude".into(),
                project: "p".into(),
                kind: "bash".into(),
                value: "Bash".into(),
                cost_nanos: 100,
                source_path: "/tmp/x.jsonl".into(),
            },
            crate::models::ToolEvent {
                dedup_key: "c:4".into(),
                ts_epoch: 4,
                session_id: "s1".into(),
                provider: "claude".into(),
                project: "p".into(),
                kind: "mcp".into(),
                value: "read_file".into(),
                cost_nanos: 300,
                source_path: "/tmp/x.jsonl".into(),
            },
        ];
        insert_tool_events(&conn, &events).unwrap();

        let by_kind = tool_event_cost_by_kind(&conn).unwrap();
        // Sorted descending: file=600, mcp=300, bash=100
        assert_eq!(by_kind.len(), 3);
        assert_eq!(by_kind[0], ("file".into(), 600));
        assert_eq!(by_kind[1], ("mcp".into(), 300));
        assert_eq!(by_kind[2], ("bash".into(), 100));

        let by_value = tool_event_cost_by_value(&conn, "file", 10).unwrap();
        assert_eq!(by_value.len(), 2);
        assert_eq!(by_value[0], ("Edit".into(), 400));
        assert_eq!(by_value[1], ("Read".into(), 200));
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
}
