//! Read the ChatGPT for macOS app's local conversation cache.
//!
//! ChatGPT.app (Electron) stores its conversation cache under
//! `~/Library/Application Support/com.openai.chat/`. Two layouts have
//! existed in the wild:
//!
//! 1. **Pre-July 2024 (`conversations-{uuid}/`)** — plaintext per-file JSON
//!    in OpenAI's mapping/messages tree shape, world-readable. Newer app
//!    versions delete these on upgrade, but they may still exist on
//!    machines that never upgraded or where the upgrade was rolled back.
//!
//! 2. **Post-July 2024 (`conversations-v2-{uuid}/`)** — encrypted `.data`
//!    files (per-file random nonce in the first 16 bytes; the body looks
//!    like an AES-GCM ciphertext). The wrapping key lives in macOS
//!    Keychain at service `com.openai.chat.conversations_v2_cache` with
//!    an ACL bound to ChatGPT.app's code signature, requiring an
//!    interactive grant prompt that ChatGPT itself drives.
//!
//! ## What this module does
//!
//! - **Detection** (`scan_caches`): walks the support directory and reports
//!   every cache it finds, classifying each as `Plaintext` (v1) or
//!   `Encrypted` (v2). Always safe to call; never opens the encrypted
//!   blobs.
//!
//! - **Plaintext ingest** (`ingest_plaintext_into_archive`): walks any v1
//!   plaintext directories and writes each conversation through the
//!   existing `archive::web::write_web_conversation` storage primitive,
//!   under vendor `chatgpt.com`. Idempotent + history-rotated for free.
//!
//! - **Encrypted reporting**: counts files / bytes for v2 caches and
//!   surfaces them as `unreadable_reason: "v2 encrypted; format
//!   reverse-engineering required, not implemented"` so the user knows
//!   the data is on disk but not yet importable.
//!
//! ## What this module does NOT do
//!
//! - Decrypt v2 `.data` files. The cipher, key derivation, and per-file
//!   layout are not publicly documented as of 2026-04. Implementing this
//!   would require reverse-engineering the binary format from the
//!   ChatGPT app and is out of scope; the browser-extension companion
//!   (Phase 3b) and the ZIP-import path (Phase 2) cover ChatGPT cleanly
//!   without it.
//!
//! - Bypass the macOS Keychain ACL. The wrapping key is held under an
//!   access-controlled item that requires the user (or ChatGPT.app) to
//!   approve each retrieval. Heimdall does not attempt to silently
//!   coerce that access.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::archive::imports::openai;
use crate::archive::web::{WebConversation, WriteOutcome, write_web_conversation};

/// macOS bundle identifier of the ChatGPT desktop app.
pub const CHATGPT_BUNDLE_DIR: &str = "com.openai.chat";

/// Default cache root: `~/Library/Application Support/com.openai.chat/`.
pub fn default_cache_root() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    Some(
        home.join("Library")
            .join("Application Support")
            .join(CHATGPT_BUNDLE_DIR),
    )
}

/// Layout family for a discovered cache directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheKind {
    /// Pre-July-2024 plaintext layout (`conversations-{uuid}/`).
    Plaintext,
    /// Post-July-2024 encrypted layout (`conversations-v2-{uuid}/`).
    Encrypted,
}

/// One ChatGPT-cache directory found under the bundle dir.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheReport {
    pub kind: CacheKind,
    pub path: PathBuf,
    pub file_count: usize,
    pub byte_count: u64,
    /// Always populated for `Encrypted`; absent for `Plaintext`.
    pub unreadable_reason: Option<String>,
}

/// Walk the cache root and yield one `CacheReport` per discovered
/// `conversations*` directory. Returns `Ok(vec![])` if the root does not
/// exist (ChatGPT.app never installed).
pub fn scan_caches(cache_root: &Path) -> Result<Vec<CacheReport>> {
    if !cache_root.is_dir() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in
        fs::read_dir(cache_root).with_context(|| format!("reading {}", cache_root.display()))?
    {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let kind = classify_dir_name(&name);
        let Some(kind) = kind else { continue };
        let (file_count, byte_count) = count_dir(&entry.path())?;
        let unreadable_reason = match kind {
            CacheKind::Plaintext => None,
            CacheKind::Encrypted => Some(
                "v2 encrypted; format reverse-engineering required, not implemented".to_string(),
            ),
        };
        out.push(CacheReport {
            kind,
            path: entry.path(),
            file_count,
            byte_count,
            unreadable_reason,
        });
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

fn classify_dir_name(name: &str) -> Option<CacheKind> {
    if name.starts_with("conversations-v2-") {
        return Some(CacheKind::Encrypted);
    }
    if name.starts_with("conversations-") && !name.starts_with("conversations-v") {
        return Some(CacheKind::Plaintext);
    }
    None
}

fn count_dir(dir: &Path) -> Result<(usize, u64)> {
    let mut files = 0usize;
    let mut bytes = 0u64;
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let meta = entry.metadata()?;
        if meta.is_file() {
            files += 1;
            bytes += meta.len();
        }
    }
    Ok((files, bytes))
}

/// Outcome of an `ingest_plaintext_into_archive` run.
#[derive(Debug, Clone, Default, Serialize)]
pub struct IngestReport {
    /// Plaintext directories scanned (regardless of whether they yielded conversations).
    pub plaintext_dirs: usize,
    /// Total files inside those plaintext directories.
    pub plaintext_files: usize,
    /// Conversations parsed.
    pub parsed: usize,
    /// Conversations newly written or rotated to history.
    pub written: usize,
    /// Conversations that already matched the on-disk current copy.
    pub unchanged: usize,
    /// Per-conversation parse / write errors.
    pub errors: Vec<String>,
    /// Encrypted caches detected but not ingested (reported only).
    pub encrypted_dirs: usize,
    /// File count across encrypted caches.
    pub encrypted_files: usize,
}

/// Walk every plaintext cache under `cache_root` and ingest each
/// conversation into `<archive_root>/web/chatgpt.com/...` via the
/// existing storage primitive. Encrypted caches are counted but not
/// touched.
pub fn ingest_plaintext_into_archive(
    cache_root: &Path,
    archive_root: &Path,
) -> Result<IngestReport> {
    let mut report = IngestReport::default();
    let caches = scan_caches(cache_root)?;
    for cache in caches {
        match cache.kind {
            CacheKind::Encrypted => {
                report.encrypted_dirs += 1;
                report.encrypted_files += cache.file_count;
                debug!(
                    target: "archive::macos_cache",
                    "skipping encrypted cache {} ({} files)",
                    cache.path.display(),
                    cache.file_count
                );
            }
            CacheKind::Plaintext => {
                report.plaintext_dirs += 1;
                report.plaintext_files += cache.file_count;
                ingest_one_plaintext_dir(&cache.path, archive_root, &mut report)?;
            }
        }
    }
    info!(
        target: "archive::macos_cache",
        "macos cache ingest: {} parsed, {} written, {} unchanged, {} errors, {} encrypted dirs skipped",
        report.parsed, report.written, report.unchanged, report.errors.len(), report.encrypted_dirs,
    );
    Ok(report)
}

fn ingest_one_plaintext_dir(
    dir: &Path,
    archive_root: &Path,
    report: &mut IngestReport,
) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let path = entry.path();
        let bytes = match fs::read(&path) {
            Ok(b) => b,
            Err(e) => {
                report.errors.push(format!("{}: read: {e}", path.display()));
                continue;
            }
        };
        // Plaintext files contain either a single conversation object or
        // an array of them; reuse the import parser by trying both shapes.
        let parsed: Vec<openai::Conversation> = match openai::parse_conversations(&bytes) {
            Ok(arr) => arr,
            Err(_) => match serde_json::from_slice::<openai::Conversation>(&bytes) {
                Ok(single) => vec![single],
                Err(e) => {
                    warn!(
                        target: "archive::macos_cache",
                        "skipping unparseable {}: {e}", path.display()
                    );
                    report
                        .errors
                        .push(format!("{}: parse: {e}", path.display()));
                    continue;
                }
            },
        };
        for conv in parsed {
            report.parsed += 1;
            let key = openai::conversation_key(&conv).unwrap_or_else(|| {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string()
            });
            let payload = match serde_json::to_value(&conv) {
                Ok(v) => v,
                Err(e) => {
                    report.errors.push(format!("{key}: serialize: {e}"));
                    continue;
                }
            };
            let web = WebConversation {
                vendor: "chatgpt.com".into(),
                conversation_id: key.clone(),
                captured_at: Utc::now().format("%Y-%m-%dT%H%M%S%.6fZ").to_string(),
                schema_fingerprint: "chatgpt.com/macos-plaintext-v1".into(),
                payload,
            };
            match write_web_conversation(archive_root, &web) {
                Ok(WriteOutcome::Saved { .. }) => report.written += 1,
                Ok(WriteOutcome::Unchanged) => report.unchanged += 1,
                Err(e) => report.errors.push(format!("{key}: write: {e}")),
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_cache_dir(root: &Path, name: &str) -> PathBuf {
        let dir = root.join(name);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn classify_recognises_plaintext_and_encrypted_names() {
        assert_eq!(
            classify_dir_name("conversations-abc-123"),
            Some(CacheKind::Plaintext)
        );
        assert_eq!(
            classify_dir_name("conversations-v2-xyz-456"),
            Some(CacheKind::Encrypted)
        );
        assert_eq!(classify_dir_name("conversations"), None);
        assert_eq!(classify_dir_name("Cache"), None);
        // Future-proofing: a hypothetical v3 should NOT be misclassified
        // as plaintext.
        assert_eq!(classify_dir_name("conversations-v3-abc"), None);
    }

    #[test]
    fn scan_caches_returns_empty_when_root_missing() {
        let tmp = TempDir::new().unwrap();
        let absent = tmp.path().join("nonexistent");
        assert!(scan_caches(&absent).unwrap().is_empty());
    }

    #[test]
    fn scan_caches_classifies_both_layouts() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let pt = make_cache_dir(root, "conversations-aaa");
        fs::write(pt.join("file1.json"), b"{}").unwrap();
        fs::write(pt.join("file2.json"), b"{}").unwrap();
        let enc = make_cache_dir(root, "conversations-v2-bbb");
        fs::write(enc.join("blob.data"), b"\x00\x01\x02\x03").unwrap();
        // Decoy: a sibling dir that should be ignored.
        make_cache_dir(root, "Cache");

        let reports = scan_caches(root).unwrap();
        assert_eq!(reports.len(), 2);
        let plain = reports
            .iter()
            .find(|r| r.kind == CacheKind::Plaintext)
            .unwrap();
        assert_eq!(plain.file_count, 2);
        assert!(plain.unreadable_reason.is_none());
        let enc = reports
            .iter()
            .find(|r| r.kind == CacheKind::Encrypted)
            .unwrap();
        assert_eq!(enc.file_count, 1);
        assert_eq!(enc.byte_count, 4);
        assert!(
            enc.unreadable_reason
                .as_deref()
                .unwrap_or("")
                .contains("v2 encrypted"),
            "expected v2 encrypted reason, got {:?}",
            enc.unreadable_reason
        );
    }

    #[test]
    fn ingest_skips_encrypted_dirs() {
        let tmp = TempDir::new().unwrap();
        let cache = tmp.path().join("cache");
        let archive = tmp.path().join("archive");
        let enc = make_cache_dir(&cache, "conversations-v2-only");
        fs::write(enc.join("a.data"), b"\x00\x00\x00\x00").unwrap();
        fs::write(enc.join("b.data"), b"\x00\x00").unwrap();

        let report = ingest_plaintext_into_archive(&cache, &archive).unwrap();
        assert_eq!(report.parsed, 0);
        assert_eq!(report.written, 0);
        assert_eq!(report.encrypted_dirs, 1);
        assert_eq!(report.encrypted_files, 2);
        // No web tree should have been created — encrypted skip is total.
        assert!(!archive.join("web").exists());
    }

    #[test]
    fn ingest_writes_plaintext_conversation_into_web_tree() {
        let tmp = TempDir::new().unwrap();
        let cache = tmp.path().join("cache");
        let archive = tmp.path().join("archive");
        let dir = make_cache_dir(&cache, "conversations-abc");
        let conv = serde_json::json!({
            "id": "conv-from-cache",
            "title": "hello",
            "create_time": 1.0,
            "mapping": {}
        });
        fs::write(dir.join("conv-from-cache.json"), conv.to_string()).unwrap();

        let report = ingest_plaintext_into_archive(&cache, &archive).unwrap();
        assert_eq!(report.parsed, 1);
        assert_eq!(report.written, 1);
        assert_eq!(report.unchanged, 0);
        let on_disk = archive
            .join("web")
            .join("chatgpt.com")
            .join("conv-from-cache.json");
        assert!(on_disk.is_file(), "expected {}", on_disk.display());
    }

    #[test]
    fn ingest_handles_array_shaped_plaintext_files() {
        let tmp = TempDir::new().unwrap();
        let cache = tmp.path().join("cache");
        let archive = tmp.path().join("archive");
        let dir = make_cache_dir(&cache, "conversations-zzz");
        let arr = serde_json::json!([
            { "id": "a", "title": "t1", "mapping": {} },
            { "id": "b", "title": "t2", "mapping": {} },
        ]);
        fs::write(dir.join("bundle.json"), arr.to_string()).unwrap();

        let report = ingest_plaintext_into_archive(&cache, &archive).unwrap();
        assert_eq!(report.parsed, 2);
        assert_eq!(report.written, 2);
    }

    #[test]
    fn ingest_records_parse_errors_and_continues() {
        let tmp = TempDir::new().unwrap();
        let cache = tmp.path().join("cache");
        let archive = tmp.path().join("archive");
        let dir = make_cache_dir(&cache, "conversations-mixed");
        fs::write(dir.join("good.json"), r#"{"id":"g","mapping":{}}"#).unwrap();
        fs::write(dir.join("bad.json"), b"{not json").unwrap();

        let report = ingest_plaintext_into_archive(&cache, &archive).unwrap();
        assert_eq!(report.parsed, 1);
        assert_eq!(report.written, 1);
        assert_eq!(report.errors.len(), 1);
        assert!(
            report.errors[0].contains("bad.json"),
            "got: {:?}",
            report.errors
        );
    }
}
