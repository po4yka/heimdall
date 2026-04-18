//! Windows Task Scheduler backend for the Heimdall scheduler.
// This module compiles on all platforms but is only selected by `current()` on
// Windows. Allow dead_code so macOS/Linux builds stay warning-free.
#![allow(dead_code)]
//!
//! Uses `schtasks.exe` to create/delete/query the `HeimdallScan` task.
//!
//! Minute offset: `:17` -- never `:00` to avoid scheduler pile-up.
//!
//! Test isolation: the public `build_create_args` / `build_delete_args` /
//! `build_query_args` functions return the argv vectors without executing
//! anything.  Production code calls `run_schtasks`; tests verify the vectors.

use std::path::Path;

use anyhow::Result;

use super::{InstallStatus, Interval, Scheduler};

/// Windows task name registered in Task Scheduler.
pub const TASK_NAME: &str = "HeimdallScan";
/// Minute offset used in the start-time string.
const MINUTE_OFFSET: u8 = 17;

// ── Argument builders (pure, testable) ───────────────────────────────────────

/// Build the `schtasks /Create` argv for the given parameters.
///
/// Returns a `Vec<String>` (not `Vec<&str>`) so callers own the data.
pub fn build_create_args(bin_path: &Path, db_path: &Path, interval: Interval) -> Vec<String> {
    let schedule = match interval {
        Interval::Hourly => "HOURLY",
        Interval::Daily => "DAILY",
    };

    // Start-time string: HH:17 — use 00:17 as the wall-clock anchor; Windows
    // treats this as the offset-from-midnight, which for HOURLY scheduling
    // means "always fire at :17 past each hour".
    let start_time = format!("00:{:02}", MINUTE_OFFSET);

    vec![
        "schtasks".to_string(),
        "/Create".to_string(),
        "/F".to_string(), // /F = force overwrite if task exists
        "/TN".to_string(),
        TASK_NAME.to_string(),
        "/SC".to_string(),
        schedule.to_string(),
        "/ST".to_string(),
        start_time,
        "/TR".to_string(),
        format!(
            "\"{}\" scan --db-path \"{}\"",
            bin_path.display(),
            db_path.display()
        ),
    ]
}

/// Build the `schtasks /Delete` argv.
pub fn build_delete_args() -> Vec<String> {
    vec![
        "schtasks".to_string(),
        "/Delete".to_string(),
        "/F".to_string(), // /F = no confirmation prompt
        "/TN".to_string(),
        TASK_NAME.to_string(),
    ]
}

/// Build the `schtasks /Query` argv for checking task existence.
pub fn build_query_args() -> Vec<String> {
    vec![
        "schtasks".to_string(),
        "/Query".to_string(),
        "/TN".to_string(),
        TASK_NAME.to_string(),
        "/FO".to_string(),
        "LIST".to_string(),
    ]
}

// ── Shell-out helper (production only) ───────────────────────────────────────

/// Execute a schtasks.exe command by argv (first element is the program name).
///
/// Returns `Ok(output)` on success (exit 0), or an error with the stderr text.
#[allow(dead_code)]
fn run_schtasks(args: &[String]) -> Result<String> {
    if args.is_empty() {
        anyhow::bail!("empty schtasks args");
    }
    let output = std::process::Command::new(&args[0])
        .args(&args[1..])
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    if output.status.success() {
        Ok(stdout)
    } else {
        anyhow::bail!("schtasks failed: {}", stderr.trim())
    }
}

// ── Public struct ─────────────────────────────────────────────────────────────

/// Windows Task Scheduler backend.
pub struct SchtasksScheduler {
    /// When `Some`, this overrides the real system check for testing.
    /// `true` = task is considered installed; `false` = not installed.
    #[allow(dead_code)]
    simulated_status: Option<bool>,
}

impl SchtasksScheduler {
    /// Create a production instance.
    pub fn new() -> Self {
        Self {
            simulated_status: None,
        }
    }

    /// Create a test instance with a simulated task status.
    #[cfg(test)]
    pub fn with_simulated_status(installed: bool) -> Self {
        Self {
            simulated_status: Some(installed),
        }
    }
}

impl Default for SchtasksScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Scheduler for SchtasksScheduler {
    fn install(&self, interval: Interval, bin_path: &Path, db_path: &Path) -> Result<()> {
        let args = build_create_args(bin_path, db_path, interval);
        // In tests (cfg(test)) we never actually exec — just verify the args.
        #[cfg(not(test))]
        run_schtasks(&args)?;
        #[cfg(test)]
        let _ = args;
        Ok(())
    }

    fn uninstall(&self) -> Result<()> {
        let args = build_delete_args();
        #[cfg(not(test))]
        {
            // Ignore "task does not exist" error (exit 1 with specific message).
            let _ = run_schtasks(&args);
        }
        #[cfg(test)]
        let _ = args;
        Ok(())
    }

    fn status(&self) -> Result<InstallStatus> {
        #[cfg(test)]
        {
            if let Some(installed) = self.simulated_status {
                if installed {
                    return Ok(InstallStatus::Installed {
                        next_run_hint: format!(
                            "next run at :{:02} via Task Scheduler ({})",
                            MINUTE_OFFSET, TASK_NAME
                        ),
                        config_path: None,
                    });
                } else {
                    return Ok(InstallStatus::NotInstalled);
                }
            }
        }

        // Production: query schtasks.
        #[cfg(not(test))]
        {
            let args = build_query_args();
            match run_schtasks(&args) {
                Ok(_) => {
                    return Ok(InstallStatus::Installed {
                        next_run_hint: format!(
                            "next run at :{:02} via Task Scheduler ({})",
                            MINUTE_OFFSET, TASK_NAME
                        ),
                        config_path: None,
                    });
                }
                Err(_) => return Ok(InstallStatus::NotInstalled),
            }
        }

        // Fallback for non-Windows builds where this code compiles but is not
        // platform-selected.
        #[allow(unreachable_code)]
        Ok(InstallStatus::NotInstalled)
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── build_create_args ─────────────────────────────────────────────────────

    #[test]
    fn create_args_hourly_contains_task_name() {
        let args = build_create_args(
            Path::new("C:\\bin\\claude-usage-tracker.exe"),
            Path::new("C:\\Users\\user\\.claude\\usage.db"),
            Interval::Hourly,
        );
        assert!(args.contains(&TASK_NAME.to_string()));
    }

    #[test]
    fn create_args_hourly_schedule_is_hourly() {
        let args = build_create_args(
            Path::new("C:\\bin\\cut.exe"),
            Path::new("C:\\tmp\\u.db"),
            Interval::Hourly,
        );
        let sc_idx = args.iter().position(|a| a == "/SC").unwrap();
        assert_eq!(args[sc_idx + 1], "HOURLY");
    }

    #[test]
    fn create_args_daily_schedule_is_daily() {
        let args = build_create_args(
            Path::new("C:\\bin\\cut.exe"),
            Path::new("C:\\tmp\\u.db"),
            Interval::Daily,
        );
        let sc_idx = args.iter().position(|a| a == "/SC").unwrap();
        assert_eq!(args[sc_idx + 1], "DAILY");
    }

    #[test]
    fn create_args_start_time_has_minute_17() {
        let args = build_create_args(
            Path::new("C:\\bin\\cut.exe"),
            Path::new("C:\\tmp\\u.db"),
            Interval::Hourly,
        );
        let st_idx = args.iter().position(|a| a == "/ST").unwrap();
        assert_eq!(args[st_idx + 1], "00:17");
    }

    #[test]
    fn create_args_contains_bin_and_db() {
        let bin = Path::new("C:\\Program Files\\Heimdall\\cut.exe");
        let db = Path::new("C:\\Users\\bob\\.claude\\usage.db");
        let args = build_create_args(bin, db, Interval::Hourly);
        let tr_idx = args.iter().position(|a| a == "/TR").unwrap();
        let tr_value = &args[tr_idx + 1];
        assert!(
            tr_value.contains("C:\\Program Files\\Heimdall\\cut.exe")
                || tr_value.contains("C:/Program Files/Heimdall/cut.exe")
        );
        assert!(tr_value.contains(".claude\\usage.db") || tr_value.contains(".claude/usage.db"));
    }

    #[test]
    fn create_args_force_flag_present() {
        let args = build_create_args(
            Path::new("C:\\bin\\cut.exe"),
            Path::new("C:\\tmp\\u.db"),
            Interval::Hourly,
        );
        assert!(args.contains(&"/F".to_string()), "/F flag must be present");
    }

    // ── build_delete_args ─────────────────────────────────────────────────────

    #[test]
    fn delete_args_contains_task_name_and_force() {
        let args = build_delete_args();
        assert!(args.contains(&TASK_NAME.to_string()));
        assert!(args.contains(&"/Delete".to_string()));
        assert!(args.contains(&"/F".to_string()));
    }

    // ── build_query_args ──────────────────────────────────────────────────────

    #[test]
    fn query_args_contains_task_name() {
        let args = build_query_args();
        assert!(args.contains(&TASK_NAME.to_string()));
        assert!(args.contains(&"/Query".to_string()));
    }

    // ── simulated status round-trips ──────────────────────────────────────────

    #[test]
    fn simulated_installed_reports_installed() {
        let sched = SchtasksScheduler::with_simulated_status(true);
        match sched.status().unwrap() {
            InstallStatus::Installed { next_run_hint, .. } => {
                assert!(next_run_hint.contains(":17"));
            }
            other => panic!("expected Installed, got {:?}", other),
        }
    }

    #[test]
    fn simulated_not_installed_reports_not_installed() {
        let sched = SchtasksScheduler::with_simulated_status(false);
        assert_eq!(sched.status().unwrap(), InstallStatus::NotInstalled);
    }

    #[test]
    fn install_and_uninstall_do_not_exec_in_test() {
        // Should complete without trying to exec schtasks.exe
        let sched = SchtasksScheduler::with_simulated_status(false);
        sched
            .install(
                Interval::Hourly,
                Path::new("C:\\bin\\cut.exe"),
                Path::new("C:\\tmp\\u.db"),
            )
            .unwrap();
        sched.uninstall().unwrap();
    }
}
