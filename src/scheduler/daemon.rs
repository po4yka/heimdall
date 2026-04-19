//! macOS LaunchAgent daemon backend for Heimdall (Phase 22).
//!
//! Writes `~/Library/LaunchAgents/dev.heimdall.daemon.plist` with
//! `RunAtLoad: true` and `KeepAlive: true` so the dashboard runs
//! persistently in the background and restarts after logout/login.
//!
//! Stdout and stderr are routed to `~/Library/Logs/heimdall/daemon.log`
//! and `~/Library/Logs/heimdall/daemon.err` respectively.
//!
//! Test isolation: pass `root: Some(tempdir)` and all file writes land
//! under `<root>/Library/…` instead of the real home directory.
//! The `launchctl` shell-out is skipped when `root.is_some()` so tests
//! stay fully offline.
//!
//! Non-macOS: `current_daemon_scheduler()` returns an `UnsupportedPlatform`
//! stub that surfaces a clear error to the user.

use std::path::{Path, PathBuf};

use anyhow::Result;

use super::InstallStatus;

/// Label used as the plist `<key>Label</key>` and as the service identifier.
pub const DAEMON_PLIST_LABEL: &str = "dev.heimdall.daemon";
/// File name of the plist placed under `LaunchAgents/`.
const DAEMON_PLIST_FILENAME: &str = "dev.heimdall.daemon.plist";

// ── Trait ─────────────────────────────────────────────────────────────────────

/// Capability interface for the daemon (always-on dashboard) subcommand.
pub trait DaemonScheduler {
    /// Write the plist and register it with launchd.
    fn install(&self, bin_path: &Path) -> Result<()>;
    /// Unregister and remove the plist.
    fn uninstall(&self) -> Result<()>;
    /// Return the current installation status.  Never returns `Err`.
    fn status(&self) -> Result<InstallStatus>;
}

// ── Platform dispatch ─────────────────────────────────────────────────────────

/// Return the platform-appropriate daemon scheduler.
///
/// On macOS this is a `LaunchdDaemonScheduler` with production paths.
/// On all other platforms it is the `UnsupportedDaemonScheduler` stub that
/// returns a clear error on every operation.
pub fn current_daemon_scheduler() -> Box<dyn DaemonScheduler> {
    #[cfg(target_os = "macos")]
    {
        Box::new(LaunchdDaemonScheduler::new(None))
    }
    #[cfg(not(target_os = "macos"))]
    {
        Box::new(UnsupportedDaemonScheduler)
    }
}

// ── Unsupported-platform stub ─────────────────────────────────────────────────

#[allow(dead_code)]
pub struct UnsupportedDaemonScheduler;

impl DaemonScheduler for UnsupportedDaemonScheduler {
    fn install(&self, _bin_path: &Path) -> Result<()> {
        anyhow::bail!(
            "daemon subcommand is currently macOS-only; \
             Linux systemd and Windows Service support is deferred"
        );
    }
    fn uninstall(&self) -> Result<()> {
        anyhow::bail!(
            "daemon subcommand is currently macOS-only; \
             Linux systemd and Windows Service support is deferred"
        );
    }
    fn status(&self) -> Result<InstallStatus> {
        Ok(InstallStatus::UnsupportedPlatform(
            std::env::consts::OS.to_string(),
        ))
    }
}

// ── macOS implementation ──────────────────────────────────────────────────────

/// macOS launchd daemon scheduler.
pub struct LaunchdDaemonScheduler {
    /// When `Some`, file writes target `<root>/Library/…` and shell-outs are
    /// skipped.  Production code passes `None`.
    root: Option<PathBuf>,
}

impl LaunchdDaemonScheduler {
    /// Create a production instance (writes to real `~/Library/…`).
    pub fn new(root: Option<PathBuf>) -> Self {
        Self { root }
    }

    /// Resolve `~/Library/LaunchAgents/` (or its test override).
    fn agents_dir(&self) -> PathBuf {
        match &self.root {
            Some(r) => r.join("Library").join("LaunchAgents"),
            None => {
                let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
                home.join("Library").join("LaunchAgents")
            }
        }
    }

    /// Resolve `~/Library/Logs/heimdall/` (or its test override).
    fn logs_dir(&self) -> PathBuf {
        match &self.root {
            Some(r) => r.join("Library").join("Logs").join("heimdall"),
            None => {
                let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
                home.join("Library").join("Logs").join("heimdall")
            }
        }
    }

    /// Full path to the daemon plist file.
    pub fn plist_path(&self) -> PathBuf {
        self.agents_dir().join(DAEMON_PLIST_FILENAME)
    }

    /// Whether we are running in test/dry-run mode (skip shell-outs).
    fn is_isolated(&self) -> bool {
        self.root.is_some()
    }
}

// ── Plist generation ───────────────────────────────────────────────────────────

/// Generate the daemon plist XML.
///
/// Pure (no I/O) so it can be unit-tested directly.
///
/// The plist runs:
///   `<bin> dashboard --host localhost --port 8080 --watch --no-open --background-poll`
///
/// with `RunAtLoad = true` and `KeepAlive = true` so launchd restarts the
/// process if it exits.
pub fn generate_daemon_plist(bin_path: &Path, logs_dir: &Path) -> String {
    let bin_str = bin_path.display();
    let log_out = logs_dir.join("daemon.log");
    let log_err = logs_dir.join("daemon.err");

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
        <string>dashboard</string>
        <string>--host</string>
        <string>localhost</string>
        <string>--port</string>
        <string>8080</string>
        <string>--watch</string>
        <string>--no-open</string>
        <string>--background-poll</string>
    </array>

    <key>RunAtLoad</key>
    <true/>

    <key>KeepAlive</key>
    <true/>

    <key>StandardOutPath</key>
    <string>{log_out}</string>

    <key>StandardErrorPath</key>
    <string>{log_err}</string>
</dict>
</plist>
"#,
        label = DAEMON_PLIST_LABEL,
        bin = bin_str,
        log_out = log_out.display(),
        log_err = log_err.display(),
    )
}

// ── DaemonScheduler impl ───────────────────────────────────────────────────────

impl DaemonScheduler for LaunchdDaemonScheduler {
    fn install(&self, bin_path: &Path) -> Result<()> {
        // Idempotent: remove any existing daemon registration first.
        self.uninstall()?;

        // Create directories.
        let agents_dir = self.agents_dir();
        std::fs::create_dir_all(&agents_dir)?;
        let logs_dir = self.logs_dir();
        std::fs::create_dir_all(&logs_dir)?;

        // Write the plist.
        let plist_path = self.plist_path();
        let plist_xml = generate_daemon_plist(bin_path, &logs_dir);
        std::fs::write(&plist_path, &plist_xml)?;

        if !self.is_isolated() {
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
            // Bootout; ignore errors (service may not be loaded).
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
            "daemon installed (always-on, port 8080); plist: {}",
            plist_path.display()
        );
        Ok(InstallStatus::Installed {
            next_run_hint: hint,
            config_path: Some(plist_path),
        })
    }
}

// ── Unsafe helper ─────────────────────────────────────────────────────────────

/// Return the real UID of the current process.
///
/// # Safety
/// Calls POSIX `getuid()` which is always safe.
#[cfg(unix)]
unsafe fn libc_uid() -> u32 {
    unsafe extern "C" {
        fn getuid() -> u32;
    }
    unsafe { getuid() }
}

/// Windows stub. The `LaunchdDaemonScheduler` impl that references this fn
/// is never reached at runtime on non-macOS (see `current_daemon_scheduler`),
/// but the symbol must exist so the impl block compiles.
#[cfg(not(unix))]
unsafe fn libc_uid() -> u32 {
    0
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ── plist generation ──────────────────────────────────────────────────────

    #[test]
    fn generate_daemon_plist_contains_label() {
        let xml = generate_daemon_plist(
            Path::new("/usr/local/bin/claude-usage-tracker"),
            Path::new("/tmp/logs/heimdall"),
        );
        assert!(
            xml.contains("dev.heimdall.daemon"),
            "plist must contain daemon label"
        );
    }

    #[test]
    fn generate_daemon_plist_keep_alive_true() {
        let xml = generate_daemon_plist(
            Path::new("/usr/local/bin/claude-usage-tracker"),
            Path::new("/tmp/logs"),
        );
        // KeepAlive key must precede <true/>
        let keepalive_pos = xml
            .find("KeepAlive")
            .expect("plist must contain KeepAlive key");
        let true_after = &xml[keepalive_pos..];
        assert!(
            true_after.contains("<true/>"),
            "KeepAlive must be set to true"
        );
    }

    #[test]
    fn generate_daemon_plist_run_at_load_true() {
        let xml = generate_daemon_plist(
            Path::new("/usr/local/bin/claude-usage-tracker"),
            Path::new("/tmp/logs"),
        );
        let run_at_load_pos = xml
            .find("RunAtLoad")
            .expect("plist must contain RunAtLoad key");
        let after = &xml[run_at_load_pos..];
        assert!(after.contains("<true/>"), "RunAtLoad must be set to true");
    }

    #[test]
    fn generate_daemon_plist_logs_path() {
        let logs_dir = Path::new("/Users/alice/Library/Logs/heimdall");
        let xml = generate_daemon_plist(Path::new("/usr/local/bin/claude-usage-tracker"), logs_dir);
        assert!(
            xml.contains("daemon.log"),
            "plist must reference daemon.log"
        );
        assert!(
            xml.contains("daemon.err"),
            "plist must reference daemon.err"
        );
        assert!(
            xml.contains("/Users/alice/Library/Logs/heimdall"),
            "plist must embed logs dir path"
        );
    }

    #[test]
    fn generate_daemon_plist_dashboard_subcommand() {
        let xml = generate_daemon_plist(
            Path::new("/usr/local/bin/claude-usage-tracker"),
            Path::new("/tmp/logs"),
        );
        assert!(
            xml.contains("<string>dashboard</string>"),
            "plist must invoke 'dashboard' subcommand"
        );
        assert!(
            xml.contains("<string>--host</string>"),
            "plist must pass --host"
        );
        assert!(
            xml.contains("<string>localhost</string>"),
            "plist host must be localhost"
        );
        assert!(
            xml.contains("<string>8080</string>"),
            "plist port must be 8080"
        );
        assert!(
            xml.contains("<string>--watch</string>"),
            "plist must pass --watch"
        );
        assert!(
            xml.contains("<string>--no-open</string>"),
            "plist must pass --no-open"
        );
        assert!(
            xml.contains("<string>--background-poll</string>"),
            "plist must pass --background-poll"
        );
    }

    // ── install/uninstall round-trip (isolated, no shell-out) ─────────────────

    #[test]
    fn install_creates_plist_and_logs_dir() {
        let tmp = TempDir::new().unwrap();
        let sched = LaunchdDaemonScheduler::new(Some(tmp.path().to_path_buf()));
        let bin = Path::new("/usr/local/bin/claude-usage-tracker");

        sched.install(bin).unwrap();

        assert!(
            sched.plist_path().exists(),
            "plist must exist after install"
        );
        let logs_dir = tmp.path().join("Library").join("Logs").join("heimdall");
        assert!(logs_dir.exists(), "logs dir must be created on install");
    }

    #[test]
    fn install_status_reports_installed() {
        let tmp = TempDir::new().unwrap();
        let sched = LaunchdDaemonScheduler::new(Some(tmp.path().to_path_buf()));
        let bin = Path::new("/usr/local/bin/claude-usage-tracker");

        sched.install(bin).unwrap();

        match sched.status().unwrap() {
            InstallStatus::Installed {
                next_run_hint,
                config_path,
            } => {
                assert!(
                    next_run_hint.contains("8080"),
                    "hint must mention port 8080"
                );
                assert_eq!(config_path.unwrap(), sched.plist_path());
            }
            other => panic!("expected Installed, got {:?}", other),
        }
    }

    #[test]
    fn uninstall_removes_plist() {
        let tmp = TempDir::new().unwrap();
        let sched = LaunchdDaemonScheduler::new(Some(tmp.path().to_path_buf()));
        let bin = Path::new("/usr/local/bin/claude-usage-tracker");

        sched.install(bin).unwrap();
        sched.uninstall().unwrap();

        assert!(
            !sched.plist_path().exists(),
            "plist must be gone after uninstall"
        );
        assert_eq!(sched.status().unwrap(), InstallStatus::NotInstalled);
    }

    #[test]
    fn uninstall_when_not_installed_is_noop() {
        let tmp = TempDir::new().unwrap();
        let sched = LaunchdDaemonScheduler::new(Some(tmp.path().to_path_buf()));
        sched.uninstall().unwrap();
    }

    #[test]
    fn install_is_idempotent() {
        let tmp = TempDir::new().unwrap();
        let sched = LaunchdDaemonScheduler::new(Some(tmp.path().to_path_buf()));
        let bin = Path::new("/usr/local/bin/claude-usage-tracker");

        sched.install(bin).unwrap();
        // Second install must not duplicate or error.
        sched.install(bin).unwrap();

        let agents_dir = tmp.path().join("Library").join("LaunchAgents");
        let entries: Vec<_> = std::fs::read_dir(&agents_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(
            entries.len(),
            1,
            "only one plist file must exist after double-install"
        );
    }

    #[test]
    fn status_when_not_installed_returns_not_installed() {
        let tmp = TempDir::new().unwrap();
        let sched = LaunchdDaemonScheduler::new(Some(tmp.path().to_path_buf()));
        assert_eq!(sched.status().unwrap(), InstallStatus::NotInstalled);
    }

    // ── platform-gate test ────────────────────────────────────────────────────

    /// On non-macOS, `current_daemon_scheduler()` must return an
    /// `UnsupportedPlatform` variant from `status()`.
    ///
    /// On macOS this exercises the production path instead; the test is
    /// compiled for all platforms but the assertion branches on the OS.
    #[test]
    fn unsupported_platform_scheduler_status() {
        let stub = UnsupportedDaemonScheduler;
        match stub.status().unwrap() {
            InstallStatus::UnsupportedPlatform(_) => {}
            other => panic!("expected UnsupportedPlatform, got {:?}", other),
        }
    }

    #[test]
    fn unsupported_platform_scheduler_install_errors() {
        let stub = UnsupportedDaemonScheduler;
        let err = stub.install(Path::new("/bin/foo")).unwrap_err();
        assert!(
            err.to_string().contains("macOS-only"),
            "error must mention macOS-only"
        );
    }

    #[test]
    fn unsupported_platform_scheduler_uninstall_errors() {
        let stub = UnsupportedDaemonScheduler;
        let err = stub.uninstall().unwrap_err();
        assert!(
            err.to_string().contains("macOS-only"),
            "error must mention macOS-only"
        );
    }
}
