//! Detector: tool retry and rework-cycle patterns.
//!
//! Two finding flavors are emitted from a single pass over `tool_invocations`:
//!
//! **Flavor A — repeated invocations:**
//! The same tool with the same canonical input is invoked ≥ N times in a
//! single session.  Thresholds are lower for edit-class tools (≥3) where
//! repetition is most costly, and higher for lookup tools (Read ≥5, mcp ≥5).
//! Findings that include at least one `is_error=1` row receive a severity
//! upgrade and an "(N errors)" annotation in the title.
//!
//! **Flavor B — rework cycles:**
//! An edit-class tool touches file `F`, then ≥1 unrelated tool runs, then an
//! edit-class tool touches `F` again in the same session.  Each such
//! edit→other→edit arc counts as one cycle.  Generalises the session-scalar
//! `one_shot` flag (from `scanner/oneshot.rs`) into per-file findings.
//!
//! # Cost formula
//! Flavor A: `(count - 1) × tokens_per_tool × 20_000_000 / 1_000`
//! Flavor B: `n_cycles × 600 × 20_000_000 / 1_000`
//!
//! # Severity (Flavor A)
//! | count      | errors | Severity |
//! |------------|--------|----------|
//! | ≤7, 0 err  | 0      | Low      |
//! | >7 or ≥1 err | ≥1   | Medium   |
//! | >15 or ≥2 err | ≥2  | High     |
//!
//! # Severity (Flavor B)
//! | cycles | Severity |
//! |--------|----------|
//! | 1      | Low      |
//! | 2      | Medium   |
//! | ≥3     | High     |

use std::collections::HashMap;

use anyhow::Result;
use chrono::{Duration, Utc};
use rusqlite::Connection;

use super::{Detector, Finding, Severity};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Characters taken from a Bash command for canonical grouping.
const BASH_CANON_LEN: usize = 60;

/// Nanos per 1 000 tokens (rough Sonnet input rate: $0.02 / 1K tokens).
const NANOS_PER_1K_TOKENS: i64 = 20_000_000;

/// Maximum total findings returned across both flavors.
const MAX_FINDINGS: usize = 30;

/// Rolling window for the SQL cutoff.
const WINDOW_DAYS: i64 = 30;

/// Tokens assumed per rework-cycle waste arc.
const REWORK_TOKENS_PER_CYCLE: i64 = 600;

// ---------------------------------------------------------------------------
// Tool families
// ---------------------------------------------------------------------------

const EDIT_TOOLS: &[&str] = &["Edit", "Write", "MultiEdit", "NotebookEdit"];

fn is_edit_tool(name: &str) -> bool {
    EDIT_TOOLS.contains(&name)
}

/// Minimum invocations before a Flavor-A finding fires.
fn min_repeats(tool_name: &str) -> u32 {
    if is_edit_tool(tool_name) {
        3
    } else if tool_name == "Read" {
        5
    } else if matches!(tool_name, "WebFetch" | "Task") {
        3
    } else if tool_name.starts_with("mcp__") {
        5
    } else {
        // Bash, Grep, Glob, and unknown tools.
        4
    }
}

/// Conservative token count per wasted repeat invocation.
fn tokens_per_tool(tool_name: &str) -> i64 {
    if is_edit_tool(tool_name) {
        500
    } else if tool_name == "Task" {
        1_000
    } else {
        200
    }
}

// ---------------------------------------------------------------------------
// Canonical input extraction
// ---------------------------------------------------------------------------

/// Derive a stable grouping key from `(tool_name, tool_input_json)`.
/// Returns `None` when the tool is not tracked or the JSON cannot be parsed.
fn canonical_input(tool_name: &str, tool_input_json: Option<&str>) -> Option<String> {
    // MCP tools: group by full tool name with no per-input matching.
    if tool_name.starts_with("mcp__") {
        return Some(tool_name.to_string());
    }

    let json_str = tool_input_json?;
    let v: serde_json::Value = serde_json::from_str(json_str).ok()?;

    match tool_name {
        "Read" | "Edit" | "Write" | "MultiEdit" => v
            .get("file_path")
            .and_then(|f| f.as_str())
            .map(str::to_string),

        "NotebookEdit" => v
            .get("notebook_path")
            .or_else(|| v.get("file_path"))
            .and_then(|f| f.as_str())
            .map(str::to_string),

        "Bash" => v.get("command").and_then(|c| c.as_str()).map(|s| {
            s.chars()
                .take(BASH_CANON_LEN)
                .collect::<String>()
                .to_ascii_lowercase()
        }),

        "Grep" | "Glob" => v
            .get("pattern")
            .and_then(|p| p.as_str())
            .map(str::to_string),

        "WebFetch" => v.get("url").and_then(|u| u.as_str()).map(|s| {
            // Strip query string and fragment for canonical comparison.
            let s = s.split('?').next().unwrap_or(s);
            s.split('#').next().unwrap_or(s).to_string()
        }),

        "Task" => v
            .get("subagent_type")
            .and_then(|t| t.as_str())
            .map(str::to_string),

        _ => None,
    }
}

/// Extract the target file path for rework-cycle tracking (edit-class tools only).
fn edit_file_path(tool_name: &str, tool_input_json: Option<&str>) -> Option<String> {
    if !is_edit_tool(tool_name) {
        return None;
    }
    let json_str = tool_input_json?;
    let v: serde_json::Value = serde_json::from_str(json_str).ok()?;
    if tool_name == "NotebookEdit" {
        v.get("notebook_path")
            .or_else(|| v.get("file_path"))
            .and_then(|f| f.as_str())
            .map(str::to_string)
    } else {
        v.get("file_path")
            .and_then(|f| f.as_str())
            .map(str::to_string)
    }
}

// ---------------------------------------------------------------------------
// Session ID shortener (mirrors reread.rs / bash.rs)
// ---------------------------------------------------------------------------

fn short_session(session_id: &str) -> &str {
    session_id
        .split_once(':')
        .map(|(_, raw)| {
            let raw = raw.trim_end_matches(|c: char| !c.is_alphanumeric());
            if raw.len() > 8 { &raw[raw.len() - 8..] } else { raw }
        })
        .unwrap_or(session_id)
}

// ---------------------------------------------------------------------------
// Severity helpers
// ---------------------------------------------------------------------------

fn severity_for_repeats(count: u32, errors: u32) -> Severity {
    if count > 15 || errors >= 2 {
        Severity::High
    } else if count > 7 || errors >= 1 {
        Severity::Medium
    } else {
        Severity::Low
    }
}

fn severity_for_cycles(n_cycles: u32) -> Severity {
    if n_cycles >= 3 {
        Severity::High
    } else if n_cycles >= 2 {
        Severity::Medium
    } else {
        Severity::Low
    }
}

fn severity_ord(s: Severity) -> u8 {
    match s {
        Severity::Low => 0,
        Severity::Medium => 1,
        Severity::High => 2,
    }
}

// ---------------------------------------------------------------------------
// Detector
// ---------------------------------------------------------------------------

pub struct ToolRetryDetector;

impl ToolRetryDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ToolRetryDetector {
    fn default() -> Self {
        Self::new()
    }
}

struct InvRow {
    session_id: String,
    tool_name: String,
    tool_input_json: Option<String>,
    is_error: bool,
}

impl Detector for ToolRetryDetector {
    fn name(&self) -> &'static str {
        "tool_retry"
    }

    fn run(&self, conn: &Connection) -> Result<Vec<Finding>> {
        let cutoff = (Utc::now() - Duration::days(WINDOW_DAYS))
            .format("%Y-%m-%d")
            .to_string();

        // SQL kept here for detector-local cohesion; see scanner/db.rs for shared queries.
        let mut stmt = conn.prepare(
            "SELECT session_id, tool_name, tool_input_json, is_error
             FROM tool_invocations
             WHERE timestamp != '' AND SUBSTR(timestamp, 1, 10) >= ?1
             ORDER BY session_id, timestamp",
        )?;

        let rows: Vec<InvRow> = stmt
            .query_map(rusqlite::params![cutoff], |row| {
                Ok(InvRow {
                    session_id: row.get::<_, String>(0)?,
                    tool_name: row.get::<_, String>(1)?,
                    tool_input_json: row.get::<_, Option<String>>(2)?,
                    is_error: row.get::<_, Option<i32>>(3)?.unwrap_or(0) != 0,
                })
            })?
            .filter_map(|r| match r {
                Ok(v) => Some(v),
                Err(e) => {
                    tracing::warn!("ToolRetryDetector: row error: {e}");
                    None
                }
            })
            .collect();

        if rows.is_empty() {
            return Ok(vec![]);
        }

        // Group into per-session slices (rows are sorted by session_id, timestamp).
        let mut sessions: Vec<(String, Vec<usize>)> = Vec::new();
        for (i, row) in rows.iter().enumerate() {
            match sessions.last_mut() {
                Some((sid, indices)) if *sid == row.session_id => indices.push(i),
                _ => sessions.push((row.session_id.clone(), vec![i])),
            }
        }

        let mut findings = Vec::new();
        for (session_id, indices) in &sessions {
            let session_rows: Vec<&InvRow> = indices.iter().map(|&i| &rows[i]).collect();
            emit_repeat_findings(session_id, &session_rows, self.name(), &mut findings);
            emit_rework_cycle_findings(session_id, &session_rows, self.name(), &mut findings);
        }

        // Sort: High > Medium > Low, then by waste descending, then by title for determinism.
        findings.sort_by(|a, b| {
            severity_ord(b.severity)
                .cmp(&severity_ord(a.severity))
                .then_with(|| {
                    b.estimated_monthly_waste_nanos
                        .cmp(&a.estimated_monthly_waste_nanos)
                })
                .then_with(|| a.title.cmp(&b.title))
        });
        findings.truncate(MAX_FINDINGS);

        Ok(findings)
    }
}

// ---------------------------------------------------------------------------
// Flavor A — repeated invocations
// ---------------------------------------------------------------------------

fn emit_repeat_findings(
    session_id: &str,
    rows: &[&InvRow],
    detector: &'static str,
    out: &mut Vec<Finding>,
) {
    let mut groups: HashMap<(String, String), (u32, u32)> = HashMap::new();

    for row in rows {
        let Some(canon) = canonical_input(&row.tool_name, row.tool_input_json.as_deref()) else {
            continue;
        };
        let entry = groups
            .entry((row.tool_name.clone(), canon))
            .or_insert((0, 0));
        entry.0 += 1;
        if row.is_error {
            entry.1 += 1;
        }
    }

    let session_short = short_session(session_id);

    for ((tool_name, canon), (count, errors)) in groups {
        if count < min_repeats(&tool_name) {
            continue;
        }

        let severity = severity_for_repeats(count, errors);
        let tokens = tokens_per_tool(&tool_name);
        let waste = ((count as i64 - 1) * tokens * NANOS_PER_1K_TOKENS / 1_000).max(0);

        let display: String = if canon.len() > 60 {
            format!("{}...", &canon[..60])
        } else {
            canon.clone()
        };

        let error_suffix = if errors > 0 {
            format!(" ({errors} error{})", if errors == 1 { "" } else { "s" })
        } else {
            String::new()
        };

        let error_prefix = if errors > 0 {
            format!("{errors} of {count} attempts errored. ")
        } else {
            String::new()
        };

        out.push(Finding {
            detector: detector.into(),
            severity,
            title: format!(
                "{tool_name} '{display}' invoked {count} times in session {session_short}{error_suffix}"
            ),
            detail: format!(
                "{error_prefix}{tool_name} was invoked {count} times with the same input in \
                 session '{session_id}'. Consider caching the result, using MultiEdit to batch \
                 edits, or reducing redundant invocations. Estimated overhead: {count} × \
                 ~{tokens} tokens per call."
            ),
            estimated_monthly_waste_nanos: waste,
        });
    }
}

// ---------------------------------------------------------------------------
// Flavor B — rework cycles (edit → other → edit on same file)
// ---------------------------------------------------------------------------

struct FileEditState {
    seen_edit: bool,
    has_intermediate: bool,
    cycle_count: u32,
}

fn emit_rework_cycle_findings(
    session_id: &str,
    rows: &[&InvRow],
    detector: &'static str,
    out: &mut Vec<Finding>,
) {
    let mut file_states: HashMap<String, FileEditState> = HashMap::new();

    for row in rows {
        match edit_file_path(&row.tool_name, row.tool_input_json.as_deref()) {
            Some(file) => {
                let state = file_states.entry(file).or_insert(FileEditState {
                    seen_edit: false,
                    has_intermediate: false,
                    cycle_count: 0,
                });
                if state.seen_edit && state.has_intermediate {
                    state.cycle_count += 1;
                    state.has_intermediate = false;
                }
                state.seen_edit = true;
            }
            None => {
                // Non-edit tool: mark every tracked file as having an intermediate.
                for state in file_states.values_mut() {
                    if state.seen_edit {
                        state.has_intermediate = true;
                    }
                }
            }
        }
    }

    let session_short = short_session(session_id);

    for (file, state) in file_states {
        if state.cycle_count == 0 {
            continue;
        }
        let n = state.cycle_count;
        let severity = severity_for_cycles(n);
        let waste = n as i64 * REWORK_TOKENS_PER_CYCLE * NANOS_PER_1K_TOKENS / 1_000;

        out.push(Finding {
            detector: detector.into(),
            severity,
            title: format!(
                "File '{file}' rewritten across {} edit batches in session {session_short}",
                n + 1
            ),
            detail: format!(
                "{n} rework cycle{} detected on '{file}' in session '{session_id}' \
                 (edit-class tool → other tool → edit-class tool on the same file). \
                 This suggests trial-and-error edits; consider asking the model to \
                 plan the full change before editing.",
                if n == 1 { "" } else { "s" }
            ),
            estimated_monthly_waste_nanos: waste,
        });
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizer::mod_tests::{empty_db, insert_sessions};

    /// Returns an RFC3339 timestamp `offset_secs` seconds before now.
    /// Use higher offsets for earlier events so SQL ORDER BY timestamp is chronological.
    fn ts(offset_secs: u32) -> String {
        use chrono::{Duration, Utc};
        (Utc::now() - Duration::seconds(i64::from(offset_secs)))
            .format("%Y-%m-%dT%H:%M:%SZ")
            .to_string()
    }

    fn insert_invocation(
        conn: &rusqlite::Connection,
        session_id: &str,
        timestamp: &str,
        tool_name: &str,
        tool_input_json: Option<&str>,
        is_error: bool,
    ) {
        // Ensure the session row exists so foreign-key-like lookups don't fail.
        conn.execute(
            "INSERT OR IGNORE INTO sessions
             (session_id, provider, first_timestamp, last_timestamp)
             VALUES (?1, 'claude', ?2, ?2)",
            rusqlite::params![session_id, timestamp],
        )
        .unwrap();

        // Generate a unique dedup key from session + timestamp + tool.
        let dedup = format!(
            "{session_id}:{timestamp}:{tool_name}:{}",
            tool_input_json.unwrap_or("")
        );
        conn.execute(
            "INSERT OR IGNORE INTO tool_invocations
             (session_id, provider, message_id, tool_name, mcp_server, mcp_tool,
              tool_category, tool_use_id, is_error, source_path, timestamp,
              error_text, tool_input_json)
             VALUES (?1, 'claude', NULL, ?2, NULL, NULL,
                     'builtin', ?3, ?4, '/tmp/t.jsonl', ?5,
                     NULL, ?6)",
            rusqlite::params![
                session_id,
                tool_name,
                dedup,
                is_error as i32,
                timestamp,
                tool_input_json,
            ],
        )
        .unwrap();
    }

    fn det() -> ToolRetryDetector {
        ToolRetryDetector::new()
    }

    // -----------------------------------------------------------------------
    // No-finding baseline
    // -----------------------------------------------------------------------

    #[test]
    fn no_invocations_no_finding() {
        let (_dir, conn) = empty_db();
        assert!(det().run(&conn).unwrap().is_empty());
    }

    #[test]
    fn below_threshold_silent_edit() {
        let (_dir, conn) = empty_db();
        // 2 edits < threshold of 3
        for i in 0..2u32 {
            insert_invocation(
                &conn, "claude:s1", &ts((2 - i) * 100),
                "Edit",
                Some(r#"{"file_path":"/src/foo.rs","old_string":"a","new_string":"b"}"#),
                false,
            );
        }
        assert!(det().run(&conn).unwrap().is_empty());
    }

    #[test]
    fn below_threshold_silent_read() {
        let (_dir, conn) = empty_db();
        // 4 reads < threshold of 5
        for i in 0..4u32 {
            insert_invocation(
                &conn, "claude:s1", &ts((4 - i) * 100),
                "Read",
                Some(r#"{"file_path":"/src/foo.rs"}"#),
                false,
            );
        }
        assert!(det().run(&conn).unwrap().is_empty());
    }

    // -----------------------------------------------------------------------
    // Flavor A — repeat detection severity levels
    // -----------------------------------------------------------------------

    #[test]
    fn repeat_read_same_file_triggers_low_then_high() {
        let (_dir, conn) = empty_db();
        // 5 reads → low (count == 5, ≤7, 0 errors)
        for i in 0..5u32 {
            insert_invocation(
                &conn, "claude:s1", &ts((5 - i) * 100),
                "Read",
                Some(r#"{"file_path":"/src/foo.rs"}"#),
                false,
            );
        }
        let findings = det().run(&conn).unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Low);
        assert!(findings[0].title.contains("5 times"), "{}", findings[0].title);
        assert!(findings[0].estimated_monthly_waste_nanos > 0);
    }

    #[test]
    fn repeat_edit_same_file_low() {
        let (_dir, conn) = empty_db();
        for i in 0..3u32 {
            insert_invocation(
                &conn, "claude:s1", &ts((3 - i) * 100),
                "Edit",
                Some(r#"{"file_path":"/src/bar.rs","old_string":"x","new_string":"y"}"#),
                false,
            );
        }
        let findings = det().run(&conn).unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Low);
    }

    #[test]
    fn repeat_edit_eight_times_medium() {
        let (_dir, conn) = empty_db();
        for i in 0..8u32 {
            insert_invocation(
                &conn, "claude:s1", &ts((8 - i) * 100),
                "Edit",
                Some(r#"{"file_path":"/src/bar.rs","old_string":"x","new_string":"y"}"#),
                false,
            );
        }
        let findings = det().run(&conn).unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Medium);
    }

    #[test]
    fn repeat_with_errors_upgraded_to_high() {
        let (_dir, conn) = empty_db();
        // 4 edits, 2 with is_error — should upgrade to High (errors >= 2)
        for i in 0..4u32 {
            insert_invocation(
                &conn, "claude:s1", &ts((4 - i) * 100),
                "Edit",
                Some(r#"{"file_path":"/src/baz.rs","old_string":"a","new_string":"b"}"#),
                i < 2, // first two errored
            );
        }
        let findings = det().run(&conn).unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::High);
        assert!(
            findings[0].title.contains("2 errors"),
            "expected '2 errors' in title: {}",
            findings[0].title
        );
    }

    #[test]
    fn repeat_with_one_error_medium() {
        let (_dir, conn) = empty_db();
        // 4 edits, 1 error — Medium (errors >= 1)
        for i in 0..4u32 {
            insert_invocation(
                &conn, "claude:s1", &ts((4 - i) * 100),
                "Edit",
                Some(r#"{"file_path":"/src/qux.rs","old_string":"a","new_string":"b"}"#),
                i == 0,
            );
        }
        let findings = det().run(&conn).unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Medium);
        assert!(findings[0].title.contains("1 error"), "{}", findings[0].title);
    }

    // -----------------------------------------------------------------------
    // Bash normalization
    // -----------------------------------------------------------------------

    #[test]
    fn bash_normalization_across_variants() {
        let (_dir, conn) = empty_db();
        // Commands that share the same first-60-char lowercased prefix collapse into one group.
        // "cargo build --release --target x86_64-unknown-linux-musl -j4" is exactly 60 chars.
        let prefix = "cargo build --release --target x86_64-unknown-linux-musl -j4";
        assert_eq!(prefix.len(), 60, "prefix must be exactly 60 chars for this test");
        let cmds = [
            prefix.to_string(),
            format!("{prefix} 2>&1"),
            format!("{prefix} --verbose"),
            format!("{prefix} | head -20"),
            format!("{prefix} && echo done"),
        ];
        for (i, cmd) in cmds.iter().enumerate() {
            insert_invocation(
                &conn, "claude:s1", &ts((5 - i as u32) * 100),
                "Bash",
                Some(&format!(r#"{{"command":"{cmd}"}}"#)),
                false,
            );
        }
        let findings = det().run(&conn).unwrap();
        // All 5 share the same 60-char lowercased prefix → one group; Bash threshold is 4.
        assert_eq!(
            findings.len(),
            1,
            "expected 1 finding, got: {:?}",
            findings.iter().map(|f| &f.title).collect::<Vec<_>>()
        );
        assert!(findings[0].title.contains("Bash"), "{}", findings[0].title);
    }

    // -----------------------------------------------------------------------
    // MCP repeat
    // -----------------------------------------------------------------------

    #[test]
    fn mcp_repeat_uses_tool_name_as_key() {
        let (_dir, conn) = empty_db();
        // 5 calls to the same MCP tool (different args) → 1 finding.
        for i in 0..5u32 {
            insert_invocation(
                &conn, "claude:s1", &ts((5 - i) * 100),
                "mcp__github__get_pr",
                Some(&format!(r#"{{"pr_number":{i}}}"#)),
                false,
            );
        }
        let findings = det().run(&conn).unwrap();
        assert_eq!(findings.len(), 1);
        assert!(
            findings[0].title.contains("mcp__github__get_pr"),
            "{}",
            findings[0].title
        );
    }

    // -----------------------------------------------------------------------
    // Flavor B — rework cycles
    // -----------------------------------------------------------------------

    #[test]
    fn rework_cycle_basic() {
        let (_dir, conn) = empty_db();
        // Edit → Bash → Edit on same file = 1 cycle → Low.
        // Higher ts() offset = older timestamp, ensuring chronological order.
        insert_invocation(
            &conn, "claude:s1", &ts(300), "Edit",
            Some(r#"{"file_path":"/src/foo.rs","old_string":"a","new_string":"b"}"#), false,
        );
        insert_invocation(
            &conn, "claude:s1", &ts(200), "Bash",
            Some(r#"{"command":"cargo check"}"#), false,
        );
        insert_invocation(
            &conn, "claude:s1", &ts(100), "Edit",
            Some(r#"{"file_path":"/src/foo.rs","old_string":"b","new_string":"c"}"#), false,
        );

        let findings = det().run(&conn).unwrap();
        let cycle = findings
            .iter()
            .find(|f| f.title.contains("rewritten across"))
            .expect("expected a rework-cycle finding");
        assert!(cycle.title.contains("2 edit batches"), "{}", cycle.title);
        assert_eq!(cycle.severity, Severity::Low);
        assert!(cycle.estimated_monthly_waste_nanos > 0);
    }

    #[test]
    fn rework_cycle_no_intermediate_not_counted() {
        let (_dir, conn) = empty_db();
        // Back-to-back edits with no intermediate tool → no cycle.
        for i in 0..3u32 {
            insert_invocation(
                &conn, "claude:s1", &ts((3 - i) * 100), "Edit",
                Some(r#"{"file_path":"/src/foo.rs","old_string":"a","new_string":"b"}"#), false,
            );
        }
        let findings = det().run(&conn).unwrap();
        let cycles: Vec<_> = findings
            .iter()
            .filter(|f| f.title.contains("rewritten across"))
            .collect();
        assert!(cycles.is_empty(), "no cycle without intermediate: {:?}", cycles);
    }

    #[test]
    fn rework_cycle_per_file_isolation() {
        let (_dir, conn) = empty_db();
        // foo.rs has a cycle; bar.rs does not.
        insert_invocation(&conn, "claude:s1", &ts(400), "Edit",
            Some(r#"{"file_path":"/src/foo.rs","old_string":"a","new_string":"b"}"#), false);
        insert_invocation(&conn, "claude:s1", &ts(300), "Bash",
            Some(r#"{"command":"cargo check"}"#), false);
        insert_invocation(&conn, "claude:s1", &ts(200), "Edit",
            Some(r#"{"file_path":"/src/foo.rs","old_string":"b","new_string":"c"}"#), false);
        // bar.rs: single edit only
        insert_invocation(&conn, "claude:s1", &ts(100), "Edit",
            Some(r#"{"file_path":"/src/bar.rs","old_string":"x","new_string":"y"}"#), false);

        let findings = det().run(&conn).unwrap();
        let cycles: Vec<_> = findings
            .iter()
            .filter(|f| f.title.contains("rewritten across"))
            .collect();
        assert_eq!(cycles.len(), 1, "only foo.rs should have a cycle");
        assert!(cycles[0].title.contains("foo.rs"), "{}", cycles[0].title);
    }

    #[test]
    fn rework_three_cycles_high_severity() {
        let (_dir, conn) = empty_db();
        // 4 edits with Bash between each pair = 3 cycles → High.
        // Offsets: edit0=700, bash0=600, edit1=500, bash1=400, edit2=300, bash2=200, edit3=100
        for i in 0..4u32 {
            insert_invocation(
                &conn, "claude:s1", &ts(700 - i * 200), "Edit",
                Some(r#"{"file_path":"/src/foo.rs","old_string":"a","new_string":"b"}"#), false,
            );
            if i < 3 {
                insert_invocation(
                    &conn, "claude:s1", &ts(700 - i * 200 - 100), "Bash",
                    Some(r#"{"command":"cargo check"}"#), false,
                );
            }
        }
        let findings = det().run(&conn).unwrap();
        let cycle = findings
            .iter()
            .find(|f| f.title.contains("rewritten across"))
            .expect("expected rework-cycle finding");
        assert_eq!(cycle.severity, Severity::High, "{}", cycle.title);
    }

    // -----------------------------------------------------------------------
    // Cross-session isolation
    // -----------------------------------------------------------------------

    #[test]
    fn cross_session_isolation() {
        let (_dir, conn) = empty_db();
        // 3 edits in s1 + 3 edits in s2 — should fire separately, not aggregate.
        for sess in ["claude:s1", "claude:s2"] {
            for i in 0..3u32 {
                insert_invocation(
                    &conn, sess, &ts((3 - i) * 100), "Edit",
                    Some(r#"{"file_path":"/src/foo.rs","old_string":"a","new_string":"b"}"#),
                    false,
                );
            }
        }
        let findings = det().run(&conn).unwrap();
        // Each session independently fires — 2 separate Low findings.
        let repeat_findings: Vec<_> = findings
            .iter()
            .filter(|f| f.title.contains("invoked"))
            .collect();
        assert_eq!(repeat_findings.len(), 2, "should have one finding per session");
    }

    // -----------------------------------------------------------------------
    // Unknown tool / malformed JSON skipped
    // -----------------------------------------------------------------------

    #[test]
    fn unknown_tool_skipped() {
        let (_dir, conn) = empty_db();
        // 100 invocations of an unrecognised tool — no input extraction rule.
        for i in 0..100u32 {
            insert_invocation(
                &conn, "claude:s1", &ts((100 - i) * 60),
                "UnknownTool", Some(r#"{"foo":"bar"}"#), false,
            );
        }
        assert!(det().run(&conn).unwrap().is_empty());
    }

    #[test]
    fn malformed_json_skipped() {
        let (_dir, conn) = empty_db();
        for i in 0..5u32 {
            insert_invocation(
                &conn, "claude:s1", &ts((5 - i) * 100),
                "Edit", Some("{not valid json"), false,
            );
        }
        // No parseable file_path → all skipped.
        assert!(det().run(&conn).unwrap().is_empty());
    }

    #[test]
    fn null_tool_input_json_skipped() {
        let (_dir, conn) = empty_db();
        insert_sessions(&conn, 1);
        for i in 0..5u32 {
            insert_invocation(
                &conn, "claude:session_0", &ts((5 - i) * 100),
                "Read", None, false,
            );
        }
        assert!(det().run(&conn).unwrap().is_empty());
    }

    // -----------------------------------------------------------------------
    // Waste nanos sanity
    // -----------------------------------------------------------------------

    #[test]
    fn waste_nanos_positive_and_nonzero() {
        let (_dir, conn) = empty_db();
        for i in 0..5u32 {
            insert_invocation(
                &conn, "claude:s1", &ts((5 - i) * 100),
                "Read", Some(r#"{"file_path":"/src/x.rs"}"#), false,
            );
        }
        let findings = det().run(&conn).unwrap();
        assert_eq!(findings.len(), 1);
        // (5-1) * 200 * 20_000_000 / 1000 = 16_000_000
        assert_eq!(findings[0].estimated_monthly_waste_nanos, 16_000_000);
    }
}
