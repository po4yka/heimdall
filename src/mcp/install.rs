/// `mcp install` / `mcp uninstall` / `mcp status` logic.
///
/// Writes the heimdall MCP server entry into the per-client mcp.json file.
///
/// Supported clients:
///   - `claude-code`    → ~/.claude/.mcp.json
///   - `claude-desktop` → `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS)
///     → `~/.config/Claude/claude_desktop_config.json` (Linux)
///     → `%APPDATA%\Claude\claude_desktop_config.json` (Windows)
///   - `cursor`         → ~/.cursor/mcp.json
///
/// Tag sentinel: `"_heimdall_mcp_version": "v1"` inside the `"heimdall"` entry.
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::install_json::{read_or_empty_object, write_object, write_object_backup_if_present};

const SENTINEL_KEY: &str = "_heimdall_mcp_version";
const SENTINEL_VAL: &str = "v1";
const SERVER_KEY: &str = "heimdall";

// ── Public types ─────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq)]
pub enum McpInstallResult {
    Installed { path: PathBuf },
    AlreadyInstalled { path: PathBuf },
    Uninstalled { path: PathBuf },
    NothingToUninstall,
}

#[derive(Debug, PartialEq)]
pub enum McpInstallStatus {
    /// Our sentinel is present — installed by us.
    Installed { path: PathBuf },
    /// A `heimdall` entry exists but lacks our sentinel — user-customized.
    Customized { path: PathBuf },
    /// No `heimdall` entry.
    Absent,
}

// ── Path resolution ───────────────────────────────────────────────────────────

/// Resolve the mcp.json path for a given client name.
///
/// Respects `HEIMDALL_MCP_CONFIG_DIR` env override (used in tests).
pub fn resolve_client_path(client: &str) -> Result<PathBuf> {
    let override_dir = std::env::var("HEIMDALL_MCP_CONFIG_DIR").ok();

    if let Some(ref dir) = override_dir {
        let dir = PathBuf::from(dir);
        let file = match client {
            "claude-code" => ".mcp.json",
            "claude-desktop" => "claude_desktop_config.json",
            "cursor" => "mcp.json",
            other => anyhow::bail!(
                "unknown client '{}'; expected claude-code | claude-desktop | cursor",
                other
            ),
        };
        return Ok(dir.join(file));
    }

    let home = dirs::home_dir().context("cannot determine home directory")?;

    match client {
        "claude-code" => Ok(home.join(".claude").join(".mcp.json")),
        "claude-desktop" => {
            #[cfg(target_os = "macos")]
            {
                Ok(dirs::config_dir()
                    .unwrap_or_else(|| home.join("Library").join("Application Support"))
                    .join("Claude")
                    .join("claude_desktop_config.json"))
            }
            #[cfg(target_os = "linux")]
            {
                Ok(dirs::config_dir()
                    .unwrap_or_else(|| home.join(".config"))
                    .join("Claude")
                    .join("claude_desktop_config.json"))
            }
            #[cfg(target_os = "windows")]
            {
                Ok(dirs::config_dir()
                    .context("cannot determine AppData dir")?
                    .join("Claude")
                    .join("claude_desktop_config.json"))
            }
            #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
            {
                Ok(home
                    .join(".config")
                    .join("Claude")
                    .join("claude_desktop_config.json"))
            }
        }
        "cursor" => Ok(home.join(".cursor").join("mcp.json")),
        other => anyhow::bail!(
            "unknown client '{}'; expected claude-code | claude-desktop | cursor",
            other
        ),
    }
}

// ── Build the server entry ────────────────────────────────────────────────────

fn make_entry() -> serde_json::Value {
    serde_json::json!({
        "command": "claude-usage-tracker",
        "args": ["mcp", "serve", "--transport=stdio"],
        SENTINEL_KEY: SENTINEL_VAL
    })
}

// ── Core operations ───────────────────────────────────────────────────────────

pub fn install(client: &str) -> Result<McpInstallResult> {
    let path = resolve_client_path(client)?;
    install_into(&path)
}

pub fn install_into(path: &std::path::Path) -> Result<McpInstallResult> {
    let mut root = read_or_empty_object(path)?;

    // Ensure mcpServers object exists.
    {
        root.as_object_mut()
            .context("mcp.json root must be an object")?
            .entry("mcpServers")
            .or_insert_with(|| serde_json::json!({}));
    }

    // Check for existing heimdall entry (borrow released at end of block).
    let already_installed = root
        .get("mcpServers")
        .and_then(|s| s.get(SERVER_KEY))
        .map(|e| e.get(SENTINEL_KEY).and_then(|v| v.as_str()) == Some(SENTINEL_VAL))
        .unwrap_or(false);
    let already_present = root
        .get("mcpServers")
        .and_then(|s| s.get(SERVER_KEY))
        .is_some();

    if already_present {
        if already_installed {
            // Our sentinel is present — idempotent install.
            return Ok(McpInstallResult::AlreadyInstalled {
                path: path.to_path_buf(),
            });
        } else {
            // User-customized entry without our sentinel: do NOT overwrite.
            return Err(anyhow::anyhow!(
                "refusing to overwrite user-customized 'heimdall' entry in {} \
                (run `heimdall mcp status --client=<...>` to inspect; \
                remove the entry manually before reinstalling)",
                path.display()
            ));
        }
    }

    write_object_backup_if_present(path, &root)?;

    root.as_object_mut()
        .context("mcp.json root must be an object")?
        .get_mut("mcpServers")
        .and_then(|s| s.as_object_mut())
        .context("mcpServers must be an object")?
        .insert(SERVER_KEY.to_string(), make_entry());

    write_object(path, &root)?;
    Ok(McpInstallResult::Installed {
        path: path.to_path_buf(),
    })
}

pub fn uninstall(client: &str) -> Result<McpInstallResult> {
    let path = resolve_client_path(client)?;
    uninstall_from(&path)
}

pub fn uninstall_from(path: &std::path::Path) -> Result<McpInstallResult> {
    if !path.exists() {
        return Ok(McpInstallResult::NothingToUninstall);
    }

    let mut root = read_or_empty_object(path)?;

    // Check for our sentinel (immutable borrows released before mutation).
    let has_sentinel = root
        .get("mcpServers")
        .and_then(|s| s.get(SERVER_KEY))
        .map(|e| e.get(SENTINEL_KEY).and_then(|v| v.as_str()) == Some(SENTINEL_VAL))
        .unwrap_or(false);
    let has_entry = root
        .get("mcpServers")
        .and_then(|s| s.get(SERVER_KEY))
        .is_some();

    if !has_entry || !has_sentinel {
        return Ok(McpInstallResult::NothingToUninstall);
    }

    write_object_backup_if_present(path, &root)?;

    root.as_object_mut()
        .context("mcp.json root must be an object")?
        .get_mut("mcpServers")
        .and_then(|s| s.as_object_mut())
        .context("mcpServers must be an object")?
        .remove(SERVER_KEY);

    write_object(path, &root)?;
    Ok(McpInstallResult::Uninstalled {
        path: path.to_path_buf(),
    })
}

pub fn status(client: &str) -> Result<McpInstallStatus> {
    let path = resolve_client_path(client)?;
    status_from(&path)
}

pub fn status_from(path: &std::path::Path) -> Result<McpInstallStatus> {
    if !path.exists() {
        return Ok(McpInstallStatus::Absent);
    }
    let root = read_or_empty_object(path)?;
    let servers = match root.get("mcpServers") {
        Some(s) => s,
        None => return Ok(McpInstallStatus::Absent),
    };
    let entry = match servers.get(SERVER_KEY) {
        Some(e) => e,
        None => return Ok(McpInstallStatus::Absent),
    };
    if entry.get(SENTINEL_KEY).and_then(|v| v.as_str()) == Some(SENTINEL_VAL) {
        Ok(McpInstallStatus::Installed {
            path: path.to_path_buf(),
        })
    } else {
        Ok(McpInstallStatus::Customized {
            path: path.to_path_buf(),
        })
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn tmp_path(dir: &TempDir, name: &str) -> PathBuf {
        dir.path().join(name)
    }

    #[test]
    fn install_into_empty_creates_entry() {
        let dir = TempDir::new().unwrap();
        let path = tmp_path(&dir, ".mcp.json");
        let r = install_into(&path).unwrap();
        assert!(matches!(r, McpInstallResult::Installed { .. }));

        let v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(v["mcpServers"]["heimdall"]["_heimdall_mcp_version"], "v1");
        assert_eq!(
            v["mcpServers"]["heimdall"]["command"],
            "claude-usage-tracker"
        );
    }

    #[test]
    fn install_is_idempotent() {
        let dir = TempDir::new().unwrap();
        let path = tmp_path(&dir, ".mcp.json");
        install_into(&path).unwrap();
        let r2 = install_into(&path).unwrap();
        assert!(matches!(r2, McpInstallResult::AlreadyInstalled { .. }));
        let v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(v["mcpServers"].as_object().unwrap().len(), 1);
    }

    #[test]
    fn install_preserves_other_servers() {
        let dir = TempDir::new().unwrap();
        let path = tmp_path(&dir, ".mcp.json");
        let initial = serde_json::json!({ "mcpServers": { "other": { "command": "other-tool" } } });
        std::fs::write(&path, serde_json::to_string_pretty(&initial).unwrap()).unwrap();

        install_into(&path).unwrap();

        let v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert!(v["mcpServers"]["other"].is_object());
        assert!(v["mcpServers"]["heimdall"].is_object());
    }

    #[test]
    fn uninstall_removes_entry() {
        let dir = TempDir::new().unwrap();
        let path = tmp_path(&dir, ".mcp.json");
        install_into(&path).unwrap();
        let r = uninstall_from(&path).unwrap();
        assert!(matches!(r, McpInstallResult::Uninstalled { .. }));

        let v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert!(v["mcpServers"]["heimdall"].is_null());
    }

    #[test]
    fn uninstall_respects_user_customization() {
        let dir = TempDir::new().unwrap();
        let path = tmp_path(&dir, ".mcp.json");
        // Write a heimdall entry WITHOUT the sentinel.
        let custom = serde_json::json!({
            "mcpServers": {
                "heimdall": { "command": "my-custom-binary", "args": [] }
            }
        });
        std::fs::write(&path, serde_json::to_string_pretty(&custom).unwrap()).unwrap();

        let r = uninstall_from(&path).unwrap();
        assert_eq!(r, McpInstallResult::NothingToUninstall);

        // Entry must still be there.
        let v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(v["mcpServers"]["heimdall"]["command"], "my-custom-binary");
    }

    #[test]
    fn status_absent_when_no_file() {
        let dir = TempDir::new().unwrap();
        let path = tmp_path(&dir, ".mcp.json");
        assert_eq!(status_from(&path).unwrap(), McpInstallStatus::Absent);
    }

    #[test]
    fn status_installed_after_install() {
        let dir = TempDir::new().unwrap();
        let path = tmp_path(&dir, ".mcp.json");
        install_into(&path).unwrap();
        assert!(matches!(
            status_from(&path).unwrap(),
            McpInstallStatus::Installed { .. }
        ));
    }

    #[test]
    fn status_customized_without_sentinel() {
        let dir = TempDir::new().unwrap();
        let path = tmp_path(&dir, ".mcp.json");
        let custom = serde_json::json!({ "mcpServers": { "heimdall": {} } });
        std::fs::write(&path, serde_json::to_string_pretty(&custom).unwrap()).unwrap();
        assert!(matches!(
            status_from(&path).unwrap(),
            McpInstallStatus::Customized { .. }
        ));
    }

    #[test]
    fn status_absent_after_uninstall() {
        let dir = TempDir::new().unwrap();
        let path = tmp_path(&dir, ".mcp.json");
        install_into(&path).unwrap();
        uninstall_from(&path).unwrap();
        assert_eq!(status_from(&path).unwrap(), McpInstallStatus::Absent);
    }

    #[test]
    fn backup_written_on_install() {
        let dir = TempDir::new().unwrap();
        let path = tmp_path(&dir, ".mcp.json");
        // Create file so backup is written.
        std::fs::write(&path, "{}").unwrap();
        install_into(&path).unwrap();
        let bak = dir.path().join(".mcp.json.heimdall-bak");
        assert!(bak.exists(), "backup should exist after install");
    }

    #[test]
    fn install_refuses_to_overwrite_customized_entry() {
        let dir = TempDir::new().unwrap();
        let path = tmp_path(&dir, ".mcp.json");
        // Seed a heimdall entry WITHOUT the sentinel — user-customized.
        let custom = serde_json::json!({
            "mcpServers": {
                "heimdall": { "command": "my-custom-binary", "args": [] }
            }
        });
        std::fs::write(&path, serde_json::to_string_pretty(&custom).unwrap()).unwrap();
        let result = install_into(&path);
        assert!(
            result.is_err(),
            "install_into must refuse to overwrite a customized entry"
        );
        // The original entry must be untouched.
        let v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(v["mcpServers"]["heimdall"]["command"], "my-custom-binary");
    }

    #[test]
    fn install_errors_on_malformed_json_target() {
        let dir = TempDir::new().unwrap();
        let path = tmp_path(&dir, ".mcp.json");
        std::fs::write(&path, "{ this is not json").unwrap();
        let result = install_into(&path);
        assert!(
            result.is_err(),
            "install_into must return Err for malformed JSON"
        );
    }
}
