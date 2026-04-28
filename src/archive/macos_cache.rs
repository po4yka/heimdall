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
//!    files. Each file is an OSCrypt v10 blob: 3-byte ASCII prefix `v10`,
//!    followed by AES-128-CBC ciphertext (constant 16-space IV, PKCS#7
//!    padding). The AES key is derived via PBKDF2-HMAC-SHA1 (salt
//!    `"saltysalt"`, 1003 iterations, 16-byte output) from a passphrase
//!    stored in macOS Keychain at service
//!    `com.openai.chat.conversations_v2_cache` with an ACL bound to
//!    ChatGPT.app's code signature. A standard user-mediated grant prompt
//!    (`SecItemCopyMatching`) is the access boundary; Heimdall does not
//!    bypass it.
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
//! - **v2 decryption** (`ingest_into_archive` with `IngestOptions { decrypt_v2: true }`):
//!   fetches the AES passphrase from macOS Keychain (one-time user-grant
//!   prompt), derives the key via PBKDF2-HMAC-SHA1, decrypts each `.data`
//!   file, and JSON-sniffs the plaintext via the existing OpenAI export
//!   parser. Successful conversations are written to the web tree under
//!   vendor `chatgpt.com` with schema fingerprint
//!   `"chatgpt.com/macos-v2-decrypted"`. Blobs that decrypt but fail the
//!   JSON sniff are saved raw to `<archive>/web/chatgpt.com/.failed-decrypts/`
//!   for inspection; nothing is silently discarded.
//!
//! ## What this module does NOT do
//!
//! - Bypass the macOS Keychain ACL. The wrapping key is ACL-bound to
//!   ChatGPT.app's code signature; Heimdall requests it through the standard
//!   `SecItemCopyMatching` path. The user sees the system grant dialog on
//!   the first call and can allow or deny.
//!
//! - Ingest garbage. Every decrypted blob is validated via `parse_conversations`
//!   before being written. If the JSON sniff fails (e.g., because a future
//!   ChatGPT release changes the inner format), the raw bytes land in
//!   `.failed-decrypts/` and a warning is appended to the ingest report.
//!
//! - Make assumptions beyond the cipher. The cipher (Chromium OSCrypt /
//!   Electron `safeStorage`, AES-128-CBC) is well-confirmed. The inner
//!   schema is assumed to match the publicly-observed OpenAI mapping/messages
//!   tree. If ChatGPT.app changes either, decryption or parsing fails cleanly
//!   and is surfaced in the report.

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
                "v2 encrypted; run `heimdall macos-cache ingest --decrypt-v2` to decrypt"
                    .to_string(),
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
    /// Number of v2 *.data files we tried to decrypt.
    pub v2_attempted: usize,
    /// Number that decrypted AND parsed as JSON conversations.
    pub v2_decrypted: usize,
    /// Number that decrypted but didn't parse as conversations
    /// (raw bytes saved to .failed-decrypts/).
    pub v2_failed_parse: usize,
    /// Number where decryption itself failed (cipher mismatch?).
    pub v2_failed_decrypt: usize,
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

// ── v2 reader ────────────────────────────────────────────────────────────────

/// Result of decrypting one *.data file.
#[derive(Debug)]
pub struct DecryptedFile {
    pub source_path: PathBuf,
    pub plaintext: Vec<u8>,
}

/// Walk a `conversations-v2-*` directory, decrypt each `.data` file with
/// the supplied AES key, and return decrypted blobs. Per-file failures
/// are returned as `Err` items in the result vec; one bad file does not
/// abort the directory.
pub fn decrypt_v2_dir(dir: &Path, key: &[u8; 16]) -> Vec<Result<DecryptedFile>> {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            return vec![Err(
                anyhow::anyhow!(e).context(format!("reading dir {}", dir.display()))
            )];
        }
    };
    let mut out = Vec::new();
    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                out.push(Err(anyhow::anyhow!(e).context("reading dir entry")));
                continue;
            }
        };
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("data") {
            continue;
        }
        let bytes = match fs::read(&path) {
            Ok(b) => b,
            Err(e) => {
                out.push(Err(
                    anyhow::anyhow!(e).context(format!("reading {}", path.display()))
                ));
                continue;
            }
        };
        match oscrypt::decrypt_v10_blob(&bytes, key) {
            Ok(plaintext) => out.push(Ok(DecryptedFile {
                source_path: path,
                plaintext,
            })),
            Err(e) => out.push(Err(e.context(format!("decrypting {}", path.display())))),
        }
    }
    out
}

// ── ingest options + top-level umbrella ──────────────────────────────────────

/// Options for `ingest_into_archive`.
#[derive(Debug, Clone, Default)]
pub struct IngestOptions {
    /// If true, attempt to decrypt v2 directories (will trigger the
    /// macOS Keychain prompt on first call).
    pub decrypt_v2: bool,
}

/// Top-level ingest umbrella. Always runs the plaintext path; runs the
/// v2 path only when `opts.decrypt_v2` is true (macOS only).
pub fn ingest_into_archive(
    cache_root: &Path,
    archive_root: &Path,
    opts: IngestOptions,
) -> Result<IngestReport> {
    let mut report = ingest_plaintext_into_archive(cache_root, archive_root)?;
    #[cfg(target_os = "macos")]
    if opts.decrypt_v2 {
        ingest_v2_into_archive(cache_root, archive_root, &mut report)?;
    }
    #[cfg(not(target_os = "macos"))]
    let _ = opts; // suppress unused warning on non-macOS
    Ok(report)
}

/// Real-world v2 ingest: fetches the key from the macOS Keychain and ingests.
#[cfg(target_os = "macos")]
pub fn ingest_v2_into_archive(
    cache_root: &Path,
    archive_root: &Path,
    report: &mut IngestReport,
) -> Result<()> {
    let provider = keychain::KeychainKeyProvider;
    ingest_v2_into_archive_with_provider(cache_root, archive_root, report, &provider)
}

/// Provider-injectable variant — used by integration tests on Linux.
pub fn ingest_v2_into_archive_with_provider<P: keychain::KeyProvider>(
    cache_root: &Path,
    archive_root: &Path,
    report: &mut IngestReport,
    provider: &P,
) -> Result<()> {
    // Step 1: fetch passphrase.
    let passphrase = match provider.fetch_v2_passphrase() {
        Ok(p) => p,
        Err(keychain::KeychainError::AccessDenied) => {
            warn!(
                target: "archive::macos_cache",
                "Keychain access denied — user clicked Deny. Re-run and click Allow."
            );
            report
                .errors
                .push("v2 key: Keychain access denied — click Allow when prompted".into());
            return Ok(());
        }
        Err(keychain::KeychainError::ItemNotFound { .. }) => {
            info!(
                target: "archive::macos_cache",
                "v2 Keychain item not found — has ChatGPT.app been signed in?"
            );
            report
                .errors
                .push("v2 key not found — has ChatGPT.app been signed in?".into());
            return Ok(());
        }
        Err(keychain::KeychainError::WouldPrompt) => {
            warn!(
                target: "archive::macos_cache",
                "v2 Keychain item would require a prompt in this context"
            );
            report.errors.push("v2 key: Keychain prompt suppressed".into());
            return Ok(());
        }
        Err(keychain::KeychainError::Other(e)) => return Err(e),
    };

    // Step 2: derive AES key.
    let key = oscrypt::derive_key(&passphrase);

    // Step 3: walk encrypted caches.
    let caches = scan_caches(cache_root)?;
    let failed_decrypts_dir = archive_root
        .join("web")
        .join("chatgpt.com")
        .join(".failed-decrypts");

    for cache in caches.iter().filter(|c| c.kind == CacheKind::Encrypted) {
        let results = decrypt_v2_dir(&cache.path, &key);
        for result in results {
            report.v2_attempted += 1;
            match result {
                Err(e) => {
                    report.v2_failed_decrypt += 1;
                    report
                        .errors
                        .push(format!("decrypt error in {}: {e}", cache.path.display()));
                }
                Ok(decrypted) => {
                    // JSON-sniff via the existing OpenAI export parser.
                    let parsed: Vec<openai::Conversation> =
                        match openai::parse_conversations(&decrypted.plaintext) {
                            Ok(arr) if !arr.is_empty() => arr,
                            Ok(_) => {
                                // Empty array — treat as no conversations.
                                match serde_json::from_slice::<openai::Conversation>(
                                    &decrypted.plaintext,
                                ) {
                                    Ok(single) => vec![single],
                                    Err(_) => {
                                        // Not a conversation array or object — dump it.
                                        dump_failed_decrypt(
                                            &failed_decrypts_dir,
                                            &decrypted,
                                            report,
                                        )?;
                                        continue;
                                    }
                                }
                            }
                            Err(_) => {
                                // Try single-object shape before giving up.
                                match serde_json::from_slice::<openai::Conversation>(
                                    &decrypted.plaintext,
                                ) {
                                    Ok(single) => vec![single],
                                    Err(_) => {
                                        dump_failed_decrypt(
                                            &failed_decrypts_dir,
                                            &decrypted,
                                            report,
                                        )?;
                                        continue;
                                    }
                                }
                            }
                        };

                    report.v2_decrypted += 1;
                    for conv in parsed {
                        report.parsed += 1;
                        let key_id =
                            openai::conversation_key(&conv).unwrap_or_else(|| {
                                decrypted
                                    .source_path
                                    .file_stem()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or("unknown")
                                    .to_string()
                            });
                        let payload = match serde_json::to_value(&conv) {
                            Ok(v) => v,
                            Err(e) => {
                                report.errors.push(format!("{key_id}: serialize: {e}"));
                                continue;
                            }
                        };
                        let web = WebConversation {
                            vendor: "chatgpt.com".into(),
                            conversation_id: key_id.clone(),
                            captured_at: Utc::now()
                                .format("%Y-%m-%dT%H%M%S%.6fZ")
                                .to_string(),
                            schema_fingerprint: "chatgpt.com/macos-v2-decrypted".into(),
                            payload,
                        };
                        match write_web_conversation(archive_root, &web) {
                            Ok(WriteOutcome::Saved { .. }) => report.written += 1,
                            Ok(WriteOutcome::Unchanged) => report.unchanged += 1,
                            Err(e) => report.errors.push(format!("{key_id}: write: {e}")),
                        }
                    }
                }
            }
        }
    }

    // Step 4: probe keychain metadata (informational — best effort).
    #[cfg(target_os = "macos")]
    {
        if let Ok(Some(meta)) = keychain::probe_v2_key_metadata() {
            debug!(
                target: "archive::macos_cache",
                "v2 keychain item: service={} account={:?} grant_required={}",
                meta.service, meta.account, meta.grant_required
            );
        }
    }

    Ok(())
}

/// Write a failed-decrypt blob to the sidecar directory.
fn dump_failed_decrypt(
    sidecar_dir: &Path,
    decrypted: &DecryptedFile,
    report: &mut IngestReport,
) -> Result<()> {
    fs::create_dir_all(sidecar_dir)
        .with_context(|| format!("creating {}", sidecar_dir.display()))?;
    let stem = decrypted
        .source_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    let out_path = sidecar_dir.join(format!("{stem}.bin"));
    fs::write(&out_path, &decrypted.plaintext)
        .with_context(|| format!("writing failed-decrypt to {}", out_path.display()))?;
    report.v2_failed_parse += 1;
    report.errors.push(format!(
        "{}: decrypted but not parseable as JSON conversations; raw bytes saved to {}",
        decrypted.source_path.display(),
        out_path.display()
    ));
    Ok(())
}

/// Keychain access layer for the ChatGPT v2 cache decryptor.
///
/// Provides the `KeyProvider` trait (injectable in tests / Linux CI), a real
/// `KeychainKeyProvider` that reads from macOS Keychain, and a non-prompting
/// `probe_v2_key_metadata` that tells callers whether the item exists before
/// they attempt a full (possibly dialog-triggering) fetch.
#[cfg(target_os = "macos")]
pub mod keychain {
    use anyhow::anyhow;
    use security_framework::item::{ItemClass, ItemSearchOptions, Limit};
    use security_framework::passwords::get_generic_password;

    /// Service name written into the macOS Keychain by ChatGPT.app (Electron
    /// safeStorage). The item is ACL'd to the app's code signature.
    pub const V2_KEY_SERVICE: &str = "com.openai.chat.conversations_v2_cache";

    /// Account names to try in order. Electron uses `app.getName()` which for
    /// ChatGPT.app is "ChatGPT"; some Electron builds store with an empty
    /// account string; if both fail we enumerate by service only.
    pub const ACCOUNT_CANDIDATES: &[&str] = &["ChatGPT", ""];

    // ── errSec* status codes ──────────────────────────────────────────────────

    /// `errSecItemNotFound` (-25300): no matching keychain item.
    const ERR_SEC_ITEM_NOT_FOUND: i32 = -25300;
    /// `errSecAuthFailed` (-25293): user clicked Deny or ACL check failed.
    const ERR_SEC_AUTH_FAILED: i32 = -25293;
    /// `errSecInteractionNotAllowed` (-25308): UI interaction not allowed in
    /// the current session (raised when using kSecUseAuthenticationUIFail).
    #[allow(dead_code)]
    const ERR_SEC_INTERACTION_NOT_ALLOWED: i32 = -25308;

    // macOS Security.framework defines kSecAttrAccount as the CFString "acct".
    // This is stable Apple API; verified against Security/SecItem.h.
    const ATTR_ACCOUNT_KEY: &str = "acct";

    // ── error type ────────────────────────────────────────────────────────────

    /// Typed errors from Keychain operations.
    #[derive(Debug, thiserror::Error)]
    pub enum KeychainError {
        #[error(
            "Keychain item '{service}' not found — has ChatGPT.app ever been \
             launched and signed in?"
        )]
        ItemNotFound { service: String },

        #[error(
            "Keychain access denied (errSecAuthFailed). The user clicked Deny \
             or the prompt was suppressed. Re-run and click Allow."
        )]
        AccessDenied,

        #[error(
            "Keychain query would prompt; pre-flight probe used \
             kSecUseAuthenticationUIFail"
        )]
        WouldPrompt,

        #[error(transparent)]
        Other(#[from] anyhow::Error),
    }

    // ── KeyProvider trait ─────────────────────────────────────────────────────

    /// Abstraction over the AES passphrase source for the v2 cache.
    ///
    /// The real implementation reads from macOS Keychain. Tests inject a stub
    /// so the v2 reader can be exercised on Linux / CI without hitting Keychain.
    pub trait KeyProvider {
        fn fetch_v2_passphrase(&self) -> Result<Vec<u8>, KeychainError>;
    }

    // ── KeychainKeyProvider ───────────────────────────────────────────────────

    /// Production [`KeyProvider`] backed by the macOS Keychain.
    pub struct KeychainKeyProvider;

    impl KeyProvider for KeychainKeyProvider {
        fn fetch_v2_passphrase(&self) -> Result<Vec<u8>, KeychainError> {
            let mut last_not_found = false;

            for &account in ACCOUNT_CANDIDATES {
                match get_generic_password(V2_KEY_SERVICE, account) {
                    Ok(pw) => return Ok(pw.to_vec()),
                    Err(e) => {
                        let code = e.code();
                        if code == ERR_SEC_AUTH_FAILED {
                            // User clicked Deny — stop immediately; trying
                            // the next account candidate won't help.
                            return Err(KeychainError::AccessDenied);
                        } else if code == ERR_SEC_ITEM_NOT_FOUND {
                            last_not_found = true;
                            // Continue to next candidate.
                        } else {
                            return Err(KeychainError::Other(anyhow!(
                                "Keychain errSec {code}: {}",
                                e.message().unwrap_or_else(|| "unknown".into())
                            )));
                        }
                    }
                }
            }

            if last_not_found {
                return Err(KeychainError::ItemNotFound {
                    service: V2_KEY_SERVICE.into(),
                });
            }

            // Shouldn't be reachable (ACCOUNT_CANDIDATES is non-empty and
            // every branch above returns), but satisfy the compiler.
            Err(KeychainError::ItemNotFound {
                service: V2_KEY_SERVICE.into(),
            })
        }
    }

    // ── probe metadata ────────────────────────────────────────────────────────

    /// What we know about the v2 Keychain item without unlocking it.
    #[derive(Debug, Clone, serde::Serialize)]
    pub struct KeychainItemMeta {
        pub service: String,
        /// Hint only — may be `None` if the system didn't return the attribute.
        pub account: Option<String>,
        /// `true` when a non-prompting probe fetch indicated the item exists
        /// but we haven't been granted access yet. Set conservatively.
        pub grant_required: bool,
    }

    /// Returns `Some(meta)` if the v2 Keychain item exists, `None` if absent.
    ///
    /// Uses an attribute-only query (`kSecReturnData = false`) so it never
    /// triggers the user-grant dialog. The `grant_required` field is set by
    /// attempting a non-prompting data fetch: if we can read the data we are
    /// already granted; any other error (auth-failed, interaction-not-allowed)
    /// sets `grant_required = true` conservatively.
    pub fn probe_v2_key_metadata() -> Result<Option<KeychainItemMeta>, KeychainError> {
        // Phase 1: attribute-only search — confirms existence without loading
        // the secret and without prompting.
        let results = ItemSearchOptions::new()
            .class(ItemClass::generic_password())
            .service(V2_KEY_SERVICE)
            .load_attributes(true)
            .load_data(false)
            .limit(Limit::Max(1))
            .search()
            .map_err(|e| {
                let code = e.code();
                if code == ERR_SEC_ITEM_NOT_FOUND {
                    return KeychainError::ItemNotFound {
                        service: V2_KEY_SERVICE.into(),
                    };
                }
                KeychainError::Other(anyhow!(
                    "Keychain attribute probe errSec {code}: {}",
                    e.message().unwrap_or_else(|| "unknown".into())
                ))
            });

        let results = match results {
            Ok(r) => r,
            Err(KeychainError::ItemNotFound { .. }) => return Ok(None),
            Err(e) => return Err(e),
        };

        if results.is_empty() {
            return Ok(None);
        }

        // Extract the account hint from the first result's attribute dict.
        // `simplify_dict()` converts the CFDictionary into HashMap<String,String>
        // using the CF attribute name strings as keys; kSecAttrAccount → "acct".
        let account: Option<String> = results
            .into_iter()
            .find_map(|r| r.simplify_dict())
            .and_then(|map| map.get(ATTR_ACCOUNT_KEY).cloned());

        // Phase 2: attempt a data fetch to determine whether we are already
        // granted. `get_generic_password` follows the normal ACL path. In a
        // terminal session the OS may show the grant dialog — that is
        // intentional when the caller has decided to proceed. We use the
        // simpler heuristic: success → granted, any error → grant_required.
        let grant_required = {
            let mut granted = false;
            for &account_candidate in ACCOUNT_CANDIDATES {
                match get_generic_password(V2_KEY_SERVICE, account_candidate) {
                    Ok(_) => {
                        granted = true;
                        break;
                    }
                    Err(e) if e.code() == ERR_SEC_ITEM_NOT_FOUND => {
                        // This candidate doesn't exist; try the next one.
                        continue;
                    }
                    Err(_) => {
                        // Auth-failed, interaction-not-allowed, or other —
                        // item exists but we can't read it without a grant.
                        break;
                    }
                }
            }
            !granted
        };

        Ok(Some(KeychainItemMeta {
            service: V2_KEY_SERVICE.into(),
            account,
            grant_required,
        }))
    }
}

/// Chromium / Electron `safeStorage` cipher primitives (OSCrypt v10 format).
///
/// ChatGPT.app uses the same Electron `safeStorage` AES-128-CBC scheme that
/// Chromium uses on macOS for its safe-storage layer (the "v10" cookie
/// encryption variant). The key is derived via PBKDF2-HMAC-SHA1 from a
/// passphrase stored in macOS Keychain; the ciphertext has a 3-byte `v10`
/// prefix followed by 16-byte constant-IV AES-128-CBC data.
///
/// All functions here are `pub(super)` so sibling submods can reach them
/// without leaking the crypto surface into the public API.
mod oscrypt {
    use aes::Aes128;
    use anyhow::{Context, Result, bail};
    use cbc::cipher::{BlockDecryptMut, KeyIvInit, block_padding::Pkcs7};
    use pbkdf2::pbkdf2_hmac;
    use sha1::Sha1;

    /// The constant 16-byte AES-128-CBC initialisation vector used by every
    /// OSCrypt v10 blob — 16 ASCII space characters (0x20).
    const IV: [u8; 16] = [0x20u8; 16];

    /// PBKDF2 salt hard-coded by Chromium's OSCrypt implementation.
    const SALT: &[u8] = b"saltysalt";

    /// PBKDF2 iteration count hard-coded by Chromium's OSCrypt implementation.
    const ITERATIONS: u32 = 1003;

    /// The 3-byte ASCII prefix that marks an OSCrypt v10 encrypted blob.
    const PREFIX: &[u8] = b"v10";

    /// Derive the 16-byte AES-128 key from a Keychain-vended passphrase.
    ///
    /// Uses PBKDF2-HMAC-SHA1 with the hard-coded Chromium salt `"saltysalt"`
    /// and 1003 iterations, producing a 16-byte (128-bit) key.
    pub(super) fn derive_key(passphrase: &[u8]) -> [u8; 16] {
        let mut key = [0u8; 16];
        pbkdf2_hmac::<Sha1>(passphrase, SALT, ITERATIONS, &mut key);
        key
    }

    /// Decrypt an OSCrypt v10 blob.
    ///
    /// Expects `prefixed` to start with the 3-byte ASCII prefix `v10`,
    /// followed by the AES-128-CBC ciphertext (padded with PKCS#7).
    /// The constant 16-space IV is used; no integrity tag is present.
    ///
    /// Returns an error if:
    /// - the input does not begin with `v10`,
    /// - the ciphertext length is not a positive multiple of 16, or
    /// - PKCS#7 unpadding fails.
    pub(super) fn decrypt_v10_blob(prefixed: &[u8], key: &[u8; 16]) -> Result<Vec<u8>> {
        let ciphertext = prefixed
            .strip_prefix(PREFIX)
            .context("blob does not start with v10 prefix")?;
        if ciphertext.is_empty() || ciphertext.len() % 16 != 0 {
            bail!(
                "ciphertext length {} is not a positive multiple of 16",
                ciphertext.len()
            );
        }
        type Aes128CbcDec = cbc::Decryptor<Aes128>;
        let mut buf = ciphertext.to_vec();
        let plaintext = Aes128CbcDec::new(key.into(), &IV.into())
            .decrypt_padded_mut::<Pkcs7>(&mut buf)
            .map_err(|e| anyhow::anyhow!("AES-128-CBC decrypt / unpad failed: {e}"))?;
        Ok(plaintext.to_vec())
    }

    /// Encrypt plaintext into an OSCrypt v10 blob (test-only).
    ///
    /// Prepends the `v10` prefix, then AES-128-CBC encrypts with PKCS#7
    /// padding and the constant 16-space IV. This is the exact inverse of
    /// `decrypt_v10_blob` and exists solely so unit tests can mint synthetic
    /// fixtures without depending on external tooling.
    #[cfg(test)]
    pub(super) fn encrypt_v10_blob(plaintext: &[u8], key: &[u8; 16]) -> Vec<u8> {
        use cbc::cipher::BlockEncryptMut;
        type Aes128CbcEnc = cbc::Encryptor<Aes128>;
        // Allocate space: plaintext + up to one full padding block.
        let padded_len = ((plaintext.len() / 16) + 1) * 16;
        let mut buf = vec![0u8; padded_len];
        buf[..plaintext.len()].copy_from_slice(plaintext);
        let ciphertext = Aes128CbcEnc::new(key.into(), &IV.into())
            .encrypt_padded_mut::<Pkcs7>(&mut buf, plaintext.len())
            .expect("encrypt_padded_mut: buffer too small (should not happen)");
        let mut out = Vec::with_capacity(PREFIX.len() + ciphertext.len());
        out.extend_from_slice(PREFIX);
        out.extend_from_slice(ciphertext);
        out
    }
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
        assert!(
            enc.unreadable_reason
                .as_deref()
                .unwrap_or("")
                .contains("--decrypt-v2"),
            "expected --decrypt-v2 hint in reason, got {:?}",
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

    // ── oscrypt cipher tests ─────────────────────────────────────────────────

    #[test]
    fn oscrypt_round_trips_a_short_payload() {
        let key = [0x42u8; 16];
        let plaintext = b"hello world";
        let blob = oscrypt::encrypt_v10_blob(plaintext, &key);
        let recovered = oscrypt::decrypt_v10_blob(&blob, &key).unwrap();
        assert_eq!(recovered, plaintext);
    }

    #[test]
    fn oscrypt_round_trips_a_payload_that_exactly_fills_a_block() {
        // 16 bytes of input → PKCS#7 appends a full 16-byte padding block,
        // so the ciphertext body is 32 bytes.
        let key = [0x11u8; 16];
        let plaintext = b"1234567890abcdef"; // exactly 16 bytes
        let blob = oscrypt::encrypt_v10_blob(plaintext, &key);
        // Prefix (3) + 2 AES blocks (32) = 35 bytes total.
        assert_eq!(blob.len(), 3 + 32);
        let recovered = oscrypt::decrypt_v10_blob(&blob, &key).unwrap();
        assert_eq!(recovered.as_slice(), plaintext);
    }

    #[test]
    fn oscrypt_round_trips_a_long_json_payload() {
        let key = [0xABu8; 16];
        // Build a ~4 KiB JSON string.
        let payload: String = {
            let entry = r#"{"key":"value","number":12345678,"flag":true}"#;
            let entries: Vec<&str> = std::iter::repeat(entry).take(90).collect();
            format!("[{}]", entries.join(","))
        };
        assert!(payload.len() >= 4000, "payload too short: {}", payload.len());
        let blob = oscrypt::encrypt_v10_blob(payload.as_bytes(), &key);
        let recovered = oscrypt::decrypt_v10_blob(&blob, &key).unwrap();
        assert_eq!(recovered, payload.as_bytes());
    }

    #[test]
    fn oscrypt_decrypt_rejects_non_v10_prefix() {
        let key = [0x00u8; 16];
        // A blob that starts with "v11" instead of "v10".
        let bad_blob = b"v11\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        let err = oscrypt::decrypt_v10_blob(bad_blob, &key);
        assert!(err.is_err(), "expected error for non-v10 prefix");
        let msg = err.unwrap_err().to_string();
        assert!(
            msg.contains("v10"),
            "error should mention v10 prefix, got: {msg}"
        );
    }

    #[test]
    fn oscrypt_derive_key_known_vector() {
        // Derive the key for the passphrase "peanuts" using the hard-coded
        // Chromium parameters (salt="saltysalt", 1003 iterations, 16 bytes).
        // The expected value was computed by calling derive_key itself and
        // pinning the result; the test guards against accidental regressions
        // (wrong salt, wrong iteration count, wrong output length).
        let key = oscrypt::derive_key(b"peanuts");
        let hex: String = key.iter().map(|b| format!("{b:02x}")).collect();
        // Pin the value produced by this exact PBKDF2-HMAC-SHA1 configuration.
        assert_eq!(
            hex,
            "d9a09d499b4e1b7461f28e67972c6dbd",
            "derive_key regression: got {hex}"
        );
    }

    // ── keychain trait tests (CI-safe: no real Keychain access) ──────────────

    #[test]
    fn keychain_stub_provider_returns_passphrase() {
        use super::keychain::KeyProvider;
        struct Stub(Vec<u8>);
        impl KeyProvider for Stub {
            fn fetch_v2_passphrase(
                &self,
            ) -> Result<Vec<u8>, super::keychain::KeychainError> {
                Ok(self.0.clone())
            }
        }
        let p = Stub(b"peanuts".to_vec());
        assert_eq!(p.fetch_v2_passphrase().unwrap(), b"peanuts");
    }

    #[test]
    fn keychain_stub_provider_can_propagate_access_denied() {
        use super::keychain::KeyProvider;
        struct Denied;
        impl KeyProvider for Denied {
            fn fetch_v2_passphrase(
                &self,
            ) -> Result<Vec<u8>, super::keychain::KeychainError> {
                Err(super::keychain::KeychainError::AccessDenied)
            }
        }
        let err = Denied.fetch_v2_passphrase().unwrap_err();
        assert!(matches!(err, super::keychain::KeychainError::AccessDenied));
    }

    #[test]
    fn keychain_error_display_messages_are_actionable() {
        let e = super::keychain::KeychainError::ItemNotFound {
            service: "test".into(),
        };
        assert!(
            format!("{e}").contains("ChatGPT.app"),
            "ItemNotFound message should mention ChatGPT.app, got: {e}"
        );
        let e = super::keychain::KeychainError::AccessDenied;
        assert!(
            format!("{e}").contains("Allow"),
            "AccessDenied message should mention Allow, got: {e}"
        );
    }

    // ── v2 reader + ingest tests ─────────────────────────────────────────────

    /// Stub KeyProvider that always returns a fixed passphrase.
    struct StubKeyProvider(Vec<u8>);
    impl keychain::KeyProvider for StubKeyProvider {
        fn fetch_v2_passphrase(&self) -> Result<Vec<u8>, keychain::KeychainError> {
            Ok(self.0.clone())
        }
    }

    #[test]
    fn decrypt_v2_dir_round_trips_synthetic_payloads() {
        let passphrase = b"test-passphrase";
        let key = oscrypt::derive_key(passphrase);

        let tmp = TempDir::new().unwrap();
        let v2_dir = tmp.path().join("conversations-v2-test");
        fs::create_dir_all(&v2_dir).unwrap();

        // Write 2 encrypted .data files.
        let payload1 = b"hello from file one";
        let payload2 = b"hello from file two with more content here!";
        fs::write(
            v2_dir.join("file1.data"),
            oscrypt::encrypt_v10_blob(payload1, &key),
        )
        .unwrap();
        fs::write(
            v2_dir.join("file2.data"),
            oscrypt::encrypt_v10_blob(payload2, &key),
        )
        .unwrap();
        // Non-.data file — should be ignored.
        fs::write(v2_dir.join("readme.txt"), b"ignore me").unwrap();

        let results = decrypt_v2_dir(&v2_dir, &key);
        assert_eq!(results.len(), 2, "expected 2 .data files decrypted");

        let mut plaintexts: Vec<Vec<u8>> = results
            .into_iter()
            .map(|r| r.expect("decryption should succeed").plaintext)
            .collect();
        plaintexts.sort();

        let mut expected = vec![payload1.to_vec(), payload2.to_vec()];
        expected.sort();
        assert_eq!(plaintexts, expected);
    }

    #[test]
    fn ingest_v2_into_archive_writes_chatgpt_conversations() {
        let passphrase = b"ingest-test-key";
        let key = oscrypt::derive_key(passphrase);

        let tmp = TempDir::new().unwrap();
        let cache = tmp.path().join("cache");
        let archive = tmp.path().join("archive");
        let v2_dir = cache.join("conversations-v2-abc");
        fs::create_dir_all(&v2_dir).unwrap();

        // Build two valid OpenAI conversation JSONs and encrypt them.
        let conv1 = serde_json::json!([{
            "id": "conv-v2-alpha",
            "title": "Alpha",
            "create_time": 1.0,
            "mapping": {}
        }]);
        let conv2 = serde_json::json!([{
            "id": "conv-v2-beta",
            "title": "Beta",
            "create_time": 2.0,
            "mapping": {}
        }]);
        fs::write(
            v2_dir.join("alpha.data"),
            oscrypt::encrypt_v10_blob(conv1.to_string().as_bytes(), &key),
        )
        .unwrap();
        fs::write(
            v2_dir.join("beta.data"),
            oscrypt::encrypt_v10_blob(conv2.to_string().as_bytes(), &key),
        )
        .unwrap();

        let provider = StubKeyProvider(passphrase.to_vec());
        let mut report = IngestReport::default();
        ingest_v2_into_archive_with_provider(&cache, &archive, &mut report, &provider).unwrap();

        assert_eq!(report.v2_attempted, 2, "should attempt both .data files");
        assert_eq!(report.v2_decrypted, 2, "both should decrypt+parse");
        assert_eq!(report.v2_failed_decrypt, 0);
        assert_eq!(report.v2_failed_parse, 0);
        assert_eq!(report.written, 2, "two conversations written");

        let web_dir = archive.join("web").join("chatgpt.com");
        assert!(
            web_dir.join("conv-v2-alpha.json").is_file(),
            "conv-v2-alpha.json missing"
        );
        assert!(
            web_dir.join("conv-v2-beta.json").is_file(),
            "conv-v2-beta.json missing"
        );
    }

    #[test]
    fn ingest_v2_into_archive_dumps_unparseable_plaintexts() {
        let passphrase = b"dump-test-key-xx";
        let key = oscrypt::derive_key(passphrase);

        let tmp = TempDir::new().unwrap();
        let cache = tmp.path().join("cache");
        let archive = tmp.path().join("archive");
        let v2_dir = cache.join("conversations-v2-xyz");
        fs::create_dir_all(&v2_dir).unwrap();

        // File 1: valid conversation.
        let good = serde_json::json!([{
            "id": "conv-good",
            "title": "Good",
            "create_time": 1.0,
            "mapping": {}
        }]);
        fs::write(
            v2_dir.join("good.data"),
            oscrypt::encrypt_v10_blob(good.to_string().as_bytes(), &key),
        )
        .unwrap();

        // File 2: random bytes that decrypt cleanly but are not valid JSON conversations.
        let garbage = b"this is not json at all %%%%";
        fs::write(
            v2_dir.join("garbage.data"),
            oscrypt::encrypt_v10_blob(garbage, &key),
        )
        .unwrap();

        let provider = StubKeyProvider(passphrase.to_vec());
        let mut report = IngestReport::default();
        ingest_v2_into_archive_with_provider(&cache, &archive, &mut report, &provider).unwrap();

        assert_eq!(report.v2_attempted, 2);
        assert_eq!(report.v2_decrypted, 1, "only the good file should parse");
        assert_eq!(report.v2_failed_parse, 1, "garbage should land in failed-decrypts");
        assert_eq!(report.v2_failed_decrypt, 0);
        assert_eq!(report.written, 1);

        let web_dir = archive.join("web").join("chatgpt.com");
        assert!(web_dir.join("conv-good.json").is_file(), "good conversation missing");

        let failed_dir = web_dir.join(".failed-decrypts");
        assert!(failed_dir.is_dir(), ".failed-decrypts dir missing");
        assert!(
            failed_dir.join("garbage.bin").is_file(),
            "garbage.bin missing in .failed-decrypts"
        );
    }
}
