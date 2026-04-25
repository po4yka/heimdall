# Claude Usage Tracker Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Rust CLI + web dashboard that parses Claude Code JSONL transcripts, stores usage in SQLite, and serves an interactive analytics UI.

**Architecture:** Single binary with embedded web assets. Scanner module parses JSONL files incrementally into SQLite. Server module (axum) serves a Chart.js dashboard with JSON API. CLI (clap) dispatches commands.

**Tech Stack:** Rust 1.94, clap, axum, tokio, rusqlite (bundled), serde/serde_json, chrono, walkdir, tracing, open

---

## File Structure

```
Cargo.toml
src/
  main.rs              -- CLI entry point (clap), dispatches commands
  models.rs            -- Session, Turn, ScanResult, DashboardData types
  pricing.rs           -- PRICING table, get_pricing(), calc_cost()
  scanner/
    mod.rs             -- scan() orchestration, incremental logic
    parser.rs          -- parse_jsonl_file(), streaming dedup
    db.rs              -- init_db(), insert_turns(), upsert_sessions(), queries
  server/
    mod.rs             -- axum router, serve()
    api.rs             -- handler functions for /api/*
    assets.rs          -- include_str! for HTML/CSS/JS
  ui/
    index.html         -- Dashboard HTML (dark theme, Chart.js)
    style.css          -- Dashboard CSS
    app.js             -- Dashboard JS (filtering, charts, tables, CSV export)
```

---

### Task 1: Project Setup + Cargo.toml

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs` (stub)

- [ ] **Step 1: Initialize cargo project**

```bash
cd /Users/npochaev/GitHub/claude-usage-tracker
cargo init --name claude-usage-tracker
```

- [ ] **Step 2: Replace Cargo.toml with full dependencies**

```toml
[package]
name = "claude-usage-tracker"
version = "0.1.0"
edition = "2021"
description = "Local analytics dashboard for Claude Code usage and token consumption"
license = "BSD-3-Clause"

[dependencies]
anyhow = "1"
axum = "0.8"
chrono = { version = "0.4", features = ["serde"] }
clap = { version = "4", features = ["derive"] }
open = "5"
rusqlite = { version = "0.34", features = ["bundled"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tokio = { version = "1", features = ["rt", "net", "macros", "time"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
walkdir = "2"

[profile.release]
lto = true
strip = true
```

- [ ] **Step 3: Write minimal main.rs stub**

```rust
fn main() {
    println!("claude-usage-tracker");
}
```

- [ ] **Step 4: Verify it compiles**

Run: `cargo build`
Expected: Compiles with no errors.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml src/main.rs
git commit -m "feat: initialize Rust project with dependencies"
```

---

### Task 2: Shared Types (models.rs)

**Files:**
- Create: `src/models.rs`
- Modify: `src/main.rs` (add mod declaration)

- [ ] **Step 1: Create models.rs with all shared types**

```rust
// src/models.rs
use serde::Serialize;

#[derive(Debug, Clone, Default)]
pub struct Session {
    pub session_id: String,
    pub project_name: String,
    pub project_slug: String,
    pub first_timestamp: String,
    pub last_timestamp: String,
    pub git_branch: String,
    pub model: Option<String>,
    pub entrypoint: String,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_cache_read: i64,
    pub total_cache_creation: i64,
    pub turn_count: i64,
}

#[derive(Debug, Clone, Default)]
pub struct Turn {
    pub session_id: String,
    pub timestamp: String,
    pub model: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_creation_tokens: i64,
    pub tool_name: Option<String>,
    pub cwd: String,
    pub message_id: String,
    pub service_tier: Option<String>,
    pub inference_geo: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SessionMeta {
    pub session_id: String,
    pub project_name: String,
    pub project_slug: String,
    pub first_timestamp: String,
    pub last_timestamp: String,
    pub git_branch: String,
    pub model: Option<String>,
    pub entrypoint: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ScanResult {
    pub new: usize,
    pub updated: usize,
    pub skipped: usize,
    pub turns: usize,
    pub sessions: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct DashboardData {
    pub all_models: Vec<String>,
    pub daily_by_model: Vec<DailyModelRow>,
    pub sessions_all: Vec<SessionRow>,
    pub generated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DailyModelRow {
    pub day: String,
    pub model: String,
    pub input: i64,
    pub output: i64,
    pub cache_read: i64,
    pub cache_creation: i64,
    pub turns: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionRow {
    pub session_id: String,
    pub project: String,
    pub last: String,
    pub last_date: String,
    pub duration_min: f64,
    pub model: String,
    pub turns: i64,
    pub input: i64,
    pub output: i64,
    pub cache_read: i64,
    pub cache_creation: i64,
}
```

- [ ] **Step 2: Add mod declaration to main.rs**

```rust
mod models;

fn main() {
    println!("claude-usage-tracker");
}
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo build`
Expected: Compiles with no errors.

- [ ] **Step 4: Commit**

```bash
git add src/models.rs src/main.rs
git commit -m "feat: add shared data types"
```

---

### Task 3: Pricing Module (pricing.rs)

**Files:**
- Create: `src/pricing.rs`
- Modify: `src/main.rs` (add mod declaration)

- [ ] **Step 1: Write pricing tests first**

Add at the bottom of `src/pricing.rs`:

```rust
// src/pricing.rs

pub struct ModelPricing {
    pub input: f64,
    pub output: f64,
    pub cache_write: f64,
    pub cache_read: f64,
}

const PRICING_TABLE: &[(&str, ModelPricing)] = &[
    ("claude-opus-4-6",   ModelPricing { input: 15.0, output: 75.0, cache_write: 18.75, cache_read: 1.50 }),
    ("claude-opus-4-5",   ModelPricing { input: 15.0, output: 75.0, cache_write: 18.75, cache_read: 1.50 }),
    ("claude-sonnet-4-6", ModelPricing { input:  3.0, output: 15.0, cache_write:  3.75, cache_read: 0.30 }),
    ("claude-sonnet-4-5", ModelPricing { input:  3.0, output: 15.0, cache_write:  3.75, cache_read: 0.30 }),
    ("claude-haiku-4-5",  ModelPricing { input:  1.0, output:  5.0, cache_write:  1.25, cache_read: 0.10 }),
    ("claude-haiku-4-6",  ModelPricing { input:  1.0, output:  5.0, cache_write:  1.25, cache_read: 0.10 }),
];

/// Look up pricing for a model. Tries exact match, prefix match, then substring fallback.
pub fn get_pricing(model: &str) -> Option<&'static ModelPricing> {
    if model.is_empty() {
        return None;
    }
    // Exact match
    for (name, pricing) in PRICING_TABLE {
        if *name == model {
            return Some(pricing);
        }
    }
    // Prefix match
    for (name, pricing) in PRICING_TABLE {
        if model.starts_with(name) {
            return Some(pricing);
        }
    }
    // Substring fallback (case-insensitive)
    let lower = model.to_lowercase();
    if lower.contains("opus") {
        return get_pricing("claude-opus-4-6");
    }
    if lower.contains("sonnet") {
        return get_pricing("claude-sonnet-4-6");
    }
    if lower.contains("haiku") {
        return get_pricing("claude-haiku-4-5");
    }
    None
}

/// Returns true if this model is an Anthropic model we can price.
pub fn is_billable(model: &str) -> bool {
    get_pricing(model).is_some()
}

/// Calculate cost in dollars for the given token counts.
pub fn calc_cost(model: &str, input: i64, output: i64, cache_read: i64, cache_creation: i64) -> f64 {
    let Some(p) = get_pricing(model) else {
        return 0.0;
    };
    input as f64 * p.input / 1_000_000.0
        + output as f64 * p.output / 1_000_000.0
        + cache_read as f64 * p.cache_read / 1_000_000.0
        + cache_creation as f64 * p.cache_write / 1_000_000.0
}

/// Format a token count for display (e.g., 1.5M, 2.3K, 999).
pub fn fmt_tokens(n: i64) -> String {
    if n >= 1_000_000 {
        format!("{:.2}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

/// Format cost for display.
pub fn fmt_cost(c: f64) -> String {
    format!("${:.4}", c)
}

/// Return the pricing table as JSON for injection into dashboard JS.
pub fn pricing_table_json() -> String {
    let entries: Vec<String> = PRICING_TABLE
        .iter()
        .map(|(name, p)| {
            format!(
                "  '{}': {{ input: {}, output: {}, cache_write: {}, cache_read: {} }}",
                name, p.input, p.output, p.cache_write, p.cache_read
            )
        })
        .collect();
    format!("{{\n{}\n}}", entries.join(",\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let p = get_pricing("claude-sonnet-4-6").unwrap();
        assert_eq!(p.input, 3.0);
        assert_eq!(p.output, 15.0);
    }

    #[test]
    fn test_all_known_models() {
        for (name, _) in PRICING_TABLE {
            assert!(get_pricing(name).is_some(), "Missing pricing for {name}");
        }
    }

    #[test]
    fn test_prefix_match() {
        let p = get_pricing("claude-sonnet-4-6-20260401").unwrap();
        assert_eq!(p.input, 3.0);
    }

    #[test]
    fn test_substring_opus() {
        let p = get_pricing("new-opus-5-model").unwrap();
        assert_eq!(p.input, 15.0);
    }

    #[test]
    fn test_substring_case_insensitive() {
        let p = get_pricing("Claude-Opus-Next").unwrap();
        assert_eq!(p.input, 15.0);
    }

    #[test]
    fn test_unknown_returns_none() {
        assert!(get_pricing("gpt-4o").is_none());
        assert!(get_pricing("").is_none());
    }

    #[test]
    fn test_calc_cost_sonnet_input() {
        let cost = calc_cost("claude-sonnet-4-6", 1_000_000, 0, 0, 0);
        assert!((cost - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_calc_cost_opus_output() {
        let cost = calc_cost("claude-opus-4-6", 0, 1_000_000, 0, 0);
        assert!((cost - 75.0).abs() < 0.001);
    }

    #[test]
    fn test_calc_cost_cache_read() {
        let cost = calc_cost("claude-opus-4-6", 0, 0, 1_000_000, 0);
        assert!((cost - 1.50).abs() < 0.001);
    }

    #[test]
    fn test_calc_cost_cache_write() {
        let cost = calc_cost("claude-opus-4-6", 0, 0, 0, 1_000_000);
        assert!((cost - 18.75).abs() < 0.001);
    }

    #[test]
    fn test_calc_cost_unknown_model_zero() {
        let cost = calc_cost("gpt-4o", 1_000_000, 500_000, 0, 0);
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn test_fmt_tokens() {
        assert_eq!(fmt_tokens(1_500_000), "1.50M");
        assert_eq!(fmt_tokens(1_500), "1.5K");
        assert_eq!(fmt_tokens(999), "999");
    }

    #[test]
    fn test_fmt_cost() {
        assert_eq!(fmt_cost(3.0), "$3.0000");
    }

    #[test]
    fn test_is_billable() {
        assert!(is_billable("claude-sonnet-4-6"));
        assert!(!is_billable("gpt-4o"));
    }

    #[test]
    fn test_pricing_table_json() {
        let json = pricing_table_json();
        assert!(json.contains("claude-opus-4-6"));
        assert!(json.contains("claude-haiku-4-5"));
    }
}
```

- [ ] **Step 2: Add mod to main.rs**

```rust
mod models;
mod pricing;

fn main() {
    println!("claude-usage-tracker");
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test pricing`
Expected: All tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/pricing.rs src/main.rs
git commit -m "feat: add pricing module with cost calculation"
```

---

### Task 4: Scanner Database (scanner/db.rs)

**Files:**
- Create: `src/scanner/mod.rs` (stub re-exports)
- Create: `src/scanner/db.rs`
- Modify: `src/main.rs` (add mod)

- [ ] **Step 1: Create scanner/mod.rs stub**

```rust
// src/scanner/mod.rs
pub mod db;
pub mod parser;

// scan() will be added in Task 6
```

- [ ] **Step 2: Create scanner/db.rs**

```rust
// src/scanner/db.rs
use anyhow::Result;
use rusqlite::Connection;
use crate::models::{Turn, Session, SessionMeta, DashboardData, DailyModelRow, SessionRow};

pub fn open_db(path: &std::path::Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;
    Ok(conn)
}

pub fn init_db(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS sessions (
            session_id          TEXT PRIMARY KEY,
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
            turn_count          INTEGER DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS turns (
            id                      INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id              TEXT NOT NULL,
            timestamp               TEXT,
            model                   TEXT,
            input_tokens            INTEGER DEFAULT 0,
            output_tokens           INTEGER DEFAULT 0,
            cache_read_tokens       INTEGER DEFAULT 0,
            cache_creation_tokens   INTEGER DEFAULT 0,
            tool_name               TEXT,
            cwd                     TEXT,
            message_id              TEXT,
            service_tier            TEXT,
            inference_geo           TEXT
        );

        CREATE TABLE IF NOT EXISTS processed_files (
            path    TEXT PRIMARY KEY,
            mtime   REAL,
            lines   INTEGER
        );

        CREATE INDEX IF NOT EXISTS idx_turns_session ON turns(session_id);
        CREATE INDEX IF NOT EXISTS idx_turns_timestamp ON turns(timestamp);
        CREATE INDEX IF NOT EXISTS idx_sessions_first ON sessions(first_timestamp);
        "
    )?;

    // Conditional unique index for message_id dedup
    conn.execute_batch(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_turns_message_id
         ON turns(message_id) WHERE message_id IS NOT NULL AND message_id != '';"
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

pub fn insert_turns(conn: &Connection, turns: &[Turn]) -> Result<()> {
    let mut stmt = conn.prepare(
        "INSERT OR IGNORE INTO turns
            (session_id, timestamp, model, input_tokens, output_tokens,
             cache_read_tokens, cache_creation_tokens, tool_name, cwd,
             message_id, service_tier, inference_geo)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)"
    )?;
    for t in turns {
        let msg_id = if t.message_id.is_empty() { None } else { Some(&t.message_id) };
        stmt.execute(rusqlite::params![
            t.session_id, t.timestamp, t.model,
            t.input_tokens, t.output_tokens,
            t.cache_read_tokens, t.cache_creation_tokens,
            t.tool_name, t.cwd, msg_id,
            t.service_tier, t.inference_geo,
        ])?;
    }
    Ok(())
}

pub fn upsert_sessions(conn: &Connection, sessions: &[Session]) -> Result<()> {
    for s in sessions {
        let exists: bool = conn.query_row(
            "SELECT COUNT(*) FROM sessions WHERE session_id = ?1",
            [&s.session_id],
            |row| row.get::<_, i64>(0).map(|c| c > 0),
        )?;

        if exists {
            conn.execute(
                "UPDATE sessions SET
                    last_timestamp = MAX(last_timestamp, ?1),
                    total_input_tokens = total_input_tokens + ?2,
                    total_output_tokens = total_output_tokens + ?3,
                    total_cache_read = total_cache_read + ?4,
                    total_cache_creation = total_cache_creation + ?5,
                    turn_count = turn_count + ?6,
                    model = COALESCE(?7, model)
                WHERE session_id = ?8",
                rusqlite::params![
                    s.last_timestamp,
                    s.total_input_tokens, s.total_output_tokens,
                    s.total_cache_read, s.total_cache_creation,
                    s.turn_count, s.model,
                    s.session_id,
                ],
            )?;
        } else {
            conn.execute(
                "INSERT INTO sessions
                    (session_id, project_name, project_slug, first_timestamp, last_timestamp,
                     git_branch, total_input_tokens, total_output_tokens,
                     total_cache_read, total_cache_creation, model, entrypoint, turn_count)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                rusqlite::params![
                    s.session_id, s.project_name, s.project_slug,
                    s.first_timestamp, s.last_timestamp, s.git_branch,
                    s.total_input_tokens, s.total_output_tokens,
                    s.total_cache_read, s.total_cache_creation,
                    s.model, s.entrypoint, s.turn_count,
                ],
            )?;
        }
    }
    Ok(())
}

/// Recompute session totals from actual turns in DB.
/// This ensures correctness when INSERT OR IGNORE skips duplicate turns.
pub fn recompute_session_totals(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "UPDATE sessions SET
            total_input_tokens = COALESCE((SELECT SUM(input_tokens) FROM turns WHERE turns.session_id = sessions.session_id), 0),
            total_output_tokens = COALESCE((SELECT SUM(output_tokens) FROM turns WHERE turns.session_id = sessions.session_id), 0),
            total_cache_read = COALESCE((SELECT SUM(cache_read_tokens) FROM turns WHERE turns.session_id = sessions.session_id), 0),
            total_cache_creation = COALESCE((SELECT SUM(cache_creation_tokens) FROM turns WHERE turns.session_id = sessions.session_id), 0),
            turn_count = COALESCE((SELECT COUNT(*) FROM turns WHERE turns.session_id = sessions.session_id), 0)"
    )?;
    Ok(())
}

pub fn get_dashboard_data(conn: &Connection) -> Result<DashboardData> {
    // All models
    let mut stmt = conn.prepare(
        "SELECT COALESCE(model, 'unknown') as model
         FROM turns GROUP BY model
         ORDER BY SUM(input_tokens + output_tokens) DESC"
    )?;
    let all_models: Vec<String> = stmt.query_map([], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();

    // Daily by model
    let mut stmt = conn.prepare(
        "SELECT substr(timestamp, 1, 10) as day, COALESCE(model, 'unknown') as model,
                SUM(input_tokens) as input, SUM(output_tokens) as output,
                SUM(cache_read_tokens) as cache_read, SUM(cache_creation_tokens) as cache_creation,
                COUNT(*) as turns
         FROM turns GROUP BY day, model ORDER BY day, model"
    )?;
    let daily_by_model: Vec<DailyModelRow> = stmt.query_map([], |row| {
        Ok(DailyModelRow {
            day: row.get(0)?,
            model: row.get(1)?,
            input: row.get(2)?,
            output: row.get(3)?,
            cache_read: row.get(4)?,
            cache_creation: row.get(5)?,
            turns: row.get(6)?,
        })
    })?.filter_map(|r| r.ok()).collect();

    // Sessions
    let mut stmt = conn.prepare(
        "SELECT session_id, project_name, first_timestamp, last_timestamp,
                total_input_tokens, total_output_tokens,
                total_cache_read, total_cache_creation, model, turn_count
         FROM sessions ORDER BY last_timestamp DESC"
    )?;
    let sessions_all: Vec<SessionRow> = stmt.query_map([], |row| {
        let session_id: String = row.get(0)?;
        let first_ts: String = row.get::<_, Option<String>>(2)?.unwrap_or_default();
        let last_ts: String = row.get::<_, Option<String>>(3)?.unwrap_or_default();

        let duration_min = compute_duration_min(&first_ts, &last_ts);

        Ok(SessionRow {
            session_id: session_id.chars().take(8).collect(),
            project: row.get::<_, Option<String>>(1)?.unwrap_or_else(|| "unknown".into()),
            last: last_ts.chars().take(16).collect::<String>().replace('T', " "),
            last_date: last_ts.chars().take(10).collect(),
            duration_min,
            model: row.get::<_, Option<String>>(8)?.unwrap_or_else(|| "unknown".into()),
            turns: row.get(9)?,
            input: row.get(4)?,
            output: row.get(5)?,
            cache_read: row.get(6)?,
            cache_creation: row.get(7)?,
        })
    })?.filter_map(|r| r.ok()).collect();

    let generated_at = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    Ok(DashboardData {
        all_models,
        daily_by_model,
        sessions_all,
        generated_at,
    })
}

fn compute_duration_min(first: &str, last: &str) -> f64 {
    let parse = |s: &str| -> Option<chrono::DateTime<chrono::FixedOffset>> {
        chrono::DateTime::parse_from_rfc3339(s).ok()
            .or_else(|| chrono::DateTime::parse_from_rfc3339(&format!("{}+00:00", s.trim_end_matches('Z'))).ok())
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
        init_db(&conn).unwrap(); // second call should not error
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
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM turns", [], |row| row.get(0)).unwrap();
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
        insert_turns(&conn, &[turn]).unwrap(); // duplicate
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM turns", [], |row| row.get(0)).unwrap();
        assert_eq!(count, 1); // deduped
    }

    #[test]
    fn test_null_message_id_not_deduped() {
        let conn = test_conn();
        let t1 = Turn { session_id: "s1".into(), input_tokens: 100, ..Default::default() };
        let t2 = Turn { session_id: "s1".into(), input_tokens: 200, ..Default::default() };
        insert_turns(&conn, &[t1]).unwrap();
        insert_turns(&conn, &[t2]).unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM turns", [], |row| row.get(0)).unwrap();
        assert_eq!(count, 2); // both kept
    }

    #[test]
    fn test_processed_file_roundtrip() {
        let conn = test_conn();
        assert!(get_processed_file(&conn, "/tmp/test.jsonl").unwrap().is_none());
        upsert_processed_file(&conn, "/tmp/test.jsonl", 1234.5, 100).unwrap();
        let (mtime, lines) = get_processed_file(&conn, "/tmp/test.jsonl").unwrap().unwrap();
        assert!((mtime - 1234.5).abs() < 0.01);
        assert_eq!(lines, 100);
    }

    #[test]
    fn test_compute_duration_min() {
        let d = compute_duration_min("2026-04-08T09:00:00Z", "2026-04-08T10:00:00Z");
        assert!((d - 60.0).abs() < 0.1);
    }
}
```

- [ ] **Step 3: Add scanner mod to main.rs**

```rust
mod models;
mod pricing;
mod scanner;

fn main() {
    println!("claude-usage-tracker");
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test scanner::db`
Expected: All tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/scanner/ src/main.rs
git commit -m "feat: add scanner database module with SQLite schema"
```

---

### Task 5: JSONL Parser (scanner/parser.rs)

**Files:**
- Create: `src/scanner/parser.rs`

- [ ] **Step 1: Create parser.rs with parsing + dedup logic**

```rust
// src/scanner/parser.rs
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::Path;
use tracing::warn;

use crate::models::{Turn, SessionMeta};

/// Derive a friendly project name from cwd (last 2 path components).
pub fn project_name_from_cwd(cwd: &str) -> String {
    if cwd.is_empty() {
        return "unknown".into();
    }
    let normalized = cwd.replace('\\', "/");
    let trimmed = normalized.trim_end_matches('/');
    let parts: Vec<&str> = trimmed.split('/').collect();
    match parts.len() {
        0 => "unknown".into(),
        1 => parts[0].to_string(),
        _ => format!("{}/{}", parts[parts.len() - 2], parts[parts.len() - 1]),
    }
}

pub struct ParseResult {
    pub session_metas: Vec<SessionMeta>,
    pub turns: Vec<Turn>,
    pub line_count: i64,
}

/// Parse a JSONL file, deduplicating streaming events by message.id.
/// If `skip_lines > 0`, skips that many lines from the start (for incremental updates).
pub fn parse_jsonl_file(filepath: &Path, skip_lines: i64) -> ParseResult {
    let mut seen_messages: HashMap<String, Turn> = HashMap::new();
    let mut turns_no_id: Vec<Turn> = Vec::new();
    let mut session_meta: HashMap<String, SessionMeta> = HashMap::new();
    let mut line_count: i64 = 0;

    let file = match std::fs::File::open(filepath) {
        Ok(f) => f,
        Err(e) => {
            warn!("Error opening {}: {}", filepath.display(), e);
            return ParseResult { session_metas: vec![], turns: vec![], line_count: 0 };
        }
    };

    let reader = BufReader::new(file);
    for line_result in reader.lines() {
        line_count += 1;
        if line_count <= skip_lines {
            continue;
        }

        let line = match line_result {
            Ok(l) => l,
            Err(_) => continue,
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let record: serde_json::Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let rtype = record.get("type").and_then(|v| v.as_str()).unwrap_or("");
        if rtype != "assistant" && rtype != "user" {
            continue;
        }

        let session_id = match record.get("sessionId").and_then(|v| v.as_str()) {
            Some(s) if !s.is_empty() => s.to_string(),
            _ => continue,
        };

        let timestamp = record.get("timestamp").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let cwd = record.get("cwd").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let git_branch = record.get("gitBranch").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let entrypoint = record.get("entrypoint").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let slug = record.get("slug").and_then(|v| v.as_str()).unwrap_or("").to_string();

        // Update session metadata
        session_meta.entry(session_id.clone())
            .and_modify(|meta| {
                if !timestamp.is_empty() {
                    if meta.first_timestamp.is_empty() || timestamp < meta.first_timestamp {
                        meta.first_timestamp = timestamp.clone();
                    }
                    if meta.last_timestamp.is_empty() || timestamp > meta.last_timestamp {
                        meta.last_timestamp = timestamp.clone();
                    }
                }
                if !git_branch.is_empty() && meta.git_branch.is_empty() {
                    meta.git_branch = git_branch.clone();
                }
            })
            .or_insert_with(|| SessionMeta {
                session_id: session_id.clone(),
                project_name: project_name_from_cwd(&cwd),
                project_slug: slug.clone(),
                first_timestamp: timestamp.clone(),
                last_timestamp: timestamp.clone(),
                git_branch: git_branch.clone(),
                model: None,
                entrypoint: entrypoint.clone(),
            });

        if rtype == "assistant" {
            let msg = match record.get("message") {
                Some(m) => m,
                None => continue,
            };
            let usage = msg.get("usage").cloned().unwrap_or(serde_json::Value::Object(Default::default()));
            let model = msg.get("model").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let message_id = msg.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();

            let input_tokens = usage.get("input_tokens").and_then(|v| v.as_i64()).unwrap_or(0);
            let output_tokens = usage.get("output_tokens").and_then(|v| v.as_i64()).unwrap_or(0);
            let cache_read = usage.get("cache_read_input_tokens").and_then(|v| v.as_i64()).unwrap_or(0);
            let cache_creation = usage.get("cache_creation_input_tokens").and_then(|v| v.as_i64()).unwrap_or(0);

            // Skip zero-token records
            if input_tokens + output_tokens + cache_read + cache_creation == 0 {
                continue;
            }

            // Extract tool name
            let tool_name = msg.get("content")
                .and_then(|c| c.as_array())
                .and_then(|arr| arr.iter().find(|item| {
                    item.get("type").and_then(|t| t.as_str()) == Some("tool_use")
                }))
                .and_then(|item| item.get("name").and_then(|n| n.as_str()))
                .map(String::from);

            let service_tier = usage.get("service_tier").and_then(|v| v.as_str()).map(String::from);
            let inference_geo = usage.get("inference_geo").and_then(|v| v.as_str()).map(String::from);

            if !model.is_empty() {
                if let Some(meta) = session_meta.get_mut(&session_id) {
                    meta.model = Some(model.clone());
                }
            }

            let turn = Turn {
                session_id: session_id.clone(),
                timestamp: timestamp.clone(),
                model,
                input_tokens,
                output_tokens,
                cache_read_tokens: cache_read,
                cache_creation_tokens: cache_creation,
                tool_name,
                cwd,
                message_id: message_id.clone(),
                service_tier,
                inference_geo,
            };

            if !message_id.is_empty() {
                seen_messages.insert(message_id, turn);
            } else {
                turns_no_id.push(turn);
            }
        }
    }

    let mut turns = turns_no_id;
    turns.extend(seen_messages.into_values());

    ParseResult {
        session_metas: session_meta.into_values().collect(),
        turns,
        line_count,
    }
}

/// Aggregate turn data into session-level stats.
pub fn aggregate_sessions(metas: &[SessionMeta], turns: &[Turn]) -> Vec<Session> {
    use crate::models::Session;
    use std::collections::HashMap;

    struct Stats {
        total_input: i64,
        total_output: i64,
        total_cache_read: i64,
        total_cache_creation: i64,
        turn_count: i64,
        model: Option<String>,
    }

    let mut stats_map: HashMap<&str, Stats> = HashMap::new();
    for t in turns {
        let entry = stats_map.entry(&t.session_id).or_insert(Stats {
            total_input: 0, total_output: 0, total_cache_read: 0,
            total_cache_creation: 0, turn_count: 0, model: None,
        });
        entry.total_input += t.input_tokens;
        entry.total_output += t.output_tokens;
        entry.total_cache_read += t.cache_read_tokens;
        entry.total_cache_creation += t.cache_creation_tokens;
        entry.turn_count += 1;
        if !t.model.is_empty() {
            entry.model = Some(t.model.clone());
        }
    }

    metas.iter().map(|meta| {
        let empty = Stats {
            total_input: 0, total_output: 0, total_cache_read: 0,
            total_cache_creation: 0, turn_count: 0, model: None,
        };
        let s = stats_map.get(meta.session_id.as_str()).unwrap_or(&empty);
        Session {
            session_id: meta.session_id.clone(),
            project_name: meta.project_name.clone(),
            project_slug: meta.project_slug.clone(),
            first_timestamp: meta.first_timestamp.clone(),
            last_timestamp: meta.last_timestamp.clone(),
            git_branch: meta.git_branch.clone(),
            model: s.model.clone().or_else(|| meta.model.clone()),
            entrypoint: meta.entrypoint.clone(),
            total_input_tokens: s.total_input,
            total_output_tokens: s.total_output,
            total_cache_read: s.total_cache_read,
            total_cache_creation: s.total_cache_creation,
            turn_count: s.turn_count,
        }
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn make_assistant_record(
        session_id: &str, model: &str, input: i64, output: i64,
        message_id: &str,
    ) -> String {
        let mut msg = serde_json::json!({
            "model": model,
            "usage": {
                "input_tokens": input,
                "output_tokens": output,
                "cache_read_input_tokens": 0,
                "cache_creation_input_tokens": 0,
            },
            "content": [],
        });
        if !message_id.is_empty() {
            msg["id"] = serde_json::json!(message_id);
        }
        serde_json::json!({
            "type": "assistant",
            "sessionId": session_id,
            "timestamp": "2026-04-08T10:00:00Z",
            "cwd": "/home/user/project",
            "message": msg,
        }).to_string()
    }

    fn make_user_record(session_id: &str) -> String {
        serde_json::json!({
            "type": "user",
            "sessionId": session_id,
            "timestamp": "2026-04-08T09:59:00Z",
            "cwd": "/home/user/project",
        }).to_string()
    }

    fn write_jsonl(dir: &TempDir, name: &str, lines: &[String]) -> std::path::PathBuf {
        let path = dir.path().join(name);
        let mut f = std::fs::File::create(&path).unwrap();
        for line in lines {
            writeln!(f, "{}", line).unwrap();
        }
        path
    }

    #[test]
    fn test_project_name_from_cwd() {
        assert_eq!(project_name_from_cwd("/home/user/project"), "user/project");
        assert_eq!(project_name_from_cwd("C:\\Users\\me\\proj"), "me/proj");
        assert_eq!(project_name_from_cwd("/a/b/c/d"), "c/d");
        assert_eq!(project_name_from_cwd(""), "unknown");
        assert_eq!(project_name_from_cwd("/home/user/project/"), "user/project");
    }

    #[test]
    fn test_basic_parsing() {
        let dir = TempDir::new().unwrap();
        let path = write_jsonl(&dir, "test.jsonl", &[
            make_user_record("s1"),
            make_assistant_record("s1", "claude-sonnet-4-6", 100, 50, ""),
        ]);
        let result = parse_jsonl_file(&path, 0);
        assert_eq!(result.session_metas.len(), 1);
        assert_eq!(result.turns.len(), 1);
        assert_eq!(result.turns[0].input_tokens, 100);
        assert_eq!(result.line_count, 2);
    }

    #[test]
    fn test_skips_zero_tokens() {
        let dir = TempDir::new().unwrap();
        let path = write_jsonl(&dir, "test.jsonl", &[
            make_assistant_record("s1", "claude-sonnet-4-6", 0, 0, ""),
        ]);
        let result = parse_jsonl_file(&path, 0);
        assert_eq!(result.turns.len(), 0);
    }

    #[test]
    fn test_streaming_dedup() {
        let dir = TempDir::new().unwrap();
        let path = write_jsonl(&dir, "test.jsonl", &[
            make_assistant_record("s1", "claude-sonnet-4-6", 50, 10, "msg-1"),
            make_assistant_record("s1", "claude-sonnet-4-6", 100, 50, "msg-1"),
            make_assistant_record("s1", "claude-sonnet-4-6", 150, 80, "msg-1"),
        ]);
        let result = parse_jsonl_file(&path, 0);
        assert_eq!(result.turns.len(), 1);
        assert_eq!(result.turns[0].input_tokens, 150); // last wins
    }

    #[test]
    fn test_different_message_ids_kept() {
        let dir = TempDir::new().unwrap();
        let path = write_jsonl(&dir, "test.jsonl", &[
            make_assistant_record("s1", "claude-sonnet-4-6", 100, 50, "msg-1"),
            make_assistant_record("s1", "claude-sonnet-4-6", 200, 100, "msg-2"),
        ]);
        let result = parse_jsonl_file(&path, 0);
        assert_eq!(result.turns.len(), 2);
    }

    #[test]
    fn test_skip_lines() {
        let dir = TempDir::new().unwrap();
        let path = write_jsonl(&dir, "test.jsonl", &[
            make_assistant_record("s1", "claude-sonnet-4-6", 100, 50, "msg-1"),
            make_assistant_record("s1", "claude-sonnet-4-6", 200, 100, "msg-2"),
        ]);
        let result = parse_jsonl_file(&path, 1);
        assert_eq!(result.turns.len(), 1);
        assert_eq!(result.turns[0].input_tokens, 200);
        assert_eq!(result.line_count, 2);
    }

    #[test]
    fn test_malformed_json_skipped() {
        let dir = TempDir::new().unwrap();
        let path = write_jsonl(&dir, "test.jsonl", &[
            "not valid json".into(),
            make_assistant_record("s1", "claude-sonnet-4-6", 100, 50, ""),
        ]);
        let result = parse_jsonl_file(&path, 0);
        assert_eq!(result.turns.len(), 1);
    }

    #[test]
    fn test_aggregate_sessions() {
        let metas = vec![SessionMeta {
            session_id: "s1".into(),
            project_name: "test".into(),
            ..Default::default()
        }];
        let turns = vec![
            Turn { session_id: "s1".into(), input_tokens: 100, output_tokens: 50, model: "claude-sonnet-4-6".into(), ..Default::default() },
            Turn { session_id: "s1".into(), input_tokens: 200, output_tokens: 100, model: "claude-sonnet-4-6".into(), ..Default::default() },
        ];
        let sessions = aggregate_sessions(&metas, &turns);
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].total_input_tokens, 300);
        assert_eq!(sessions[0].total_output_tokens, 150);
        assert_eq!(sessions[0].turn_count, 2);
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test scanner::parser`
Expected: All tests pass.

- [ ] **Step 3: Commit**

```bash
git add src/scanner/parser.rs
git commit -m "feat: add JSONL parser with streaming deduplication"
```

---

### Task 6: Scanner Orchestration (scanner/mod.rs)

**Files:**
- Modify: `src/scanner/mod.rs`

- [ ] **Step 1: Implement scan() in mod.rs**

```rust
// src/scanner/mod.rs
pub mod db;
pub mod parser;

use std::path::{Path, PathBuf};
use anyhow::Result;
use tracing::{debug, info, warn};
use walkdir::WalkDir;

use crate::models::ScanResult;
use db::{open_db, init_db, get_processed_file, upsert_processed_file,
         insert_turns, upsert_sessions, recompute_session_totals};
use parser::{parse_jsonl_file, aggregate_sessions};

fn default_projects_dirs() -> Vec<PathBuf> {
    let home = dirs_path();
    let mut dirs = vec![home.join(".claude").join("projects")];
    #[cfg(target_os = "macos")]
    dirs.push(home.join("Library/Developer/Xcode/CodingAssistant/ClaudeAgentConfig/projects"));
    dirs
}

fn dirs_path() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

pub fn default_db_path() -> PathBuf {
    dirs_path().join(".claude").join("usage.db")
}

pub fn scan(
    projects_dirs: Option<Vec<PathBuf>>,
    db_path: &Path,
    verbose: bool,
) -> Result<ScanResult> {
    let conn = open_db(db_path)?;
    init_db(&conn)?;

    let dirs = projects_dirs.unwrap_or_else(default_projects_dirs);

    let mut jsonl_files: Vec<PathBuf> = Vec::new();
    for d in &dirs {
        if !d.exists() {
            continue;
        }
        if verbose {
            info!("Scanning {} ...", d.display());
        }
        for entry in WalkDir::new(d).into_iter().filter_map(|e| e.ok()) {
            if entry.path().extension().is_some_and(|ext| ext == "jsonl") {
                jsonl_files.push(entry.path().to_path_buf());
            }
        }
    }
    jsonl_files.sort();

    let mut result = ScanResult::default();
    let mut any_changes = false;

    for filepath in &jsonl_files {
        let filepath_str = filepath.to_string_lossy().to_string();
        let mtime = match std::fs::metadata(filepath) {
            Ok(m) => m.modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs_f64())
                .unwrap_or(0.0),
            Err(_) => continue,
        };

        let prev = get_processed_file(&conn, &filepath_str)?;

        if let Some((prev_mtime, _)) = prev {
            if (prev_mtime - mtime).abs() < 0.01 {
                result.skipped += 1;
                continue;
            }
        }

        let is_new = prev.is_none();
        let skip_lines = if is_new { 0 } else { prev.unwrap().1 };

        debug!("[{}] {}", if is_new { "NEW" } else { "UPD" }, filepath_str);

        let parsed = parse_jsonl_file(filepath, skip_lines);

        // If file didn't grow, just update mtime
        if !is_new && parsed.line_count <= skip_lines {
            upsert_processed_file(&conn, &filepath_str, mtime, skip_lines)?;
            conn.execute_batch("COMMIT; BEGIN;")?; // no-op if autocommit
            result.skipped += 1;
            continue;
        }

        if !parsed.turns.is_empty() || !parsed.session_metas.is_empty() {
            let sessions = aggregate_sessions(&parsed.session_metas, &parsed.turns);
            upsert_sessions(&conn, &sessions)?;
            insert_turns(&conn, &parsed.turns)?;

            for s in &sessions {
                result.sessions += 1; // approximate; uses set in final count
            }
            result.turns += parsed.turns.len();
            any_changes = true;
        }

        if is_new {
            result.new += 1;
        } else {
            result.updated += 1;
        }

        upsert_processed_file(&conn, &filepath_str, mtime, parsed.line_count)?;
    }

    // Recompute session totals from turns for dedup correctness
    if any_changes {
        recompute_session_totals(&conn)?;
    }

    if verbose {
        info!(
            "Scan complete: {} new, {} updated, {} skipped, {} turns",
            result.new, result.updated, result.skipped, result.turns
        );
    }

    Ok(result)
}
```

- [ ] **Step 2: Add `dirs` dependency to Cargo.toml** (for home_dir)

Add to `[dependencies]`: `dirs = "6"`

- [ ] **Step 3: Run full tests**

Run: `cargo test`
Expected: All tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/scanner/mod.rs Cargo.toml
git commit -m "feat: add scan orchestration with incremental processing"
```

---

### Task 7: Dashboard UI Files

**Files:**
- Create: `src/ui/index.html`
- Create: `src/ui/style.css`
- Create: `src/ui/app.js`

These are static assets embedded at compile time. The HTML loads Chart.js from CDN. The JS receives pricing data injected by the server via a `__PRICING_JSON__` placeholder.

- [ ] **Step 1: Create src/ui/ directory and all three files**

The files are large (HTML ~100 lines, CSS ~180 lines, JS ~600 lines). Create them with the full dashboard implementation including:
- Dark theme matching the spec
- Chart.js charts (daily stacked bar, model doughnut, project horizontal bar)
- Sortable tables (sessions, cost by model, cost by project)
- Model filter checkboxes + date range buttons
- CSV export
- Rescan button
- Auto-refresh every 30s
- URL persistence for filters
- `__PRICING_JSON__` placeholder that gets replaced by server at runtime

- [ ] **Step 2: Verify files exist**

Run: `ls src/ui/`
Expected: `index.html style.css app.js`

- [ ] **Step 3: Commit**

```bash
git add src/ui/
git commit -m "feat: add dashboard UI assets"
```

---

### Task 8: HTTP Server (server/)

**Files:**
- Create: `src/server/mod.rs`
- Create: `src/server/api.rs`
- Create: `src/server/assets.rs`
- Modify: `src/main.rs` (add mod)

- [ ] **Step 1: Create server/assets.rs**

```rust
// src/server/assets.rs
use crate::pricing::pricing_table_json;

const INDEX_HTML: &str = include_str!("../ui/index.html");
const STYLE_CSS: &str = include_str!("../ui/style.css");
const APP_JS: &str = include_str!("../ui/app.js");

pub fn render_dashboard() -> String {
    let pricing_json = pricing_table_json();
    INDEX_HTML
        .replace("/* __STYLE_CSS__ */", STYLE_CSS)
        .replace("/* __APP_JS__ */", APP_JS)
        .replace("\"__PRICING_JSON__\"", &pricing_json)
}
```

- [ ] **Step 2: Create server/api.rs**

```rust
// src/server/api.rs
use std::path::PathBuf;
use std::sync::Arc;
use axum::{extract::State, http::StatusCode, response::Json};
use serde_json::Value;

use crate::models::{DashboardData, ScanResult};
use crate::scanner::db;
use crate::scanner;

pub struct AppState {
    pub db_path: PathBuf,
    pub projects_dirs: Option<Vec<PathBuf>>,
}

pub async fn api_data(State(state): State<Arc<AppState>>) -> Result<Json<Value>, StatusCode> {
    let db_path = state.db_path.clone();
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<DashboardData> {
        let conn = db::open_db(&db_path)?;
        db::init_db(&conn)?;
        db::get_dashboard_data(&conn)
    }).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::to_value(result).unwrap()))
}

pub async fn api_rescan(State(state): State<Arc<AppState>>) -> Result<Json<Value>, StatusCode> {
    let db_path = state.db_path.clone();
    let projects_dirs = state.projects_dirs.clone();

    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<ScanResult> {
        // Atomic rescan: write to temp, then rename
        let temp_path = db_path.with_extension("db.tmp");
        let scan_result = scanner::scan(projects_dirs, &temp_path, false)?;
        if temp_path.exists() {
            std::fs::rename(&temp_path, &db_path)?;
        }
        Ok(scan_result)
    }).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::to_value(result).unwrap()))
}

pub async fn api_health() -> &'static str {
    "ok"
}
```

- [ ] **Step 3: Create server/mod.rs**

```rust
// src/server/mod.rs
pub mod api;
pub mod assets;

use std::path::PathBuf;
use std::sync::Arc;
use axum::{Router, routing::{get, post}, response::Html};

use api::AppState;

pub async fn serve(host: String, port: u16, db_path: PathBuf, projects_dirs: Option<Vec<PathBuf>>) -> anyhow::Result<()> {
    let state = Arc::new(AppState { db_path, projects_dirs });
    let dashboard_html = assets::render_dashboard();

    let app = Router::new()
        .route("/", get({
            let html = dashboard_html.clone();
            move || async { Html(html) }
        }))
        .route("/index.html", get({
            let html = dashboard_html;
            move || async { Html(html) }
        }))
        .route("/api/data", get(api::api_data))
        .route("/api/rescan", post(api::api_rescan))
        .route("/api/health", get(api::api_health))
        .with_state(state);

    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Dashboard running at http://{}", addr);
    eprintln!("Dashboard running at http://{}", addr);
    eprintln!("Press Ctrl+C to stop.");
    axum::serve(listener, app).await?;
    Ok(())
}
```

- [ ] **Step 4: Add server mod to main.rs**

```rust
mod models;
mod pricing;
mod scanner;
mod server;

fn main() {
    println!("claude-usage-tracker");
}
```

- [ ] **Step 5: Verify it compiles**

Run: `cargo build`
Expected: Compiles (UI files must exist first from Task 7).

- [ ] **Step 6: Commit**

```bash
git add src/server/ src/main.rs
git commit -m "feat: add axum HTTP server with API endpoints"
```

---

### Task 9: CLI Entry Point (main.rs)

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Implement full CLI with clap**

```rust
// src/main.rs
mod models;
mod pricing;
mod scanner;
mod server;

use std::path::PathBuf;
use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "claude-usage-tracker", version, about = "Local analytics dashboard for Claude Code usage")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan JSONL files and update the database
    Scan {
        #[arg(long)]
        projects_dir: Option<PathBuf>,
        #[arg(long)]
        db_path: Option<PathBuf>,
    },
    /// Show today's usage summary
    Today {
        #[arg(long)]
        db_path: Option<PathBuf>,
    },
    /// Show all-time statistics
    Stats {
        #[arg(long)]
        db_path: Option<PathBuf>,
    },
    /// Scan + start web dashboard
    Dashboard {
        #[arg(long)]
        projects_dir: Option<PathBuf>,
        #[arg(long)]
        db_path: Option<PathBuf>,
        #[arg(long, default_value = "localhost")]
        host: String,
        #[arg(long, default_value = "8080")]
        port: u16,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into())
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Scan { projects_dir, db_path } => {
            let db = db_path.unwrap_or_else(scanner::default_db_path);
            let dirs = projects_dir.map(|d| vec![d]);
            scanner::scan(dirs, &db, true)?;
        }
        Commands::Today { db_path } => {
            let db = db_path.unwrap_or_else(scanner::default_db_path);
            cmd_today(&db)?;
        }
        Commands::Stats { db_path } => {
            let db = db_path.unwrap_or_else(scanner::default_db_path);
            cmd_stats(&db)?;
        }
        Commands::Dashboard { projects_dir, db_path, host, port } => {
            let db = db_path.unwrap_or_else(scanner::default_db_path);
            let dirs = projects_dir.map(|d| vec![d]);

            eprintln!("Running scan first...");
            scanner::scan(dirs.clone(), &db, true)?;

            let host_env = std::env::var("HOST").unwrap_or(host);
            let port_env = std::env::var("PORT").ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(port);

            let url = format!("http://{}:{}", host_env, port_env);
            let _ = open::that(&url);

            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(server::serve(host_env, port_env, db, dirs))?;
        }
    }
    Ok(())
}

fn cmd_today(db_path: &std::path::Path) -> Result<()> {
    use chrono::Local;

    if !db_path.exists() {
        anyhow::bail!("Database not found. Run: claude-usage-tracker scan");
    }
    let conn = scanner::db::open_db(db_path)?;
    let today = Local::now().format("%Y-%m-%d").to_string();

    let mut stmt = conn.prepare(
        "SELECT COALESCE(model, 'unknown') as model,
                SUM(input_tokens) as inp, SUM(output_tokens) as out,
                SUM(cache_read_tokens) as cr, SUM(cache_creation_tokens) as cc,
                COUNT(*) as turns
         FROM turns WHERE substr(timestamp, 1, 10) = ?1
         GROUP BY model ORDER BY inp + out DESC"
    )?;

    let rows: Vec<(String, i64, i64, i64, i64, i64)> = stmt.query_map([&today], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?))
    })?.filter_map(|r| r.ok()).collect();

    println!();
    println!("{}", "-".repeat(70));
    println!("  Today's Usage  ({})", today);
    println!("{}", "-".repeat(70));

    if rows.is_empty() {
        println!("  No usage recorded today.");
        println!();
        return Ok(());
    }

    let mut total_cost = 0.0;
    for (model, inp, out, cr, cc, turns) in &rows {
        let cost = pricing::calc_cost(model, *inp, *out, *cr, *cc);
        total_cost += cost;
        println!(
            "  {:<30}  turns={:<4}  in={:<8}  out={:<8}  cost={}",
            model, turns, pricing::fmt_tokens(*inp), pricing::fmt_tokens(*out), pricing::fmt_cost(cost)
        );
    }

    println!("{}", "-".repeat(70));
    println!("  Est. total cost: {}", pricing::fmt_cost(total_cost));
    println!();
    Ok(())
}

fn cmd_stats(db_path: &std::path::Path) -> Result<()> {
    if !db_path.exists() {
        anyhow::bail!("Database not found. Run: claude-usage-tracker scan");
    }
    let conn = scanner::db::open_db(db_path)?;

    let (sessions, first, last): (i64, Option<String>, Option<String>) = conn.query_row(
        "SELECT COUNT(*), MIN(first_timestamp), MAX(last_timestamp) FROM sessions",
        [], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    )?;

    let (inp, out, cr, cc, turns): (i64, i64, i64, i64, i64) = conn.query_row(
        "SELECT COALESCE(SUM(input_tokens),0), COALESCE(SUM(output_tokens),0),
                COALESCE(SUM(cache_read_tokens),0), COALESCE(SUM(cache_creation_tokens),0),
                COUNT(*) FROM turns",
        [], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
    )?;

    let mut stmt = conn.prepare(
        "SELECT COALESCE(model,'unknown'), SUM(input_tokens), SUM(output_tokens),
                SUM(cache_read_tokens), SUM(cache_creation_tokens), COUNT(*),
                COUNT(DISTINCT session_id)
         FROM turns GROUP BY model ORDER BY SUM(input_tokens+output_tokens) DESC"
    )?;
    let by_model: Vec<(String, i64, i64, i64, i64, i64, i64)> = stmt.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?))
    })?.filter_map(|r| r.ok()).collect();

    let total_cost: f64 = by_model.iter()
        .map(|(m, i, o, cr, cc, _, _)| pricing::calc_cost(m, *i, *o, *cr, *cc))
        .sum();

    println!();
    println!("{}", "=".repeat(70));
    println!("  Claude Code Usage - All-Time Statistics");
    println!("{}", "=".repeat(70));
    let f = |s: &Option<String>| s.as_deref().unwrap_or("").chars().take(10).collect::<String>();
    println!("  Period:           {} to {}", f(&first), f(&last));
    println!("  Total sessions:   {}", sessions);
    println!("  Total turns:      {}", pricing::fmt_tokens(turns));
    println!();
    println!("  Input tokens:     {:<12}  (raw prompt tokens)", pricing::fmt_tokens(inp));
    println!("  Output tokens:    {:<12}  (generated tokens)", pricing::fmt_tokens(out));
    println!("  Cache read:       {:<12}  (90% cheaper than input)", pricing::fmt_tokens(cr));
    println!("  Cache creation:   {:<12}  (25% premium on input)", pricing::fmt_tokens(cc));
    println!();
    println!("  Est. total cost:  {}", pricing::fmt_cost(total_cost));
    println!("{}", "-".repeat(70));

    println!("  By Model:");
    for (model, mi, mo, mcr, mcc, mt, ms) in &by_model {
        let cost = pricing::calc_cost(model, *mi, *mo, *mcr, *mcc);
        println!(
            "    {:<30}  sessions={:<4}  turns={:<6}  in={:<8}  out={:<8}  cost={}",
            model, ms, pricing::fmt_tokens(*mt), pricing::fmt_tokens(*mi),
            pricing::fmt_tokens(*mo), pricing::fmt_cost(cost)
        );
    }
    println!("{}", "=".repeat(70));
    println!();
    Ok(())
}
```

- [ ] **Step 2: Verify it compiles and runs**

Run: `cargo build && cargo run – --help`
Expected: Shows CLI help text with all subcommands.

- [ ] **Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat: add CLI entry point with all commands"
```

---

### Task 10: Build, Test, Verify

- [ ] **Step 1: Run all tests**

Run: `cargo test`
Expected: All tests pass.

- [ ] **Step 2: Run clippy**

Run: `cargo clippy – -D warnings`
Expected: No warnings.

- [ ] **Step 3: Run fmt check**

Run: `cargo fmt --check`
Expected: No formatting issues.

- [ ] **Step 4: Test with real data (if available)**

Run: `cargo run – scan`
Run: `cargo run – today`
Run: `cargo run – stats`
Expected: Scans ~/.claude/projects/ and shows results.

- [ ] **Step 5: Final commit**

```bash
git add -A
git commit -m "chore: final cleanup and formatting"
```
