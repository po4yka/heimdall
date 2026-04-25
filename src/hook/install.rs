//! `hook install` / `hook uninstall` / `hook status` subcommand logic.
//!
//! Manages the Claude Code `~/.claude/settings.json` hook entry that points
//! at the `heimdall-hook` binary so every PreToolUse event is ingested in
//! real-time.
//!
//! # Ownership marker vs version stamp
//!
//! Tag sentinel used to find/remove our entries:
//!   `"description": "heimdall real-time ingest"` ([`HOOK_DESCRIPTION`]).
//!
//! Each installed entry also carries a [`HOOK_VERSION_KEY`] field whose value
//! is `env!("CARGO_PKG_VERSION")` — the heimdall package version at the time
//! `install` was run. The version stamp is **purely informational** so users
//! can answer "what version is installed?" with
//! `grep heimdall ~/.claude/settings.json` instead of running
//! `claude-usage-tracker hook status`. Pattern borrowed from talk-normal's
//! `<!-- talk-normal X.Y.Z -->` convention. Ownership detection uses
//! `HOOK_DESCRIPTION` so older-version entries are uninstallable by
//! newer-version binaries.

use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::install_json::{
    claude_settings_json_path, read_or_empty_object, write_object, write_object_backup,
};

pub const HOOK_DESCRIPTION: &str = "heimdall real-time ingest";

/// JSON key that carries the heimdall package version on each installed
/// entry. Informational only — see module docblock § "Ownership marker vs
/// version stamp".
pub const HOOK_VERSION_KEY: &str = "_heimdall_version";

/// Outcome returned by `install` and `uninstall`.
#[derive(Debug, PartialEq)]
pub enum HookActionResult {
    Installed { binary_path: PathBuf },
    AlreadyInstalled { binary_path: PathBuf },
    Uninstalled,
    NothingToUninstall,
}

/// Status of the hook entry in settings.json.
#[derive(Debug, PartialEq)]
pub enum HookStatus {
    Present { binary_path: String },
    Absent,
}

// ── Path helpers ─────────────────────────────────────────────────────────────

/// Resolve the absolute path to the `heimdall-hook` binary.
/// Looks for it next to the currently running binary.
pub fn resolve_hook_binary_path() -> Result<PathBuf> {
    let exe = std::env::current_exe().context("cannot resolve current executable path")?;
    let dir = exe
        .parent()
        .context("cannot determine executable directory")?;
    Ok(dir.join("heimdall-hook"))
}

fn settings_json_path() -> PathBuf {
    claude_settings_json_path()
}

// ── Core operations ──────────────────────────────────────────────────────────

/// Install the heimdall-hook entry into `~/.claude/settings.json`.
///
/// - Creates the file if absent.
/// - Appends to existing `PreToolUse` hooks; does not clobber.
/// - Writes a `.heimdall-bak` backup before every modification.
/// - Is idempotent: returns `AlreadyInstalled` if our tag is already present.
pub fn install(hook_binary: &std::path::Path) -> Result<HookActionResult> {
    install_into(&settings_json_path(), hook_binary)
}

pub fn install_into(
    settings_path: &std::path::Path,
    hook_binary: &std::path::Path,
) -> Result<HookActionResult> {
    let mut root = read_or_empty_object(settings_path)?;

    // Check for duplicate.
    if find_heimdall_entry(&root).is_some() {
        return Ok(HookActionResult::AlreadyInstalled {
            binary_path: hook_binary.to_path_buf(),
        });
    }

    // Backup before modification.
    write_object_backup(settings_path, &root)?;

    // Build the new entry object. The "_heimdall_version" key is the
    // grep-friendly version stamp documented in the module docblock — keep
    // its literal name in sync with HOOK_VERSION_KEY (asserted by
    // `install_writes_pkg_version_key`).
    let entry = serde_json::json!({
        "type": "command",
        "command": hook_binary.to_string_lossy().as_ref(),
        "timeout": 5,
        "description": HOOK_DESCRIPTION,
        "_heimdall_version": env!("CARGO_PKG_VERSION"),
    });

    // Ensure hooks.PreToolUse array exists and append.
    let hooks = root
        .as_object_mut()
        .context("settings.json root must be an object")?
        .entry("hooks")
        .or_insert_with(|| serde_json::json!({}));

    let pre_tool_use = hooks
        .as_object_mut()
        .context("hooks must be an object")?
        .entry("PreToolUse")
        .or_insert_with(|| serde_json::json!([]));

    pre_tool_use
        .as_array_mut()
        .context("PreToolUse is not an array")?
        .push(entry);

    write_object(settings_path, &root)?;

    Ok(HookActionResult::Installed {
        binary_path: hook_binary.to_path_buf(),
    })
}

/// Remove all heimdall-tagged entries from `~/.claude/settings.json`.
///
/// Leaves unrelated hooks intact. Backs up the file before every modification.
pub fn uninstall() -> Result<HookActionResult> {
    uninstall_from(&settings_json_path())
}

pub fn uninstall_from(settings_path: &std::path::Path) -> Result<HookActionResult> {
    if !settings_path.exists() {
        return Ok(HookActionResult::NothingToUninstall);
    }

    let mut root = read_or_empty_object(settings_path)?;

    if find_heimdall_entry(&root).is_none() {
        return Ok(HookActionResult::NothingToUninstall);
    }

    // Backup before modification.
    write_object_backup(settings_path, &root)?;

    // Remove heimdall entries from PreToolUse.
    let removed = remove_heimdall_entries(&mut root);
    if !removed {
        return Ok(HookActionResult::NothingToUninstall);
    }

    write_object(settings_path, &root)?;
    Ok(HookActionResult::Uninstalled)
}

/// Check whether the hook entry is present.
pub fn status() -> Result<HookStatus> {
    status_from(&settings_json_path())
}

pub fn status_from(settings_path: &std::path::Path) -> Result<HookStatus> {
    if !settings_path.exists() {
        return Ok(HookStatus::Absent);
    }
    let root = read_or_empty_object(settings_path)?;
    match find_heimdall_entry(&root) {
        Some(entry) => {
            let binary_path = entry
                .get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("<unknown>")
                .to_string();
            Ok(HookStatus::Present { binary_path })
        }
        None => Ok(HookStatus::Absent),
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Find the first heimdall-tagged entry in `hooks.PreToolUse`, if any.
fn find_heimdall_entry(root: &serde_json::Value) -> Option<&serde_json::Value> {
    root.get("hooks")?
        .get("PreToolUse")?
        .as_array()?
        .iter()
        .find(|entry| {
            entry
                .get("description")
                .and_then(|v| v.as_str())
                .map(|d| d == HOOK_DESCRIPTION)
                .unwrap_or(false)
        })
}

/// Remove all heimdall-tagged entries from `hooks.PreToolUse`.
/// Returns `true` if at least one entry was removed.
fn remove_heimdall_entries(root: &mut serde_json::Value) -> bool {
    let Some(hooks) = root.get_mut("hooks") else {
        return false;
    };
    let Some(pre_tool_use) = hooks.get_mut("PreToolUse") else {
        return false;
    };
    let Some(arr) = pre_tool_use.as_array_mut() else {
        return false;
    };

    let before = arr.len();
    arr.retain(|entry| {
        entry
            .get("description")
            .and_then(|v| v.as_str())
            .map(|d| d != HOOK_DESCRIPTION)
            .unwrap_or(true)
    });
    let removed = arr.len() < before;

    // If PreToolUse is now empty, remove it.
    if arr.is_empty() {
        hooks.as_object_mut().unwrap().remove("PreToolUse");
    }

    // If hooks is now empty, remove it.
    if hooks.as_object().map(|o| o.is_empty()).unwrap_or(false) {
        root.as_object_mut().unwrap().remove("hooks");
    }

    removed
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn tmp_settings(dir: &TempDir) -> PathBuf {
        dir.path().join("settings.json")
    }

    fn hook_bin(dir: &TempDir) -> PathBuf {
        dir.path().join("heimdall-hook")
    }

    /// The installed entry carries the package-version stamp documented in
    /// the module docblock § "Ownership marker vs version stamp". Catches
    /// drift between the literal `"_heimdall_version"` in `install_into`
    /// and the `HOOK_VERSION_KEY` constant.
    #[test]
    fn install_writes_pkg_version_key() {
        let dir = TempDir::new().unwrap();
        let settings = tmp_settings(&dir);
        let bin = hook_bin(&dir);

        install_into(&settings, &bin).unwrap();

        let v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&settings).unwrap()).unwrap();
        let entry = &v["hooks"]["PreToolUse"][0];

        let stamped = entry
            .get(HOOK_VERSION_KEY)
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("missing {HOOK_VERSION_KEY}: {entry:?}"));
        assert_eq!(
            stamped,
            env!("CARGO_PKG_VERSION"),
            "version stamp must match Cargo.toml package version"
        );
    }

    #[test]
    fn install_into_empty_file_adds_entry() {
        let dir = TempDir::new().unwrap();
        let settings = tmp_settings(&dir);
        let bin = hook_bin(&dir);

        let result = install_into(&settings, &bin).unwrap();
        assert!(matches!(result, HookActionResult::Installed { .. }));

        let content = std::fs::read_to_string(&settings).unwrap();
        let v: serde_json::Value = serde_json::from_str(&content).unwrap();
        let pre_tool_use = v["hooks"]["PreToolUse"].as_array().unwrap();
        assert_eq!(pre_tool_use.len(), 1);
        assert_eq!(
            pre_tool_use[0]["description"].as_str().unwrap(),
            HOOK_DESCRIPTION
        );
        assert_eq!(
            pre_tool_use[0]["command"].as_str().unwrap(),
            bin.to_string_lossy().as_ref()
        );
    }

    #[test]
    fn install_is_idempotent() {
        let dir = TempDir::new().unwrap();
        let settings = tmp_settings(&dir);
        let bin = hook_bin(&dir);

        install_into(&settings, &bin).unwrap();
        let result2 = install_into(&settings, &bin).unwrap();
        assert!(matches!(result2, HookActionResult::AlreadyInstalled { .. }));

        let content = std::fs::read_to_string(&settings).unwrap();
        let v: serde_json::Value = serde_json::from_str(&content).unwrap();
        // Only one entry should exist.
        assert_eq!(v["hooks"]["PreToolUse"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn install_rejects_non_object_root() {
        let dir = TempDir::new().unwrap();
        let settings = tmp_settings(&dir);
        let bin = hook_bin(&dir);

        std::fs::write(&settings, "[]").unwrap();

        let error = install_into(&settings, &bin).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("settings.json root must be an object")
        );
    }

    #[test]
    fn install_rejects_non_object_hooks_value() {
        let dir = TempDir::new().unwrap();
        let settings = tmp_settings(&dir);
        let bin = hook_bin(&dir);

        std::fs::write(
            &settings,
            serde_json::to_string_pretty(&serde_json::json!({
                "hooks": []
            }))
            .unwrap(),
        )
        .unwrap();

        let error = install_into(&settings, &bin).unwrap_err();
        assert!(error.to_string().contains("hooks must be an object"));
    }

    #[test]
    fn install_preserves_existing_unrelated_hooks() {
        let dir = TempDir::new().unwrap();
        let settings = tmp_settings(&dir);
        let bin = hook_bin(&dir);

        // Write a settings.json with an unrelated hook.
        let pre_existing = serde_json::json!({
            "hooks": {
                "PreToolUse": [
                    {
                        "type": "command",
                        "command": "/usr/local/bin/other-tool",
                        "description": "some other tool"
                    }
                ]
            }
        });
        std::fs::write(
            &settings,
            serde_json::to_string_pretty(&pre_existing).unwrap(),
        )
        .unwrap();

        install_into(&settings, &bin).unwrap();

        let content = std::fs::read_to_string(&settings).unwrap();
        let v: serde_json::Value = serde_json::from_str(&content).unwrap();
        let arr = v["hooks"]["PreToolUse"].as_array().unwrap();
        // Both entries must be present.
        assert_eq!(arr.len(), 2);
        assert!(arr.iter().any(|e| e["description"] == "some other tool"));
        assert!(arr.iter().any(|e| e["description"] == HOOK_DESCRIPTION));
    }

    #[test]
    fn uninstall_removes_entry_and_leaves_file_clean() {
        let dir = TempDir::new().unwrap();
        let settings = tmp_settings(&dir);
        let bin = hook_bin(&dir);

        install_into(&settings, &bin).unwrap();

        // Capture the raw JSON before install for later comparison.
        // After uninstall the file should contain no hooks key.
        let result = uninstall_from(&settings).unwrap();
        assert_eq!(result, HookActionResult::Uninstalled);

        let content = std::fs::read_to_string(&settings).unwrap();
        let v: serde_json::Value = serde_json::from_str(&content).unwrap();
        // hooks key should be absent when there are no remaining entries.
        assert!(v.get("hooks").is_none());
    }

    #[test]
    fn uninstall_preserves_unrelated_hooks() {
        let dir = TempDir::new().unwrap();
        let settings = tmp_settings(&dir);
        let bin = hook_bin(&dir);

        // Start with an unrelated hook already present.
        let initial = serde_json::json!({
            "hooks": {
                "PreToolUse": [
                    {
                        "type": "command",
                        "command": "/usr/bin/other",
                        "description": "unrelated"
                    }
                ]
            }
        });
        std::fs::write(&settings, serde_json::to_string_pretty(&initial).unwrap()).unwrap();

        install_into(&settings, &bin).unwrap();
        uninstall_from(&settings).unwrap();

        let content = std::fs::read_to_string(&settings).unwrap();
        let v: serde_json::Value = serde_json::from_str(&content).unwrap();
        let arr = v["hooks"]["PreToolUse"].as_array().unwrap();
        // Only the unrelated entry must remain.
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["description"], "unrelated");
    }

    #[test]
    fn uninstall_on_nonexistent_file_is_a_no_op() {
        let dir = TempDir::new().unwrap();
        let settings = tmp_settings(&dir);
        let result = uninstall_from(&settings).unwrap();
        assert_eq!(result, HookActionResult::NothingToUninstall);
    }

    #[test]
    fn install_then_uninstall_roundtrip() {
        let dir = TempDir::new().unwrap();
        let settings = tmp_settings(&dir);
        let bin = hook_bin(&dir);

        // Start from no file.
        install_into(&settings, &bin).unwrap();
        let after_install: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&settings).unwrap()).unwrap();
        assert!(
            after_install["hooks"]["PreToolUse"]
                .as_array()
                .unwrap()
                .len()
                == 1
        );

        uninstall_from(&settings).unwrap();
        let after_uninstall: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&settings).unwrap()).unwrap();
        // No hooks key — clean state.
        assert!(after_uninstall.get("hooks").is_none());
    }

    #[test]
    fn backup_is_written_on_install() {
        let dir = TempDir::new().unwrap();
        let settings = tmp_settings(&dir);
        let bin = hook_bin(&dir);

        install_into(&settings, &bin).unwrap();

        let bak = crate::install_json::backup_path(&settings);
        assert!(bak.exists(), "backup file should exist after install");
    }

    #[test]
    fn backup_is_written_on_uninstall() {
        let dir = TempDir::new().unwrap();
        let settings = tmp_settings(&dir);
        let bin = hook_bin(&dir);

        install_into(&settings, &bin).unwrap();
        // Remove the backup from install so we can verify uninstall creates it.
        let bak = crate::install_json::backup_path(&settings);
        let _ = std::fs::remove_file(&bak);

        uninstall_from(&settings).unwrap();
        assert!(bak.exists(), "backup file should exist after uninstall");
    }

    #[test]
    fn status_absent_when_no_file() {
        let dir = TempDir::new().unwrap();
        let settings = tmp_settings(&dir);
        let s = status_from(&settings).unwrap();
        assert_eq!(s, HookStatus::Absent);
    }

    #[test]
    fn status_present_after_install() {
        let dir = TempDir::new().unwrap();
        let settings = tmp_settings(&dir);
        let bin = hook_bin(&dir);

        install_into(&settings, &bin).unwrap();
        let s = status_from(&settings).unwrap();
        assert!(matches!(s, HookStatus::Present { .. }));
        if let HookStatus::Present { binary_path } = s {
            assert_eq!(binary_path, bin.to_string_lossy().as_ref());
        }
    }

    #[test]
    fn status_absent_after_uninstall() {
        let dir = TempDir::new().unwrap();
        let settings = tmp_settings(&dir);
        let bin = hook_bin(&dir);

        install_into(&settings, &bin).unwrap();
        uninstall_from(&settings).unwrap();
        let s = status_from(&settings).unwrap();
        assert_eq!(s, HookStatus::Absent);
    }
}
