//! Install / uninstall the `statusLine` entry in `~/.claude/settings.json`.
//!
//! # Ownership marker vs version stamp
//!
//! Ownership of the entry is detected by the *presence* of
//! [`STATUSLINE_VERSION_KEY`] — not its value. The value carries the
//! heimdall package version at install time (`env!("CARGO_PKG_VERSION")`),
//! so users can answer "what version is installed?" with
//! `grep heimdall ~/.claude/settings.json` instead of running
//! `claude-usage-tracker statusline status`. Pattern borrowed from
//! talk-normal's `<!-- talk-normal X.Y.Z -->` convention.
//!
//! Statusline shares the `_heimdall_version` key name with the hook
//! entry. The two live at different nesting levels in `settings.json`
//! (statusline at the root, hook inside each `hooks.PreToolUse[]`
//! entry) so they cannot collide, and a single
//! `grep _heimdall_version ~/.claude/settings.json` covers both
//! surfaces.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::install_json::{
    claude_settings_json_path, read_or_empty_object, write_object, write_object_backup,
};

/// Ownership / version-stamp key. Shares the `_heimdall_version` namespace
/// with the hook entry — see module docblock.
pub const STATUSLINE_VERSION_KEY: &str = "_heimdall_version";

pub const STATUSLINE_COMMAND: &str = "claude-usage-tracker statusline";

/// Package version stamped into the entry's value field at install time.
/// Sourced from `Cargo.toml` via `env!`. Informational — see module
/// docblock § "Ownership marker vs version stamp".
pub fn statusline_version_value() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Outcome returned by `install` and `uninstall`.
///
/// Install is *replace-in-place idempotent* in the talk-normal sense: every
/// re-run reaches the current state, including refreshing the version stamp
/// written by improvement (5). First run returns `Installed`; subsequent
/// runs return `Updated`. There is no `AlreadyInstalled` no-op variant —
/// see `src/hook/install.rs::HookActionResult` for the symmetric rationale.
#[derive(Debug, PartialEq)]
pub enum StatuslineActionResult {
    /// First-time install — entry was not previously present.
    Installed,
    /// Re-run install — existing entry was replaced with the current
    /// command and version stamp.
    Updated,
    Uninstalled,
    NothingToUninstall,
}

#[derive(Debug, PartialEq)]
pub enum StatuslineStatus {
    Present { command: String },
    Absent,
}

// ── Path helpers ──────────────────────────────────────────────────────────────

fn settings_json_path() -> PathBuf {
    claude_settings_json_path()
}

// ── Public API ────────────────────────────────────────────────────────────────

pub fn install() -> Result<StatuslineActionResult> {
    install_into(&settings_json_path())
}

pub fn install_into(settings_path: &Path) -> Result<StatuslineActionResult> {
    let mut root = read_or_empty_object(settings_path)?;

    // Was our entry already present? Determines `Installed` vs `Updated`.
    // Replace-in-place idempotency: re-running always reaches the current
    // state, including refreshing the version stamp from improvement (5).
    let was_installed = is_installed(&root);

    write_object_backup(settings_path, &root)?;

    let obj = root
        .as_object_mut()
        .context("settings.json is not an object")?;
    // `obj.insert` overwrites by default, so an existing entry is replaced
    // with the current command + version stamp without an explicit strip.
    obj.insert(
        "statusLine".to_string(),
        serde_json::Value::String(STATUSLINE_COMMAND.to_string()),
    );
    obj.insert(
        STATUSLINE_VERSION_KEY.to_string(),
        serde_json::Value::String(statusline_version_value().to_string()),
    );

    write_object(settings_path, &root)?;
    Ok(if was_installed {
        StatuslineActionResult::Updated
    } else {
        StatuslineActionResult::Installed
    })
}

pub fn uninstall() -> Result<StatuslineActionResult> {
    uninstall_from(&settings_json_path())
}

pub fn uninstall_from(settings_path: &Path) -> Result<StatuslineActionResult> {
    if !settings_path.exists() {
        return Ok(StatuslineActionResult::NothingToUninstall);
    }

    let mut root = read_or_empty_object(settings_path)?;

    // Only remove if we own it (STATUSLINE_VERSION_KEY present).
    if !is_installed(&root) {
        return Ok(StatuslineActionResult::NothingToUninstall);
    }

    write_object_backup(settings_path, &root)?;

    let obj = root
        .as_object_mut()
        .context("settings.json is not an object")?;
    obj.remove("statusLine");
    obj.remove(STATUSLINE_VERSION_KEY);

    write_object(settings_path, &root)?;
    Ok(StatuslineActionResult::Uninstalled)
}

pub fn status() -> Result<StatuslineStatus> {
    status_from(&settings_json_path())
}

pub fn status_from(settings_path: &Path) -> Result<StatuslineStatus> {
    if !settings_path.exists() {
        return Ok(StatuslineStatus::Absent);
    }
    let root = read_or_empty_object(settings_path)?;
    if !is_installed(&root) {
        return Ok(StatuslineStatus::Absent);
    }
    let command = root
        .get("statusLine")
        .and_then(|v| v.as_str())
        .unwrap_or("<unknown>")
        .to_string();
    Ok(StatuslineStatus::Present { command })
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn is_installed(root: &serde_json::Value) -> bool {
    // Ownership is key-presence, not value-equality — see module docblock.
    root.get(STATUSLINE_VERSION_KEY)
        .and_then(|v| v.as_str())
        .is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn tmp_settings(dir: &TempDir) -> PathBuf {
        dir.path().join("settings.json")
    }

    #[test]
    fn install_into_empty_file() {
        let dir = TempDir::new().unwrap();
        let settings = tmp_settings(&dir);

        let result = install_into(&settings).unwrap();
        assert_eq!(result, StatuslineActionResult::Installed);

        let v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&settings).unwrap()).unwrap();
        assert_eq!(v["statusLine"].as_str().unwrap(), STATUSLINE_COMMAND);
        // The value carries the heimdall package version (talk-normal-style
        // grep-friendly stamp); see module docblock § "Ownership marker vs
        // version stamp".
        assert_eq!(
            v[STATUSLINE_VERSION_KEY].as_str().unwrap(),
            env!("CARGO_PKG_VERSION"),
        );
    }

    /// Re-running install reports `Updated` and reaches the current state.
    /// The entry's value stays the canonical command (no drift if a stale
    /// `statusLine` was previously written by an older heimdall) and the
    /// version key reflects the running binary.
    #[test]
    fn install_is_replace_in_place_idempotent() {
        let dir = TempDir::new().unwrap();
        let settings = tmp_settings(&dir);

        let r1 = install_into(&settings).unwrap();
        assert_eq!(r1, StatuslineActionResult::Installed);

        let r2 = install_into(&settings).unwrap();
        assert_eq!(r2, StatuslineActionResult::Updated);

        let v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&settings).unwrap()).unwrap();
        assert_eq!(v["statusLine"].as_str().unwrap(), STATUSLINE_COMMAND);
        assert_eq!(
            v[STATUSLINE_VERSION_KEY].as_str().unwrap(),
            env!("CARGO_PKG_VERSION"),
        );
    }

    #[test]
    fn install_preserves_existing_keys() {
        let dir = TempDir::new().unwrap();
        let settings = tmp_settings(&dir);
        let initial = serde_json::json!({"someKey": "someValue"});
        std::fs::write(&settings, serde_json::to_string_pretty(&initial).unwrap()).unwrap();

        install_into(&settings).unwrap();
        let v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&settings).unwrap()).unwrap();
        assert_eq!(v["someKey"].as_str().unwrap(), "someValue");
        assert_eq!(v["statusLine"].as_str().unwrap(), STATUSLINE_COMMAND);
    }

    #[test]
    fn uninstall_removes_both_keys() {
        let dir = TempDir::new().unwrap();
        let settings = tmp_settings(&dir);

        install_into(&settings).unwrap();
        let result = uninstall_from(&settings).unwrap();
        assert_eq!(result, StatuslineActionResult::Uninstalled);

        let v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&settings).unwrap()).unwrap();
        assert!(v.get("statusLine").is_none());
        assert!(v.get(STATUSLINE_VERSION_KEY).is_none());
    }

    #[test]
    fn uninstall_without_version_tag_is_noop() {
        let dir = TempDir::new().unwrap();
        let settings = tmp_settings(&dir);
        // User has their own statusLine — no version tag.
        let initial = serde_json::json!({"statusLine": "my custom command"});
        std::fs::write(&settings, serde_json::to_string_pretty(&initial).unwrap()).unwrap();

        let result = uninstall_from(&settings).unwrap();
        assert_eq!(result, StatuslineActionResult::NothingToUninstall);

        // User's value must be preserved.
        let v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&settings).unwrap()).unwrap();
        assert_eq!(v["statusLine"].as_str().unwrap(), "my custom command");
    }

    #[test]
    fn round_trip_install_uninstall() {
        let dir = TempDir::new().unwrap();
        let settings = tmp_settings(&dir);
        let initial = serde_json::json!({"other": 42});
        std::fs::write(&settings, serde_json::to_string_pretty(&initial).unwrap()).unwrap();
        let before: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&settings).unwrap()).unwrap();

        install_into(&settings).unwrap();
        uninstall_from(&settings).unwrap();

        let after: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&settings).unwrap()).unwrap();
        // Pre-existing keys survive the round-trip.
        assert_eq!(before["other"], after["other"]);
        assert!(after.get("statusLine").is_none());
    }

    #[test]
    fn status_absent_when_no_file() {
        let dir = TempDir::new().unwrap();
        let settings = tmp_settings(&dir);
        assert_eq!(status_from(&settings).unwrap(), StatuslineStatus::Absent);
    }

    #[test]
    fn status_present_after_install() {
        let dir = TempDir::new().unwrap();
        let settings = tmp_settings(&dir);
        install_into(&settings).unwrap();
        let s = status_from(&settings).unwrap();
        assert!(matches!(s, StatuslineStatus::Present { .. }));
    }

    #[test]
    fn status_absent_after_uninstall() {
        let dir = TempDir::new().unwrap();
        let settings = tmp_settings(&dir);
        install_into(&settings).unwrap();
        uninstall_from(&settings).unwrap();
        assert_eq!(status_from(&settings).unwrap(), StatuslineStatus::Absent);
    }

    #[test]
    fn backup_written_on_install() {
        let dir = TempDir::new().unwrap();
        let settings = tmp_settings(&dir);
        install_into(&settings).unwrap();
        assert!(crate::install_json::backup_path(&settings).exists());
    }

    #[test]
    fn backup_written_on_uninstall() {
        let dir = TempDir::new().unwrap();
        let settings = tmp_settings(&dir);
        install_into(&settings).unwrap();
        let _ = std::fs::remove_file(crate::install_json::backup_path(&settings));
        uninstall_from(&settings).unwrap();
        assert!(crate::install_json::backup_path(&settings).exists());
    }
}
