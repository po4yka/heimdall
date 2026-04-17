//! Sidecar cache for Cursor `state.vscdb` parse results.
//!
//! Stores per-workspace metadata at `~/.cache/heimdall/cursor/<workspace-hash>.json`
//! so that unchanged DBs are not re-parsed on every scan.

use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Serialised metadata written alongside each parsed workspace DB.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CursorCacheEntry {
    /// Source file modification time as Unix seconds (f64 for sub-second precision).
    pub source_mtime: f64,
    /// Source file size in bytes.
    pub source_size: u64,
    /// Number of `Turn`s produced by the last successful parse.
    pub parsed_turns_count: usize,
    /// RFC-3339 timestamp of when this cache entry was written.
    pub parsed_at: String,
}

/// Load a `CursorCacheEntry` from `cache_path`.
///
/// Returns `None` (rather than an error) when the file is absent or
/// unparseable — both are treated as "no valid cache".
pub fn load_cache(cache_path: &Path) -> Option<CursorCacheEntry> {
    let bytes = std::fs::read(cache_path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

/// Persist a `CursorCacheEntry` to `cache_path`.
///
/// Creates parent directories if they are absent.
pub fn save_cache(cache_path: &Path, entry: &CursorCacheEntry) -> Result<()> {
    if let Some(parent) = cache_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_vec_pretty(entry)?;
    std::fs::write(cache_path, json)?;
    Ok(())
}

/// Returns `true` when the source file's mtime and size match the cached
/// entry exactly, indicating that the DB has not changed since last parse.
pub fn is_cache_fresh(entry: &CursorCacheEntry, current_mtime: f64, current_size: u64) -> bool {
    (entry.source_mtime - current_mtime).abs() < f64::EPSILON && entry.source_size == current_size
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_entry() -> CursorCacheEntry {
        CursorCacheEntry {
            source_mtime: 1_700_000_000.5,
            source_size: 4096,
            parsed_turns_count: 3,
            parsed_at: "2026-04-17T10:00:00Z".to_string(),
        }
    }

    #[test]
    fn cache_round_trip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("abc123.json");
        let entry = make_entry();

        save_cache(&path, &entry).unwrap();
        let loaded = load_cache(&path).expect("should load saved entry");
        assert_eq!(loaded, entry);
    }

    #[test]
    fn cache_creates_parent_dirs() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nested").join("deep").join("entry.json");
        let entry = make_entry();
        save_cache(&path, &entry).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn load_cache_returns_none_for_missing_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nonexistent.json");
        assert!(load_cache(&path).is_none());
    }

    #[test]
    fn load_cache_returns_none_for_corrupt_json() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("corrupt.json");
        std::fs::write(&path, b"not valid json{{{").unwrap();
        assert!(load_cache(&path).is_none());
    }

    #[test]
    fn freshness_same_mtime_and_size() {
        let entry = make_entry();
        assert!(is_cache_fresh(
            &entry,
            entry.source_mtime,
            entry.source_size
        ));
    }

    #[test]
    fn freshness_different_mtime() {
        let entry = make_entry();
        assert!(!is_cache_fresh(
            &entry,
            entry.source_mtime + 1.0,
            entry.source_size
        ));
    }

    #[test]
    fn freshness_different_size() {
        let entry = make_entry();
        assert!(!is_cache_fresh(
            &entry,
            entry.source_mtime,
            entry.source_size + 1
        ));
    }
}
