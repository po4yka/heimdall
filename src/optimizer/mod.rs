//! ROADMAP Phase 6 (complete) -- `optimize` waste detector framework.
//!
//! Implements all five detectors:
//!   - `ClaudeMdBloatDetector`  (`claude_md.rs`) -- CLAUDE.md token cost × session count
//!   - `UnusedMcpDetector`      (`mcp.rs`)        -- MCP servers configured but never invoked
//!   - `GhostAgentDetector`     (`agents.rs`)     -- agent .md files never referenced in turns
//!   - `RereadDetector`         (`reread.rs`)     -- same file read ≥3× per session
//!   - `BashNoiseDetector`      (`bash.rs`)       -- trivial shell commands repeated ≥5× per session
//!
//! The `RereadDetector` and `BashNoiseDetector` require tool-argument data in `tool_events.value`.
//! The scanner now populates `value` with the actual file path (for file tools) or the command
//! text (for Bash).  Legacy DB rows where `value` equals the tool name are automatically skipped
//! by both detectors so no false-positive findings are produced on old data.

pub mod agents;
pub mod bash;
pub mod claude_md;
pub mod grade;
pub mod mcp;
pub mod reread;

use std::path::Path;

use anyhow::Result;
use rusqlite::Connection;
use serde::Serialize;

use crate::scanner::db::open_db;

// ---------------------------------------------------------------------------
// Core types
// ---------------------------------------------------------------------------

/// Severity of a finding, used for grading and display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Low,
    Medium,
    High,
}

/// A single waste-detector finding.
#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    /// Stable slug identifying the detector, e.g. `"claude_md_bloat"`.
    pub detector: String,
    pub severity: Severity,
    /// Short human-readable title, shown in the text report header.
    pub title: String,
    /// Longer explanation or recommendation.
    pub detail: String,
    /// Estimated monthly waste in integer nanos (billionths of a dollar).
    /// Zero when the waste is qualitative (configuration clutter) rather than
    /// directly token-costed.
    pub estimated_monthly_waste_nanos: i64,
}

/// Aggregate result returned by `run_optimize`.
#[derive(Debug, Serialize)]
pub struct OptimizeReport {
    pub findings: Vec<Finding>,
    /// A–F letter grade computed from the findings.
    pub grade: char,
    /// Sum of `estimated_monthly_waste_nanos` across all findings.
    pub total_monthly_waste_nanos: i64,
}

// ---------------------------------------------------------------------------
// Detector trait
// ---------------------------------------------------------------------------

/// Every waste detector implements this trait.
pub trait Detector {
    /// Stable slug used as `Finding::detector`. Must be unique across detectors.
    fn name(&self) -> &'static str;

    /// Run the detector against the live database and return any findings.
    /// Returning an empty `Vec` means the detector found nothing to report.
    fn run(&self, conn: &Connection) -> Result<Vec<Finding>>;
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Open `db_path`, run all registered detectors, and return the aggregate report.
///
/// Exit code semantics: callers should exit 0 always; only I/O errors from this
/// function warrant a non-zero exit.
pub fn run_optimize(db_path: &Path) -> Result<OptimizeReport> {
    run_optimize_with_overrides(db_path, None, None, None)
}

/// Like `run_optimize` but accepts optional path overrides for each detector,
/// used by tests to point at fixture files instead of `~/.claude/`.
pub fn run_optimize_with_overrides(
    db_path: &Path,
    claude_md_path: Option<&Path>,
    settings_json_path: Option<&Path>,
    agents_dir_path: Option<&Path>,
) -> Result<OptimizeReport> {
    let conn = open_db(db_path)?;

    let detectors: Vec<Box<dyn Detector>> = vec![
        Box::new(claude_md::ClaudeMdBloatDetector::new(
            claude_md_path.map(Path::to_path_buf),
        )),
        Box::new(mcp::UnusedMcpDetector::new(
            settings_json_path.map(Path::to_path_buf),
        )),
        Box::new(agents::GhostAgentDetector::new(
            agents_dir_path.map(Path::to_path_buf),
        )),
        Box::new(reread::RereadDetector::new()),
        Box::new(bash::BashNoiseDetector::new()),
    ];

    let mut findings = Vec::new();
    for detector in &detectors {
        match detector.run(&conn) {
            Ok(mut f) => findings.append(&mut f),
            Err(e) => {
                tracing::warn!("Detector '{}' failed: {}", detector.name(), e);
            }
        }
    }

    let grade = grade::compute_grade(&findings);
    let total_monthly_waste_nanos = findings
        .iter()
        .map(|f| f.estimated_monthly_waste_nanos)
        .sum();

    Ok(OptimizeReport {
        findings,
        grade,
        total_monthly_waste_nanos,
    })
}

// ---------------------------------------------------------------------------
// Shared test helpers (accessible from child detector modules via
// `crate::optimizer::mod_tests::{empty_db, insert_sessions, write_temp_file}`)
// ---------------------------------------------------------------------------

#[cfg(test)]
pub(crate) mod mod_tests {
    use std::io::Write;

    use rusqlite::Connection;
    use tempfile::TempDir;

    use crate::scanner::db::init_db;

    /// Build an empty but properly initialised DB in a temp dir.
    /// Returns both the TempDir (so it stays alive) and the Connection.
    pub fn empty_db() -> (TempDir, Connection) {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = Connection::open(&db_path).unwrap();
        init_db(&conn).unwrap();
        (dir, conn)
    }

    /// Insert `n` minimal sessions into the DB.
    pub fn insert_sessions(conn: &Connection, n: usize) {
        for i in 0..n {
            conn.execute(
                "INSERT OR IGNORE INTO sessions
                 (session_id, provider, first_timestamp, last_timestamp)
                 VALUES (?1, 'claude', '2024-01-01T00:00:00Z', '2024-01-01T01:00:00Z')",
                rusqlite::params![format!("claude:session_{i}")],
            )
            .unwrap();
        }
    }

    /// Write a temp file and return its path.
    pub fn write_temp_file(dir: &TempDir, name: &str, contents: &str) -> std::path::PathBuf {
        let path = dir.path().join(name);
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(contents.as_bytes()).unwrap();
        path
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use mod_tests::{empty_db, insert_sessions, write_temp_file};

    // -----------------------------------------------------------------------
    // run_optimize on a clean setup
    // -----------------------------------------------------------------------

    #[test]
    fn clean_setup_produces_no_findings_and_grade_a() {
        let (dir, _conn) = empty_db();
        let db_path = dir.path().join("test.db");

        // Point each detector at a nonexistent path so no config files are found.
        let nonexistent = dir.path().join("nonexistent");
        let report = run_optimize_with_overrides(
            &db_path,
            Some(&nonexistent.join("CLAUDE.md")),
            Some(&nonexistent.join("settings.json")),
            Some(&nonexistent.join("agents")),
        )
        .unwrap();

        assert!(
            report.findings.is_empty(),
            "expected no findings: {:?}",
            report.findings
        );
        assert_eq!(report.grade, 'A');
        assert_eq!(report.total_monthly_waste_nanos, 0);
    }

    // -----------------------------------------------------------------------
    // run_optimize: aggregates findings from multiple detectors
    // -----------------------------------------------------------------------

    #[test]
    fn multiple_detectors_aggregate_correctly() {
        let (dir, _conn) = empty_db();
        let db_path = dir.path().join("test.db");

        // CLAUDE.md with content + sessions so claude_md detector fires.
        let claude_md = mod_tests::write_temp_file(&dir, "CLAUDE.md", &"x".repeat(4000));
        // Insert 100 sessions via a fresh connection.
        {
            let conn = rusqlite::Connection::open(&db_path).unwrap();
            insert_sessions(&conn, 100);
        }

        // settings.json with one unused MCP server.
        let settings = mod_tests::write_temp_file(
            &dir,
            "settings.json",
            r#"{"mcpServers": {"unused-srv": {}}}"#,
        );

        // Agents dir with one ghost agent.
        let adir = dir.path().join("agents");
        std::fs::create_dir_all(&adir).unwrap();
        std::fs::write(adir.join("ghost.md"), "# ghost").unwrap();

        let report =
            run_optimize_with_overrides(&db_path, Some(&claude_md), Some(&settings), Some(&adir))
                .unwrap();

        // Should have at least 3 findings (one per detector).
        assert!(
            report.findings.len() >= 3,
            "expected >=3 findings, got {}",
            report.findings.len()
        );
        let total: i64 = report
            .findings
            .iter()
            .map(|f| f.estimated_monthly_waste_nanos)
            .sum();
        assert_eq!(report.total_monthly_waste_nanos, total);
    }

    // -----------------------------------------------------------------------
    // JSON schema stability (golden test)
    // -----------------------------------------------------------------------

    #[test]
    fn optimize_report_json_schema() {
        let finding = Finding {
            detector: "claude_md_bloat".into(),
            severity: Severity::High,
            title: "CLAUDE.md is 1000 tokens, replayed across 500 sessions".into(),
            detail: "Estimated waste: $5.00/month".into(),
            estimated_monthly_waste_nanos: 5_000_000_000,
        };
        let report = OptimizeReport {
            findings: vec![finding],
            grade: 'F',
            total_monthly_waste_nanos: 5_000_000_000,
        };
        let json = serde_json::to_value(&report).unwrap();

        // Verify top-level keys exist and have the right types.
        assert!(json["findings"].is_array());
        assert!(json["grade"].is_string());
        assert!(json["total_monthly_waste_nanos"].is_number());

        // Verify finding schema keys and types.
        let f = &json["findings"][0];
        assert_eq!(f["detector"], "claude_md_bloat");
        assert_eq!(f["severity"], "high");
        assert!(f["title"].is_string());
        assert!(f["detail"].is_string());
        assert!(f["estimated_monthly_waste_nanos"].is_number());

        // grade is serialised as a single-character string.
        assert_eq!(json["grade"].as_str().unwrap().len(), 1);
    }

    // -----------------------------------------------------------------------
    // Unused import guard — ensure write_temp_file is referenced
    // -----------------------------------------------------------------------

    #[test]
    fn write_temp_file_helper_works() {
        let (dir, _conn) = empty_db();
        let p = write_temp_file(&dir, "hello.txt", "hello");
        assert!(p.exists());
    }
}
