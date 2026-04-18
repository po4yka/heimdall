//! ROADMAP Phase 15 -- Cross-Platform Scheduler Subcommand.
//!
//! Installs, uninstalls, and reports the status of a platform-native scheduled
//! job that runs `claude-usage-tracker scan` periodically.
//!
//! Platform dispatch:
//!   - macOS  → launchd (`~/Library/LaunchAgents/dev.heimdall.scan.plist`)
//!   - Linux  → crontab (user crontab, tagged with `# heimdall-scheduler:v1`)
//!   - Windows → schtasks (`HeimdallScan` task)
//!
//! The minute offset `:17` is used for all platforms to avoid pile-up at :00.
//!
//! Test isolation: each platform impl accepts an optional `root: PathBuf` that
//! redirects file writes to a temp directory.  Production code passes `None`;
//! tests pass a `tempdir`.  The cron impl additionally accepts an optional
//! `crontab_file: PathBuf` that replaces the live `crontab -l` shell-out so
//! tests never touch the real user crontab.

pub mod cron;
pub mod launchd;
pub mod schtasks;

use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::Result;

// ── Common types ──────────────────────────────────────────────────────────────

/// How often the scan job should run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interval {
    Hourly,
    Daily,
}

impl Interval {
    /// Return the interval as a short lowercase label.
    #[allow(dead_code)]
    pub fn as_str(self) -> &'static str {
        match self {
            Interval::Hourly => "hourly",
            Interval::Daily => "daily",
        }
    }
}

impl FromStr for Interval {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "hourly" => Ok(Interval::Hourly),
            "daily" => Ok(Interval::Daily),
            other => anyhow::bail!("unknown interval '{}'; valid values: hourly, daily", other),
        }
    }
}

/// Current installation state of the scheduled job.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum InstallStatus {
    /// The job is installed and will run at the given time hint.
    Installed {
        /// Human-readable description, e.g. "next run at :17 via launchd".
        next_run_hint: String,
        /// Path to the config file created (plist / crontab note / task xml).
        config_path: Option<PathBuf>,
    },
    /// No Heimdall job found.
    NotInstalled,
    /// The current platform is not supported.
    UnsupportedPlatform(String),
}

/// Capability interface implemented by each platform backend.
pub trait Scheduler {
    /// Install (or re-install) the scheduled job.
    ///
    /// Idempotent: if a job already exists it is removed first.
    fn install(&self, interval: Interval, bin_path: &Path, db_path: &Path) -> Result<()>;

    /// Remove the scheduled job.  A no-op when nothing is installed.
    fn uninstall(&self) -> Result<()>;

    /// Return the current installation status.  Never returns `Err`.
    fn status(&self) -> Result<InstallStatus>;
}

// ── Platform dispatch ─────────────────────────────────────────────────────────

/// Return the platform-appropriate scheduler with default (production) paths.
pub fn current() -> Box<dyn Scheduler> {
    #[cfg(target_os = "macos")]
    {
        Box::new(launchd::LaunchdScheduler::new(None))
    }
    #[cfg(target_os = "linux")]
    {
        Box::new(cron::CronScheduler::new(None, None))
    }
    #[cfg(target_os = "windows")]
    {
        Box::new(schtasks::SchtasksScheduler::new())
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Box::new(UnsupportedScheduler)
    }
}

// ── Unsupported-platform stub ─────────────────────────────────────────────────

#[allow(dead_code)]
struct UnsupportedScheduler;

impl Scheduler for UnsupportedScheduler {
    fn install(&self, _interval: Interval, _bin_path: &Path, _db_path: &Path) -> Result<()> {
        anyhow::bail!("scheduler: unsupported platform");
    }
    fn uninstall(&self) -> Result<()> {
        Ok(())
    }
    fn status(&self) -> Result<InstallStatus> {
        Ok(InstallStatus::UnsupportedPlatform(
            std::env::consts::OS.to_string(),
        ))
    }
}

// ── Shared helpers ─────────────────────────────────────────────────────────────

/// Resolve the absolute path to the running binary, canonicalized.
///
/// Critical for nvm/Homebrew/fnm-style installs where symlinks abound.
pub fn resolve_bin_path() -> Result<PathBuf> {
    let exe = std::env::current_exe()?;
    let canonical = exe.canonicalize().unwrap_or(exe);
    Ok(canonical)
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interval_from_str_hourly() {
        assert_eq!(Interval::from_str("hourly").unwrap(), Interval::Hourly);
    }

    #[test]
    fn interval_from_str_daily() {
        assert_eq!(Interval::from_str("daily").unwrap(), Interval::Daily);
    }

    #[test]
    fn interval_from_str_case_insensitive() {
        assert_eq!(Interval::from_str("HOURLY").unwrap(), Interval::Hourly);
        assert_eq!(Interval::from_str("Daily").unwrap(), Interval::Daily);
    }

    #[test]
    fn interval_from_str_error() {
        assert!(Interval::from_str("weekly").is_err());
        assert!(Interval::from_str("").is_err());
        assert!(Interval::from_str("month").is_err());
    }

    #[test]
    fn interval_as_str() {
        assert_eq!(Interval::Hourly.as_str(), "hourly");
        assert_eq!(Interval::Daily.as_str(), "daily");
    }
}
