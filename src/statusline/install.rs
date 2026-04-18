/// Install / uninstall the `statusLine` entry in `~/.claude/settings.json`.
///
/// Tag sentinel: `_heimdall_statusline_version` = `"v1"`.
/// The `statusLine` key is a plain string at the root of the JSON object.
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

pub const STATUSLINE_VERSION_KEY: &str = "_heimdall_statusline_version";
pub const STATUSLINE_VERSION_VAL: &str = "v1";
pub const STATUSLINE_COMMAND: &str = "claude-usage-tracker statusline";

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
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
        .join("settings.json")
}

fn backup_path(settings: &Path) -> PathBuf {
    let mut p = settings.to_path_buf();
    let mut name = p.file_name().unwrap_or_default().to_os_string();
    name.push(".heimdall-bak");
    p.set_file_name(name);
    p
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

    write_backup(settings_path, &root)?;

    let obj = root
        .as_object_mut()
        .context("settings.json is not an object")?;
    obj.insert(
        "statusLine".to_string(),
        serde_json::Value::String(STATUSLINE_COMMAND.to_string()),
    );
    obj.insert(
        STATUSLINE_VERSION_KEY.to_string(),
        serde_json::Value::String(STATUSLINE_VERSION_VAL.to_string()),
    );

    write_settings(settings_path, &root)?;
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

    // Only remove if we own it (version tag present).
    let version_present = root
        .get(STATUSLINE_VERSION_KEY)
        .and_then(|v| v.as_str())
        .map(|v| v == STATUSLINE_VERSION_VAL)
        .unwrap_or(false);

    if !version_present {
        return Ok(StatuslineActionResult::NothingToUninstall);
    }

    write_backup(settings_path, &root)?;

    let obj = root
        .as_object_mut()
        .context("settings.json is not an object")?;
    obj.remove("statusLine");
    obj.remove(STATUSLINE_VERSION_KEY);

    write_settings(settings_path, &root)?;
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
    root.get(STATUSLINE_VERSION_KEY)
        .and_then(|v| v.as_str())
        .map(|v| v == STATUSLINE_VERSION_VAL)
        .unwrap_or(false)
}

fn read_or_empty_object(path: &Path) -> Result<serde_json::Value> {
    if !path.exists() {
        return Ok(serde_json::json!({}));
    }
    let text =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    if text.trim().is_empty() {
        return Ok(serde_json::json!({}));
    }
    serde_json::from_str(&text).with_context(|| format!("parsing JSON from {}", path.display()))
}

fn write_settings(path: &Path, value: &serde_json::Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(value)?;
    let mut file =
        std::fs::File::create(path).with_context(|| format!("writing {}", path.display()))?;
    file.write_all(text.as_bytes())?;
    Ok(())
}

fn write_backup(settings_path: &Path, value: &serde_json::Value) -> Result<()> {
    let bak = backup_path(settings_path);
    let text = serde_json::to_string_pretty(value)?;
    let mut file =
        std::fs::File::create(&bak).with_context(|| format!("writing backup {}", bak.display()))?;
    file.write_all(text.as_bytes())?;
    Ok(())
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
        assert_eq!(
            v[STATUSLINE_VERSION_KEY].as_str().unwrap(),
            STATUSLINE_VERSION_VAL
        );
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
        assert!(backup_path(&settings).exists());
    }

    #[test]
    fn backup_written_on_uninstall() {
        let dir = TempDir::new().unwrap();
        let settings = tmp_settings(&dir);
        install_into(&settings).unwrap();
        let _ = std::fs::remove_file(backup_path(&settings));
        uninstall_from(&settings).unwrap();
        assert!(backup_path(&settings).exists());
    }
}
