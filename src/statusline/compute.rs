/// Compute layer: pull session/today/block totals from the DB.
use std::path::Path;

use anyhow::Result;
use chrono::{DateTime, Local, Utc};

use crate::analytics::blocks::{BurnRate, calculate_burn_rate, identify_blocks_with_now};
use crate::scanner::db::{load_turns_since, open_db};
use crate::statusline::context_window;
use crate::statusline::input::HookInput;

// ── Cost source ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CostSource {
    /// Prefer hook-supplied cost; fall back to local DB.
    Auto,
    /// Always compute from local DB.
    Local,
    /// Always use the hook-supplied cost.
    Hook,
    /// Phase 8 placeholder — treated as Auto for now.
    Both,
}

impl CostSource {
    pub fn parse(s: &str) -> std::result::Result<Self, String> {
        match s.to_ascii_lowercase().as_str() {
            "auto" => Ok(Self::Auto),
            "local" => Ok(Self::Local),
            "hook" => Ok(Self::Hook),
            "both" => Ok(Self::Both),
            other => Err(format!(
                "unknown cost-source '{other}': expected auto|local|hook|both"
            )),
        }
    }
}

// ── Output ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ComputedStats {
    pub model: String,
    pub session_cost_nanos: i64,
    pub today_cost_nanos: i64,
    pub active_block: Option<ActiveBlockInfo>,
    pub context_tokens: Option<i64>,
    pub context_size: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct ActiveBlockInfo {
    pub cost_nanos: i64,
    pub block_end: DateTime<Utc>,
    pub burn_rate: Option<BurnRate>,
}

// ── Main entry ────────────────────────────────────────────────────────────────

pub fn compute(
    db_path: &Path,
    input: &HookInput,
    cost_source: CostSource,
) -> Result<ComputedStats> {
    // Try to open the DB; if it doesn't exist yet, return zeroed stats.
    let conn = match open_db(db_path) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("statusline: cannot open DB {}: {}", db_path.display(), e);
            return Ok(zeroed_stats(input, cost_source));
        }
    };

    let now = Utc::now();

    // ── Session cost ──────────────────────────────────────────────────────────
    let session_cost_nanos = query_session_cost(&conn, &input.session_id, cost_source, input)?;

    // ── Today cost ────────────────────────────────────────────────────────────
    let today_cost_nanos = query_today_cost(&conn, now)?;

    // ── Active block ──────────────────────────────────────────────────────────
    let active_block = query_active_block(&conn, now)?;

    // ── Context window ────────────────────────────────────────────────────────
    let cw = context_window::resolve(input);
    let context_tokens = cw.map(|c| c.total_input_tokens);
    let context_size = cw.map(|c| c.context_window_size);

    Ok(ComputedStats {
        model: input.model.clone().unwrap_or_else(|| "unknown".to_string()),
        session_cost_nanos,
        today_cost_nanos,
        active_block,
        context_tokens,
        context_size,
    })
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn zeroed_stats(input: &HookInput, cost_source: CostSource) -> ComputedStats {
    let hook_cost = input
        .cost
        .as_ref()
        .map(|c| (c.total_cost_usd * 1_000_000_000.0) as i64)
        .unwrap_or(0);
    let session_cost_nanos = match cost_source {
        CostSource::Hook | CostSource::Auto | CostSource::Both => hook_cost,
        CostSource::Local => 0,
    };
    let cw = context_window::resolve(input);
    let context_tokens = cw.map(|c| c.total_input_tokens);
    let context_size = cw.map(|c| c.context_window_size);
    ComputedStats {
        model: input.model.clone().unwrap_or_else(|| "unknown".to_string()),
        session_cost_nanos,
        today_cost_nanos: 0,
        active_block: None,
        context_tokens,
        context_size,
    }
}

fn query_session_cost(
    conn: &rusqlite::Connection,
    raw_session_id: &str,
    cost_source: CostSource,
    input: &HookInput,
) -> Result<i64> {
    // If hook cost is preferred and available, use it directly.
    let hook_nanos = input
        .cost
        .as_ref()
        .map(|c| (c.total_cost_usd * 1_000_000_000.0) as i64);

    match cost_source {
        CostSource::Hook => return Ok(hook_nanos.unwrap_or(0)),
        CostSource::Auto | CostSource::Both => {
            if let Some(n) = hook_nanos {
                return Ok(n);
            }
        }
        CostSource::Local => {}
    }

    // Fall back to DB: try raw id, then "claude:<id>".
    let db_nanos = query_session_cost_from_db(conn, raw_session_id)?;
    Ok(db_nanos)
}

fn query_session_cost_from_db(conn: &rusqlite::Connection, raw_session_id: &str) -> Result<i64> {
    // Try the raw session_id first.
    let raw_cost: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(estimated_cost_nanos), 0) FROM turns WHERE session_id = ?1",
            rusqlite::params![raw_session_id],
            |r| r.get(0),
        )
        .unwrap_or(0);

    if raw_cost > 0 {
        return Ok(raw_cost);
    }

    // Try prefixed form "claude:<id>".
    let prefixed = format!("claude:{}", raw_session_id);
    let prefixed_cost: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(estimated_cost_nanos), 0) FROM turns WHERE session_id = ?1",
            rusqlite::params![prefixed],
            |r| r.get(0),
        )
        .unwrap_or(0);

    Ok(prefixed_cost)
}

fn query_today_cost(conn: &rusqlite::Connection, now: DateTime<Utc>) -> Result<i64> {
    // Derive UTC bounds for the user's local "today".
    // Comparing UTC-stored timestamps against a local date string misses the
    // first hours of the day in non-UTC timezones; using a UTC range is exact.
    let local_today = now.with_timezone(&Local).date_naive();
    let start_local = local_today
        .and_hms_opt(0, 0, 0)
        .and_then(|ndt| ndt.and_local_timezone(Local).single())
        .unwrap_or_else(|| now.with_timezone(&Local));
    let end_local = start_local + chrono::Duration::days(1);
    let start_utc = start_local.with_timezone(&Utc).to_rfc3339();
    let end_utc = end_local.with_timezone(&Utc).to_rfc3339();

    let cost: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(estimated_cost_nanos), 0) FROM turns \
             WHERE timestamp >= ?1 AND timestamp < ?2",
            rusqlite::params![start_utc, end_utc],
            |r| r.get(0),
        )
        .unwrap_or(0);

    Ok(cost)
}

fn query_active_block(
    conn: &rusqlite::Connection,
    now: DateTime<Utc>,
) -> Result<Option<ActiveBlockInfo>> {
    // Only load turns from the last 24 h — avoids a full-table scan on large DBs.
    let cutoff = now - chrono::Duration::hours(24);
    let cutoff_iso = cutoff.to_rfc3339();

    let turns = match load_turns_since(conn, &cutoff_iso) {
        Ok(t) => t,
        Err(e) => {
            // On first-run machines the `turns` table may not exist yet.
            if is_missing_table_error(&e) {
                return Ok(None);
            }
            return Err(e);
        }
    };

    if turns.is_empty() {
        return Ok(None);
    }

    let blocks = identify_blocks_with_now(&turns, 5.0, now);
    let active = blocks.into_iter().find(|b| b.is_active);

    Ok(active.map(|b| {
        let burn = calculate_burn_rate(&b, now);
        ActiveBlockInfo {
            cost_nanos: b.cost_nanos,
            block_end: b.end,
            burn_rate: burn,
        }
    }))
}

/// Returns `true` when `e` indicates the queried table does not exist yet.
/// This happens on first-run machines where `heimdall scan` has not been
/// executed (so the DB schema is absent or incomplete).
fn is_missing_table_error(e: &anyhow::Error) -> bool {
    let msg = e.to_string();
    msg.contains("no such table") || msg.contains("no such column")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::db::{init_db, open_db};
    use crate::statusline::input::{ContextWindow, HookCost, HookInput};
    use rusqlite::Connection;
    use tempfile::TempDir;

    fn make_input(session_id: &str, cost: Option<f64>) -> HookInput {
        HookInput {
            session_id: session_id.to_string(),
            transcript_path: "/tmp/t.jsonl".to_string(),
            model: Some("claude-sonnet-4-6".to_string()),
            cost: cost.map(|c| HookCost {
                total_cost_usd: c,
                total_duration_ms: None,
                total_api_duration_ms: None,
            }),
            context_window: Some(ContextWindow {
                total_input_tokens: Some(45231),
                context_window_size: Some(200000),
            }),
        }
    }

    fn seed_turn(conn: &Connection, session_id: &str, cost_nanos: i64, ts: &str) {
        conn.execute(
            "INSERT INTO turns (session_id, provider, timestamp, model, estimated_cost_nanos)
             VALUES (?1, 'claude', ?2, 'claude-sonnet-4-6', ?3)",
            rusqlite::params![session_id, ts, cost_nanos],
        )
        .unwrap();
    }

    #[test]
    fn session_absent_returns_hook_cost() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = open_db(&db_path).unwrap();
        init_db(&conn).unwrap();
        drop(conn);

        let input = make_input("no-such-session", Some(0.05));
        let stats = compute(&db_path, &input, CostSource::Auto).unwrap();
        // Hook cost preferred in Auto mode.
        assert_eq!(stats.session_cost_nanos, 50_000_000); // 0.05 USD * 1e9
    }

    #[test]
    fn session_present_in_db_raw_id() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = open_db(&db_path).unwrap();
        init_db(&conn).unwrap();
        seed_turn(&conn, "ses1", 120_000_000, "2026-01-01T10:00:00Z");
        drop(conn);

        // No hook cost — falls through to DB.
        let input = HookInput {
            session_id: "ses1".to_string(),
            transcript_path: "/tmp/t.jsonl".to_string(),
            model: Some("claude-sonnet-4-6".to_string()),
            cost: None,
            context_window: None,
        };
        let stats = compute(&db_path, &input, CostSource::Local).unwrap();
        assert_eq!(stats.session_cost_nanos, 120_000_000);
    }

    #[test]
    fn today_cost_sums_todays_turns() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = open_db(&db_path).unwrap();
        init_db(&conn).unwrap();

        // Use today's timestamp in proper UTC RFC3339 so FIX-6 range query matches.
        let today = Utc::now().to_rfc3339();
        seed_turn(&conn, "ses1", 100_000_000, &today);
        seed_turn(&conn, "ses1", 200_000_000, &today);
        // Old turn — should not count.
        seed_turn(&conn, "ses1", 999_000_000, "2020-01-01T00:00:00Z");
        drop(conn);

        let input = HookInput {
            session_id: "ses1".to_string(),
            transcript_path: "/tmp/t.jsonl".to_string(),
            model: None,
            cost: None,
            context_window: None,
        };
        let stats = compute(&db_path, &input, CostSource::Local).unwrap();
        assert_eq!(stats.today_cost_nanos, 300_000_000);
    }

    #[test]
    fn active_block_none_when_no_recent_turns() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = open_db(&db_path).unwrap();
        init_db(&conn).unwrap();
        // Very old turn — outside 24 h window.
        seed_turn(&conn, "s1", 100_000_000, "2020-01-01T00:00:00Z");
        drop(conn);

        let input = HookInput {
            session_id: "s1".to_string(),
            transcript_path: "/tmp/t.jsonl".to_string(),
            model: None,
            cost: None,
            context_window: None,
        };
        let stats = compute(&db_path, &input, CostSource::Local).unwrap();
        assert!(stats.active_block.is_none());
    }

    #[test]
    fn cost_source_local_ignores_hook_cost() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = open_db(&db_path).unwrap();
        init_db(&conn).unwrap();
        seed_turn(&conn, "s1", 50_000_000, "2026-01-01T10:00:00Z");
        drop(conn);

        let input = make_input("s1", Some(9999.0)); // hook says $9999
        let stats = compute(&db_path, &input, CostSource::Local).unwrap();
        // Local mode: uses DB, not hook.
        assert_eq!(stats.session_cost_nanos, 50_000_000);
    }

    #[test]
    fn parse_cost_source_accepts_all_variants() {
        assert_eq!(CostSource::parse("auto").unwrap(), CostSource::Auto);
        assert_eq!(CostSource::parse("AUTO").unwrap(), CostSource::Auto);
        assert_eq!(CostSource::parse("local").unwrap(), CostSource::Local);
        assert_eq!(CostSource::parse("hook").unwrap(), CostSource::Hook);
        assert_eq!(CostSource::parse("both").unwrap(), CostSource::Both);
    }

    #[test]
    fn parse_cost_source_rejects_bogus() {
        assert!(CostSource::parse("bogus").is_err());
        assert!(CostSource::parse("").is_err());
    }
}
