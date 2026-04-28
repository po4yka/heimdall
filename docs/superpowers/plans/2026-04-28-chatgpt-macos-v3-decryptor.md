# ChatGPT macOS v3 Cache — Findings + Plan

> **For agentic workers:** the implementation portion of this plan
> (Phase A only) uses superpowers:subagent-driven-development. Phase B
> is deferred indefinitely pending external public reverse engineering;
> do not implement it speculatively.

## TL;DR

ChatGPT for macOS shipped a **v3** conversation cache format
(`conversations-v3-{uuid}/`) sometime between July 2024 and early 2026.
The v2 decryptor we just shipped does **not** apply to v3 because:

1. **Cipher changed**: v3 is almost certainly AES-GCM (or another AEAD),
   not the Chromium OSCrypt AES-128-CBC stack used by v2.
2. **Key location changed**: the v3 key is in the macOS **data-protection
   keychain**, not the legacy login keychain. Apple's data-protection
   keychain is **partitioned by app entitlements / Team ID**, and a
   third-party CLI without OpenAI's entitlement gets `errSecItemNotFound`
   — indistinguishable from "doesn't exist" — for items it cannot read.
   No `Allow / Deny` user prompt is offered because this is not a
   user-mediated ACL boundary; it is an architectural one.

So unlike v2, where the user-mediated Keychain ACL prompt was the
boundary and we accepted it, **v3 has no documented user-consent path
that grants a third-party CLI access**. The cipher is implementable; the
key is not retrievable. Without the key, the cipher is useless.

This plan therefore has only one *implementable* phase (Phase A —
honest detection + UX). Phase B (cipher implementation) is documented
for future-readiness but **not slated for implementation** until a
public RE writeup appears that solves the key-access problem.

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
- File sizes are **not multiples of 16** (one example: `23868 mod 16 = 12`).
  → Rules out plain AES-CBC. Consistent with AES-GCM
  (`plaintext_len + 16-byte tag`) or another AEAD with similar tag.
- First 12 bytes are **distinct per file** across 5 samples I checked.
  → Consistent with per-file 96-bit AES-GCM nonces.
- No `v10` ASCII prefix (the v2 / Chromium-OSCrypt magic). v3 is binary
  from byte 0.

Keychain probe:

- `security find-generic-password -s "com.openai.chat.conversations_v3_cache"`
  → "item could not be found".
- `security dump-keychain` reveals only the OpenAI Bearer API token
  (`https://api.openai.com/v1`). Zero entries with "conversations" in
  service or account.
- This confirms the v3 key is **not in the legacy file-based keychain**.

## Public-source confirmation

- Zero open-source projects, blog posts, or academic papers document the
  v3 format as of April 2026.
- Pedro Vieito's July 2024 disclosure (`pvieito.com`) covers only v1→v2.
  No v3 follow-up.
- Apple's own DTS engineer ("eskimo") confirms data-protection keychain
  semantics on developer.apple.com:
  > "The data protection keychain is only available to code that can
  > carry an entitlement, that is, main executables like an app or an
  > app extension."
- `SecItemCopyMatching` does not surface metadata for items outside the
  caller's access groups — `errSecItemNotFound` is indistinguishable
  between "absent" and "present-but-inaccessible".

## What we cannot do, and why we won't try

Hypothetical paths to the v3 key, all out of scope per Heimdall's
operational ground rules:

- **Code-sign as ChatGPT.app**: requires faking OpenAI's Team ID
  (`2DC432JVRJ`). Forbidden.
- **Inject into the running ChatGPT process**: requires defeating
  hardened runtime + amfid + SIP. Forbidden.
- **Dump SEP-protected memory**: requires kernel-level access. Forbidden.
- **`chainbreaker`-style direct DB parsing**: doesn't yet handle the
  data-protection keychain SQLite shape; even if it did, the per-row
  AES-256-GCM table key wraps each item and the access-group enforcement
  is at the framework level.

The honest user-facing answer for v3 is: *use the alternatives we
already shipped*:

- **Phase 3b browser extension** — captures conversations from
  `chatgpt.com` directly. No desktop cache needed.
- **Phase 2 account-export importer** — full account export ZIP, slow
  but complete, including archived conversations.
- **Phase 3a cookie-paste CLI** — `heimdall scrape chatgpt …` for
  headless setups.

---

## Phase A — Honest v3 detection (implementable now)

**Goal:** make `heimdall macos-cache scan` and `ingest` surface v3
directories clearly instead of silently ignoring them, with a pointed
recommendation to use one of the existing alternatives.

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

~80 LoC across `macos_cache.rs` + `main.rs`, plus 3 tests. Single
commit titled `feat(archive): detect ChatGPT v3 cache (data-protection
keychain — not extractable)`.

### Verification

```
cargo test --lib archive::macos_cache       # 18 + 3 = 21 unit tests pass
cargo test --test macos_cache_v2_integration  # 4 still pass
cargo clippy --lib --bin claude-usage-tracker -- -D warnings
cargo fmt --check
./target/release/claude-usage-tracker macos-cache scan   # real-machine
```

The real-machine `scan` should print the v3 dir + the explanation +
recommended alternatives.

---

## Phase B — Cipher implementation (DEFERRED, do not start)

Documented here only so future readers understand what would need to
become true before Phase B is implementable, and so the eventual
implementation is shorter when it ships.

### Trigger conditions (any one suffices)

1. A public reverse-engineering writeup appears that documents the v3
   key-extraction path (entitlement details, alternative location, any
   user-consent flow that's gated rather than blocked).
2. OpenAI publishes an official key-export or data-portability path that
   bypasses the keychain entirely.
3. Apple introduces a documented user-consent flow for cross-app
   data-protection-keychain reads (no announcement as of macOS 26;
   monitor `developer.apple.com` release notes).

### Implementation sketch (when Phase B is unblocked)

- Cipher: assume AES-256-GCM with `[12-byte nonce][ciphertext][16-byte
  tag]` framing. Verified by file-size-mod-16 = 12 evidence and distinct
  per-file first-12-bytes evidence. If 256 doesn't validate, fall back
  to 128 (Electron OSCrypt's other common size).
- Reuse the existing `KeyProvider` trait. Add a v3 variant that
  fetches from whichever access path Phase B's research identifies.
- New `oscrypt_v3` private submod inside `macos_cache.rs` with
  `decrypt_v3_blob(bytes: &[u8], key: &[u8]) -> Result<Vec<u8>>`.
- Plaintext schema sniff via the existing `imports::openai::parse_conversations`
  (same as v2). Failures land in `.failed-decrypts/` (same primitive).
- Add `--decrypt-v3` CLI flag; surface v3 ingest counts in `IngestReport`.

### Rough effort estimate (when unblocked)

~250 LoC + 4 tests + integration test, mirroring v2's shape. ~3 hours
implementation once research closes the key-access question.

---

## Self-review

- **Does Phase A pretend to do anything it can't?** No. It detects v3,
  reports it, recommends alternatives. Zero decryption attempts.
- **Does Phase A leak the user's expectations?** No. The message is
  explicit: v3 cache exists, you cannot extract it via this tool, here
  are three working alternatives.
- **Does Phase A regress v2 functionality?** No. The v2 ingest path is
  untouched; v3 is a new variant that the existing v2 code paths skip
  cleanly. The integration test `ingest_v2_does_not_attempt_v3` pins
  this.
- **Does Phase B leave a mess if it never ships?** No. It's a comment
  in this plan + a reference point in code (the `EncryptedV3` enum
  variant naturally extends to a future `decrypt_v3_dir` function).
