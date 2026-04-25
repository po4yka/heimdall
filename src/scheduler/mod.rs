//! ROADMAP Phase 15 -- Cross-Platform Scheduler Subcommand.
//!
//! Installs, uninstalls, and reports the status of a platform-native scheduled
//! job that runs `claude-usage-tracker scan` periodically.
//!
//! Platform dispatch:
//!   - macOS → launchd (`~/Library/LaunchAgents/dev.heimdall.scan.plist`)
//!   - Linux → crontab (user crontab, tagged with `# heimdall-scheduler:v1`)
//!   - Other → `UnsupportedScheduler` stub
//!
//! The minute offset `:17` is used for all platforms to avoid pile-up at :00.
//!
//! Test isolation: each platform impl accepts an optional `root: PathBuf` that
//! redirects file writes to a temp directory.  Production code passes `None`;
//! tests pass a `tempdir`.  The cron impl additionally accepts an optional
//! `crontab_file: PathBuf` that replaces the live `crontab -l` shell-out so
//! tests never touch the real user crontab.

pub mod cron;
pub mod daemon;
pub mod launchd;

use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScheduledJob {
    pub slug: &'static str,
    pub launchd_label: &'static str,
    pub launchd_filename: &'static str,
    pub cron_tag: &'static str,
    pub command: &'static [&'static str],
}

pub const SCAN_JOB: ScheduledJob = ScheduledJob {
    slug: "scan",
    launchd_label: "dev.heimdall.scan",
    launchd_filename: "dev.heimdall.scan.plist",
    cron_tag: "# heimdall-scheduler:v1",
    command: &["scan"],
};

pub const USAGE_MONITOR_JOB: ScheduledJob = ScheduledJob {
    slug: "usage-monitor",
    launchd_label: "dev.heimdall.usage-monitor",
    launchd_filename: "dev.heimdall.usage-monitor.plist",
    cron_tag: "# heimdall-usage-monitor:v1",
    command: &["usage-monitor", "capture"],
};

pub const PRICING_SYNC_JOB: ScheduledJob = ScheduledJob {
    slug: "pricing-sync",
    launchd_label: "dev.heimdall.pricing-sync",
    launchd_filename: "dev.heimdall.pricing-sync.plist",
    cron_tag: "# heimdall-pricing-sync:v1",
    command: &["pricing", "sync"],
};

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
    current_for(SCAN_JOB)
}

/// Return the platform-appropriate scheduler for the given Heimdall job.
pub fn current_for(job: ScheduledJob) -> Box<dyn Scheduler> {
    #[cfg(target_os = "macos")]
    {
        Box::new(launchd::LaunchdScheduler::for_job(None, job))
    }
    #[cfg(target_os = "linux")]
    {
        Box::new(cron::CronScheduler::for_job(None, None, job))
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
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

pub(crate) fn xml_escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

pub(crate) fn shell_quote(value: &str) -> String {
    let escaped = value.replace('\'', "'\\''");
    format!("'{escaped}'")
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

    #[test]
    fn xml_escape_escapes_reserved_chars() {
        assert_eq!(
            xml_escape("a&b<c>d\"e'f"),
            "a&amp;b&lt;c&gt;d&quot;e&apos;f"
        );
    }

    #[test]
    fn shell_quote_wraps_and_escapes_single_quotes() {
        assert_eq!(shell_quote("/tmp/it's here"), "'/tmp/it'\\''s here'");
    }
}
