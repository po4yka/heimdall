/// Real-time PreToolUse hook ingest module (Phase 19).
///
/// The `heimdall-hook` binary is a thin wrapper around `main_impl()` defined
/// here. All logic lives in the library crate so it is directly testable.
pub mod bypass;
pub mod ingest;
pub mod install;

use std::io::Read;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use anyhow::Result;
use chrono::Utc;
use tracing::warn;

use crate::config::load_config_resolved;
use crate::scanner::db::{init_db, open_db};

/// Entry point for the `heimdall-hook` binary, extracted for testability.
///
/// Contract:
/// - Always prints `{}` on stdout (Claude Code ignores non-empty output only
///   when the hook returns non-zero; we always return 0).
/// - Never exits non-zero — doing so would surface an error to the user.
/// - Returns within ~50 ms for normal operation; stdin read is guarded by a
///   1-second timeout.
pub fn main_impl() {
    // 1. Bypass check: if any ancestor has --dangerously-skip-permissions, skip.
    if bypass::is_bypass_active() {
        print!("{{}}");
        return;
    }

    // 2. Read stdin with a 1-second timeout via a background thread + channel.
    let json = match read_stdin_with_timeout(Duration::from_secs(1)) {
        Some(s) => s,
        None => {
            // Timeout or empty — output {} and exit cleanly.
            print!("{{}}");
            return;
        }
    };

    if json.trim().is_empty() {
        print!("{{}}");
        return;
    }

    // 3. Run the ingest logic; swallow any error.
    if let Err(e) = ingest_event(&json) {
        warn!("heimdall-hook ingest error: {}", e);
    }

    // 6. Always output {} regardless of success or failure.
    print!("{{}}");
}

/// Parse the hook payload and write a `live_events` row to SQLite.
fn ingest_event(json: &str) -> Result<()> {
    let received_at = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true);

    // 3. Parse JSON — returns None on invalid JSON (not an error we surface).
    let event = match ingest::parse_hook_payload(json, &received_at) {
        Some(e) => e,
        None => {
            warn!("heimdall-hook: failed to parse payload");
            return Ok(());
        }
    };

    // 4. Resolve DB path from config.
    let db_path = resolve_db_path();

    // 5. Open DB (create if missing), run migrations, INSERT OR IGNORE.
    let conn = open_db(&db_path)?;
    init_db(&conn)?;

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

/// Resolve the SQLite database path via config, falling back to the default.
fn resolve_db_path() -> PathBuf {
    let cfg = load_config_resolved();
    cfg.db_path.unwrap_or_else(crate::scanner::default_db_path)
}

/// Read all of stdin with a hard timeout.
/// Returns `None` on timeout or read error; `Some(String)` on success.
fn read_stdin_with_timeout(timeout: Duration) -> Option<String> {
    let (tx, rx) = mpsc::channel::<String>();

    std::thread::spawn(move || {
        let mut buf = String::new();
        let mut stdin = std::io::stdin().lock();
        let _ = stdin.read_to_string(&mut buf);
        let _ = tx.send(buf);
    });

    match rx.recv_timeout(timeout) {
        Ok(s) if !s.is_empty() => Some(s),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Write a live_events row directly and verify it can be read back.
    #[test]
    fn live_events_migration_is_idempotent() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");

        // Run init_db twice — should not fail.
        {
            let conn = open_db(&db_path).unwrap();
            init_db(&conn).unwrap();
        }
        {
            let conn = open_db(&db_path).unwrap();
            init_db(&conn).unwrap();
        }

        // Insert a row and read it back.
        let conn = open_db(&db_path).unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO live_events
                (dedup_key, received_at, session_id, tool_name,
                 cost_usd_nanos, input_tokens, output_tokens, raw_json)
             VALUES ('k1', '2024-01-01T00:00:00Z', 'ses1', 'Edit',
                     1000000, 100, 50, '{}')",
            [],
        )
        .unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM live_events", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn ingest_event_writes_row_to_db() {
        let _guard = crate::config::HEIMDALL_CONFIG_MUTEX
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("hook_test.db");

        // Ensure DB and table exist.
        {
            let conn = open_db(&db_path).unwrap();
            init_db(&conn).unwrap();
        }

        // Temporarily redirect the DB path via a minimal config file.
        // We call ingest_event directly after setting up the DB path via
        // a temp config file pointed at by HEIMDALL_CONFIG.
        let cfg_path = dir.path().join("config.toml");
        std::fs::write(
            &cfg_path,
            format!("db_path = {:?}\n", db_path.to_string_lossy().as_ref()),
        )
        .unwrap();
        // SAFETY: single-threaded test; env mutation is safe here.
        unsafe { std::env::set_var("HEIMDALL_CONFIG", &cfg_path) };

        let json = r#"{
            "session_id": "test-session",
            "tool_name": "Read",
            "tool_use_id": "tu_abc",
            "hook_input": {
                "cost": { "total_cost_usd": 0.0005 },
                "usage": { "input_tokens": 200, "output_tokens": 50 }
            }
        }"#;

        ingest_event(json).unwrap();

        // Clean up env var.
        unsafe { std::env::remove_var("HEIMDALL_CONFIG") };

        let conn = open_db(&db_path).unwrap();
        let (session_id, tool_name, cost_nanos, hook_reported): (String, String, i64, Option<i64>) =
            conn.query_row(
                "SELECT session_id, tool_name, cost_usd_nanos, hook_reported_cost_nanos FROM live_events LIMIT 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .unwrap();

        assert_eq!(session_id, "test-session");
        assert_eq!(tool_name, "Read");
        assert_eq!(cost_nanos, 500_000); // 0.0005 USD * 1e9
        assert_eq!(hook_reported, Some(500_000)); // hook_reported_cost_nanos = 500_000 nanos
    }

    /// Phase 5: context_window fields in hook payload are persisted to DB.
    #[test]
    fn ingest_event_persists_context_window_columns() {
        let _guard = crate::config::HEIMDALL_CONFIG_MUTEX
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("ctx_test.db");

        {
            let conn = open_db(&db_path).unwrap();
            init_db(&conn).unwrap();
        }

        let cfg_path = dir.path().join("config.toml");
        std::fs::write(
            &cfg_path,
            format!("db_path = {:?}\n", db_path.to_string_lossy().as_ref()),
        )
        .unwrap();
        unsafe { std::env::set_var("HEIMDALL_CONFIG", &cfg_path) };

        let json = r#"{
            "session_id": "ctx-session",
            "tool_name": "Read",
            "tool_use_id": "tu_ctx1",
            "hook_input": {
                "cost": { "total_cost_usd": 0.001 },
                "usage": { "input_tokens": 200, "output_tokens": 50 },
                "context_window": {
                    "total_input_tokens": 45231,
                    "context_window_size": 200000
                }
            }
        }"#;

        ingest_event(json).unwrap();

        unsafe { std::env::remove_var("HEIMDALL_CONFIG") };

        let conn = open_db(&db_path).unwrap();
        let (ctx_tokens, ctx_size): (Option<i64>, Option<i64>) = conn
            .query_row(
                "SELECT context_input_tokens, context_window_size FROM live_events LIMIT 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();

        assert_eq!(ctx_tokens, Some(45231));
        assert_eq!(ctx_size, Some(200_000));
    }

    /// Phase 8: cost-bearing payload persists hook_reported_cost_nanos.
    #[test]
    fn ingest_event_persists_hook_reported_cost_nanos() {
        let _guard = crate::config::HEIMDALL_CONFIG_MUTEX
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("hook_cost_test.db");

        {
            let conn = open_db(&db_path).unwrap();
            init_db(&conn).unwrap();
        }

        let cfg_path = dir.path().join("config.toml");
        std::fs::write(
            &cfg_path,
            format!("db_path = {:?}\n", db_path.to_string_lossy().as_ref()),
        )
        .unwrap();
        unsafe { std::env::set_var("HEIMDALL_CONFIG", &cfg_path) };

        // Payload with cost object shape
        let json = r#"{
            "session_id": "cost-session",
            "tool_name": "Bash",
            "tool_use_id": "tu_cost1",
            "hook_input": {
                "cost": { "total_cost_usd": 0.14 },
                "usage": { "input_tokens": 100, "output_tokens": 50 }
            }
        }"#;

        ingest_event(json).unwrap();

        unsafe { std::env::remove_var("HEIMDALL_CONFIG") };

        let conn = open_db(&db_path).unwrap();
        let hook_nanos: Option<i64> = conn
            .query_row(
                "SELECT hook_reported_cost_nanos FROM live_events LIMIT 1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(hook_nanos, Some(140_000_000)); // 0.14 USD * 1e9
    }

    /// Phase 8: cost-less payload leaves hook_reported_cost_nanos NULL.
    #[test]
    fn ingest_event_cost_absent_leaves_hook_column_null() {
        let _guard = crate::config::HEIMDALL_CONFIG_MUTEX
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("hook_null_test.db");

        {
            let conn = open_db(&db_path).unwrap();
            init_db(&conn).unwrap();
        }

        let cfg_path = dir.path().join("config.toml");
        std::fs::write(
            &cfg_path,
            format!("db_path = {:?}\n", db_path.to_string_lossy().as_ref()),
        )
        .unwrap();
        unsafe { std::env::set_var("HEIMDALL_CONFIG", &cfg_path) };

        // Payload with NO cost field
        let json = r#"{
            "session_id": "no-cost-session",
            "tool_name": "Read",
            "tool_use_id": "tu_nocost1",
            "hook_input": {
                "usage": { "input_tokens": 100, "output_tokens": 50 }
            }
        }"#;

        ingest_event(json).unwrap();

        unsafe { std::env::remove_var("HEIMDALL_CONFIG") };

        let conn = open_db(&db_path).unwrap();
        let hook_nanos: Option<i64> = conn
            .query_row(
                "SELECT hook_reported_cost_nanos FROM live_events LIMIT 1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(hook_nanos, None);
    }

    #[test]
    fn ingest_event_is_idempotent_on_duplicate_dedup_key() {
        let _guard = crate::config::HEIMDALL_CONFIG_MUTEX
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("dedup_test.db");

        let cfg_path = dir.path().join("config.toml");
        std::fs::write(
            &cfg_path,
            format!("db_path = {:?}\n", db_path.to_string_lossy().as_ref()),
        )
        .unwrap();
        // SAFETY: single-threaded test; env mutation is safe here.
        unsafe { std::env::set_var("HEIMDALL_CONFIG", &cfg_path) };

        let json = r#"{
            "session_id": "s1",
            "tool_name": "Bash",
            "tool_use_id": "tu_same",
            "hook_input": { "cost": { "total_cost_usd": 0.001 } }
        }"#;

        ingest_event(json).unwrap();
        // Second call with same tool_use_id must not fail (INSERT OR IGNORE).
        ingest_event(json).unwrap();

        unsafe { std::env::remove_var("HEIMDALL_CONFIG") };

        let conn = open_db(&db_path).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM live_events", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1, "duplicate dedup_key must be silently ignored");
    }
}
