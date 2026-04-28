# Heimdall — local chat backup design

Status: draft, for review
Date: 2026-04-28
Owner: Nikita Pochaev

## 1. Goal

Give the user a complete, daily-cadence local copy of every conversation they
have with Anthropic Claude and OpenAI ChatGPT, across:

- CLI tools that already write to disk (Claude Code, Codex, Cursor, OpenCode,
  Pi, Copilot, Xcode CodingAssistant, Cowork, Amp);
- claude.ai web app and Claude Desktop;
- chatgpt.com web app and ChatGPT Desktop;

so that account loss, machine wipe, vendor outage, vendor account suspension,
or vendor-side deletion does not destroy the user's conversation history.
Archived conversations are in scope.

## 2. Non-goals

- Multi-user / multi-tenant backup. This is a single-user local tool.
- Backing up *other people's* accounts. The user must be the account holder.
- Bypassing vendor anti-bot defenses against the vendor's wishes. Tier 3 lives
  in the user's own logged-in browser; we do not fight Cloudflare from a
  headless client.
- Cloud backup, off-device sync, or shared archive servers. Storage is local
  only. (Users may sync `~/.heimdall/archive/` themselves via Time Machine,
  rsync, etc. — out of scope here.)
- Re-deriving statistics from the archive. Heimdall's existing scanner already
  does observability; this feature is about preservation, not analytics.

## 3. Three-tier architecture

The backup surfaces split cleanly into three tiers with very different
properties:

| Tier | Source | Cadence | Brittleness | Coverage |
|------|--------|---------|-------------|----------|
| 1. CLI snapshots | Local files written by CLI tools | Daily auto via scheduler | None — pure file copy | Full and authoritative |
| 2. Account-export importer | Vendor-issued ZIP exports (claude.ai, chatgpt.com) | When the user requests an export (~weeks) | Schema drift only | Full *at export time*, including archived |
| 3. Web-chat live capture | Browser extension reading the user's logged-in tab; or copy-paste cookie CLI | Daily / on tab open | Moderate — vendor schema or route changes | Full at capture time |

Tier 1 is the bulk of conversation volume for technical users and is also the
easiest tier. Tier 2 is the safety net that backstops everything (it is the
only tier that survives "vendor deletes my account"). Tier 3 closes the daily
gap between exports for web/desktop chats.

## 4. Storage layout

A single archive root under `$XDG_DATA_HOME/heimdall/archive/` (default
`~/.heimdall/archive/`). Path is configurable via `[archive] root = ...` in
the existing TOML config and overridable per-invocation with `--archive-root`.

```
~/.heimdall/archive/
  objects/                       # content-addressed store (SHA-256)
    sha256/ab/cdef...             # immutable, deduped across snapshots
  snapshots/                     # tier 1
    2026-04-28T08-00-00Z/
      manifest.json              # {provider -> [{logical_path, sha256, size, mtime}]}
      summary.json               # totals, providers seen, source file counts
  exports/                       # tier 2
    anthropic/2026-04-15/
      original.zip                # untouched vendor ZIP
      conversations/<conv_id>.json   # split per conversation
      metadata.json               # parser version, schema seen, parse warnings
    openai/2026-04-12/
      original.zip
      conversations.json          # untouched from ZIP
      conversations/<conv_id>.json
      metadata.json
  web/                           # tier 3
    claude-ai/<conv_id>.json     # latest captured version
    claude-ai/<conv_id>.history/<timestamp>.json   # prior versions on update
    chatgpt-com/<conv_id>.json
    chatgpt-com/<conv_id>.history/...
  index.sqlite                   # cross-tier index for dashboard queries
  archive.lock                   # process lock (advisory) for snapshot/import
```

Design notes:

- **Content addressing for tier 1.** Files in `~/.claude/projects/` and
  similar grow append-only over a session and only flip when the session
  ends. Daily snapshots that hash file content into `objects/` and reference
  hashes from `manifest.json` make near-zero-delta days nearly free on disk
  and trivially deduped.
- **Original ZIP is preserved verbatim** for Tier 2. The split-per-conversation
  JSON files are derived data we can re-derive if our parser improves.
- **Tier 3 versioning.** Conversations on claude.ai/chatgpt.com are mutable —
  rerunning a prompt rewrites the message tree. We keep the *current* state
  at `<conv_id>.json` and any prior state in `<conv_id>.history/<ts>.json`.
- **Index is derivative, not authoritative.** `index.sqlite` is rebuildable
  from the on-disk tree. Do not store anything in it that is not also in the
  files. This guarantees a future Heimdall version can recover from a corrupt
  index, and that users who copy `archive/` to another machine retain
  everything.

## 5. Tier 1 — CLI snapshots

### Components

- New module `src/archive/mod.rs` with `snapshot()`, `restore()`, `prune()`,
  `verify()` entry points.
- `src/archive/objects.rs` — content-addressed object store (write, read,
  exists, gc).
- `src/archive/manifest.rs` — manifest schema and serde types.
- `src/archive/sources.rs` — discovers source roots by reusing the existing
  `Provider` trait. Each provider exposes a new `archive_paths(&self) ->
  Vec<PathBuf>` method (default empty for providers with no on-disk
  representation).

### Data flow

```
heimdall archive snapshot
  -> for each Provider in providers::all():
       for each path in provider.archive_paths():
         walkdir(path), filter to provider-relevant files
         hash each file -> write to objects/sha256/...
         record (logical_path, sha256, size, mtime) in manifest entries
  -> write snapshots/<ts>/manifest.json + summary.json
  -> update index.sqlite
  -> emit event for dashboard SSE / menubar refresh
```

Restore writes the file tree under `--to <dir>` (defaulting to a fresh
`./heimdall-restore-<ts>/`); we never overwrite the original `~/.claude/...`
locations.

`heimdall archive verify` does two jobs: (a) checksums every object against
its filename to detect on-disk corruption, and (b) rebuilds `index.sqlite`
from the on-disk manifests, so a corrupted or missing index can always be
recovered without re-running snapshots.

### Error handling

- A file that disappears mid-walk → log `warn!`, skip, continue.
- A hash collision against the object store → if content differs, panic in
  debug, log error in release (effectively impossible for SHA-256, but the
  guard is cheap).
- A snapshot interrupted partway → next run retries cleanly because objects
  are content-addressed and the manifest is the only stateful artifact;
  an in-flight snapshot writes to `snapshots/<ts>/.partial/` and renames
  on success. Objects written by an aborted snapshot are *not* removed
  immediately — they sit unreferenced in `objects/` and are reclaimed by
  the next `archive prune` (which GCs anything not referenced by a
  surviving manifest).

### Testing

- `tempfile` per-test, fixture provider trees of 1–10 files, round-trip
  snapshot → restore → byte-equality assertion.
- Property test: snapshot then re-snapshot identical state produces the same
  manifest hashes (idempotent).
- Prune-keep-N test: only the most recent N snapshots remain; objects
  unreferenced after prune are GC'd.

## 6. Tier 2 — Account-export importer

### Components

- `src/archive/imports/mod.rs` — entry point and watch-folder loop.
- `src/archive/imports/openai.rs` — parser for `chatgpt-export-*.zip`.
- `src/archive/imports/anthropic.rs` — parser for `claude-export-*.zip`.

### Data flow

```
heimdall import-export <zip>             # one-off
heimdall import-export --watch [<dir>]   # daemon, default ~/Downloads
  -> detect vendor by zip content (presence of conversations.json => OpenAI;
     vendor-specific top-level filenames otherwise)
  -> validate signatures we know:
       OpenAI: conversations.json with mapping/messages tree
       Anthropic: schema not publicly documented, parse defensively
  -> copy original zip to exports/<vendor>/<date>/original.zip
  -> extract + split per-conversation -> exports/<vendor>/<date>/conversations/
  -> write metadata.json with: parser_version, schema_fingerprint,
     conversation_count, parse_warnings[]
  -> update index.sqlite
```

### OpenAI parser

The format is well-documented enough to write a typed parser:
`conversations.json` is an array; each entry has `id`, `title`,
`create_time`, `update_time`, `mapping: {<node_uuid>: {id, message, parent,
children}}`, `current_node`, `is_archived`, plus auxiliary fields. We model
it with `serde` and `#[serde(default, flatten)] extra: HashMap<String, Value>`
for forward compatibility.

### Anthropic parser

The export schema is **not publicly documented** as of the research date.
We parse defensively:

1. Probe for top-level files; record what we find.
2. Walk JSON values, treat `Value` as the source of truth for unknown fields.
3. Extract a normalized form `(conversation_id, title, created_at, updated_at,
   messages[])` using best-effort field name mapping with a drop-in
   override table. Fields that don't map are preserved verbatim under
   `extras` in the per-conversation JSON.
4. `metadata.json` records the schema fingerprint we encountered so future
   Heimdall versions can offer to re-parse old imports under a newer parser.

A failed parse never destroys data — we keep `original.zip` verbatim and
write `parse-errors.json` next to the (possibly empty) extracted directory.

### Testing

- Golden-file fixtures from real OpenAI exports (small, redacted).
- Parameterized fixtures for synthetic Anthropic exports under several
  hypothesized schemas.
- "Unknown ZIP" test: a ZIP that doesn't match either signature — must not
  crash, must surface a clear error.

## 7. Tier 3 — Web-chat live capture

Two cooperating implementations covering the same surface:

### 7a. Browser extension companion (primary path)

- Separate sibling project at `extensions/heimdall-companion/`.
- Tooling: TypeScript + WebExtension Manifest v3, esbuild bundle, Vitest +
  happy-dom for tests.
- Targets: Chrome, Firefox, Safari (single codebase via webextension-polyfill).
- Permissions: `storage`, `activeTab`, host permissions for `*://claude.ai/*`
  and `*://chatgpt.com/*`. No remote-code execution; no third-party network
  calls; only POSTs to the user's local Heimdall.
- Content script reuses the user's logged-in session by issuing in-page
  fetches to the same private endpoints the page itself uses
  (`/api/organizations/.../chat_conversations` and
  `/backend-api/conversations`). This sidesteps Cloudflare entirely because
  the request originates from the live tab.
- Sync strategy: on tab focus and on a per-site interval (default 6 h, user
  configurable), enumerate conversation IDs, diff against
  `lastSeenUpdatedAt` per conversation, fetch only changed conversations,
  POST each to Heimdall.
- Options page: pair with Heimdall (paste the per-install bearer secret),
  per-site enable/disable, capture interval, manual "Sync now" button,
  status log.
- The extension never stores conversations itself — it forwards to Heimdall
  and forgets, so the extension's own quota is bounded.

### 7b. Cookie-paste CLI fallback

For headless installs and "the extension just broke and I need a copy
*now*" cases:

```
heimdall scrape claude  --session-key <sk-ant-sid01...> [--once|--daemon]
heimdall scrape chatgpt --session-token <__Secure-next-auth.session-token>
                        --access-token <bearer> [--once|--daemon]
```

Implementation lives in `src/scrape/{claude,chatgpt}.rs` using the existing
`reqwest` client. We do **not** bundle a browser engine, do **not** try to
solve Cloudflare challenges, and require the user to copy fresh cookies
from DevTools when the previous one expires. This path is documented as
brittle and explicitly secondary; the primary recommendation is the
extension.

### Heimdall ingest endpoint

A new endpoint accepts captures from either path:

```
POST /api/archive/web-conversation
Authorization: Bearer <companion-token>
Content-Type: application/json
{
  "vendor": "claude.ai" | "chatgpt.com",
  "conversation_id": "<vendor id>",
  "captured_at": "<rfc3339>",
  "schema_fingerprint": "<sha256 of keys-sorted JSON>",
  "payload": { ... vendor JSON verbatim ... }
}
```

Server side: validate bearer, write to `web/<vendor>/<conv_id>.json`,
move previous content (if any and different) to
`web/<vendor>/<conv_id>.history/<prev_captured_at>.json`, update
`index.sqlite`, return `204 No Content`.

### Error handling

- Auth failure → 401, structured response so the extension can prompt for
  re-pairing.
- Body too large (> 50 MB) → 413, suggest the user open an issue.
- Conflict detection: if `payload` is byte-identical to current stored
  state, return 200 OK with `{"unchanged": true}` and skip history rotation.

### Testing

- Rust side: handler unit tests with mocked archive root and bearer.
- Extension side: Vitest tests with fixture HTML/fetch responses simulating
  claude.ai's `chat_conversations` index and a single-conversation payload.
- End-to-end: a smoke test with the dev dashboard up, the extension loaded
  unpacked, and a recorded HAR replaying the vendor responses.

## 8. Surfaces

### CLI

Add a single `archive` subcommand group plus two convenience subcommands:

```
heimdall archive snapshot                # tier 1
heimdall archive list
heimdall archive show <snapshot>
heimdall archive restore <snapshot> [--to <dir>]
heimdall archive prune --keep-last 30
heimdall archive verify

heimdall import-export <zip>             # tier 2
heimdall import-export --watch [<dir>]

heimdall scrape claude  ...              # tier 3 fallback
heimdall scrape chatgpt ...

heimdall scheduler install --include-archive    # extends existing scheduler
                                                # to run snapshot daily
```

Existing flags (`--json`, timezone) supported where applicable.

### Web dashboard

A new top-level "Backup" tab in the existing Preact UI. Three panels:

1. **Snapshots** — last snapshot timestamp, total archive size, file count
   per provider, "Snapshot now" button (POSTs `/api/archive/snapshot`),
   table of recent snapshots with size and source counts.
2. **Imports** — drag-drop ZIP area (uploads to `POST /api/archive/import`),
   list of imports with vendor/date/conversation count, link out to vendor
   "request export" settings page.
3. **Web captures** — companion extension status (heartbeat from
   `POST /api/archive/companion-heartbeat`, last seen N minutes ago),
   captured-conversations counts per vendor, manual "open extension store"
   link, and a fold-out cookie-paste form for the CLI fallback.

No new dashboard chart types; reuse `DataTable`, status pills, and the
existing inline `[STATUS]` helper.

### SwiftBar menubar plugin

Existing `heimdall menubar` already emits SwiftBar-formatted markdown.
Extend its output (in `src/menubar.rs`) with a new section. SwiftBar's
plugin protocol natively supports submenus, click-to-shell, refresh
intervals, status icons, and parameter passing — no second native app
needed:

```
[H] Heimdall · backed up 2h ago · 4.2 GB
---
Snapshots
  Last: 2026-04-28 08:00 UTC (4.2 GB)
  Snapshot now | bash=heimdall param0=archive param1=snapshot terminal=false
  Open archive folder | bash=open param0=~/.heimdall/archive
---
Web chats
  Browser extension: connected (Claude + ChatGPT)
  Last capture: 14m ago
  Open dashboard | href=http://localhost:9999/#/backup
---
Imports
  Watching: ~/Downloads
  Last import: 2026-04-15 (Anthropic)
```

The output is sanitized through the existing menubar XSS hardening
(`menubar.rs` already has injection-resistant escape helpers).

## 9. Authentication

Heimdall's dashboard binds `127.0.0.1` only and is currently unauthenticated.
The new `POST /api/archive/*` endpoints accept untrusted-on-disk data and
must not be hit by random localhost software, so they require a bearer:

- On first run after upgrade, generate a 256-bit random token and store at
  `~/.heimdall/companion-token` (mode 0600).
- The token is also surfaced in the dashboard "Backup" tab behind a
  click-to-reveal, and printed by `heimdall companion-token show`.
- The browser extension's options page accepts the token; both sides keep
  it locally only.
- Read-only endpoints (`GET /api/archive`, `GET /api/data`, etc.) remain
  unauthenticated (status quo) — only the *write* endpoints gate on the
  bearer.

Token rotation: `heimdall companion-token rotate` writes a new value;
the extension shows a "re-pair required" banner on first 401.

## 10. Phased rollout

The work splits cleanly by tier; ship in this order so users get value
at each step.

- **Phase 1: Tier 1 + scheduler integration + dashboard panel + menubar.**
  Ship the CLI archive subcommands and the snapshot panel. Most of the
  user's daily conversation volume gets backed up immediately.
- **Phase 2: Tier 2 importer.** Both vendors. Adds long-tail safety net
  for the user's web chats — slow-cadence but complete.
- **Phase 3a: Tier 3 cookie-paste CLI fallback.** Ship the brittle
  pure-Rust path first because it requires no second project. Validates
  the ingest endpoint and the storage layout against real claude.ai /
  chatgpt.com payloads.
- **Phase 3b: Tier 3 browser extension.** Final piece — the comfortable
  daily UX. Ships as a sibling project under `extensions/heimdall-companion/`
  with its own README, package.json, and release pipeline.

Each phase is an independent shippable increment and gets its own
implementation plan when its turn comes.

## 11. Risks and explicit acknowledgements

- **Tier 3 ToS posture.** The cookie/extension path uses private,
  undocumented vendor endpoints. The user is the rights-holder of their own
  account data, but neither vendor has a public carve-out for unofficial
  access. The dashboard "Backup" tab and the extension options page show
  a one-line disclaimer to that effect. We never share or upload the
  user's session credentials.
- **Anthropic export schema is undocumented.** The Tier 2 Anthropic parser
  is best-effort and may need updates as Anthropic publishes more or as
  we observe real exports. Defensive parsing prevents data loss; users
  always retain `original.zip`.
- **Disk usage.** Daily snapshots of mostly-stable JSONL trees deduplicate
  well via content addressing, but a user with 50 GB of project sessions
  will see real disk growth. Default `prune --keep-last 30` runs after
  each snapshot when scheduler is `--include-archive`.
- **Concurrent runs.** The `archive.lock` file prevents two `snapshot`
  invocations from racing. Dashboard "Snapshot now" returns 409 if a
  snapshot is in flight.

## 12. Out of scope (revisit later)

- ChatGPT macOS desktop encrypted Keychain cache reader
  (`com.openai.chat.conversations_v2_cache`). Format isn't publicly
  documented; the browser-extension path already covers ChatGPT cleanly.
- Claude Desktop Electron LevelDB / IndexedDB cache mining. Anthropic's
  desktop app does not appear to keep authoritative conversation cache
  on disk (server-state model); browser-extension is the better route.
- Cross-machine archive sync. Users can sync `~/.heimdall/archive/`
  themselves; we don't implement Resilio/iCloud/etc. integration here.
- Conversation diff/restore *into* the vendor account (one-way out only).
- Encryption-at-rest for the archive directory. Out of scope; users with
  this requirement use FileVault.

## 13. Open questions for review

1. **Default archive root.** I'm proposing `~/.heimdall/archive/` to
   parallel `~/.claude/`. Alternatives: `~/Library/Application Support/
   Heimdall/archive/` (macOS-conventional) or `$XDG_DATA_HOME/heimdall/
   archive/` (XDG-conventional). Let me know which feels right.
2. **Daily snapshot time.** Default proposal: 03:30 local time, off
   the existing scheduler. OK or should it follow the user's existing
   `scan` schedule?
3. **Browser extension distribution.** Chrome Web Store + Firefox AMO +
   Safari TestFlight, or unpacked/sideload only for v1? Store
   distribution adds review-cycle friction but legitimizes the extension.
4. **`heimdall scrape` retention without the archive.** If a user runs
   the cookie CLI without setting up `archive snapshot` first, do we
   create the archive root lazily, or require explicit `archive init`?
5. **Anthropic data-export schema.** I will fetch a real export from your
   own account during implementation to lock in the parser. OK to use
   your account as the schema reference?
