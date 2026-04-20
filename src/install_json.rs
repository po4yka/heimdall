use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

pub(crate) fn claude_settings_json_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
        .join("settings.json")
}

pub(crate) fn backup_path(path: &Path) -> PathBuf {
    let mut backup = path.to_path_buf();
    let mut name = backup.file_name().unwrap_or_default().to_os_string();
    name.push(".heimdall-bak");
    backup.set_file_name(name);
    backup
}

pub(crate) fn read_or_empty_object(path: &Path) -> Result<serde_json::Value> {
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

pub(crate) fn write_object(path: &Path, value: &serde_json::Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(value)?;
    let mut file =
        std::fs::File::create(path).with_context(|| format!("writing {}", path.display()))?;
    file.write_all(text.as_bytes())?;
    Ok(())
}

pub(crate) fn write_object_backup(path: &Path, value: &serde_json::Value) -> Result<()> {
    write_object(&backup_path(path), value)
}

pub(crate) fn write_object_backup_if_present(path: &Path, value: &serde_json::Value) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    if std::fs::metadata(path)
        .map(|meta| meta.len() == 0)
        .unwrap_or(true)
    {
        return Ok(());
    }
    write_object_backup(path, value)
}
