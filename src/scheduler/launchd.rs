//! macOS launchd backend for the Heimdall scheduler.
//!
//! Writes a plist to `~/Library/LaunchAgents/dev.heimdall.scan.plist` and
//! loads it with `launchctl bootstrap gui/<uid> <plist>`.
//!
//! Minute offset: `:17` -- never `:00` to avoid scheduler pile-up.
//!
//! Test isolation: pass `root: Some(tempdir)` and the plist is written under
//! `<root>/Library/LaunchAgents/` instead of the real home directory.
//! The `launchctl` shell-out is skipped when `root.is_some()` so tests stay
//! fully offline.

use std::path::{Path, PathBuf};

use anyhow::Result;

use super::{InstallStatus, Interval, Scheduler};

/// Label used as the plist `<key>Label</key>` and as the service identifier.
pub const PLIST_LABEL: &str = "dev.heimdall.scan";
/// File name of the plist placed under `LaunchAgents/`.
const PLIST_FILENAME: &str = "dev.heimdall.scan.plist";
/// Minute offset used in `StartCalendarInterval` to avoid :00 pile-up.
const MINUTE_OFFSET: u8 = 17;

// ── Public struct ─────────────────────────────────────────────────────────────

/// macOS launchd scheduler backend.
pub struct LaunchdScheduler {
    /// When `Some`, file writes target `<root>/Library/LaunchAgents/` instead
    /// of `~/Library/LaunchAgents/`.  Shell-outs are skipped.
    root: Option<PathBuf>,
}

impl LaunchdScheduler {
    /// Create a production instance (writes to real `~/Library/LaunchAgents`).
    pub fn new(root: Option<PathBuf>) -> Self {
        Self { root }
    }

    /// Resolve the directory where the plist should live.
    fn agents_dir(&self) -> PathBuf {
        match &self.root {
            Some(r) => r.join("Library").join("LaunchAgents"),
            None => {
                let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
                home.join("Library").join("LaunchAgents")
            }
        }
    }

    /// Full path to the plist file.
    pub fn plist_path(&self) -> PathBuf {
        self.agents_dir().join(PLIST_FILENAME)
    }

    /// Whether we are running in test/dry-run mode (skip shell-outs).
    fn is_isolated(&self) -> bool {
        self.root.is_some()
    }
}

// ── Plist generation ───────────────────────────────────────────────────────────

/// Generate the plist XML for the given parameters.
///
/// This function is pure (no I/O) so it can be unit-tested directly.
pub fn generate_plist(bin_path: &Path, db_path: &Path, interval: Interval) -> String {
    let bin_str = bin_path.display();
    let db_str = db_path.display();

    // For hourly: fire at minute :17 every hour.
    // For daily:  fire at 02:17 every day (low-traffic hour).
    let calendar_entries = match interval {
        Interval::Hourly => format!(
            "            <dict>\n\
             \t\t\t\t<key>Minute</key>\n\
             \t\t\t\t<integer>{}</integer>\n\
             \t\t\t</dict>",
            MINUTE_OFFSET
        ),
        Interval::Daily => format!(
            "            <dict>\n\
             \t\t\t\t<key>Hour</key>\n\
             \t\t\t\t<integer>2</integer>\n\
             \t\t\t\t<key>Minute</key>\n\
             \t\t\t\t<integer>{}</integer>\n\
             \t\t\t</dict>",
            MINUTE_OFFSET
        ),
    };

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
    "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{label}</string>

    <key>ProgramArguments</key>
    <array>
        <string>{bin}</string>
        <string>scan</string>
        <string>--db-path</string>
        <string>{db}</string>
    </array>

    <key>StartCalendarInterval</key>
    <array>
{entries}
    </array>

    <key>StandardOutPath</key>
    <string>/tmp/heimdall-scan.log</string>

    <key>StandardErrorPath</key>
    <string>/tmp/heimdall-scan.log</string>

    <key>RunAtLoad</key>
    <false/>
</dict>
</plist>
"#,
        label = PLIST_LABEL,
        bin = bin_str,
        db = db_str,
        entries = calendar_entries,
    )
}

// ── Scheduler trait impl ───────────────────────────────────────────────────────

impl Scheduler for LaunchdScheduler {
    fn install(&self, interval: Interval, bin_path: &Path, db_path: &Path) -> Result<()> {
        // Idempotent: remove any existing job first.
        self.uninstall()?;

        let agents_dir = self.agents_dir();
        std::fs::create_dir_all(&agents_dir)?;

        let plist_path = self.plist_path();
        let plist_xml = generate_plist(bin_path, db_path, interval);
        std::fs::write(&plist_path, &plist_xml)?;

        if !self.is_isolated() {
            // Load the job via launchctl bootstrap.
            let uid = unsafe { libc_uid() };
            let status = std::process::Command::new("launchctl")
                .args([
                    "bootstrap",
                    &format!("gui/{}", uid),
                    &plist_path.display().to_string(),
                ])
                .status()?;
            if !status.success() {
                anyhow::bail!(
                    "launchctl bootstrap failed (exit {:?}); plist written to {}",
                    status.code(),
                    plist_path.display()
                );
            }
        }

        Ok(())
    }

    fn uninstall(&self) -> Result<()> {
        let plist_path = self.plist_path();
        if !plist_path.exists() {
            return Ok(());
        }

        if !self.is_isolated() {
            let uid = unsafe { libc_uid() };
            // Attempt bootout; ignore errors (service may not be loaded).
            let _ = std::process::Command::new("launchctl")
                .args([
                    "bootout",
                    &format!("gui/{}", uid),
                    &plist_path.display().to_string(),
                ])
                .status();
        }

        std::fs::remove_file(&plist_path)?;
        Ok(())
    }

    fn status(&self) -> Result<InstallStatus> {
        let plist_path = self.plist_path();
        if !plist_path.exists() {
            return Ok(InstallStatus::NotInstalled);
        }

        let hint = format!(
            "next run at :{:02} via launchd ({})",
            MINUTE_OFFSET,
            plist_path.display()
        );
        Ok(InstallStatus::Installed {
            next_run_hint: hint,
            config_path: Some(plist_path),
        })
    }
}

// ── Unsafe helper: get real UID for launchctl domain ─────────────────────────

/// Return the real UID of the current process.
///
/// # Safety
/// Calls POSIX `getuid()` which is always safe.
unsafe fn libc_uid() -> u32 {
    unsafe extern "C" {
        fn getuid() -> u32;
    }
    unsafe { getuid() }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ── plist generation tests ────────────────────────────────────────────────

    #[test]
    fn plist_hourly_contains_label() {
        let xml = generate_plist(
            Path::new("/usr/local/bin/claude-usage-tracker"),
            Path::new("/home/user/.claude/usage.db"),
            Interval::Hourly,
        );
        assert!(
            xml.contains("dev.heimdall.scan"),
            "plist must contain label"
        );
    }

    #[test]
    fn plist_hourly_contains_minute_17() {
        let xml = generate_plist(
            Path::new("/usr/local/bin/claude-usage-tracker"),
            Path::new("/home/user/.claude/usage.db"),
            Interval::Hourly,
        );
        assert!(
            xml.contains("<integer>17</integer>"),
            "plist must contain minute :17"
        );
        assert!(
            !xml.contains("<key>Hour</key>"),
            "hourly plist must not specify an hour"
        );
    }

    #[test]
    fn plist_daily_contains_hour_and_minute() {
        let xml = generate_plist(
            Path::new("/usr/local/bin/claude-usage-tracker"),
            Path::new("/home/user/.claude/usage.db"),
            Interval::Daily,
        );
        assert!(xml.contains("<key>Hour</key>"), "daily plist must set Hour");
        assert!(
            xml.contains("<integer>2</integer>"),
            "daily plist must use hour 2"
        );
        assert!(
            xml.contains("<integer>17</integer>"),
            "daily plist must use minute 17"
        );
    }

    #[test]
    fn plist_embeds_bin_and_db_paths() {
        let bin = Path::new("/opt/homebrew/bin/claude-usage-tracker");
        let db = Path::new("/Users/alice/.claude/usage.db");
        let xml = generate_plist(bin, db, Interval::Hourly);
        assert!(
            xml.contains("/opt/homebrew/bin/claude-usage-tracker"),
            "plist must embed bin path"
        );
        assert!(
            xml.contains("/Users/alice/.claude/usage.db"),
            "plist must embed db path"
        );
    }

    #[test]
    fn plist_contains_scan_subcommand() {
        let xml = generate_plist(
            Path::new("/bin/claude-usage-tracker"),
            Path::new("/tmp/usage.db"),
            Interval::Hourly,
        );
        assert!(
            xml.contains("<string>scan</string>"),
            "plist must include 'scan' argument"
        );
        assert!(
            xml.contains("<string>--db-path</string>"),
            "plist must include '--db-path' argument"
        );
    }

    // ── install/uninstall/status round-trip (isolated, no shell-out) ─────────

    #[test]
    fn install_creates_plist_and_status_reports_installed() {
        let tmp = TempDir::new().unwrap();
        let sched = LaunchdScheduler::new(Some(tmp.path().to_path_buf()));

        let bin = Path::new("/usr/local/bin/claude-usage-tracker");
        let db = Path::new("/tmp/usage.db");

        sched.install(Interval::Hourly, bin, db).unwrap();

        let plist = sched.plist_path();
        assert!(plist.exists(), "plist file must exist after install");

        match sched.status().unwrap() {
            InstallStatus::Installed {
                next_run_hint,
                config_path,
            } => {
                assert!(
                    next_run_hint.contains(":17"),
                    "hint must reference minute :17"
                );
                assert_eq!(config_path.unwrap(), plist);
            }
            other => panic!("expected Installed, got {:?}", other),
        }
    }

    #[test]
    fn uninstall_removes_plist_and_status_reports_not_installed() {
        let tmp = TempDir::new().unwrap();
        let sched = LaunchdScheduler::new(Some(tmp.path().to_path_buf()));

        let bin = Path::new("/usr/local/bin/claude-usage-tracker");
        let db = Path::new("/tmp/usage.db");

        sched.install(Interval::Hourly, bin, db).unwrap();
        sched.uninstall().unwrap();

        assert!(
            !sched.plist_path().exists(),
            "plist must be removed after uninstall"
        );
        assert_eq!(sched.status().unwrap(), InstallStatus::NotInstalled);
    }

    #[test]
    fn uninstall_when_not_installed_is_noop() {
        let tmp = TempDir::new().unwrap();
        let sched = LaunchdScheduler::new(Some(tmp.path().to_path_buf()));
        // Should not error.
        sched.uninstall().unwrap();
    }

    #[test]
    fn install_is_idempotent() {
        let tmp = TempDir::new().unwrap();
        let sched = LaunchdScheduler::new(Some(tmp.path().to_path_buf()));
        let bin = Path::new("/bin/cut");
        let db = Path::new("/tmp/u.db");

        sched.install(Interval::Hourly, bin, db).unwrap();
        // Second install must not duplicate or error.
        sched.install(Interval::Daily, bin, db).unwrap();

        let agents_dir = tmp.path().join("Library").join("LaunchAgents");
        let entries: Vec<_> = std::fs::read_dir(&agents_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(entries.len(), 1, "only one plist file must exist");
    }

    #[test]
    fn status_when_not_installed_returns_not_installed() {
        let tmp = TempDir::new().unwrap();
        let sched = LaunchdScheduler::new(Some(tmp.path().to_path_buf()));
        assert_eq!(sched.status().unwrap(), InstallStatus::NotInstalled);
    }
}
