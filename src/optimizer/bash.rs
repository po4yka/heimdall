//! Detector: bash noise.
//!
//! Queries `tool_events` for trivial shell commands that are repeated many
//! times in a single session.  Commands like `ls`, `pwd`, or `git status`
//! add token overhead without contributing to the task — they are typically
//! emitted by the model as orientation moves rather than meaningful work.
//!
//! # Trivial command list
//! The following command prefixes are considered trivial (hardcoded; chosen
//! based on frequency analysis of typical Claude Code sessions):
//!
//! - Navigation / inspection: `ls`, `pwd`, `cd`, `echo`, `cat`, `head`,
//!   `tail`, `wc`, `clear`
//! - Git read-only: `git status`, `git log`, `git diff`
//! - Environment probes: `date`, `whoami`, `which`
//!
//! Matching is case-insensitive on the first 20 characters of the stored
//! command string (the `value` column).
//!
//! # Cost formula
//! Each Bash invocation contributes approximately 50 input tokens (command
//! text + prompt overhead):
//!
//! `estimated_monthly_waste_nanos = count × 50 × 20_000_000 / 1_000`
//!
//! Derivation:
//!   - Assumed tokens per invocation: 50
//!   - Rough Sonnet input rate: $0.02 per 1 000 tokens = 20 000 000 nanos / 1 000 tokens
//!
//! # Severity thresholds
//! | Instances | Severity |
//! |-----------|----------|
//! | 5–10      | Low      |
//! | 11–30     | Medium   |
//! | >30       | High     |

use anyhow::Result;
use rusqlite::Connection;

use super::{Detector, Finding, Severity};

/// Number of characters used as the command "prefix" for grouping.
const PREFIX_LEN: usize = 20;
/// Minimum occurrences of a trivial command before it is flagged.
const MIN_OCCURRENCES: usize = 5;
/// Approximate input tokens per Bash invocation (conservative).
const TOKENS_PER_BASH: i64 = 50;
/// Nanos per 1 000 tokens (rough Sonnet input rate: $0.02 / 1K tokens).
const NANOS_PER_1K_TOKENS: i64 = 20_000_000;

/// Known-trivial command prefixes (lowercase, no leading whitespace).
/// Matching is done against the first `PREFIX_LEN` characters of the stored
/// command value, lowercased.
///
/// Rationale for each entry:
/// - `ls`/`pwd`/`cd`/`echo`: orientation commands with no lasting effect.
/// - `cat`/`head`/`tail`/`wc`: read-only file inspection; should use Read tool instead.
/// - `clear`: no-op in a non-interactive context.
/// - `git status`/`git log`/`git diff`: read-only git queries with no side effects.
/// - `date`/`whoami`/`which`: environment probes.
const TRIVIAL_PREFIXES: &[&str] = &[
    "ls",
    "pwd",
    "cd",
    "echo",
    "cat",
    "head",
    "tail",
    "wc",
    "clear",
    "git status",
    "git log",
    "git diff",
    "date",
    "whoami",
    "which",
];

pub struct BashNoiseDetector;

impl BashNoiseDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for BashNoiseDetector {
    fn default() -> Self {
        Self::new()
    }
}

fn severity_for_count(count: i64) -> Severity {
    if count > 30 {
        Severity::High
    } else if count > 10 {
        Severity::Medium
    } else {
        Severity::Low
    }
}

/// Estimate waste nanos for `count` trivial bash invocations.
fn waste_nanos(count: i64) -> i64 {
    count * TOKENS_PER_BASH * NANOS_PER_1K_TOKENS / 1_000
}

/// If `cmd` (lowercased prefix) matches a trivial prefix, return that canonical
/// prefix string so callers can group different variants under the same bucket.
/// Returns `None` if the command is not trivial.
fn trivial_prefix(cmd_lower_prefix: &str) -> Option<&'static str> {
    for &prefix in TRIVIAL_PREFIXES {
        // Match exactly or followed by whitespace (so "git status --short" matches
        // "git status" but "git statuscheck" does not).
        if cmd_lower_prefix == prefix
            || cmd_lower_prefix.starts_with(&format!("{prefix} "))
            || cmd_lower_prefix.starts_with(&format!("{prefix}\t"))
        {
            return Some(prefix);
        }
    }
    None
}

/// Return `true` if `cmd` (lowercased prefix) matches any trivial prefix.
#[cfg_attr(not(test), allow(dead_code))]
fn is_trivial(cmd_lower_prefix: &str) -> bool {
    trivial_prefix(cmd_lower_prefix).is_some()
}

impl Detector for BashNoiseDetector {
    fn name(&self) -> &'static str {
        "bash_noise"
    }

    fn run(&self, conn: &Connection) -> Result<Vec<Finding>> {
        // Fetch all bash events. We cannot do the trivial-prefix filter in SQL
        // in a portable way without LOWER() on a substring, so we fetch all
        // bash values and filter in Rust.
        let mut stmt = conn.prepare(
            "SELECT session_id, value, COUNT(*) AS cnt
             FROM tool_events
             WHERE kind = 'bash' AND value != '' AND value != 'Bash'
             GROUP BY session_id, value
             ORDER BY cnt DESC",
        )?;

        let rows: Vec<(String, String, i64)> = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })?
            .filter_map(|r| match r {
                Ok(val) => Some(val),
                Err(e) => {
                    tracing::warn!("BashNoiseDetector: row error: {e}");
                    None
                }
            })
            .collect();

        // Group by session_id + command prefix (first PREFIX_LEN chars, lowercased).
        // Aggregate counts across different full commands that share the same prefix.
        use std::collections::HashMap;
        let mut grouped: HashMap<(String, String), i64> = HashMap::new();

        for (session_id, value, cnt) in rows {
            let lower: String = value
                .chars()
                .take(PREFIX_LEN)
                .collect::<String>()
                .to_ascii_lowercase();
            // Use the canonical trivial prefix as the grouping key so that
            // "ls", "ls -la", "ls -lah" all aggregate into the "ls" bucket.
            if let Some(canonical) = trivial_prefix(&lower) {
                *grouped
                    .entry((session_id, canonical.to_string()))
                    .or_insert(0) += cnt;
            }
        }

        // Filter to sessions/prefixes that exceed the threshold.
        let mut flagged: Vec<(String, String, i64)> = grouped
            .into_iter()
            .filter(|(_, count)| *count >= MIN_OCCURRENCES as i64)
            .map(|((session_id, prefix), count)| (session_id, prefix, count))
            .collect();

        // Sort deterministically: by count descending, then session_id, then prefix.
        flagged.sort_by(|a, b| {
            b.2.cmp(&a.2)
                .then_with(|| a.0.cmp(&b.0))
                .then_with(|| a.1.cmp(&b.1))
        });

        let findings = flagged
            .into_iter()
            .map(|(session_id, prefix, count)| {
                let session_short = session_id
                    .split_once(':')
                    .map(|(_, raw)| {
                        if raw.len() > 8 {
                            &raw[raw.len() - 8..]
                        } else {
                            raw
                        }
                    })
                    .unwrap_or(session_id.as_str());

                let severity = severity_for_count(count);
                let estimated_monthly_waste_nanos = waste_nanos(count);

                Finding {
                    detector: self.name().into(),
                    severity,
                    title: format!(
                        "Trivial command '{prefix}' repeated {count}× in session {session_short}"
                    ),
                    detail: format!(
                        "The command starting with '{prefix}' was run {count} times in session \
                         '{session_id}'. Repeating trivial shell commands adds token overhead \
                         without contributing to the task outcome. Consider using the appropriate \
                         built-in tool (Read, Glob, Grep) instead of shell commands, or reduce \
                         orientation probes. Estimated waste: {count} × ~{TOKENS_PER_BASH} tokens."
                    ),
                    estimated_monthly_waste_nanos,
                }
            })
            .collect();

        Ok(findings)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizer::mod_tests::empty_db;

    fn insert_bash_event(
        conn: &rusqlite::Connection,
        dedup_key: &str,
        session_id: &str,
        command: &str,
    ) {
        conn.execute(
            "INSERT OR IGNORE INTO tool_events
             (dedup_key, ts_epoch, session_id, provider, project, kind, value, cost_nanos, source_path)
             VALUES (?1, 0, ?2, 'claude', 'proj', 'bash', ?3, 100, '/tmp/t.jsonl')",
            rusqlite::params![dedup_key, session_id, command],
        )
        .unwrap();
    }

    fn det() -> BashNoiseDetector {
        BashNoiseDetector::new()
    }

    // -----------------------------------------------------------------------
    // No-finding cases
    // -----------------------------------------------------------------------

    #[test]
    fn no_bash_events_no_finding() {
        let (_dir, conn) = empty_db();
        assert!(det().run(&conn).unwrap().is_empty());
    }

    #[test]
    fn four_trivial_commands_no_finding() {
        // Threshold is 5; 4 occurrences should not fire.
        let (_dir, conn) = empty_db();
        for i in 0..4 {
            insert_bash_event(&conn, &format!("k{i}"), "claude:s1", "ls -la");
        }
        let findings = det().run(&conn).unwrap();
        assert!(
            findings.is_empty(),
            "4 occurrences should not trigger: {:?}",
            findings
        );
    }

    #[test]
    fn legacy_bash_tool_name_not_flagged() {
        // Legacy rows have value = "Bash" (the tool name, not a command).
        let (_dir, conn) = empty_db();
        for i in 0..10 {
            insert_bash_event(&conn, &format!("k{i}"), "claude:s1", "Bash");
        }
        assert!(det().run(&conn).unwrap().is_empty());
    }

    #[test]
    fn non_trivial_command_not_flagged() {
        let (_dir, conn) = empty_db();
        for i in 0..10 {
            insert_bash_event(
                &conn,
                &format!("k{i}"),
                "claude:s1",
                "cargo build --release",
            );
        }
        assert!(det().run(&conn).unwrap().is_empty());
    }

    // -----------------------------------------------------------------------
    // Detection: severity levels
    // -----------------------------------------------------------------------

    #[test]
    fn five_trivial_commands_triggers_low() {
        let (_dir, conn) = empty_db();
        for i in 0..5 {
            insert_bash_event(&conn, &format!("k{i}"), "claude:s1", "ls");
        }
        let findings = det().run(&conn).unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Low);
        assert!(findings[0].title.contains("ls"));
        assert!(findings[0].title.contains("5"));
    }

    #[test]
    fn eleven_trivial_commands_triggers_medium() {
        let (_dir, conn) = empty_db();
        for i in 0..11 {
            insert_bash_event(&conn, &format!("k{i}"), "claude:s1", "git status");
        }
        let findings = det().run(&conn).unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Medium);
    }

    #[test]
    fn thirty_one_trivial_commands_triggers_high() {
        let (_dir, conn) = empty_db();
        for i in 0..31 {
            insert_bash_event(&conn, &format!("k{i}"), "claude:s1", "pwd");
        }
        let findings = det().run(&conn).unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::High);
    }

    // -----------------------------------------------------------------------
    // Prefix matching
    // -----------------------------------------------------------------------

    #[test]
    fn git_status_short_variant_is_trivial() {
        let (_dir, conn) = empty_db();
        for i in 0..5 {
            insert_bash_event(&conn, &format!("k{i}"), "claude:s1", "git status --short");
        }
        let findings = det().run(&conn).unwrap();
        assert_eq!(findings.len(), 1, "git status --short should be trivial");
    }

    #[test]
    fn prefix_aggregation_across_variants() {
        // "ls" and "ls -la" should both count toward the "ls" prefix bucket.
        let (_dir, conn) = empty_db();
        for i in 0..3 {
            insert_bash_event(&conn, &format!("a{i}"), "claude:s1", "ls");
        }
        for i in 0..3 {
            insert_bash_event(&conn, &format!("b{i}"), "claude:s1", "ls -la");
        }
        // Total 6 "ls*" commands in one session — should trigger.
        let findings = det().run(&conn).unwrap();
        assert_eq!(findings.len(), 1);
        assert!(findings[0].title.contains("ls"));
    }

    // -----------------------------------------------------------------------
    // Waste calculation
    // -----------------------------------------------------------------------

    #[test]
    fn waste_nanos_formula_correct() {
        // 5 × 50 tokens × 20_000_000 / 1000 = 5_000_000
        assert_eq!(waste_nanos(5), 5_000_000);
        // 10 × 50 × 20_000_000 / 1000 = 10_000_000
        assert_eq!(waste_nanos(10), 10_000_000);
    }

    #[test]
    fn estimated_waste_matches_formula() {
        let (_dir, conn) = empty_db();
        for i in 0..5 {
            insert_bash_event(&conn, &format!("k{i}"), "claude:s1", "ls");
        }
        let findings = det().run(&conn).unwrap();
        assert_eq!(findings[0].estimated_monthly_waste_nanos, waste_nanos(5));
    }

    // -----------------------------------------------------------------------
    // Cross-session isolation
    // -----------------------------------------------------------------------

    #[test]
    fn counts_do_not_aggregate_across_sessions() {
        // 4 in s1 + 4 in s2 = no finding (< 5 per session for same prefix).
        let (_dir, conn) = empty_db();
        for i in 0..4 {
            insert_bash_event(&conn, &format!("a{i}"), "claude:s1", "ls");
        }
        for i in 0..4 {
            insert_bash_event(&conn, &format!("b{i}"), "claude:s2", "ls");
        }
        assert!(det().run(&conn).unwrap().is_empty());
    }

    // -----------------------------------------------------------------------
    // is_trivial helper unit tests
    // -----------------------------------------------------------------------

    #[test]
    fn is_trivial_exact_match() {
        assert!(is_trivial("ls"));
        assert!(is_trivial("pwd"));
        assert!(is_trivial("git status"));
        assert!(is_trivial("git log"));
        assert!(is_trivial("git diff"));
        assert!(is_trivial("date"));
        assert!(is_trivial("whoami"));
        assert!(is_trivial("which"));
        assert!(is_trivial("echo"));
        assert!(is_trivial("cat"));
        assert!(is_trivial("head"));
        assert!(is_trivial("tail"));
        assert!(is_trivial("wc"));
        assert!(is_trivial("clear"));
        assert!(is_trivial("cd"));
    }

    #[test]
    fn is_trivial_with_args() {
        assert!(is_trivial("ls -la"));
        assert!(is_trivial("git status --short"));
        assert!(is_trivial("echo hello"));
        assert!(is_trivial("cat file.txt"));
    }

    #[test]
    fn is_trivial_false_for_non_trivial() {
        assert!(!is_trivial("cargo build"));
        assert!(!is_trivial("npm install"));
        assert!(!is_trivial("git commit"));
        assert!(!is_trivial("git push"));
        assert!(!is_trivial("git statuscheck")); // No space after "git status"
    }
}
