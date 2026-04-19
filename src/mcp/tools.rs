/// MCP tool definitions and handlers for the Heimdall MCP server.
///
/// Each tool is implemented as an async method on `HeimdallMcpServer` and
/// registered via the rmcp `#[tool_router]` / `#[tool]` macros.
use std::path::{Path, PathBuf};

use anyhow::Result;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::schemars::JsonSchema;
use rmcp::{tool, tool_router};
use serde::Deserialize;

use crate::analytics::blocks::{calculate_burn_rate, identify_blocks, project_block_usage};
use crate::analytics::quota::compute_quota;
use crate::optimizer;
use crate::scanner::db as sdb;

// ── Shared state ──────────────────────────────────────────────────────────────

pub struct HeimdallMcpServer {
    pub db_path: PathBuf,
}

// ── Input types ───────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, JsonSchema)]
pub struct EmptyInput {}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WeeklyInput {
    /// Which day starts the week (monday | sunday | tuesday | ...).
    pub start_of_week: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SessionsInput {
    /// Max number of sessions to return (default: 50).
    pub limit: Option<u32>,
    /// Offset for pagination (default: 0).
    pub offset: Option<u32>,
    /// Filter to a specific project name substring.
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct BlocksInput {
    /// Billing block duration in hours (default: 5.0).
    pub session_length_hours: Option<f64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CostReconciliationInput {
    /// Rolling period: day | week | month (default: month).
    pub period: Option<String>,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn json_ok(v: serde_json::Value) -> String {
    serde_json::to_string_pretty(&v).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}"))
}

fn json_err(msg: impl std::fmt::Display) -> String {
    format!("{{\"error\":\"{msg}\"}}")
}

fn open_conn(db_path: &Path) -> Result<rusqlite::Connection> {
    sdb::open_db(db_path)
}

fn parse_weekday(s: &str) -> chrono::Weekday {
    match s.to_ascii_lowercase().as_str() {
        "monday" | "mon" => chrono::Weekday::Mon,
        "tuesday" | "tue" => chrono::Weekday::Tue,
        "wednesday" | "wed" => chrono::Weekday::Wed,
        "thursday" | "thu" => chrono::Weekday::Thu,
        "friday" | "fri" => chrono::Weekday::Fri,
        "saturday" | "sat" => chrono::Weekday::Sat,
        "sunday" | "sun" => chrono::Weekday::Sun,
        other => {
            tracing::warn!("heimdall_weekly: unknown weekday '{other}', defaulting to Monday");
            chrono::Weekday::Mon
        }
    }
}

fn weekday_to_u8(day: chrono::Weekday) -> u8 {
    match day {
        chrono::Weekday::Sun => 0,
        chrono::Weekday::Mon => 1,
        chrono::Weekday::Tue => 2,
        chrono::Weekday::Wed => 3,
        chrono::Weekday::Thu => 4,
        chrono::Weekday::Fri => 5,
        chrono::Weekday::Sat => 6,
    }
}

// ── Tool implementations ──────────────────────────────────────────────────────

#[tool_router(server_handler)]
impl HeimdallMcpServer {
    /// Return today's token and cost usage summary grouped by model and provider.
    #[tool(
        description = "Return today's token and cost usage summary grouped by model and provider"
    )]
    async fn heimdall_today(&self, _params: Parameters<EmptyInput>) -> String {
        let db = self.db_path.clone();
        match tokio::task::spawn_blocking(move || query_today(&db)).await {
            Ok(Ok(v)) => json_ok(v),
            Ok(Err(e)) => json_err(e),
            Err(e) => json_err(e),
        }
    }

    /// Return all-time aggregate usage statistics (tokens, cost, sessions, providers).
    #[tool(
        description = "Return all-time aggregate usage statistics (tokens, cost, sessions, providers)"
    )]
    async fn heimdall_stats(&self, _params: Parameters<EmptyInput>) -> String {
        let db = self.db_path.clone();
        match tokio::task::spawn_blocking(move || query_stats(&db)).await {
            Ok(Ok(v)) => json_ok(v),
            Ok(Err(e)) => json_err(e),
            Err(e) => json_err(e),
        }
    }

    /// Return weekly usage report grouped by ISO calendar week.
    #[tool(description = "Return weekly usage report grouped by ISO calendar week")]
    async fn heimdall_weekly(&self, params: Parameters<WeeklyInput>) -> String {
        let db = self.db_path.clone();
        let sow = params
            .0
            .start_of_week
            .clone()
            .unwrap_or_else(|| "monday".into());
        match tokio::task::spawn_blocking(move || query_weekly(&db, &sow)).await {
            Ok(Ok(v)) => json_ok(v),
            Ok(Err(e)) => json_err(e),
            Err(e) => json_err(e),
        }
    }

    /// Return a paginated list of recorded sessions with token and cost totals.
    #[tool(description = "Return a paginated list of recorded sessions with token and cost totals")]
    async fn heimdall_sessions(&self, params: Parameters<SessionsInput>) -> String {
        let db = self.db_path.clone();
        let limit = params.0.limit.unwrap_or(50).min(500);
        let offset = params.0.offset.unwrap_or(0);
        let project = params.0.project.clone();
        match tokio::task::spawn_blocking(move || {
            query_sessions(&db, limit, offset, project.as_deref())
        })
        .await
        {
            Ok(Ok(v)) => json_ok(v),
            Ok(Err(e)) => json_err(e),
            Err(e) => json_err(e),
        }
    }

    /// Return the currently active Claude billing block with burn rate and cost projection.
    #[tool(
        description = "Return the currently active Claude billing block with burn rate and cost projection"
    )]
    async fn heimdall_blocks_active(&self, params: Parameters<BlocksInput>) -> String {
        let db = self.db_path.clone();
        let hours = params.0.session_length_hours.unwrap_or(5.0);
        match tokio::task::spawn_blocking(move || query_active_block(&db, hours)).await {
            Ok(Ok(v)) => json_ok(v),
            Ok(Err(e)) => json_err(e),
            Err(e) => json_err(e),
        }
    }

    /// Run waste detectors and return an A-F grade plus findings with estimated monthly waste.
    #[tool(
        description = "Run waste detectors and return an A-F grade plus findings with estimated monthly waste"
    )]
    async fn heimdall_optimize_grade(&self, _params: Parameters<EmptyInput>) -> String {
        let db = self.db_path.clone();
        match tokio::task::spawn_blocking(move || optimizer::run_optimize(&db)).await {
            Ok(Ok(report)) => {
                let waste_usd = report.total_monthly_waste_nanos as f64 / 1_000_000_000.0;
                json_ok(serde_json::json!({
                    "grade": report.grade.to_string(),
                    "total_estimated_waste_usd": (waste_usd * 10000.0).round() / 10000.0,
                    "findings": report.findings,
                }))
            }
            Ok(Err(e)) => json_err(e),
            Err(e) => json_err(e),
        }
    }

    /// Return the latest OAuth usage-limits rate window snapshot.
    #[tool(description = "Return the latest OAuth usage-limits rate window snapshot")]
    async fn heimdall_rate_windows(&self, _params: Parameters<EmptyInput>) -> String {
        let db = self.db_path.clone();
        match tokio::task::spawn_blocking(move || query_rate_windows(&db)).await {
            Ok(Ok(v)) => json_ok(v),
            Ok(Err(e)) => json_err(e),
            Err(e) => json_err(e),
        }
    }

    /// Return the most recent context-window utilization from live PreToolUse events.
    #[tool(
        description = "Return the most recent context-window utilization from live PreToolUse events"
    )]
    async fn heimdall_context_window(&self, _params: Parameters<EmptyInput>) -> String {
        let db = self.db_path.clone();
        match tokio::task::spawn_blocking(move || query_context_window(&db)).await {
            Ok(Ok(v)) => json_ok(v),
            Ok(Err(e)) => json_err(e),
            Err(e) => json_err(e),
        }
    }

    /// Return quota status for the active billing block if a token limit is configured.
    #[tool(
        description = "Return quota status for the active billing block if a token limit is configured"
    )]
    async fn heimdall_quota(&self, _params: Parameters<EmptyInput>) -> String {
        let db = self.db_path.clone();
        match tokio::task::spawn_blocking(move || query_quota(&db)).await {
            Ok(Ok(v)) => json_ok(v),
            Ok(Err(e)) => json_err(e),
            Err(e) => json_err(e),
        }
    }

    /// Return hook-reported vs. local-estimate cost reconciliation for a rolling period.
    #[tool(
        description = "Return hook-reported vs. local-estimate cost reconciliation (day|week|month)"
    )]
    async fn heimdall_cost_reconciliation(
        &self,
        params: Parameters<CostReconciliationInput>,
    ) -> String {
        let db = self.db_path.clone();
        let period = params
            .0
            .period
            .clone()
            .unwrap_or_else(|| "month".to_string());
        match tokio::task::spawn_blocking(move || query_cost_reconciliation(&db, &period)).await {
            Ok(Ok(v)) => json_ok(v),
            Ok(Err(e)) => json_err(e),
            Err(e) => json_err(e),
        }
    }
}

// ── Query functions (blocking, called from spawn_blocking) ────────────────────

fn query_today(db_path: &Path) -> Result<serde_json::Value> {
    let conn = open_conn(db_path)?;
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    let mut stmt = conn.prepare(
        "SELECT provider, COALESCE(model, 'unknown'),
                SUM(input_tokens), SUM(output_tokens),
                SUM(cache_read_tokens), SUM(cache_creation_tokens),
                SUM(reasoning_output_tokens), COUNT(*),
                COALESCE(SUM(estimated_cost_nanos), 0)
         FROM turns
         WHERE substr(timestamp, 1, 10) = ?1
         GROUP BY provider, model
         ORDER BY SUM(input_tokens + output_tokens) DESC",
    )?;

    let models: Vec<serde_json::Value> = stmt
        .query_map([&today], |row| {
            Ok(serde_json::json!({
                "provider": row.get::<_, String>(0)?,
                "model": row.get::<_, String>(1)?,
                "input_tokens": row.get::<_, i64>(2)?,
                "output_tokens": row.get::<_, i64>(3)?,
                "cache_read_tokens": row.get::<_, i64>(4)?,
                "cache_creation_tokens": row.get::<_, i64>(5)?,
                "reasoning_output_tokens": row.get::<_, i64>(6)?,
                "turns": row.get::<_, i64>(7)?,
                "estimated_cost": row.get::<_, i64>(8)? as f64 / 1_000_000_000.0,
            }))
        })?
        .filter_map(|r| r.ok())
        .collect();

    let mut by_provider_stmt = conn.prepare(
        "SELECT provider, COUNT(*),
                COALESCE(SUM(input_tokens), 0), COALESCE(SUM(output_tokens), 0),
                COALESCE(SUM(estimated_cost_nanos), 0)
         FROM turns
         WHERE substr(timestamp, 1, 10) = ?1
         GROUP BY provider
         ORDER BY COUNT(*) DESC",
    )?;

    let by_provider: Vec<serde_json::Value> = by_provider_stmt
        .query_map([&today], |row| {
            Ok(serde_json::json!({
                "provider": row.get::<_, String>(0)?,
                "turns": row.get::<_, i64>(1)?,
                "input_tokens": row.get::<_, i64>(2)?,
                "output_tokens": row.get::<_, i64>(3)?,
                "estimated_cost": row.get::<_, i64>(4)? as f64 / 1_000_000_000.0,
            }))
        })?
        .filter_map(|r| r.ok())
        .collect();

    let total_cost: f64 = models
        .iter()
        .map(|m| m["estimated_cost"].as_f64().unwrap_or(0.0))
        .sum();

    Ok(serde_json::json!({
        "date": today,
        "total_cost_usd": (total_cost * 10000.0).round() / 10000.0,
        "by_model": models,
        "by_provider": by_provider,
    }))
}

fn query_stats(db_path: &Path) -> Result<serde_json::Value> {
    let conn = open_conn(db_path)?;

    let (sessions, first, last): (i64, Option<String>, Option<String>) = conn.query_row(
        "SELECT COUNT(*), MIN(first_timestamp), MAX(last_timestamp) FROM sessions",
        [],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )?;

    let (inp, out, cr, cc, ro, turns): (i64, i64, i64, i64, i64, i64) = conn.query_row(
        "SELECT COALESCE(SUM(input_tokens),0), COALESCE(SUM(output_tokens),0),
                COALESCE(SUM(cache_read_tokens),0), COALESCE(SUM(cache_creation_tokens),0),
                COALESCE(SUM(reasoning_output_tokens),0), COUNT(*) FROM turns",
        [],
        |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
            ))
        },
    )?;

    let total_cost_nanos: i64 = conn.query_row(
        "SELECT COALESCE(SUM(estimated_cost_nanos), 0) FROM turns",
        [],
        |row| row.get(0),
    )?;

    let mut by_model_stmt = conn.prepare(
        "SELECT provider, COALESCE(model,'unknown'),
                SUM(input_tokens), SUM(output_tokens),
                COUNT(DISTINCT session_id), COUNT(*),
                COALESCE(SUM(estimated_cost_nanos), 0)
         FROM turns
         GROUP BY provider, model
         ORDER BY SUM(input_tokens+output_tokens) DESC",
    )?;

    let by_model: Vec<serde_json::Value> = by_model_stmt
        .query_map([], |row| {
            Ok(serde_json::json!({
                "provider": row.get::<_, String>(0)?,
                "model": row.get::<_, String>(1)?,
                "input_tokens": row.get::<_, i64>(2)?,
                "output_tokens": row.get::<_, i64>(3)?,
                "sessions": row.get::<_, i64>(4)?,
                "turns": row.get::<_, i64>(5)?,
                "estimated_cost": row.get::<_, i64>(6)? as f64 / 1_000_000_000.0,
            }))
        })?
        .filter_map(|r| r.ok())
        .collect();

    let f = |s: &Option<String>| {
        s.as_deref()
            .unwrap_or("")
            .chars()
            .take(10)
            .collect::<String>()
    };

    Ok(serde_json::json!({
        "period": { "from": f(&first), "to": f(&last) },
        "total_sessions": sessions,
        "total_turns": turns,
        "total_input_tokens": inp,
        "total_output_tokens": out,
        "total_cache_read_tokens": cr,
        "total_cache_creation_tokens": cc,
        "total_reasoning_output_tokens": ro,
        "total_estimated_cost_usd":
            (total_cost_nanos as f64 / 1_000_000_000.0 * 10000.0).round() / 10000.0,
        "by_model": by_model,
    }))
}

fn query_weekly(db_path: &Path, sow: &str) -> Result<serde_json::Value> {
    let day = parse_weekday(sow);
    let tz = crate::tz::TzParams {
        tz_offset_min: None,
        week_starts_on: Some(weekday_to_u8(day)),
    };
    let conn = open_conn(db_path)?;
    let rows = sdb::sum_by_week(&conn, tz)?;

    let weeks: Vec<String> = rows
        .iter()
        .map(|r| r.week.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();

    let mut by_week: std::collections::HashMap<String, Vec<&crate::scanner::db::WeekRow>> =
        std::collections::HashMap::new();
    for r in &rows {
        by_week.entry(r.week.clone()).or_default().push(r);
    }

    let sow_lower = sow.to_ascii_lowercase();
    let weeks_json: Vec<serde_json::Value> = weeks
        .iter()
        .map(|week| {
            let wr = by_week.get(week).cloned().unwrap_or_default();
            let total_cost: f64 = wr
                .iter()
                .map(|r| r.cost_nanos as f64 / 1_000_000_000.0)
                .sum();
            let total_in: i64 = wr.iter().map(|r| r.input_tokens).sum();
            let total_out: i64 = wr.iter().map(|r| r.output_tokens).sum();

            let models: Vec<serde_json::Value> = wr
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "model": r.model,
                        "provider": r.provider,
                        "turns": r.turns,
                        "input_tokens": r.input_tokens,
                        "output_tokens": r.output_tokens,
                        "estimated_cost": r.cost_nanos as f64 / 1_000_000_000.0,
                    })
                })
                .collect();

            serde_json::json!({
                "week": week,
                "total_input_tokens": total_in,
                "total_output_tokens": total_out,
                "total_estimated_cost_usd": (total_cost * 10000.0).round() / 10000.0,
                "models": models,
            })
        })
        .collect();

    Ok(serde_json::json!({
        "start_of_week": sow_lower,
        "weeks": weeks_json,
    }))
}

fn query_sessions(
    db_path: &Path,
    limit: u32,
    offset: u32,
    project: Option<&str>,
) -> Result<serde_json::Value> {
    let conn = open_conn(db_path)?;

    let total: i64 = if let Some(proj) = project {
        conn.query_row(
            "SELECT COUNT(*) FROM sessions WHERE project_name LIKE ?1",
            rusqlite::params![format!("%{proj}%")],
            |r| r.get(0),
        )?
    } else {
        conn.query_row("SELECT COUNT(*) FROM sessions", [], |r| r.get(0))?
    };

    let sessions: Vec<serde_json::Value> = if let Some(proj) = project {
        let mut stmt = conn.prepare(
            "SELECT session_id, provider, project_name, first_timestamp, last_timestamp,
                    total_input_tokens, total_output_tokens,
                    COALESCE(total_estimated_cost_nanos, 0), turn_count
             FROM sessions
             WHERE project_name LIKE ?1
             ORDER BY last_timestamp DESC
             LIMIT ?2 OFFSET ?3",
        )?;
        stmt.query_map(
            rusqlite::params![format!("%{proj}%"), limit, offset],
            session_row_to_json,
        )?
        .filter_map(|r| r.ok())
        .collect()
    } else {
        let mut stmt = conn.prepare(
            "SELECT session_id, provider, project_name, first_timestamp, last_timestamp,
                    total_input_tokens, total_output_tokens,
                    COALESCE(total_estimated_cost_nanos, 0), turn_count
             FROM sessions
             ORDER BY last_timestamp DESC
             LIMIT ?1 OFFSET ?2",
        )?;
        stmt.query_map(rusqlite::params![limit, offset], session_row_to_json)?
            .filter_map(|r| r.ok())
            .collect()
    };

    Ok(serde_json::json!({
        "total": total,
        "limit": limit,
        "offset": offset,
        "sessions": sessions,
    }))
}

fn session_row_to_json(row: &rusqlite::Row<'_>) -> rusqlite::Result<serde_json::Value> {
    Ok(serde_json::json!({
        "session_id": row.get::<_, String>(0)?,
        "provider": row.get::<_, String>(1)?,
        "project_name": row.get::<_, Option<String>>(2)?,
        "first_timestamp": row.get::<_, Option<String>>(3)?,
        "last_timestamp": row.get::<_, Option<String>>(4)?,
        "total_input_tokens": row.get::<_, i64>(5)?,
        "total_output_tokens": row.get::<_, i64>(6)?,
        "estimated_cost_usd": row.get::<_, i64>(7)? as f64 / 1_000_000_000.0,
        "turn_count": row.get::<_, i64>(8)?,
    }))
}

fn query_active_block(db_path: &Path, session_hours: f64) -> Result<serde_json::Value> {
    let conn = open_conn(db_path)?;
    let turns = sdb::load_all_turns(&conn)?;
    let blocks = identify_blocks(&turns, session_hours);
    let now = chrono::Utc::now();

    let active = blocks.iter().find(|b| b.is_active);
    match active {
        None => Ok(serde_json::json!({ "block": null })),
        Some(b) => {
            let rate = calculate_burn_rate(b, now);
            let proj = project_block_usage(b, rate, now);

            let burn = match rate {
                Some(r) => {
                    use crate::analytics::burn_rate::{self as br, BurnRateConfig};
                    // TODO: thread config thresholds here so MCP tier matches statusline when user overrides [statusline.burn_rate_*] in TOML.
                    let tier = br::tier(r.tokens_per_min, &BurnRateConfig::default());
                    serde_json::json!({
                        "tokens_per_min": r.tokens_per_min,
                        "cost_per_hour_nanos": r.cost_per_hour_nanos,
                        "cost_per_hour_usd": r.cost_per_hour_nanos as f64 / 1_000_000_000.0,
                        "tier": tier,
                    })
                }
                None => serde_json::Value::Null,
            };

            Ok(serde_json::json!({
                "block": {
                    "start": b.start.to_rfc3339(),
                    "end": b.end.to_rfc3339(),
                    "tokens": b.tokens,
                    "cost_nanos": b.cost_nanos,
                    "estimated_cost_usd": b.cost_nanos as f64 / 1_000_000_000.0,
                    "models": b.models,
                    "is_active": b.is_active,
                    "entry_count": b.entry_count,
                },
                "burn_rate": burn,
                "projection": {
                    "projected_cost_nanos": proj.projected_cost_nanos,
                    "projected_cost_usd": proj.projected_cost_nanos as f64 / 1_000_000_000.0,
                    "projected_tokens": proj.projected_tokens,
                },
            }))
        }
    }
}

fn query_rate_windows(db_path: &Path) -> Result<serde_json::Value> {
    let conn = open_conn(db_path)?;

    let mut stmt = conn.prepare(
        "SELECT window_type, used_percent, resets_at, timestamp
         FROM rate_window_history
         WHERE id IN (
             SELECT MAX(id) FROM rate_window_history GROUP BY window_type
         )
         ORDER BY window_type",
    )?;

    let windows: Vec<serde_json::Value> = stmt
        .query_map([], |row| {
            Ok(serde_json::json!({
                "window_type": row.get::<_, String>(0)?,
                "used_percent": row.get::<_, Option<f64>>(1)?,
                "resets_at": row.get::<_, Option<String>>(2)?,
                "captured_at": row.get::<_, String>(3)?,
            }))
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(serde_json::json!({ "rate_windows": windows }))
}

fn query_context_window(db_path: &Path) -> Result<serde_json::Value> {
    use crate::analytics::quota::severity_for_pct;
    use rusqlite::OptionalExtension;

    let conn = open_conn(db_path)?;

    let row: Option<(i64, i64, Option<String>, String)> = {
        let mut stmt = conn.prepare(
            "SELECT context_input_tokens, context_window_size, session_id, received_at
             FROM live_events
             WHERE context_input_tokens IS NOT NULL AND context_window_size > 0
             ORDER BY received_at DESC LIMIT 1",
        )?;
        stmt.query_row([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
            .optional()?
    };

    match row {
        None => Ok(serde_json::json!({ "enabled": false })),
        Some((input_tokens, window_size, session_id, captured_at)) => {
            let pct = input_tokens as f64 / window_size as f64;
            let severity = severity_for_pct(pct);
            let severity_str = match severity {
                crate::analytics::quota::Severity::Ok => "ok",
                crate::analytics::quota::Severity::Warn => "warn",
                crate::analytics::quota::Severity::Danger => "danger",
            };
            let mut v = serde_json::json!({
                "total_input_tokens": input_tokens,
                "context_window_size": window_size,
                "pct": pct,
                "severity": severity_str,
                "captured_at": captured_at,
            });
            if let Some(sid) = session_id {
                v["session_id"] = serde_json::json!(sid);
            }
            Ok(v)
        }
    }
}

fn query_cost_reconciliation(db_path: &Path, period: &str) -> Result<serde_json::Value> {
    let conn = open_conn(db_path)?;

    // Check whether any hook costs exist.
    let hook_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM live_events WHERE hook_reported_cost_nanos IS NOT NULL",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    if hook_count == 0 {
        return Ok(serde_json::json!({ "enabled": false }));
    }

    let days_back: i64 = match period {
        "day" => 1,
        "week" => 7,
        _ => 30,
    };

    let now = chrono::Utc::now();
    let cutoff = (now - chrono::Duration::days(days_back)).to_rfc3339();

    let mut hook_by_day: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    {
        let mut stmt = conn.prepare(
            "SELECT date(received_at) AS day,
                    COALESCE(SUM(hook_reported_cost_nanos), 0) AS nanos
             FROM live_events
             WHERE hook_reported_cost_nanos IS NOT NULL
               AND received_at >= ?1
             GROUP BY day ORDER BY day",
        )?;
        let rows = stmt.query_map(rusqlite::params![cutoff], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
        })?;
        for row in rows {
            let (day, nanos) = row?;
            hook_by_day.insert(day, nanos);
        }
    }

    let mut local_by_day: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    {
        let mut stmt = conn.prepare(
            "SELECT date(timestamp) AS day,
                    COALESCE(SUM(estimated_cost_nanos), 0) AS nanos
             FROM turns
             WHERE timestamp >= ?1
             GROUP BY day ORDER BY day",
        )?;
        let rows = stmt.query_map(rusqlite::params![cutoff], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
        })?;
        for row in rows {
            let (day, nanos) = row?;
            local_by_day.insert(day, nanos);
        }
    }

    let mut all_days: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    all_days.extend(hook_by_day.keys().cloned());
    all_days.extend(local_by_day.keys().cloned());

    let breakdown: Vec<serde_json::Value> = all_days
        .iter()
        .map(|day| {
            let hook_nanos = hook_by_day.get(day).copied().unwrap_or(0);
            let local_nanos = local_by_day.get(day).copied().unwrap_or(0);
            serde_json::json!({ "day": day, "hook_nanos": hook_nanos, "local_nanos": local_nanos })
        })
        .collect();

    let hook_total_nanos: i64 = hook_by_day.values().sum();
    let local_total_nanos: i64 = local_by_day.values().sum();
    let divergence_pct = if local_total_nanos > 0 {
        (hook_total_nanos - local_total_nanos) as f64 / local_total_nanos as f64
    } else {
        0.0
    };

    Ok(serde_json::json!({
        "enabled": true,
        "period": period,
        "hook_total_nanos": hook_total_nanos,
        "local_total_nanos": local_total_nanos,
        "divergence_pct": divergence_pct,
        "breakdown": breakdown,
    }))
}

fn query_quota(db_path: &Path) -> Result<serde_json::Value> {
    let cfg = crate::config::load_config_resolved();
    let Some(token_limit) = cfg.resolved_blocks().token_limit else {
        return Ok(serde_json::json!({ "enabled": false }));
    };

    let conn = open_conn(db_path)?;
    let turns = sdb::load_all_turns(&conn)?;
    let blocks = identify_blocks(&turns, 5.0);
    let now = chrono::Utc::now();

    let active = blocks.iter().find(|b| b.is_active);
    match active {
        None => Ok(serde_json::json!({ "enabled": true, "active_block": false })),
        Some(b) => {
            let rate = calculate_burn_rate(b, now);
            let proj = project_block_usage(b, rate, now);
            match compute_quota(b, &proj, token_limit) {
                None => {
                    Ok(serde_json::json!({ "enabled": true, "active_block": true, "quota": null }))
                }
                Some(q) => {
                    let sev_str = |s: crate::analytics::quota::Severity| match s {
                        crate::analytics::quota::Severity::Ok => "ok",
                        crate::analytics::quota::Severity::Warn => "warn",
                        crate::analytics::quota::Severity::Danger => "danger",
                    };
                    Ok(serde_json::json!({
                        "enabled": true,
                        "limit_tokens": q.limit_tokens,
                        "used_tokens": q.used_tokens,
                        "projected_tokens": q.projected_tokens,
                        "current_pct": q.current_pct,
                        "projected_pct": q.projected_pct,
                        "current_severity": sev_str(q.current_severity),
                        "projected_severity": sev_str(q.projected_severity),
                    }))
                }
            }
        }
    }
}
