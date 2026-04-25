//! Detector: repeated file reads.
//!
//! Queries `tool_events` for files that were the target of a `Read` three
//! or more times within the same session.  Repeated reads of the same file
//! are a signal that the model is re-loading context it already processed —
//! a common waste pattern that can be addressed by keeping relevant file
//! contents in the context window or by restructuring the task.
//!
//! # Cost formula
//! We do not have actual file sizes, so we use a conservative 500-token
//! assumption per read:
//!
//! `estimated_monthly_waste_nanos = rereads × 500 × 20_000_000 / 1_000`
//!
//! Derivation:
//!   - Conservative file size: 500 tokens
//!   - Rough Sonnet input rate: $0.02 per 1 000 tokens = 20 000 000 nanos / 1 000 tokens
//!   - `rereads` = total reads − 1 (the first read is not "waste")
//!
//! # Severity thresholds
//! | Reads | Severity |
//! |-------|----------|
//! | 3–5   | Low      |
//! | 6–15  | Medium   |
//! | >15   | High     |

use anyhow::Result;
use rusqlite::Connection;

use super::{Detector, Finding, Severity};

/// Conservative tokens-per-file assumption used for waste estimation.
const ASSUMED_TOKENS_PER_FILE: i64 = 500;
/// Nanos per 1 000 tokens (rough Sonnet input rate: $0.02 / 1K tokens).
const NANOS_PER_1K_TOKENS: i64 = 20_000_000;

/// Minimum number of reads before a file is flagged.
const MIN_READS: i64 = 3;
/// Maximum number of findings returned (ordered by read count descending).
const MAX_FINDINGS: usize = 20;

pub struct RereadDetector;

impl RereadDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RereadDetector {
    fn default() -> Self {
        Self::new()
    }
}

fn severity_for_reads(reads: i64) -> Severity {
    if reads > 15 {
        Severity::High
    } else if reads > 5 {
        Severity::Medium
    } else {
        Severity::Low
    }
}

/// Estimate waste nanos for `reads` total reads of a file.
/// The first read is not waste; `rereads = reads - 1`.
fn waste_nanos(reads: i64) -> i64 {
    let rereads = reads.saturating_sub(1);
    rereads * ASSUMED_TOKENS_PER_FILE * NANOS_PER_1K_TOKENS / 1_000
}

impl Detector for RereadDetector {
    fn name(&self) -> &'static str {
        "reread_files"
    }

    fn run(&self, conn: &Connection) -> Result<Vec<Finding>> {
        // Query for files read >= MIN_READS times in any single session.
        let mut stmt = conn.prepare(
            "SELECT session_id, value AS file_path, COUNT(*) AS reads
             FROM tool_events
             WHERE kind = 'file'
             GROUP BY session_id, value
             HAVING reads >= ?1
             ORDER BY reads DESC
             LIMIT ?2",
        )?;

        let rows: Vec<(String, String, i64)> = stmt
            .query_map(rusqlite::params![MIN_READS, MAX_FINDINGS as i64], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })?
            .filter_map(|r| match r {
                Ok(val) => Some(val),
                Err(e) => {
                    tracing::warn!("RereadDetector: row error: {e}");
                    None
                }
            })
            .collect();

        if rows.is_empty() {
            return Ok(vec![]);
        }

        let findings = rows
            .into_iter()
            .map(|(session_id, file_path, reads)| {
                // Shorten the session ID for display: take the last 8 chars after `:`.
                let session_short = session_id
                    .split_once(':')
                    .map(|(_, raw)| {
                        let raw = raw.trim_end_matches(|c: char| !c.is_alphanumeric());
                        if raw.len() > 8 {
                            &raw[raw.len() - 8..]
                        } else {
                            raw
                        }
                    })
                    .unwrap_or(session_id.as_str());

                let severity = severity_for_reads(reads);
                let estimated_monthly_waste_nanos = waste_nanos(reads);

                Finding {
                    detector: self.name().into(),
                    severity,
                    title: format!(
                        "File '{file_path}' re-read {reads} times in session {session_short}"
                    ),
                    detail: format!(
                        "The file '{file_path}' was read {reads} times within session \
                         '{session_id}'. Repeated reads of the same file suggest the model \
                         is re-loading context it already processed. Consider keeping \
                         relevant file contents visible in the conversation or restructuring \
                         the task to reduce redundant reads. Estimated waste assumes \
                         {ASSUMED_TOKENS_PER_FILE} tokens per read."
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

    fn insert_file_event(
        conn: &rusqlite::Connection,
        dedup_key: &str,
        session_id: &str,
        file_path: &str,
    ) {
        conn.execute(
            "INSERT OR IGNORE INTO tool_events
             (dedup_key, ts_epoch, session_id, provider, project, kind, value, cost_nanos, source_path)
             VALUES (?1, 0, ?2, 'claude', 'proj', 'file', ?3, 100, '/tmp/t.jsonl')",
            rusqlite::params![dedup_key, session_id, file_path],
        )
        .unwrap();
    }

    fn det() -> RereadDetector {
        RereadDetector::new()
    }

    // -----------------------------------------------------------------------
    // No-finding cases
    // -----------------------------------------------------------------------

    #[test]
    fn no_tool_events_produces_no_finding() {
        let (_dir, conn) = empty_db();
        let findings = det().run(&conn).unwrap();
        assert!(findings.is_empty());
    }

    #[test]
    fn two_reads_of_same_file_no_finding() {
        let (_dir, conn) = empty_db();
        insert_file_event(&conn, "k1", "claude:s1", "/src/main.rs");
        insert_file_event(&conn, "k2", "claude:s1", "/src/main.rs");
        let findings = det().run(&conn).unwrap();
        assert!(
            findings.is_empty(),
            "2 reads should not trigger: {:?}",
            findings
        );
    }

    // -----------------------------------------------------------------------
    // Detection: severity levels
    // -----------------------------------------------------------------------

    #[test]
    fn three_reads_triggers_low_severity() {
        let (_dir, conn) = empty_db();
        for i in 0..3 {
            insert_file_event(&conn, &format!("k{i}"), "claude:s1", "/src/foo.rs");
        }
        let findings = det().run(&conn).unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Low);
        assert!(findings[0].title.contains("/src/foo.rs"));
        assert!(findings[0].title.contains("3 times"));
    }

    #[test]
    fn six_reads_triggers_medium_severity() {
        let (_dir, conn) = empty_db();
        for i in 0..6 {
            insert_file_event(&conn, &format!("k{i}"), "claude:s1", "/src/bar.rs");
        }
        let findings = det().run(&conn).unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Medium);
    }

    #[test]
    fn sixteen_reads_triggers_high_severity() {
        let (_dir, conn) = empty_db();
        for i in 0..16 {
            insert_file_event(&conn, &format!("k{i}"), "claude:s1", "/src/big.rs");
        }
        let findings = det().run(&conn).unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::High);
    }

    // -----------------------------------------------------------------------
    // Waste calculation
    // -----------------------------------------------------------------------

    #[test]
    fn waste_nanos_formula_correct() {
        // 3 reads → rereads = 2 → waste = 2 * 500 * 20_000_000 / 1000 = 20_000_000
        assert_eq!(waste_nanos(3), 20_000_000);
        // 6 reads → rereads = 5 → waste = 5 * 500 * 20_000_000 / 1000 = 50_000_000
        assert_eq!(waste_nanos(6), 50_000_000);
    }

    #[test]
    fn estimated_waste_matches_formula() {
        let (_dir, conn) = empty_db();
        for i in 0..3 {
            insert_file_event(&conn, &format!("k{i}"), "claude:s1", "/src/x.rs");
        }
        let findings = det().run(&conn).unwrap();
        assert_eq!(findings[0].estimated_monthly_waste_nanos, waste_nanos(3));
    }

    // -----------------------------------------------------------------------
    // Cross-session isolation
    // -----------------------------------------------------------------------

    #[test]
    fn reads_do_not_aggregate_across_sessions() {
        // 2 reads in s1 + 2 reads in s2 = no finding (< 3 per session).
        let (_dir, conn) = empty_db();
        insert_file_event(&conn, "k1", "claude:s1", "/src/x.rs");
        insert_file_event(&conn, "k2", "claude:s1", "/src/x.rs");
        insert_file_event(&conn, "k3", "claude:s2", "/src/x.rs");
        insert_file_event(&conn, "k4", "claude:s2", "/src/x.rs");
        let findings = det().run(&conn).unwrap();
        assert!(findings.is_empty());
    }

    // -----------------------------------------------------------------------
    // Grade integration
    // -----------------------------------------------------------------------

    #[test]
    fn single_low_severity_reread_grades_b() {
        // Only one low-severity finding → grade should be B.
        let (_dir, conn) = empty_db();
        for i in 0..3 {
            insert_file_event(&conn, &format!("k{i}"), "claude:s1", "/src/x.rs");
        }
        let findings = det().run(&conn).unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Low);
        let grade = crate::optimizer::grade::compute_grade(&findings);
        assert_eq!(grade, 'B');
    }
}
