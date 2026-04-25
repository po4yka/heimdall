//! Install / uninstall the `statusLine` entry in `~/.claude/settings.json`.
//!
//! # Ownership marker vs version stamp
//!
//! Ownership of the entry is detected by the *presence* of
//! [`STATUSLINE_VERSION_KEY`] — not its value. The value carries the
//! heimdall package version at install time
//! (`env!("CARGO_PKG_VERSION")`), so users can answer "what version is
//! installed?" with `grep heimdall ~/.claude/settings.json` instead of
//! running `claude-usage-tracker statusline status`. Pattern borrowed
//! from talk-normal's `<!-- talk-normal X.Y.Z -->` convention.
//!
//! Pre-0.1.0 installs wrote the literal string `"v1"` as a schema
//! marker; the new key-presence detection unblocks newer binaries from
//! cleanly uninstalling those legacy entries. The schema-version idea
//! is preserved by the underscore-prefixed key name itself: a future
//! incompatible format would introduce a new key
//! (`_heimdall_statusline_v2`) rather than reusing this one.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::install_json::{
    claude_settings_json_path, read_or_empty_object, write_object, write_object_backup,
};

pub const STATUSLINE_VERSION_KEY: &str = "_heimdall_statusline_version";
pub const STATUSLINE_COMMAND: &str = "claude-usage-tracker statusline";

/// Package version stamped into the entry's value field at install time.
/// Sourced from `Cargo.toml` via `env!`. Informational — see module
/// docblock § "Ownership marker vs version stamp".
pub fn statusline_version_value() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[derive(Debug, PartialEq)]
pub enum StatuslineActionResult {
    Installed,
    AlreadyInstalled,
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

    // Idempotent: already installed?
    if is_installed(&root) {
        return Ok(StatuslineActionResult::AlreadyInstalled);
    }

    write_object_backup(settings_path, &root)?;

    let obj = root
        .as_object_mut()
        .context("settings.json is not an object")?;
    obj.insert(
        "statusLine".to_string(),
        serde_json::Value::String(STATUSLINE_COMMAND.to_string()),
    );
    obj.insert(
        STATUSLINE_VERSION_KEY.to_string(),
        serde_json::Value::String(statusline_version_value().to_string()),
    );

    write_object(settings_path, &root)?;
    Ok(StatuslineActionResult::Installed)
}

pub fn uninstall() -> Result<StatuslineActionResult> {
    uninstall_from(&settings_json_path())
}

pub fn uninstall_from(settings_path: &Path) -> Result<StatuslineActionResult> {
    if !settings_path.exists() {
        return Ok(StatuslineActionResult::NothingToUninstall);
    }

    let mut root = read_or_empty_object(settings_path)?;

    // Only remove if we own it. Ownership is the *presence* of
    // STATUSLINE_VERSION_KEY (any string value), not equality with a
    // specific version literal — that lets a 0.2.0 binary uninstall an
    // entry that 0.1.0 wrote.
    let version_present = root
        .get(STATUSLINE_VERSION_KEY)
        .and_then(|v| v.as_str())
        .is_some();

    if !version_present {
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

    /// A statusline entry written under the legacy `"v1"` schema marker
    /// must be uninstallable by a binary that writes the new package-
    /// version stamp. Ownership detection is key-presence, not value
    /// equality.
    #[test]
    fn uninstall_handles_legacy_v1_value() {
        let dir = TempDir::new().unwrap();
        let settings = tmp_settings(&dir);
        // Legacy fixture: `v1` value as written by pre-version-stamp builds.
        let initial = serde_json::json!({
            "statusLine": STATUSLINE_COMMAND,
            STATUSLINE_VERSION_KEY: "v1",
        });
        std::fs::write(&settings, serde_json::to_string_pretty(&initial).unwrap()).unwrap();

        let result = uninstall_from(&settings).unwrap();
        assert_eq!(result, StatuslineActionResult::Uninstalled);

        let v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&settings).unwrap()).unwrap();
        assert!(v.get("statusLine").is_none());
        assert!(v.get(STATUSLINE_VERSION_KEY).is_none());
    }

    #[test]
    fn install_is_idempotent() {
        let dir = TempDir::new().unwrap();
        let settings = tmp_settings(&dir);

        install_into(&settings).unwrap();
        let result2 = install_into(&settings).unwrap();
        assert_eq!(result2, StatuslineActionResult::AlreadyInstalled);
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
