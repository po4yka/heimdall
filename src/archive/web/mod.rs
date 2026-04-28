//! Web-conversation storage primitive (Phase 3a, Task 1).
//!
//! Stores captured web conversations under `<archive_root>/web/<vendor>/<conv_id>.json`.
//! On payload change the previous file is rotated into `<conv_id>.history/<prior_captured_at>.json`.
//! Writes are atomic (`.tmp-<id>.json` → rename).

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A single captured web conversation payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebConversation {
    pub vendor: String,
    pub conversation_id: String,
    pub captured_at: String,
    pub schema_fingerprint: String,
    pub payload: serde_json::Value,
}

/// Result of a [`write_web_conversation`] call.
#[derive(Debug)]
pub enum WriteOutcome {
    /// The file was written (new or updated). `path` is the current `.json` path.
    Saved { path: PathBuf },
    /// The existing file is byte-identical to the new payload; nothing was touched.
    Unchanged,
}

/// Lightweight summary returned by [`list_web_conversations`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebConversationSummary {
    pub vendor: String,
    pub conversation_id: String,
    pub captured_at: String,
    /// Number of rotated history entries found in `<conv_id>.history/`.
    pub history_count: usize,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Sanitize a path component: keep only alphanumeric, `.`, `-`, `_`.
fn sanitize(raw: &str) -> String {
    raw.chars()
        .filter(|c| c.is_alphanumeric() || matches!(c, '.' | '-' | '_'))
        .collect()
}

/// Return `<archive_root>/web/<sanitized vendor>`.
pub fn vendor_dir(archive_root: &Path, vendor: &str) -> PathBuf {
    archive_root.join("web").join(sanitize(vendor))
}

// ---------------------------------------------------------------------------
// Core write
// ---------------------------------------------------------------------------

/// Atomically write a [`WebConversation`] under the archive root.
///
/// - Sanitizes `vendor` and `conversation_id` (alphanumeric + `.` + `-` + `_`; conv_id capped at 120 chars).
/// - Returns [`WriteOutcome::Unchanged`] when the existing file is byte-identical.
/// - When the payload differs, renames the old file into `<conv_id>.history/<prior_captured_at>.json`
///   before writing the new one.
/// - The write itself is atomic: `.tmp-<conv_id>.json` → rename.
pub fn write_web_conversation(archive_root: &Path, conv: &WebConversation) -> Result<WriteOutcome> {
    let vendor_s = sanitize(&conv.vendor);
    let raw_id = sanitize(&conv.conversation_id);
    // Cap conversation_id at 120 chars after sanitization.
    let conv_id: String = raw_id.chars().take(120).collect();

    let dir = archive_root.join("web").join(&vendor_s);
    fs::create_dir_all(&dir).with_context(|| format!("creating vendor dir {}", dir.display()))?;

    let current_path = dir.join(format!("{conv_id}.json"));
    let new_bytes = serde_json::to_vec_pretty(conv).context("serializing WebConversation")?;

    // Check for byte-identical existing file.
    if current_path.is_file() {
        let existing = fs::read(&current_path)
            .with_context(|| format!("reading {}", current_path.display()))?;
        if existing == new_bytes {
            return Ok(WriteOutcome::Unchanged);
        }

        // Rotate the old file into history.
        let prior_captured_at = read_captured_at(&current_path, &conv.captured_at);
        let history_dir = dir.join(format!("{conv_id}.history"));
        fs::create_dir_all(&history_dir)
            .with_context(|| format!("creating history dir {}", history_dir.display()))?;
        let hist_file = history_dir.join(format!("{prior_captured_at}.json"));
        fs::rename(&current_path, &hist_file)
            .with_context(|| format!("rotating {} into history", current_path.display()))?;
    }

    // Atomic write: tmp → rename.
    let tmp_path = dir.join(format!(".tmp-{conv_id}.json"));
    fs::write(&tmp_path, &new_bytes)
        .with_context(|| format!("writing tmp file {}", tmp_path.display()))?;
    fs::rename(&tmp_path, &current_path)
        .with_context(|| format!("renaming tmp to {}", current_path.display()))?;

    Ok(WriteOutcome::Saved { path: current_path })
}

/// Try to read `captured_at` from the existing file; fall back to the provided `fallback`.
fn read_captured_at(path: &Path, fallback: &str) -> String {
    (|| -> Option<String> {
        let bytes = fs::read(path).ok()?;
        let v: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        v.get("captured_at")?.as_str().map(|s| s.to_owned())
    })()
    .unwrap_or_else(|| {
        // Synthetic stub: use a derived timestamp so the history slot is unique.
        Utc::now().format("%Y-%m-%dT%H%M%S%.6fZ").to_string() + "__" + fallback
    })
}

// ---------------------------------------------------------------------------
// List
// ---------------------------------------------------------------------------

/// Walk `<archive_root>/web/<vendor>/*.json`, skipping `.tmp-*` and malformed files.
///
/// Returns one [`WebConversationSummary`] per conversation, sorted newest-first by `captured_at`.
pub fn list_web_conversations(archive_root: &Path) -> Result<Vec<WebConversationSummary>> {
    let web_root = archive_root.join("web");
    if !web_root.is_dir() {
        return Ok(Vec::new());
    }

    let mut summaries = Vec::new();

    for vendor_entry in fs::read_dir(&web_root)
        .with_context(|| format!("reading web dir {}", web_root.display()))?
    {
        let vendor_entry = vendor_entry?;
        if !vendor_entry.file_type()?.is_dir() {
            continue;
        }
        let vendor_name = vendor_entry.file_name().to_string_lossy().into_owned();
        let vendor_path = vendor_entry.path();

        for file_entry in fs::read_dir(&vendor_path)
            .with_context(|| format!("reading vendor dir {}", vendor_path.display()))?
        {
            let file_entry = file_entry?;
            let file_type = file_entry.file_type()?;
            if !file_type.is_file() {
                continue;
            }
            let fname = file_entry.file_name();
            let fname_str = fname.to_string_lossy();

            // Skip temp files and non-.json files.
            if fname_str.starts_with(".tmp-") || !fname_str.ends_with(".json") {
                continue;
            }

            let file_path = file_entry.path();
            let bytes = match fs::read(&file_path) {
                Ok(b) => b,
                Err(_) => continue,
            };
            let conv: WebConversation = match serde_json::from_slice(&bytes) {
                Ok(c) => c,
                Err(_) => continue,
            };

            // Derive the conversation_id from the filename (stem without `.json`).
            let conv_id = fname_str
                .strip_suffix(".json")
                .unwrap_or(&fname_str)
                .to_owned();

            // Count history entries.
            let history_dir = vendor_path.join(format!("{conv_id}.history"));
            let history_count = if history_dir.is_dir() {
                fs::read_dir(&history_dir)
                    .map(|rd| {
                        rd.filter_map(|e| e.ok())
                            .filter(|e| {
                                e.file_type().map(|ft| ft.is_file()).unwrap_or(false)
                                    && e.file_name().to_string_lossy().ends_with(".json")
                            })
                            .count()
                    })
                    .unwrap_or(0)
            } else {
                0
            };

            summaries.push(WebConversationSummary {
                vendor: vendor_name.clone(),
                conversation_id: conv.conversation_id,
                captured_at: conv.captured_at,
                history_count,
            });
        }
    }

    // Newest-first by captured_at (lexicographic works for ISO 8601).
    summaries.sort_by(|a, b| b.captured_at.cmp(&a.captured_at));
    Ok(summaries)
}

// ---------------------------------------------------------------------------
// Heartbeat
// ---------------------------------------------------------------------------

/// Last-seen record written by the browser companion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanionHeartbeat {
    pub last_seen_at: String,
    pub extension_version: Option<String>,
    pub user_agent: Option<String>,
    pub vendors_seen: Vec<String>,
}

/// Path of the heartbeat file: `<archive_root>/web/companion-heartbeat.json`.
pub fn heartbeat_path(archive_root: &Path) -> PathBuf {
    archive_root.join("web").join("companion-heartbeat.json")
}

/// Update (or create) the heartbeat file.
///
/// - `last_seen_at` is always refreshed to the current UTC instant.
/// - `extension_version` / `user_agent` are replaced only when `Some`.
/// - `vendor` is appended to `vendors_seen` only when non-empty and not already present.
pub fn record_heartbeat(
    archive_root: &Path,
    extension_version: Option<String>,
    user_agent: Option<String>,
    vendor: &str,
) -> Result<()> {
    let path = heartbeat_path(archive_root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut h = read_heartbeat(archive_root)?.unwrap_or_else(|| CompanionHeartbeat {
        last_seen_at: String::new(),
        extension_version: None,
        user_agent: None,
        vendors_seen: Vec::new(),
    });
    h.last_seen_at = Utc::now().format("%Y-%m-%dT%H%M%S%.6fZ").to_string();
    if extension_version.is_some() {
        h.extension_version = extension_version;
    }
    if user_agent.is_some() {
        h.user_agent = user_agent;
    }
    if !vendor.is_empty() && !h.vendors_seen.iter().any(|v| v == vendor) {
        h.vendors_seen.push(vendor.to_string());
    }
    let bytes = serde_json::to_vec_pretty(&h)?;
    fs::write(&path, bytes)?;
    Ok(())
}

/// Read the heartbeat file, returning `None` when it does not exist.
pub fn read_heartbeat(archive_root: &Path) -> Result<Option<CompanionHeartbeat>> {
    let path = heartbeat_path(archive_root);
    if !path.is_file() {
        return Ok(None);
    }
    let bytes = fs::read(&path)?;
    Ok(serde_json::from_slice(&bytes).ok())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    fn make_conv(vendor: &str, id: &str, payload: serde_json::Value) -> WebConversation {
        WebConversation {
            vendor: vendor.to_owned(),
            conversation_id: id.to_owned(),
            captured_at: Utc::now().format("%Y-%m-%dT%H%M%S%.6fZ").to_string(),
            schema_fingerprint: "fp1".to_owned(),
            payload,
        }
    }

    #[test]
    fn first_write_creates_current_file() {
        let dir = TempDir::new().unwrap();
        let conv = make_conv("claude.ai", "conv-001", json!({"msg": "hello"}));
        let outcome = write_web_conversation(dir.path(), &conv).unwrap();
        match outcome {
            WriteOutcome::Saved { path } => {
                assert!(path.exists(), "current file should exist");
            }
            WriteOutcome::Unchanged => panic!("expected Saved, got Unchanged"),
        }
        // Verify path is inside <root>/web/<vendor>/
        let expected_dir = dir.path().join("web").join("claude.ai");
        assert!(expected_dir.is_dir());
        assert!(expected_dir.join("conv-001.json").is_file());
    }

    #[test]
    fn second_identical_write_is_unchanged() {
        let dir = TempDir::new().unwrap();
        let conv = make_conv("claude.ai", "conv-002", json!({"msg": "same"}));
        write_web_conversation(dir.path(), &conv).unwrap();
        let outcome = write_web_conversation(dir.path(), &conv).unwrap();
        assert!(
            matches!(outcome, WriteOutcome::Unchanged),
            "identical payload should be Unchanged"
        );
    }

    #[test]
    fn changed_payload_rotates_previous_into_history() {
        let dir = TempDir::new().unwrap();
        let conv1 = make_conv("claude.ai", "conv-003", json!({"v": 1}));
        write_web_conversation(dir.path(), &conv1).unwrap();

        // Different payload.
        let conv2 = WebConversation {
            payload: json!({"v": 2}),
            // Advance captured_at slightly for a unique history slot.
            captured_at: Utc::now().format("%Y-%m-%dT%H%M%S%.6fZ").to_string(),
            ..conv1.clone()
        };
        write_web_conversation(dir.path(), &conv2).unwrap();

        let vendor_dir = dir.path().join("web").join("claude.ai");
        let history_dir = vendor_dir.join("conv-003.history");
        assert!(history_dir.is_dir(), "history dir should exist");

        let entries: Vec<_> = fs::read_dir(&history_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(entries.len(), 1, "exactly one history entry");

        // Current file should contain v=2.
        let current: serde_json::Value =
            serde_json::from_slice(&fs::read(vendor_dir.join("conv-003.json")).unwrap()).unwrap();
        assert_eq!(current["payload"]["v"], 2);
    }

    #[test]
    fn vendor_and_id_sanitization() {
        let dir = TempDir::new().unwrap();
        // Malicious vendor and id with path-traversal characters.
        let conv = make_conv("claude.ai/../etc", "../../escape", json!({"safe": true}));
        write_web_conversation(dir.path(), &conv).unwrap();

        // "claude.ai/../etc"  → slashes stripped → "claude.ai..etc"
        // "../../escape"      → slashes stripped → "....escape"
        let sanitized_vendor = "claude.ai..etc";
        let sanitized_id = "....escape";
        let expected = dir
            .path()
            .join("web")
            .join(sanitized_vendor)
            .join(format!("{sanitized_id}.json"));
        assert!(expected.is_file(), "file should exist at sanitized path");

        // Nothing should have escaped to the parent of <root>.
        let escaped = dir.path().parent().unwrap().join("etc");
        assert!(!escaped.exists(), "path traversal must not escape root");
    }

    #[test]
    fn record_heartbeat_creates_file() {
        let tmp = TempDir::new().unwrap();
        record_heartbeat(
            tmp.path(),
            Some("0.1.0".into()),
            Some("UA".into()),
            "claude.ai",
        )
        .unwrap();
        let h = read_heartbeat(tmp.path()).unwrap().expect("present");
        assert!(!h.last_seen_at.is_empty());
        assert_eq!(h.extension_version.as_deref(), Some("0.1.0"));
        assert_eq!(h.user_agent.as_deref(), Some("UA"));
        assert_eq!(h.vendors_seen, vec!["claude.ai"]);
    }

    #[test]
    fn record_heartbeat_appends_unique_vendors() {
        let tmp = TempDir::new().unwrap();
        record_heartbeat(tmp.path(), None, None, "claude.ai").unwrap();
        record_heartbeat(tmp.path(), None, None, "claude.ai").unwrap();
        record_heartbeat(tmp.path(), None, None, "chatgpt.com").unwrap();
        let h = read_heartbeat(tmp.path()).unwrap().expect("present");
        assert_eq!(h.vendors_seen, vec!["claude.ai", "chatgpt.com"]);
    }

    #[test]
    fn list_returns_one_summary_per_conversation() {
        let dir = TempDir::new().unwrap();

        // 2 conversations under vendor-a, 1 under vendor-b.
        let convs = [
            make_conv("vendor-a", "conv-x", json!(1)),
            make_conv("vendor-a", "conv-y", json!(2)),
            make_conv("vendor-b", "conv-z", json!(3)),
        ];
        for c in &convs {
            write_web_conversation(dir.path(), c).unwrap();
        }

        let list = list_web_conversations(dir.path()).unwrap();
        assert_eq!(list.len(), 3, "should return 3 summaries");

        // All history counts should be 0 (no rotations performed).
        for s in &list {
            assert_eq!(s.history_count, 0);
        }

        // Verify vendors present.
        let vendors: std::collections::HashSet<_> =
            list.iter().map(|s| s.vendor.as_str()).collect();
        assert!(vendors.contains("vendor-a"));
        assert!(vendors.contains("vendor-b"));
    }
}
