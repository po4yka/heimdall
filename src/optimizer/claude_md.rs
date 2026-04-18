//! Detector: CLAUDE.md token bloat.
//!
//! Reads `~/.claude/CLAUDE.md` (or an injected override for tests), estimates
//! its token count, and multiplies by the total session count in the database
//! to arrive at a lifetime token replay cost.
//!
//! # Token estimation
//! Token count is estimated as `char_count / 4`. This is the well-known
//! GPT-style heuristic: on average, one token ≈ 4 characters of English text.
//! It is intentionally coarse — the goal is to flag obviously-large files, not
//! to compute a precise bill. The comment in the code marks this clearly.
//!
//! # Cost formula
//! `estimated_monthly_waste_nanos = (tokens * sessions * 20_000_000) / 1_000`
//!
//! Derivation:
//!   - Rough Sonnet input rate: $0.02 per 1 000 tokens = 20 000 000 nanos / 1 000 tokens
//!   - Formula: tokens × sessions × (20_000_000 nanos / 1_000 tokens)
//!     = tokens × sessions × 20_000 nanos / token
//!
//! Integer arithmetic only; no f64 involved in the cost path.

use std::path::PathBuf;

use anyhow::Result;
use rusqlite::Connection;

use super::{Detector, Finding, Severity};

/// Severity thresholds for token × session product.
const THRESHOLD_MEDIUM: i64 = 1_000_000;
const THRESHOLD_HIGH: i64 = 5_000_000;

pub struct ClaudeMdBloatDetector {
    /// Override path for tests. If `None`, defaults to `~/.claude/CLAUDE.md`.
    path_override: Option<PathBuf>,
}

impl ClaudeMdBloatDetector {
    pub fn new(path_override: Option<PathBuf>) -> Self {
        Self { path_override }
    }

    fn claude_md_path(&self) -> Option<PathBuf> {
        if let Some(p) = &self.path_override {
            return Some(p.clone());
        }
        dirs::home_dir().map(|h| h.join(".claude").join("CLAUDE.md"))
    }
}

impl Detector for ClaudeMdBloatDetector {
    fn name(&self) -> &'static str {
        "claude_md_bloat"
    }

    fn run(&self, conn: &Connection) -> Result<Vec<Finding>> {
        let path = match self.claude_md_path() {
            Some(p) => p,
            None => return Ok(vec![]),
        };

        // Read the file; missing or empty → no finding.
        let contents = match std::fs::read_to_string(&path) {
            Ok(s) if !s.trim().is_empty() => s,
            _ => return Ok(vec![]),
        };

        // Token estimation: char_count / 4 (GPT-style heuristic, intentionally rough).
        let token_estimate = (contents.chars().count() as i64) / 4;
        if token_estimate == 0 {
            return Ok(vec![]);
        }

        // Count sessions in the DB.
        let session_count: i64 =
            conn.query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0))?;

        if session_count == 0 {
            return Ok(vec![]);
        }

        let product = token_estimate * session_count;

        let severity = if product >= THRESHOLD_HIGH {
            Severity::High
        } else if product >= THRESHOLD_MEDIUM {
            Severity::Medium
        } else {
            Severity::Low
        };

        // Cost formula: (tokens * sessions * 20_000_000) / 1_000 nanos
        // = tokens * sessions * 20_000 nanos  (20_000 nanos per token per session)
        // Rough Sonnet input rate: $0.02 per 1K tokens = 20_000_000 nanos / 1_000 tokens
        let estimated_monthly_waste_nanos = (token_estimate * session_count * 20_000_000) / 1_000;

        let title = format!(
            "CLAUDE.md is {token_estimate} tokens, replayed across {session_count} sessions"
        );
        let detail = format!(
            "Every new Claude Code session loads CLAUDE.md into the context window. \
             At ~{token_estimate} tokens per load across {session_count} sessions, \
             this file has consumed roughly {} tokens in total. \
             Consider trimming infrequently-used sections or splitting into project-specific \
             CLAUDE.md files closer to where they are needed.",
            token_estimate * session_count
        );

        Ok(vec![Finding {
            detector: self.name().into(),
            severity,
            title,
            detail,
            estimated_monthly_waste_nanos,
        }])
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::optimizer::mod_tests::{empty_db, insert_sessions, write_temp_file};

    fn detector_for(path: &Path) -> ClaudeMdBloatDetector {
        ClaudeMdBloatDetector::new(Some(path.to_path_buf()))
    }

    // -----------------------------------------------------------------------
    // No finding cases
    // -----------------------------------------------------------------------

    #[test]
    fn missing_file_produces_no_finding() {
        let (dir, conn) = empty_db();
        insert_sessions(&conn, 10);
        let det = ClaudeMdBloatDetector::new(Some(dir.path().join("nonexistent.md")));
        let findings = det.run(&conn).unwrap();
        assert!(findings.is_empty());
    }

    #[test]
    fn empty_file_produces_no_finding() {
        let (dir, conn) = empty_db();
        insert_sessions(&conn, 10);
        let path = write_temp_file(&dir, "CLAUDE.md", "");
        let findings = detector_for(&path).run(&conn).unwrap();
        assert!(findings.is_empty());
    }

    #[test]
    fn whitespace_only_file_produces_no_finding() {
        let (dir, conn) = empty_db();
        insert_sessions(&conn, 10);
        let path = write_temp_file(&dir, "CLAUDE.md", "   \n\t\n  ");
        let findings = detector_for(&path).run(&conn).unwrap();
        assert!(findings.is_empty());
    }

    #[test]
    fn zero_sessions_produces_no_finding() {
        let (dir, conn) = empty_db();
        // No sessions inserted.
        let path = write_temp_file(&dir, "CLAUDE.md", "x".repeat(4000).as_str());
        let findings = detector_for(&path).run(&conn).unwrap();
        assert!(findings.is_empty());
    }

    // -----------------------------------------------------------------------
    // Severity thresholds
    // -----------------------------------------------------------------------

    #[test]
    fn low_severity_below_1m_product() {
        // 4000 chars => 1000 tokens; 100 sessions => product = 100_000 (< 1M)
        let (dir, conn) = empty_db();
        insert_sessions(&conn, 100);
        let path = write_temp_file(&dir, "CLAUDE.md", &"x".repeat(4000));
        let findings = detector_for(&path).run(&conn).unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Low);
    }

    #[test]
    fn medium_severity_between_1m_and_5m_product() {
        // 8000 chars => 2000 tokens; 1000 sessions => product = 2_000_000 (1M..5M)
        let (dir, conn) = empty_db();
        insert_sessions(&conn, 1000);
        let path = write_temp_file(&dir, "CLAUDE.md", &"x".repeat(8000));
        let findings = detector_for(&path).run(&conn).unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Medium);
    }

    #[test]
    fn high_severity_above_5m_product() {
        // 500_000 chars => 125_000 tokens; 1000 sessions => product = 125_000_000 (> 5M)
        let (dir, conn) = empty_db();
        insert_sessions(&conn, 1000);
        let path = write_temp_file(&dir, "CLAUDE.md", &"x".repeat(500_000));
        let findings = detector_for(&path).run(&conn).unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::High);
    }

    // -----------------------------------------------------------------------
    // Integration: 4000 chars + 100 sessions => low; 500K chars + 1000 sessions => high
    // -----------------------------------------------------------------------

    #[test]
    fn integration_low_scenario() {
        // ~1000 tokens × 100 sessions = 100_000 product => low
        let (dir, conn) = empty_db();
        insert_sessions(&conn, 100);
        let path = write_temp_file(&dir, "CLAUDE.md", &"x".repeat(4000));
        let findings = detector_for(&path).run(&conn).unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Low);
        assert_eq!(findings[0].detector, "claude_md_bloat");
        // Waste = 1000 * 100 * 20_000_000 / 1000 = 2_000_000_000 nanos
        assert_eq!(findings[0].estimated_monthly_waste_nanos, 2_000_000_000);
    }

    #[test]
    fn integration_high_scenario() {
        // 500K chars => 125_000 tokens × 1000 sessions = 125_000_000 product => high
        let (dir, conn) = empty_db();
        insert_sessions(&conn, 1000);
        let path = write_temp_file(&dir, "CLAUDE.md", &"x".repeat(500_000));
        let findings = detector_for(&path).run(&conn).unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::High);
        // Waste = 125_000 * 1000 * 20_000_000 / 1000 = 2_500_000_000_000 nanos
        assert_eq!(
            findings[0].estimated_monthly_waste_nanos,
            2_500_000_000_000_i64
        );
    }

    #[test]
    fn finding_title_contains_token_and_session_count() {
        let (dir, conn) = empty_db();
        insert_sessions(&conn, 5);
        let path = write_temp_file(&dir, "CLAUDE.md", &"x".repeat(400)); // 100 tokens
        let findings = detector_for(&path).run(&conn).unwrap();
        assert_eq!(findings.len(), 1);
        let title = &findings[0].title;
        assert!(title.contains("100 tokens"), "title: {title}");
        assert!(title.contains("5 sessions"), "title: {title}");
    }
}
