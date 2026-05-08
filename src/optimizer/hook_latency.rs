//! Detector: hook latency anomaly.
//!
//! Queries `hook_events` for p95 latency over the last 30 days.
//! High p95 latency indicates that the hook binary is slow, which impacts
//! the interactive responsiveness of every Claude Code tool invocation.
//!
//! # Severity thresholds
//! | p95 latency   | Severity |
//! |---------------|----------|
//! | 100–200ms     | Low      |
//! | 200–500ms     | Medium   |
//! | >500ms        | High     |

use anyhow::Result;
use rusqlite::Connection;

use super::{Detector, Finding, Severity};

pub struct HookLatencyDetector;

impl HookLatencyDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for HookLatencyDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for HookLatencyDetector {
    fn name(&self) -> &'static str {
        "hook_latency"
    }

    fn run(&self, conn: &Connection) -> Result<Vec<Finding>> {
        use chrono::{Duration, Utc};

        let cutoff_epoch = (Utc::now() - Duration::days(29)).timestamp();

        let mut stmt = conn.prepare(
            "SELECT latency_us FROM hook_events
             WHERE ts_epoch >= ?1
             ORDER BY latency_us ASC",
        )?;

        let latencies: Vec<i64> = stmt
            .query_map(rusqlite::params![cutoff_epoch], |r| r.get(0))?
            .filter_map(|r| match r {
                Ok(v) => Some(v),
                Err(e) => {
                    tracing::warn!("HookLatencyDetector row error: {e}");
                    None
                }
            })
            .collect();

        if latencies.len() < 10 {
            return Ok(vec![]);
        }

        let p95_idx = ((latencies.len() as f64 - 1.0) * 0.95).round() as usize;
        let p95_us = latencies[p95_idx.min(latencies.len() - 1)];
        let p95_ms = p95_us / 1_000;

        let (severity, threshold_ms) = if p95_ms > 500 {
            (Severity::High, 500u64)
        } else if p95_ms > 200 {
            (Severity::Medium, 200u64)
        } else if p95_ms > 100 {
            (Severity::Low, 100u64)
        } else {
            return Ok(vec![]);
        };

        Ok(vec![Finding {
            detector: self.name().into(),
            severity,
            title: format!("Hook p95 latency {p95_ms}ms exceeds {threshold_ms}ms threshold"),
            detail: format!(
                "The heimdall-hook binary's p95 latency over the last 30 days is {p95_ms}ms \
                 (measured across {} invocations). Every Claude Code tool invocation runs the \
                 hook; high latency adds overhead to the interactive loop. \
                 Investigate DB open time, WAL checkpoint frequency, or disk I/O contention.",
                latencies.len()
            ),
            estimated_monthly_waste_nanos: 0,
        }])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizer::mod_tests::empty_db;

    fn insert_hook_latency(conn: &Connection, latency_us: i64) {
        let now_epoch = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT INTO hook_events
                (received_at, ts_epoch, outcome, latency_us, cli_version)
             VALUES ('2024-01-01T00:00:00Z', ?1, 'ok', ?2, 'test')",
            rusqlite::params![now_epoch, latency_us],
        )
        .unwrap();
    }

    #[test]
    fn no_data_no_finding() {
        let (_dir, conn) = empty_db();
        assert!(HookLatencyDetector::new().run(&conn).unwrap().is_empty());
    }

    #[test]
    fn fast_p95_no_finding() {
        let (_dir, conn) = empty_db();
        // 20 rows at 50ms each → p95 = 50ms < 100ms threshold
        for _ in 0..20 {
            insert_hook_latency(&conn, 50_000);
        }
        assert!(HookLatencyDetector::new().run(&conn).unwrap().is_empty());
    }

    #[test]
    fn slow_p95_triggers_high() {
        let (_dir, conn) = empty_db();
        // 10 rows at 600ms each → p95 = 600ms > 500ms threshold
        for _ in 0..10 {
            insert_hook_latency(&conn, 600_000);
        }
        let findings = HookLatencyDetector::new().run(&conn).unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::High);
        assert_eq!(findings[0].estimated_monthly_waste_nanos, 0);
    }

    #[test]
    fn medium_p95_triggers_medium() {
        let (_dir, conn) = empty_db();
        for _ in 0..10 {
            insert_hook_latency(&conn, 300_000); // 300ms
        }
        let findings = HookLatencyDetector::new().run(&conn).unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Medium);
    }
}
