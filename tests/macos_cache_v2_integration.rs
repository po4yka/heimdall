//! End-to-end test: synthetic v2 cache + stub key + archive round-trip.
//!
//! Mirrors the shape of `imports_integration.rs`. Uses the provider-injectable
//! variant of the ingest pipeline so no real macOS Keychain access is needed.
//! The `archive::macos_cache` module is `cfg`-gated to macOS, so this test
//! file is gated as well — on Linux/Windows CI it compiles to an empty crate.

#![cfg(target_os = "macos")]

use claude_usage_tracker::archive::macos_cache::{
    IngestOptions, IngestReport, ingest_into_archive, ingest_v2_into_archive_with_provider,
    keychain,
};
use std::fs;
use tempfile::TempDir;

// ── stub KeyProvider ─────────────────────────────────────────────────────────

struct StubKeyProvider(Vec<u8>);

impl keychain::KeyProvider for StubKeyProvider {
    fn fetch_v2_passphrase(&self) -> Result<Vec<u8>, keychain::KeychainError> {
        Ok(self.0.clone())
    }
}

struct DeniedKeyProvider;

impl keychain::KeyProvider for DeniedKeyProvider {
    fn fetch_v2_passphrase(&self) -> Result<Vec<u8>, keychain::KeychainError> {
        Err(keychain::KeychainError::AccessDenied)
    }
}

// ── helpers ──────────────────────────────────────────────────────────────────

/// Derive the AES key from a passphrase using the same PBKDF2 params as
/// the production code. We call directly into the public surface via the
/// encrypt helper exposed in test-builds.
fn make_v10_blob(plaintext: &[u8], passphrase: &[u8]) -> Vec<u8> {
    // Use PBKDF2-HMAC-SHA1, salt "saltysalt", 1003 iters — same as oscrypt::derive_key.
    use pbkdf2::pbkdf2_hmac;
    use sha1::Sha1;
    let mut key = [0u8; 16];
    pbkdf2_hmac::<Sha1>(passphrase, b"saltysalt", 1003, &mut key);

    // AES-128-CBC with constant 16-space IV, v10 prefix, PKCS#7 padding.
    use aes::Aes128;
    use cbc::cipher::{BlockEncryptMut, KeyIvInit, block_padding::Pkcs7};
    type Aes128CbcEnc = cbc::Encryptor<Aes128>;
    let iv = [0x20u8; 16];
    let padded_len = ((plaintext.len() / 16) + 1) * 16;
    let mut buf = vec![0u8; padded_len];
    buf[..plaintext.len()].copy_from_slice(plaintext);
    let ct = Aes128CbcEnc::new(&key.into(), &iv.into())
        .encrypt_padded_mut::<Pkcs7>(&mut buf, plaintext.len())
        .expect("encrypt ok");
    let mut out = Vec::with_capacity(3 + ct.len());
    out.extend_from_slice(b"v10");
    out.extend_from_slice(ct);
    out
}

fn conv_json(id: &str, title: &str) -> String {
    serde_json::json!([{
        "id": id,
        "title": title,
        "create_time": 1.0,
        "mapping": {}
    }])
    .to_string()
}

// ── tests ────────────────────────────────────────────────────────────────────

#[test]
fn v2_ingest_writes_both_conversations_to_web_tree() {
    let passphrase = b"integration-test-pass";
    let tmp = TempDir::new().unwrap();
    let cache = tmp.path().join("cache");
    let archive = tmp.path().join("archive");

    let v2_dir = cache.join("conversations-v2-00000000-0000-0000-0000-000000000001");
    fs::create_dir_all(&v2_dir).unwrap();

    fs::write(
        v2_dir.join("alpha.data"),
        make_v10_blob(conv_json("it-conv-alpha", "Alpha").as_bytes(), passphrase),
    )
    .unwrap();
    fs::write(
        v2_dir.join("beta.data"),
        make_v10_blob(conv_json("it-conv-beta", "Beta").as_bytes(), passphrase),
    )
    .unwrap();

    let provider = StubKeyProvider(passphrase.to_vec());
    let mut report = IngestReport::default();
    ingest_v2_into_archive_with_provider(&cache, &archive, &mut report, &provider).unwrap();

    assert_eq!(report.v2_attempted, 2);
    assert_eq!(report.v2_decrypted, 2);
    assert_eq!(report.v2_failed_decrypt, 0);
    assert_eq!(report.v2_failed_parse, 0);
    assert_eq!(report.written, 2);
    assert!(
        report.errors.is_empty(),
        "unexpected errors: {:?}",
        report.errors
    );

    let web = archive.join("web").join("chatgpt.com");
    assert!(web.join("it-conv-alpha.json").is_file(), "alpha missing");
    assert!(web.join("it-conv-beta.json").is_file(), "beta missing");

    // Verify schema_fingerprint in the written file.
    let raw = fs::read(web.join("it-conv-alpha.json")).unwrap();
    let val: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    assert_eq!(
        val["schema_fingerprint"].as_str().unwrap(),
        "chatgpt.com/macos-v2-decrypted"
    );
    assert_eq!(val["vendor"].as_str().unwrap(), "chatgpt.com");
}

#[test]
fn v2_ingest_dumps_non_json_plaintext_to_failed_decrypts() {
    let passphrase = b"fail-parse-test-pp";
    let tmp = TempDir::new().unwrap();
    let cache = tmp.path().join("cache");
    let archive = tmp.path().join("archive");

    let v2_dir = cache.join("conversations-v2-00000000-0000-0000-0000-000000000002");
    fs::create_dir_all(&v2_dir).unwrap();

    // Good file.
    fs::write(
        v2_dir.join("good.data"),
        make_v10_blob(conv_json("it-good", "Good").as_bytes(), passphrase),
    )
    .unwrap();
    // Garbage: valid v10 blob, but plaintext is not JSON.
    fs::write(
        v2_dir.join("garbage.data"),
        make_v10_blob(b"not a json conversation !!!", passphrase),
    )
    .unwrap();

    let provider = StubKeyProvider(passphrase.to_vec());
    let mut report = IngestReport::default();
    ingest_v2_into_archive_with_provider(&cache, &archive, &mut report, &provider).unwrap();

    assert_eq!(report.v2_attempted, 2);
    assert_eq!(report.v2_decrypted, 1);
    assert_eq!(report.v2_failed_parse, 1);
    assert_eq!(report.v2_failed_decrypt, 0);
    assert_eq!(report.written, 1);

    let web = archive.join("web").join("chatgpt.com");
    assert!(web.join("it-good.json").is_file(), "good.json missing");
    let failed = web.join(".failed-decrypts").join("garbage.bin");
    assert!(failed.is_file(), ".failed-decrypts/garbage.bin missing");
    // Raw bytes should match the garbage plaintext.
    let raw = fs::read(&failed).unwrap();
    assert_eq!(raw, b"not a json conversation !!!");
}

#[test]
fn v2_ingest_access_denied_records_error_but_does_not_fail() {
    let tmp = TempDir::new().unwrap();
    let cache = tmp.path().join("cache");
    let archive = tmp.path().join("archive");

    // Create a v2 dir so scan finds it.
    let v2_dir = cache.join("conversations-v2-00000000-0000-0000-0000-000000000003");
    fs::create_dir_all(&v2_dir).unwrap();
    fs::write(v2_dir.join("a.data"), b"irrelevant").unwrap();

    let mut report = IngestReport::default();
    let result =
        ingest_v2_into_archive_with_provider(&cache, &archive, &mut report, &DeniedKeyProvider);

    // Should not propagate as Err — just record in report.errors.
    assert!(
        result.is_ok(),
        "access denied should not propagate: {:?}",
        result
    );
    assert_eq!(
        report.v2_attempted, 0,
        "no files attempted when key fetch fails"
    );
    assert!(
        report
            .errors
            .iter()
            .any(|e| e.contains("Allow") || e.contains("denied")),
        "expected an error mentioning Allow/denied: {:?}",
        report.errors
    );
}

#[test]
fn ingest_into_archive_umbrella_plaintext_only_when_decrypt_v2_false() {
    let tmp = TempDir::new().unwrap();
    let cache = tmp.path().join("cache");
    let archive = tmp.path().join("archive");

    // Plaintext dir.
    let pt_dir = cache.join("conversations-aaa");
    fs::create_dir_all(&pt_dir).unwrap();
    let conv = serde_json::json!([{"id": "umbrella-conv", "title": "T", "mapping": {}}]);
    fs::write(pt_dir.join("c.json"), conv.to_string()).unwrap();

    // Encrypted dir (left untouched).
    let v2_dir = cache.join("conversations-v2-bbb");
    fs::create_dir_all(&v2_dir).unwrap();
    fs::write(v2_dir.join("x.data"), b"\x00\x01\x02").unwrap();

    let opts = IngestOptions {
        decrypt_v2: false,
        v3_key: None,
    };
    let report = ingest_into_archive(&cache, &archive, opts).unwrap();

    assert_eq!(report.parsed, 1);
    assert_eq!(report.written, 1);
    assert_eq!(report.encrypted_dirs, 1);
    // v2 fields untouched when decrypt_v2 = false.
    assert_eq!(report.v2_attempted, 0);
    assert_eq!(report.v2_decrypted, 0);
}

#[test]
fn v3_ingest_into_archive_umbrella_with_key() {
    use aes_gcm::aead::{Aead, KeyInit};
    use aes_gcm::{Aes256Gcm, Key, Nonce};

    let tmp = TempDir::new().unwrap();
    let cache = tmp.path().join("cache");
    let archive = tmp.path().join("archive");

    // Build a synthetic v3 directory with one encrypted conversation.
    let v3_dir = cache.join("conversations-v3-00000000-0000-0000-0000-000000000001");
    fs::create_dir_all(&v3_dir).unwrap();

    let key_bytes = [0xbbu8; 32];
    let nonce_bytes = [0xccu8; 12];
    let conv = serde_json::json!([{
        "id": "it-v3-conv-umbrella",
        "title": "V3 Umbrella",
        "create_time": 1.0,
        "mapping": {}
    }]);
    let plaintext = serde_json::to_vec(&conv).unwrap();

    // Encrypt: nonce (12) || ciphertext+tag
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key_bytes));
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ct_and_tag = cipher.encrypt(nonce, plaintext.as_slice()).unwrap();
    let mut blob = Vec::with_capacity(12 + ct_and_tag.len());
    blob.extend_from_slice(&nonce_bytes);
    blob.extend_from_slice(&ct_and_tag);
    fs::write(v3_dir.join("conv.data"), &blob).unwrap();

    // Exercise the umbrella via ingest_into_archive with v3_key set.
    let opts = IngestOptions {
        decrypt_v2: false,
        v3_key: Some(key_bytes.to_vec()),
    };
    let report = ingest_into_archive(&cache, &archive, opts).unwrap();

    // Detection counters (populated by ingest_plaintext_into_archive scan pass).
    assert_eq!(report.v3_dirs, 1, "should detect the v3 dir");
    assert_eq!(report.v3_files, 1, "should detect 1 file");
    // Decryption counters.
    assert_eq!(report.v3_attempted, 1, "should attempt the 1 file");
    assert_eq!(report.v3_decrypted, 1, "should decrypt+parse successfully");
    assert_eq!(report.v3_failed_decrypt, 0);
    assert_eq!(report.v3_failed_parse, 0);
    assert_eq!(report.written, 1, "conversation written to archive");

    let web = archive.join("web").join("chatgpt.com");
    assert!(
        web.join("it-v3-conv-umbrella.json").is_file(),
        "it-v3-conv-umbrella.json missing"
    );

    // Verify schema_fingerprint in the written file.
    let raw = fs::read(web.join("it-v3-conv-umbrella.json")).unwrap();
    let val: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    assert_eq!(
        val["schema_fingerprint"].as_str().unwrap(),
        "chatgpt.com/macos-v3-decrypted"
    );
    assert_eq!(val["vendor"].as_str().unwrap(), "chatgpt.com");
}
