//! Detector: excessive bypass events.
//!
//! Queries `hook_events` for bypass outcomes over the last 7 days.
//! Frequent bypass events indicate that Claude Code is being invoked with
//! `--dangerously-skip-permissions`, which skips the hook entirely and
//! may indicate a misconfigured workflow or security-posture concern.
//!
//! # Severity thresholds
//! | Bypasses/week | Severity |
//! |---------------|----------|
//! | 1–30          | Low      |
//! | 31–100        | Medium   |
//! | >100          | High     |

use anyhow::Result;
use rusqlite::Connection;

use super::{Detector, Finding, Severity};

pub struct HookBypassDetector;

impl HookBypassDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for HookBypassDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for HookBypassDetector {
    fn name(&self) -> &'static str {
        "hook_bypass"
    }

    fn run(&self, conn: &Connection) -> Result<Vec<Finding>> {
        use chrono::{Duration, Utc};

        let cutoff_epoch = (Utc::now() - Duration::days(6)).timestamp();

        let bypass_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM hook_events WHERE outcome = 'bypass' AND ts_epoch >= ?1",
            rusqlite::params![cutoff_epoch],
            |r| r.get(0),
        )?;

        if bypass_count == 0 {
            return Ok(vec![]);
        }

        // Get top ancestor command for detail string.
        let top_cmd: Option<String> = conn
            .query_row(
                "SELECT bypass_ancestor_command
                 FROM hook_events
                 WHERE outcome = 'bypass' AND ts_epoch >= ?1 AND bypass_ancestor_command IS NOT NULL
                 GROUP BY bypass_ancestor_command
                 ORDER BY COUNT(*) DESC
                 LIMIT 1",
                rusqlite::params![cutoff_epoch],
                |r| r.get(0),
            )
            .ok();

        let severity = if bypass_count > 100 {
            Severity::High
        } else if bypass_count > 30 {
            Severity::Medium
        } else {
            Severity::Low
        };

        let ancestor_note = top_cmd
            .as_deref()
            .map(|c| format!(" Most frequent ancestor: '{c}'."))
            .unwrap_or_default();

        Ok(vec![Finding {
            detector: self.name().into(),
            severity,
            title: format!("{bypass_count} bypass event(s) in the last 7 days"),
            detail: format!(
                "heimdall-hook was bypassed {bypass_count} time(s) in the last 7 days because \
                 an ancestor process used `--dangerously-skip-permissions`.{ancestor_note} \
                 Bypass events are not ingested into the DB, so hook telemetry and cost \
                 attribution are incomplete for those sessions. Review whether the ancestor \
                 tool needs this flag.",
            ),
            estimated_monthly_waste_nanos: 0,
        }])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizer::mod_tests::empty_db;

    fn insert_bypass(conn: &Connection, cmd: Option<&str>) {
        let now_epoch = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT INTO hook_events
                (received_at, ts_epoch, outcome, latency_us,
                 bypass_ancestor_command, cli_version)
             VALUES ('2024-01-01T00:00:00Z', ?1, 'bypass', 5000, ?2, 'test')",
            rusqlite::params![now_epoch, cmd],
        )
        .unwrap();
    }

    #[test]
    fn no_bypasses_no_finding() {
        let (_dir, conn) = empty_db();
        assert!(HookBypassDetector::new().run(&conn).unwrap().is_empty());
    }

    #[test]
    fn one_bypass_triggers_low() {
        let (_dir, conn) = empty_db();
        insert_bypass(&conn, Some("code --dangerously-skip-permissions"));
        let findings = HookBypassDetector::new().run(&conn).unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Low);
        assert_eq!(findings[0].estimated_monthly_waste_nanos, 0);
        assert!(findings[0].detail.contains("dangerously-skip-permissions"));
    }

    #[test]
    fn over_100_triggers_high() {
        let (_dir, conn) = empty_db();
        for _ in 0..101 {
            insert_bypass(&conn, None);
        }
        let findings = HookBypassDetector::new().run(&conn).unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::High);
    }
}
