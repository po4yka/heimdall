//! Linux crontab backend for the Heimdall scheduler.
// This module compiles on all platforms but is only selected by `current()` on
// Linux. Allow dead_code so macOS/Windows builds stay warning-free.
#![allow(dead_code)]
//!
//! Modifies the user crontab by tagging our entries with a comment marker:
//!
//!   `# heimdall-scheduler:v1`
//!
//! All crontab lines between this tag and the next blank line (or EOF) are
//! owned by Heimdall.  Lines without the tag are never touched.
//!
//! Minute offset: `:17` -- never `:00` to avoid scheduler pile-up.
//!
//! Test isolation: pass `crontab_file: Some(path)` to read/write a plain file
//! instead of shelling out to `crontab -l` / `crontab -`.  All text-
//! transformation logic is exercised through the file path; no shell-outs
//! happen when the file path is set.

use std::path::{Path, PathBuf};

use anyhow::Result;

use super::{InstallStatus, Interval, SCAN_JOB, ScheduledJob, Scheduler};

/// Comment marker that identifies Heimdall-owned crontab lines.
pub const CRON_TAG: &str = "# heimdall-scheduler:v1";
/// Minute offset used in cron expressions.
const MINUTE_OFFSET: u8 = 17;

// ── Public struct ─────────────────────────────────────────────────────────────

/// Linux cron scheduler backend.
pub struct CronScheduler {
    /// When `Some`, read/write this file instead of shelling out to `crontab`.
    crontab_file: Option<PathBuf>,
    /// Unused on Linux; kept for API symmetry with the launchd backend.
    #[allow(dead_code)]
    root: Option<PathBuf>,
    job: ScheduledJob,
}

impl CronScheduler {
    /// Create a scheduler instance.
    ///
    /// - `root`: reserved for future use (path redirect).
    /// - `crontab_file`: if `Some`, use this file as the crontab source instead
    ///   of shelling out.  Pass `None` in production.
    pub fn new(root: Option<PathBuf>, crontab_file: Option<PathBuf>) -> Self {
        Self::for_job(root, crontab_file, SCAN_JOB)
    }

    pub fn for_job(
        root: Option<PathBuf>,
        crontab_file: Option<PathBuf>,
        job: ScheduledJob,
    ) -> Self {
        Self {
            crontab_file,
            root,
            job,
        }
    }

    /// Read the current crontab text.
    fn read_crontab(&self) -> Result<String> {
        if let Some(ref path) = self.crontab_file {
            return Ok(std::fs::read_to_string(path).unwrap_or_default());
        }
        // Production: shell out to `crontab -l`.
        let output = std::process::Command::new("crontab").arg("-l").output()?;
        // crontab -l exits 1 with "no crontab for user" when empty — treat as
        // empty string rather than an error.
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        } else {
            Ok(String::new())
        }
    }

    /// Write crontab text back.
    fn write_crontab(&self, content: &str) -> Result<()> {
        if let Some(ref path) = self.crontab_file {
            std::fs::write(path, content)?;
            return Ok(());
        }
        // Production: pipe to `crontab -`.
        use std::io::Write;
        let mut child = std::process::Command::new("crontab")
            .arg("-")
            .stdin(std::process::Stdio::piped())
            .spawn()?;
        if let Some(stdin) = child.stdin.take() {
            let mut stdin = stdin;
            stdin.write_all(content.as_bytes())?;
        }
        let status = child.wait()?;
        if !status.success() {
            anyhow::bail!("crontab write failed (exit {:?})", status.code());
        }
        Ok(())
    }
}

// ── Pure text-transformation helpers ─────────────────────────────────────────

/// Build the cron expression for the given interval.
///
/// Returns the two-field minute/hour prefix, e.g. `"17 * * * *"` (hourly) or
/// `"17 2 * * *"` (daily at 02:17).
pub fn cron_expression(interval: Interval) -> String {
    match interval {
        Interval::Hourly => format!("{} * * * *", MINUTE_OFFSET),
        Interval::Daily => format!("{} 2 * * *", MINUTE_OFFSET),
    }
}

/// Build the full crontab line block owned by Heimdall.
///
/// Format:
/// ```text
/// # heimdall-scheduler:v1
/// 17 * * * * /path/to/bin scan --db-path /path/to/db
/// ```
pub fn build_heimdall_block(bin_path: &Path, db_path: &Path, interval: Interval) -> String {
    build_heimdall_block_for_job(SCAN_JOB, bin_path, db_path, interval)
}

pub fn build_heimdall_block_for_job(
    job: ScheduledJob,
    bin_path: &Path,
    db_path: &Path,
    interval: Interval,
) -> String {
    format!(
        "{}\n{} {}\n",
        job.cron_tag,
        cron_expression(interval),
        job_command(bin_path, db_path, job),
    )
}

fn job_command(bin_path: &Path, db_path: &Path, job: ScheduledJob) -> String {
    let command = job.command.join(" ");
    format!(
        "{} {} --db-path {}",
        bin_path.display(),
        command,
        db_path.display()
    )
}

/// Remove all Heimdall-owned lines from a crontab text.
///
/// A Heimdall block starts with a line containing `CRON_TAG` and ends just
/// before the next blank line or EOF.  Lines without the tag are preserved
/// byte-for-byte.
pub fn remove_heimdall_entries(crontab: &str) -> String {
    remove_heimdall_entries_for_job(crontab, SCAN_JOB)
}

pub fn remove_heimdall_entries_for_job(crontab: &str, job: ScheduledJob) -> String {
    let mut output = String::with_capacity(crontab.len());
    let mut skip = false;

    for line in crontab.lines() {
        if line.contains(job.cron_tag) {
            skip = true;
            continue;
        }
        if skip && line.trim().is_empty() {
            skip = false;
            continue;
        }
        if !skip {
            output.push_str(line);
            output.push('\n');
        }
    }

    output
}

/// Merge a new Heimdall block into an existing crontab text.
///
/// 1. Remove any existing Heimdall entries.
/// 2. Append the new block.
///
/// Unrelated entries are preserved byte-for-byte.
pub fn merge_heimdall_entry(
    existing_crontab: &str,
    bin_path: &Path,
    db_path: &Path,
    interval: Interval,
) -> String {
    merge_heimdall_entry_for_job(existing_crontab, bin_path, db_path, interval, SCAN_JOB)
}

pub fn merge_heimdall_entry_for_job(
    existing_crontab: &str,
    bin_path: &Path,
    db_path: &Path,
    interval: Interval,
    job: ScheduledJob,
) -> String {
    let mut clean = remove_heimdall_entries_for_job(existing_crontab, job);
    // Ensure there is a trailing newline before we append.
    if !clean.is_empty() && !clean.ends_with('\n') {
        clean.push('\n');
    }
    clean.push_str(&build_heimdall_block_for_job(job, bin_path, db_path, interval));
    clean
}

// ── Scheduler trait impl ───────────────────────────────────────────────────────

impl Scheduler for CronScheduler {
    fn install(&self, interval: Interval, bin_path: &Path, db_path: &Path) -> Result<()> {
        let existing = self.read_crontab()?;
        let updated = merge_heimdall_entry_for_job(&existing, bin_path, db_path, interval, self.job);
        self.write_crontab(&updated)?;
        Ok(())
    }

    fn uninstall(&self) -> Result<()> {
        let existing = self.read_crontab()?;
        if !existing.contains(self.job.cron_tag) {
            return Ok(());
        }
        let updated = remove_heimdall_entries_for_job(&existing, self.job);
        self.write_crontab(&updated)?;
        Ok(())
    }

    fn status(&self) -> Result<InstallStatus> {
        let crontab = self.read_crontab()?;
        if !crontab.contains(self.job.cron_tag) {
            return Ok(InstallStatus::NotInstalled);
        }
        let hint = format!("next run at :{:02} via cron", MINUTE_OFFSET);
        Ok(InstallStatus::Installed {
            next_run_hint: hint,
            config_path: None,
        })
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scheduler::USAGE_MONITOR_JOB;
    use tempfile::NamedTempFile;

    // ── cron_expression ───────────────────────────────────────────────────────

    #[test]
    fn cron_expr_hourly_uses_minute_17() {
        let expr = cron_expression(Interval::Hourly);
        assert_eq!(expr, "17 * * * *");
    }

    #[test]
    fn cron_expr_daily_uses_hour_2_minute_17() {
        let expr = cron_expression(Interval::Daily);
        assert_eq!(expr, "17 2 * * *");
    }

    // ── build_heimdall_block ──────────────────────────────────────────────────

    #[test]
    fn block_contains_tag() {
        let block = build_heimdall_block(
            Path::new("/usr/local/bin/cut"),
            Path::new("/tmp/u.db"),
            Interval::Hourly,
        );
        assert!(block.contains(CRON_TAG));
    }

    #[test]
    fn block_contains_bin_and_db_paths() {
        let block = build_heimdall_block(
            Path::new("/opt/bin/claude-usage-tracker"),
            Path::new("/home/bob/.claude/usage.db"),
            Interval::Hourly,
        );
        assert!(block.contains("/opt/bin/claude-usage-tracker"));
        assert!(block.contains("/home/bob/.claude/usage.db"));
    }

    #[test]
    fn usage_monitor_block_uses_distinct_tag_and_subcommand() {
        let block = build_heimdall_block_for_job(
            USAGE_MONITOR_JOB,
            Path::new("/opt/bin/claude-usage-tracker"),
            Path::new("/home/bob/.claude/usage.db"),
            Interval::Daily,
        );
        assert!(block.contains(USAGE_MONITOR_JOB.cron_tag));
        assert!(block.contains("usage-monitor capture --db-path"));
    }

    // ── remove_heimdall_entries ───────────────────────────────────────────────

    #[test]
    fn remove_preserves_unrelated_entries() {
        let crontab = "# some user cron\n\
                       0 9 * * 1 /usr/bin/backup\n\
                       # heimdall-scheduler:v1\n\
                       17 * * * * /bin/cut scan --db-path /tmp/u.db\n\
                       \n\
                       30 6 * * * /usr/bin/something-else\n";

        let result = remove_heimdall_entries(crontab);

        assert!(
            result.contains("0 9 * * 1 /usr/bin/backup"),
            "unrelated cron entry must be preserved"
        );
        assert!(
            result.contains("30 6 * * * /usr/bin/something-else"),
            "entry after blank line must be preserved"
        );
        assert!(!result.contains(CRON_TAG), "heimdall tag must be removed");
        assert!(
            !result.contains("/bin/cut scan"),
            "heimdall cron line must be removed"
        );
    }

    #[test]
    fn remove_when_no_heimdall_entry_is_identity() {
        let crontab = "0 9 * * 1 /usr/bin/backup\n30 6 * * * /usr/bin/other\n";
        let result = remove_heimdall_entries(crontab);
        assert_eq!(result, crontab);
    }

    #[test]
    fn remove_empty_crontab_returns_empty() {
        assert_eq!(remove_heimdall_entries(""), "");
    }

    // ── merge_heimdall_entry ──────────────────────────────────────────────────

    #[test]
    fn merge_into_empty_crontab() {
        let result = merge_heimdall_entry(
            "",
            Path::new("/bin/cut"),
            Path::new("/tmp/u.db"),
            Interval::Hourly,
        );
        assert!(result.contains(CRON_TAG));
        assert!(result.contains("17 * * * *"));
    }

    #[test]
    fn merge_adds_exactly_one_entry() {
        let existing = "# some user cron\n0 9 * * 1 /usr/bin/backup\n";
        let result = merge_heimdall_entry(
            existing,
            Path::new("/bin/cut"),
            Path::new("/tmp/u.db"),
            Interval::Hourly,
        );
        let count = result.matches(CRON_TAG).count();
        assert_eq!(count, 1, "must have exactly one heimdall tag");
    }

    #[test]
    fn merge_replaces_existing_entry() {
        // Start with an hourly entry.
        let existing = "# unrelated\n0 0 * * * /bin/other\n\
                        # heimdall-scheduler:v1\n\
                        17 * * * * /old/bin scan --db-path /old/db\n";
        // Merge a daily entry with a new path.
        let result = merge_heimdall_entry(
            existing,
            Path::new("/new/bin"),
            Path::new("/new/db"),
            Interval::Daily,
        );
        // Only one tag.
        assert_eq!(result.matches(CRON_TAG).count(), 1);
        // New paths present.
        assert!(result.contains("/new/bin"));
        assert!(result.contains("/new/db"));
        // Old entry gone.
        assert!(!result.contains("/old/bin"));
        // Unrelated entry preserved.
        assert!(result.contains("0 0 * * * /bin/other"));
        // Daily cron expression present.
        assert!(result.contains("17 2 * * *"));
    }

    #[test]
    fn merge_is_idempotent() {
        let existing = "";
        let first = merge_heimdall_entry(
            existing,
            Path::new("/bin/cut"),
            Path::new("/tmp/u.db"),
            Interval::Hourly,
        );
        let second = merge_heimdall_entry(
            &first,
            Path::new("/bin/cut"),
            Path::new("/tmp/u.db"),
            Interval::Hourly,
        );
        assert_eq!(
            first.matches(CRON_TAG).count(),
            second.matches(CRON_TAG).count(),
            "idempotent: tag count must not change"
        );
    }

    // ── install/uninstall/status round-trip via temp file ─────────────────────

    #[test]
    fn install_then_status_reports_installed() {
        let file = NamedTempFile::new().unwrap();
        let sched = CronScheduler::new(None, Some(file.path().to_path_buf()));

        sched
            .install(
                Interval::Hourly,
                Path::new("/bin/cut"),
                Path::new("/tmp/u.db"),
            )
            .unwrap();

        match sched.status().unwrap() {
            InstallStatus::Installed { next_run_hint, .. } => {
                assert!(next_run_hint.contains(":17"));
            }
            other => panic!("expected Installed, got {:?}", other),
        }
    }

    #[test]
    fn uninstall_then_status_reports_not_installed() {
        let file = NamedTempFile::new().unwrap();
        let sched = CronScheduler::new(None, Some(file.path().to_path_buf()));

        sched
            .install(
                Interval::Hourly,
                Path::new("/bin/cut"),
                Path::new("/tmp/u.db"),
            )
            .unwrap();
        sched.uninstall().unwrap();

        assert_eq!(sched.status().unwrap(), InstallStatus::NotInstalled);
    }

    #[test]
    fn uninstall_when_nothing_installed_is_noop() {
        let file = NamedTempFile::new().unwrap();
        let sched = CronScheduler::new(None, Some(file.path().to_path_buf()));
        // Should not error.
        sched.uninstall().unwrap();
    }

    #[test]
    fn unrelated_entries_survive_install_and_uninstall() {
        let file = NamedTempFile::new().unwrap();
        std::fs::write(file.path(), "# existing\n0 9 * * 1 /usr/bin/backup\n").unwrap();

        let sched = CronScheduler::new(None, Some(file.path().to_path_buf()));

        sched
            .install(
                Interval::Daily,
                Path::new("/bin/cut"),
                Path::new("/tmp/u.db"),
            )
            .unwrap();

        let after_install = std::fs::read_to_string(file.path()).unwrap();
        assert!(after_install.contains("0 9 * * 1 /usr/bin/backup"));

        sched.uninstall().unwrap();

        let after_uninstall = std::fs::read_to_string(file.path()).unwrap();
        assert!(
            after_uninstall.contains("0 9 * * 1 /usr/bin/backup"),
            "unrelated entry must survive uninstall"
        );
        assert!(!after_uninstall.contains(CRON_TAG));
    }
}
