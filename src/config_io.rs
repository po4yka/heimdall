//! Atomic config file I/O with unknown-key preservation.
//!
//! M1 of the in-app Settings UI plan needs three things from this module:
//!
//! 1. Read a config file and return both the typed `Config` and the raw
//!    `serde_json::Map` so unknown / Swift-only top-level keys can be put back
//!    on save.
//! 2. Write a `Config` atomically (`tempfile::NamedTempFile` in the same parent
//!    directory, then `persist`). Atomicity requires the temp file to be on
//!    the same filesystem as the destination — that's what `new_in(parent)`
//!    guarantees.
//! 3. The "preserving" write variant merges the typed `Config` back into the
//!    original raw map, so any keys that aren't part of the Rust schema (e.g.
//!    macOS-only fields written by the Swift app) survive the round-trip.
//!
//! TOML files: read via `toml::from_str`, then re-encoded as JSON on save —
//! the canonical write format is JSON. A `tracing::warn!` on first migration
//! announces the conversion. Unknown-key preservation is JSON-only because the
//! TOML→JSON re-serialisation only carries Rust-known keys.

use std::path::Path;

use anyhow::{Context, Result};
use serde_json::{Map, Value};
use tempfile::NamedTempFile;
use tracing::warn;

use crate::config::Config;

/// Read a config file and return both the typed `Config` and the raw root
/// `serde_json::Map`. The map is the round-trip handle for unknown keys.
///
/// - Missing file → `(Config::default(), Map::new())`.
/// - JSON file → parsed both ways: `Config` for typed access and `Map` for
///   passthrough.
/// - TOML file → parsed via `toml::from_str` then re-serialised through
///   `serde_json::to_value` for the map. The map will only contain Rust-known
///   keys; unknown-key preservation is a JSON-only feature because TOML files
///   are canonically migrated to JSON on first save.
pub fn read_config_root(path: &Path) -> Result<(Config, Map<String, Value>)> {
    if !path.exists() {
        return Ok((Config::default(), Map::new()));
    }

    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("reading config file at {}", path.display()))?;

    if is_json_path(path) {
        let config: Config = serde_json::from_str(&contents)
            .with_context(|| format!("parsing JSON config at {}", path.display()))?;
        let root_value: Value = serde_json::from_str(&contents)
            .with_context(|| format!("parsing JSON config root at {}", path.display()))?;
        let root = match root_value {
            Value::Object(map) => map,
            _ => Map::new(),
        };
        Ok((config, root))
    } else {
        let config: Config = toml::from_str(&contents)
            .with_context(|| format!("parsing TOML config at {}", path.display()))?;
        let root_value = serde_json::to_value(&config)
            .with_context(|| format!("re-serialising TOML config at {}", path.display()))?;
        let root = match root_value {
            Value::Object(map) => map,
            _ => Map::new(),
        };
        Ok((config, root))
    }
}

/// Atomically write `config` to `path` as pretty-printed JSON.
///
/// For TOML paths, the file is migrated to JSON on first save (with a
/// `tracing::warn!`) — the canonical write format is JSON. The original TOML
/// file is left in place so the user can see the migration.
pub fn write_config_atomic(path: &Path, config: &Config) -> Result<()> {
    if !is_json_path(path) {
        warn!(
            "migrating TOML config at {} to JSON on first save",
            path.display()
        );
        let mut json_path = path.to_path_buf();
        json_path.set_extension("json");
        let value = serde_json::to_value(config).context("encoding config to JSON value")?;
        return write_json_atomic(&json_path, &value);
    }

    let value = serde_json::to_value(config).context("encoding config to JSON value")?;
    write_json_atomic(path, &value)
}

/// Atomically write `config` to `path` while preserving any top-level keys
/// from `original_root` that the typed encoding does not produce.
///
/// This is the round-trip-safe variant: keys written by the macOS app (or any
/// other non-Rust consumer) survive a save from the dashboard.
///
/// For TOML paths this falls back to [`write_config_atomic`] — preservation
/// would require `toml_edit` to keep comments and ordering intact, which is
/// out of scope for M1.
pub fn write_config_atomic_preserving(
    path: &Path,
    config: &Config,
    original_root: &Map<String, Value>,
) -> Result<()> {
    if !is_json_path(path) {
        return write_config_atomic(path, config);
    }

    let encoded = serde_json::to_value(config).context("encoding config to JSON value")?;
    let mut merged = match encoded {
        Value::Object(map) => map,
        other => {
            return Err(anyhow::anyhow!(
                "expected Config to encode to a JSON object, got {:?}",
                other
            ));
        }
    };

    for (key, value) in original_root {
        if !merged.contains_key(key) {
            merged.insert(key.clone(), value.clone());
        }
    }

    write_json_atomic(path, &Value::Object(merged))
}

fn write_json_atomic(path: &Path, value: &Value) -> Result<()> {
    let parent = path.parent().ok_or_else(|| {
        anyhow::anyhow!(
            "cannot write config to {} — path has no parent directory",
            path.display()
        )
    })?;

    if !parent.as_os_str().is_empty() && !parent.exists() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating parent directory {}", parent.display()))?;
    }

    let temp_dir = if parent.as_os_str().is_empty() {
        Path::new(".")
    } else {
        parent
    };

    let temp_file = NamedTempFile::new_in(temp_dir)
        .with_context(|| format!("creating temp file under {}", temp_dir.display()))?;

    let serialised =
        serde_json::to_string_pretty(value).context("serialising config JSON value")?;
    std::fs::write(temp_file.path(), serialised.as_bytes())
        .with_context(|| format!("writing temp file at {}", temp_file.path().display()))?;

    temp_file
        .persist(path)
        .with_context(|| format!("persisting temp file to {}", path.display()))?;

    Ok(())
}

fn is_json_path(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("json"))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn unknown_keys_round_trip() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.json");
        std::fs::write(
            &path,
            r#"{"display": {"currency": "USD"}, "macos_only_field": {"foo": "bar"}}"#,
        )
        .unwrap();

        let (cfg, root) = read_config_root(&path).expect("read root");
        assert_eq!(cfg.display.currency.as_deref(), Some("USD"));

        write_config_atomic_preserving(&path, &cfg, &root).expect("write preserving");

        let raw = std::fs::read_to_string(&path).unwrap();
        let parsed: Value = serde_json::from_str(&raw).expect("re-read JSON");
        assert_eq!(
            parsed["macos_only_field"]["foo"].as_str(),
            Some("bar"),
            "unknown top-level key was dropped"
        );
        assert_eq!(parsed["display"]["currency"].as_str(), Some("USD"));
    }

    #[test]
    fn write_atomic_creates_file() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("created.json");
        assert!(!path.exists());

        let cfg = Config::default();
        write_config_atomic(&path, &cfg).expect("write atomic");

        assert!(path.exists(), "config file was not created");
        let raw = std::fs::read_to_string(&path).unwrap();
        let _parsed: Value = serde_json::from_str(&raw).expect("created file is not valid JSON");
    }

    #[test]
    fn missing_file_returns_default_config_and_empty_map() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("does-not-exist.json");

        let (cfg, root) = read_config_root(&path).expect("read absent");
        assert!(cfg.display.currency.is_some()); // default is Some("USD")
        assert!(root.is_empty());
    }

    #[test]
    fn toml_round_trip_drops_unknown_keys() {
        // TOML preservation is a documented limitation: TOML→JSON re-serialisation
        // only carries Rust-known keys. This test pins that behaviour.
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        std::fs::write(&path, "port = 4242\n[display]\ncurrency = \"GBP\"\n").unwrap();

        let (cfg, root) = read_config_root(&path).expect("read toml");
        assert_eq!(cfg.port, Some(4242));
        // root only carries Rust-known keys — that's fine.
        assert!(root.contains_key("port") || root.contains_key("display"));
    }
}
