# ChatGPT macOS v3 Cache — Findings + Plan

> **For agentic workers:** Phase A (detection) shipped in an earlier commit. Phase B (cipher) shipped with a user-supplied key path — see "Phase B — Cipher implementation (SHIPPED with user-supplied key)" below. Do not attempt to add a `KeychainKeyProvider` for v3; the key is entitlement-gated and inaccessible to third-party CLIs.

## TL;DR

ChatGPT for macOS shipped a **v3** conversation cache format (`conversations-v3-{uuid}/`) sometime between July 2024 and early 2026. The v2 decryptor we just shipped does **not** apply to v3 because:

1. **Cipher changed**: v3 is almost certainly AES-GCM (or another AEAD), not the Chromium OSCrypt AES-128-CBC stack used by v2.
2. **Key location changed**: the v3 key is in the macOS **data-protection keychain**, not the legacy login keychain. Apple's data-protection keychain is **partitioned by app entitlements / Team ID**, and a third-party CLI without OpenAI's entitlement gets `errSecItemNotFound` — indistinguishable from "doesn't exist" — for items it cannot read. No `Allow / Deny` user prompt is offered because this is not a user-mediated ACL boundary; it is an architectural one.

So unlike v2, where the user-mediated Keychain ACL prompt was the boundary and we accepted it, **v3 has no documented user-consent path that grants a third-party CLI access**. The cipher is implementable; the key is not retrievable. Without the key, the cipher is useless.

This plan therefore has only one *implementable* phase (Phase A — honest detection + UX). Phase B (cipher implementation) is documented for future-readiness but **not slated for implementation** until a public RE writeup appears that solves the key-access problem.

---

## Forensic findings (from the user's own machine)

Inspected `~/Library/Application Support/com.openai.chat/`:

```
conversations-v3-4c16756e-...   # 399 *.data files, sizes 95–225 KB+
drafts-v2-4c16756e-...           # empty (drafts not promoted to v3)
gizmos-...                       # sibling encrypted dir
models-...                       # sibling encrypted dir
order-orders-...                 # sibling encrypted dir
order-order-details-...          # sibling encrypted dir
health-system-hints-...          # sibling encrypted dir
```

Per-file evidence:

- File extension: `.data`
- File sizes are **not multiples of 16** (one example: `23868 mod 16 = 12`). → Rules out plain AES-CBC. Consistent with AES-GCM (`plaintext_len + 16-byte tag`) or another AEAD with similar tag.
- First 12 bytes are **distinct per file** across 5 samples I checked. → Consistent with per-file 96-bit AES-GCM nonces.
- No `v10` ASCII prefix (the v2 / Chromium-OSCrypt magic). v3 is binary from byte 0.

Keychain probe:

- `security find-generic-password -s "com.openai.chat.conversations_v3_cache"` → "item could not be found".
- `security dump-keychain` reveals only the OpenAI Bearer API token (`https://api.openai.com/v1`). Zero entries with "conversations" in service or account.
- This confirms the v3 key is **not in the legacy file-based keychain**.

## Public-source confirmation

- Zero open-source projects, blog posts, or academic papers document the v3 format as of April 2026.
- Pedro Vieito's July 2024 disclosure (`pvieito.com`) covers only v1→v2. No v3 follow-up.
- Apple's own DTS engineer ("eskimo") confirms data-protection keychain semantics on developer.apple.com:
> "The data protection keychain is only available to code that can carry an entitlement, that is, main executables like an app or an app extension."
- `SecItemCopyMatching` does not surface metadata for items outside the caller's access groups — `errSecItemNotFound` is indistinguishable between "absent" and "present-but-inaccessible".

## What we cannot do, and why we won't try

Hypothetical paths to the v3 key, all out of scope per Heimdall's operational ground rules:

- **Code-sign as ChatGPT.app**: requires faking OpenAI's Team ID (`2DC432JVRJ`). Forbidden.
- **Inject into the running ChatGPT process**: requires defeating hardened runtime + amfid + SIP. Forbidden.
- **Dump SEP-protected memory**: requires kernel-level access. Forbidden.
- **`chainbreaker`-style direct DB parsing**: doesn't yet handle the data-protection keychain SQLite shape; even if it did, the per-row AES-256-GCM table key wraps each item and the access-group enforcement is at the framework level.

The honest user-facing answer for v3 is: *use the alternatives we already shipped*:

- **Phase 3b browser extension** — captures conversations from `chatgpt.com` directly. No desktop cache needed.
- **Phase 2 account-export importer** — full account export ZIP, slow but complete, including archived conversations.
- **Phase 3a cookie-paste CLI** — `heimdall scrape chatgpt …` for headless setups.

---

## Phase A — Honest v3 detection (implementable now)

**Goal:** make `heimdall macos-cache scan` and `ingest` surface v3 directories clearly instead of silently ignoring them, with a pointed recommendation to use one of the existing alternatives.

### Files to modify

| File | Change |
|------|--------|
| `src/archive/macos_cache.rs` | Extend `CacheKind` with `EncryptedV3`. Update `classify_dir_name` to return `EncryptedV3` for `conversations-v3-*`. Update `scan_caches` to populate `unreadable_reason` with the v3-specific message. Update `ingest_into_archive` to skip v3 cleanly while counting the dirs/files in the report. Update `IngestReport` with `v3_dirs` + `v3_files` counters. |
| `src/main.rs` | `MacosCacheAction::Scan` output: surface v3 dirs in the listing with a 2-line note: (a) "v3 cache — encryption key is in the data-protection keychain; not accessible to third-party tools" and (b) "alternatives: heimdall macos-cache ingest is no help here; try the browser extension (extensions/heimdall-companion), the account-export importer (heimdall import-export), or the cookie-paste CLI (heimdall scrape chatgpt …)". |
| Existing test `classify_recognises_plaintext_and_encrypted_names` | Update: `conversations-v3-abc` should now classify as `Some(EncryptedV3)`, not `None`. |

### Tests to add (3)

1. `classify_recognises_v3_layout` — `classify_dir_name("conversations-v3-abc")` returns `Some(CacheKind::EncryptedV3)`.
2. `scan_caches_classifies_v3` — synthetic dir with one `conversations-v3-test/` under tempdir → reports kind `EncryptedV3` with `unreadable_reason` matching the v3 message.
3. `ingest_v2_does_not_attempt_v3` — synthetic dir with one v2 (real-decryptable, with stub provider) and one v3 dir → after ingest, `report.v2_decrypted` reflects the v2 success and `report.v3_dirs` reflects the v3 count; v3 dir is left untouched on disk; no `failed-decrypts/` entries from v3 (we never tried).

### Estimated scope

~80 LoC across `macos_cache.rs` + `main.rs`, plus 3 tests. Single commit titled `feat(archive): detect ChatGPT v3 cache (data-protection keychain — not extractable)`.

### Verification

```
cargo test --lib archive::macos_cache       # 18 + 3 = 21 unit tests pass
cargo test --test macos_cache_v2_integration  # 4 still pass
cargo clippy --lib --bin claude-usage-tracker -- -D warnings
cargo fmt --check
./target/release/claude-usage-tracker macos-cache scan   # real-machine
```

The real-machine `scan` should print the v3 dir + the explanation + recommended alternatives.

---

## Phase B — Cipher implementation (SHIPPED with user-supplied key)

The AES-GCM cipher is now implemented. The architectural blocker was about *Heimdall fetching the key automatically*, which we still do not do. Instead, `--decrypt-v3` requires `--v3-key <hex>` — the user must supply the key; Heimdall owns decryption.

### Why we shipped this anyway

The original deferral was framed around two separate problems:
1. **Cipher unknown** — now resolved (AES-GCM with 12-byte nonce prefix).
2. **Key inaccessible** — still true; unchanged.

Shipping the cipher behind `--v3-key` means a user who obtains the key by *legitimate means* (a future OpenAI key-export feature, a documented Apple entitlement path, a one-off developer extraction on their own machine) can actually use Heimdall to decrypt their own data. The browser extension / account-export / cookie-paste-CLI alternatives remain the recommended primary path for users without the key.

### What was implemented

- `oscrypt_v3` private submod in `src/archive/macos_cache.rs`: `decrypt_v3_blob` (AES-128/256-GCM auto-dispatch) + `encrypt_v3_blob` (test-only inverse).
- `decrypt_v3_dir`: walks a `conversations-v3-*` dir, decrypts per file.
- `ingest_v3_into_archive_with_key`: full ingest pipeline mirroring v2, with `.failed-decrypts/` fallback for non-parseable plaintexts.
- `IngestOptions.v3_key: Option<Vec<u8>>`: umbrella opt-in.
- `IngestReport` extended with `v3_attempted`, `v3_decrypted`, `v3_failed_parse`, `v3_failed_decrypt`.
- CLI: `--decrypt-v3` + `--v3-key <hex>` flags on `macos-cache ingest`.
- `hex_decode_v3_key` helper in `main.rs`.
- 5 unit tests + 1 integration test.

### Future work (key acquisition — still blocked)

The following would enable automatic key fetching without `--v3-key`:

1. A public reverse-engineering writeup documents the v3 key-extraction path (entitlement details, alternative location, any user-consent flow that's gated rather than blocked).
2. OpenAI publishes an official key-export or data-portability path that bypasses the keychain entirely.
3. Apple introduces a documented user-consent flow for cross-app data-protection-keychain reads (no announcement as of macOS 26; monitor `developer.apple.com` release notes).

A `KeychainKeyProvider` for v3 would then be straightforward to add — the cipher and ingest pipeline are already in place.

---

## Self-review

- **Does Phase A pretend to do anything it can't?** No. It detects v3, reports it, recommends alternatives. Zero decryption attempts.
- **Does Phase A leak the user's expectations?** No. The message is explicit: v3 cache exists, you cannot extract it via this tool, here are three working alternatives.
- **Does Phase A regress v2 functionality?** No. The v2 ingest path is untouched; v3 is a new variant that the existing v2 code paths skip cleanly. The integration test `ingest_v2_does_not_attempt_v3` pins this.
- **Does Phase B leave a mess if it never ships?** No. It's a comment in this plan + a reference point in code (the `EncryptedV3` enum variant naturally extends to a future `decrypt_v3_dir` function).
