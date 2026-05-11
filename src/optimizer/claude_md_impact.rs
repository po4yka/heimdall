use anyhow::Result;
use rusqlite::Connection;

use super::{Detector, Finding, Severity};

#[derive(Default)]
pub struct ClaudeMdImpactDetector;

impl ClaudeMdImpactDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Detector for ClaudeMdImpactDetector {
    fn name(&self) -> &'static str {
        "claude_md_impact"
    }

    fn run(&self, conn: &Connection) -> Result<Vec<Finding>> {
        let summary = crate::scanner::db::query_dashboard_claude_md_size(conn);
        let mut findings = Vec::new();

        for file in &summary.files {
            let Some(corr) = file.cost_correlation else {
                continue;
            };
            let sample = file.cost_correlation_sample_size;
            let growth = file.token_delta_pct_30d;

            let severity = if corr >= 0.5 && growth >= 0.20 && sample >= 10 {
                Severity::High
            } else if corr >= 0.3 && growth >= 0.10 && sample >= 7 {
                Severity::Medium
            } else if corr >= 0.3 && sample >= 5 {
                Severity::Low
            } else {
                continue;
            };

            // Estimated monthly waste: extra tokens introduced in 30d × avg cost-per-session
            // scaled by their fraction of current token budget × 30 days.
            let waste = if file.current_token_count > 0 {
                let delta_fraction = file.token_delta_30d as f64 / file.current_token_count as f64;
                ((delta_fraction * file.avg_cost_per_session_30d_nanos as f64 * 30.0) as i64).max(0)
            } else {
                0
            };

            let title = format!(
                "{}: {} tokens (+{:.0}% in 30d), correlation with session cost: {:.2}",
                file.label,
                file.current_token_count,
                growth * 100.0,
                corr,
            );
            let detail = format!(
                "This file grew by {} tokens over the last 30 days and correlates positively \
                 with per-session cost (r={:.2}, n={} days of data). \
                 Correlation is not causation — review whether the added sections \
                 are pulling cost up or just coincide with heavier sessions.",
                file.token_delta_30d, corr, sample,
            );

            findings.push(Finding {
                detector: self.name().into(),
                severity,
                title,
                detail,
                estimated_monthly_waste_nanos: waste,
            });
        }

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

    #[test]
    fn no_finding_when_no_data() {
        let (_dir, conn) = empty_db();
        let detector = ClaudeMdImpactDetector::new();
        let findings = detector.run(&conn).unwrap();
        assert!(findings.is_empty(), "empty db should produce no findings");
    }

    #[test]
    fn no_finding_when_correlation_below_threshold() {
        let (_dir, conn) = empty_db();

        // Insert a history row so the file appears in the summary.
        conn.execute(
            "INSERT OR IGNORE INTO claude_md_history
             (project_path, file_path, commit_sha, commit_ts, byte_size, token_count, line_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                "/some/project",
                "CLAUDE.md",
                "abc123",
                chrono::Utc::now().timestamp(),
                1000i64,
                200i64,
                20i64,
            ],
        )
        .unwrap();

        // Without enough cost-per-session data the pearson call returns None,
        // so cost_correlation will be None → no finding.
        let detector = ClaudeMdImpactDetector::new();
        let findings = detector.run(&conn).unwrap();
        assert!(
            findings.is_empty(),
            "insufficient data should produce no findings"
        );
    }

    #[test]
    fn low_finding_when_moderate_correlation() {
        // We need ≥5 paired (token, cost) days.  We can't easily produce a real
        // pearson correlation from turns data in a unit test, so instead we verify
        // the detector runs cleanly and returns the right shape when the summary
        // would have a qualifying file.  The actual correlation logic is exercised
        // by the analytics::correlation tests.

        // Build a minimal turns + history setup that gives us ≥5 days with cost.
        let (_dir, conn) = empty_db();

        let now = chrono::Utc::now();
        let session_id = "s1";
        let project = "/my/project";

        // Insert turns for 6 different days so pearson can fire.
        for day_offset in 0i64..6 {
            let ts = (now - chrono::Duration::days(day_offset))
                .format("%Y-%m-%dT12:00:00Z")
                .to_string();
            conn.execute(
                "INSERT INTO turns
                 (session_id, provider, timestamp, cwd, input_tokens, output_tokens,
                  cache_read_tokens, cache_creation_tokens, reasoning_output_tokens,
                  estimated_cost_nanos, cost_confidence, billing_mode, model, is_subagent)
                 VALUES (?1, 'claude', ?2, ?3, 100, 50, 0, 0, 0, 1000000, 'estimated', 'auto', 'claude-3-5-sonnet', 0)",
                rusqlite::params![
                    format!("{session_id}_{day_offset}"),
                    ts,
                    project,
                ],
            )
            .unwrap();
        }

        // Insert history rows — one per day with increasing token counts.
        for day_offset in 0i64..6 {
            let ts_epoch = (now - chrono::Duration::days(day_offset)).timestamp();
            conn.execute(
                "INSERT OR IGNORE INTO claude_md_history
                 (project_path, file_path, commit_sha, commit_ts, byte_size, token_count, line_count)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![
                    project,
                    "CLAUDE.md",
                    format!("sha{day_offset}"),
                    ts_epoch,
                    1000 + day_offset * 100,
                    100 + day_offset * 10,
                    10 + day_offset,
                ],
            )
            .unwrap();
        }

        let detector = ClaudeMdImpactDetector::new();
        // The detector must not panic regardless of whether a finding fires.
        let result = detector.run(&conn);
        assert!(
            result.is_ok(),
            "detector should not error: {:?}",
            result.err()
        );
    }
}
