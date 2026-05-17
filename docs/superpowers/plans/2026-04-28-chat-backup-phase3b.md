# Chat-backup Phase 3b — Browser-Extension Companion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship the browser-extension companion (spec §7a) — a Manifest V3 WebExtension that runs in the user's logged-in claude.ai / chatgpt.com tab, issues in-page fetches against the same private endpoints the page itself uses (sidestepping Cloudflare entirely), and POSTs each captured conversation to Heimdall's `POST /api/archive/web-conversation` endpoint (already shipped in Phase 3a). Plus the dashboard "Web captures" panel that surfaces the resulting `<archive_root>/web/` tree.

**Architecture:** New sibling project at `extensions/heimdall-companion/` (TypeScript + esbuild + Vitest, mirroring the dashboard tooling). Single source tree builds Chrome and Firefox bundles via webextension-polyfill. The extension talks ONLY to localhost; no third-party network. Heimdall gains a `GET /api/archive/web-conversations` listing endpoint plus a companion-heartbeat endpoint so the dashboard can show "extension last seen N min ago". The Phase 1 dashboard "Backup" tab grows a fourth panel ("Web captures") below Snapshots and Imports.

**Tech Stack:** TypeScript 5, esbuild, `webextension-polyfill`, Vitest + happy-dom, Manifest V3 service worker pattern. Rust side reuses Phase 3a's `WebConversation`/`WriteOutcome` and adds two listing endpoints.

Spec reference: `docs/superpowers/specs/2026-04-28-chat-backup-design.md` §7a, §8, §9, and Phase 3b of §10.

---

## Non-goals for 3b

- No store distribution (Chrome Web Store / Firefox AMO / Safari TestFlight) — v1 is sideload/unpacked only. Store packaging is a follow-up.
- No automatic refresh of the user's session on the vendor side — the extension piggybacks on whatever session the live tab already has.
- No conversation deletion / merging — every capture is append-only via the existing storage primitive (history rotation handled by Phase 3a).
- No "drag-drop ZIP" feature in the Web captures panel — that lives in the existing Imports panel from Phase 2.

---

## File Structure

**Created (Rust side):**

| File | Responsibility |
|------|----------------|
| `tests/web_captures_listing_integration.rs` | E2E for `list_web_conversations` and the new heartbeat endpoint. |

**Created (extension, all under `extensions/heimdall-companion/`):**

| File | Responsibility |
|------|----------------|
| `package.json` | Project metadata, deps, scripts. |
| `tsconfig.json` | TS config — strict, ES2022 target, `--moduleResolution bundler`. |
| `vitest.config.ts` | Vitest setup. |
| `manifest.json` | WebExtension MV3 manifest (Chrome) — `manifest_version`, permissions, host permissions, background service worker, content scripts, options page, action popup. |
| `manifest.firefox.json` | Firefox-specific override (background `scripts` array + `browser_specific_settings`). The build script merges with the base manifest. |
| `src/background.ts` | Service-worker entry. Wires alarms (periodic sync), message bus, options/popup IPC. |
| `src/sync.ts` | Per-vendor sync orchestrator: enumerate IDs, diff against `lastSeenUpdatedAt`, fetch changed, POST to Heimdall. |
| `src/vendors/claude.ts` | claude.ai content-side fetcher (talks to `/api/organizations/.../chat_conversations`). |
| `src/vendors/chatgpt.ts` | chatgpt.com content-side fetcher (talks to `/backend-api/conversations`). |
| `src/heimdall.ts` | Posts captures to local Heimdall (`POST /api/archive/web-conversation`) with bearer + heartbeat. |
| `src/storage.ts` | `chrome.storage.local` wrapper for paired token, sync interval, lastSeen state, telemetry counters. |
| `src/options/options.html` | Options page shell. |
| `src/options/options.ts` | Pair token, change interval, Sync now button, status log. |
| `src/popup/popup.html` | Toolbar popup shell. |
| `src/popup/popup.ts` | Compact status: paired? last sync? counts. |
| `build.mjs` | esbuild script: bundles background, content scripts, options, popup; emits `dist/chrome/` and `dist/firefox/`. |
| `tests/sync.test.ts` | Vitest: sync.ts diff-and-fetch logic against fixture HAR. |
| `tests/storage.test.ts` | Vitest: storage helpers. |
| `tests/heimdall.test.ts` | Vitest: bearer header + heartbeat on retry. |
| `README.md` | Install instructions (sideload), pair-with-Heimdall steps. |

**Created (dashboard side, Rust + TS):**

| File | Responsibility |
|------|----------------|
| `src/ui/components/WebCapturesPanel.tsx` | "Web captures" panel inside Backup tab. |
| `src/ui/components/WebCapturesPanel.test.tsx` | Vitest. |

**Modified:**

| File | Change |
|------|--------|
| `src/archive/web/mod.rs` | Add `last_heartbeat_path()` + `read_heartbeat()` + `record_heartbeat()` helpers. |
| `src/server/api.rs` | Add `api_archive_web_conversations` (GET listing) + `api_archive_companion_heartbeat` (POST). |
| `src/server/mod.rs` | Register two new routes. |
| `src/server/tests.rs` | Coverage for both. |
| `src/ui/state/store.ts` | `WebConversationSummary` type + `webConversations` signal + `companionHeartbeat` signal. |
| `src/ui/index.html` | New `<div id="web-captures-panel" class="bento-full">` mount inside the Backup tab area. |
| `src/ui/app.tsx` | Mount `WebCapturesPanel`; add `loadWebConversations()` + `loadCompanionHeartbeat()` initialisers. |
| `src/ui/app.js`, `src/ui/style.css` | Re-generated. |
| `.github/workflows/*.yml` (optional, follow-up) | Could add an extension build job — out of scope for 3b. |
| Top-level `.gitignore` | Add `extensions/heimdall-companion/{node_modules,dist}/`. |
| `README.md` (root) | One-line pointer to the extension's README. |

---

## Storage / wire shapes

### Heartbeat file

`<archive_root>/web/companion-heartbeat.json`:

```json
{
  "last_seen_at": "2026-04-28T13:42:01.123456Z",
  "extension_version": "0.1.0",
  "user_agent": "Mozilla/5.0 ...",
  "vendors_seen": ["claude.ai", "chatgpt.com"]
}
```

Updated on every successful POST. Listing endpoint serves it back so the dashboard can render "extension last seen N min ago".

### Extension `chrome.storage.local` schema

```json
{
  "version": 1,
  "heimdallUrl": "http://localhost:8080",
  "companionToken": "...64hex...",
  "syncIntervalMinutes": 360,
  "vendors": {
    "claude.ai":   { "enabled": true,  "lastSyncAt": "...", "lastSeenUpdatedAt": { "<conv_id>": "..." } },
    "chatgpt.com": { "enabled": true,  "lastSyncAt": "...", "lastSeenUpdatedAt": { "<conv_id>": "..." } }
  },
  "telemetry": { "totalCaptures": 0, "totalErrors": 0 }
}
```

### Sync protocol (per vendor)

1. Content-script fetch: list conversation IDs + `updated_at`.
2. Diff against `lastSeenUpdatedAt[conv_id]` from local storage.
3. For each changed (or never-seen) conv: fetch its body in-page, compute `schema_fingerprint = SHA256(sorted top-level keys)`, POST `WebConversation { vendor, conversation_id, captured_at, schema_fingerprint, payload }` to `<heimdallUrl>/api/archive/web-conversation` with `Authorization: Bearer <companionToken>`.
4. On success, update `lastSeenUpdatedAt[conv_id]`.
5. Send a heartbeat POST regardless of whether any captures fired.
6. Increment `telemetry.totalCaptures` (or `totalErrors`).

---

## Conventions

- TypeScript strict mode; explicit `any` is a lint error.
- esbuild + tsc are separate steps (tsc for type-checking only).
- Vitest for unit tests, happy-dom for DOM-flavoured tests.
- Conventional-commit prefixes: `feat(extension):`, `feat(server):`, `feat(ui):`, `feat(archive):`, `chore(extension):`.
- The extension never imports Heimdall Rust source — it only knows the HTTP wire shape (documented in the manifest reader's mind, not in shared types).

---

## Task 1: Heimdall server endpoints (listing + heartbeat)

**Files:**
- Modify: `src/archive/web/mod.rs` — add heartbeat helpers.
- Modify: `src/server/api.rs`, `src/server/mod.rs`, `src/server/tests.rs`.

- [ ] **Step 1: heartbeat helpers in archive/web/mod.rs**

```rust
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanionHeartbeat {
    pub last_seen_at: String,
    pub extension_version: Option<String>,
    pub user_agent: Option<String>,
    pub vendors_seen: Vec<String>,
}

pub fn heartbeat_path(archive_root: &Path) -> PathBuf {
    archive_root.join("web").join("companion-heartbeat.json")
}

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
    if extension_version.is_some() { h.extension_version = extension_version; }
    if user_agent.is_some() { h.user_agent = user_agent; }
    if !vendor.is_empty() && !h.vendors_seen.iter().any(|v| v == vendor) {
        h.vendors_seen.push(vendor.to_string());
    }
    let bytes = serde_json::to_vec_pretty(&h)?;
    fs::write(&path, bytes)?;
    Ok(())
}

pub fn read_heartbeat(archive_root: &Path) -> Result<Option<CompanionHeartbeat>> {
    let path = heartbeat_path(archive_root);
    if !path.is_file() { return Ok(None); }
    let bytes = fs::read(&path)?;
    Ok(serde_json::from_slice(&bytes).ok())
}
```

Add 2 unit tests: `record_heartbeat_creates_file` and `record_heartbeat_appends_unique_vendors`.

- [ ] **Step 2: HTTP handlers**

In `src/server/api.rs`:

```rust
pub async fn api_archive_web_conversations(
    State(_state): State<Arc<AppState>>,
    request: Request,
) -> Result<Json<Value>, StatusCode> {
    enforce_loopback_request(&request)?;
    let archive_root = crate::archive::default_root();
    let summaries = tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        crate::archive::web::list_web_conversations(&archive_root)
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let heartbeat = tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let archive_root = crate::archive::default_root();
        crate::archive::web::read_heartbeat(&archive_root)
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok()
    .flatten();
    let body = serde_json::json!({
        "conversations": summaries,
        "heartbeat": heartbeat,
    });
    Ok(Json(body))
}

#[derive(Debug, serde::Deserialize)]
pub struct CompanionHeartbeatBody {
    #[serde(default)] pub extension_version: Option<String>,
    #[serde(default)] pub user_agent: Option<String>,
    #[serde(default)] pub vendor: Option<String>,
}

pub async fn api_archive_companion_heartbeat(
    State(_state): State<Arc<AppState>>,
    request: Request,
) -> Result<Json<Value>, StatusCode> {
    enforce_loopback_request(&request)?;
    let header = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    let candidate = header.strip_prefix("Bearer ").unwrap_or("").to_string();

    let token_path = crate::archive::companion_token::default_path();
    let token = tokio::task::spawn_blocking(move ||
        crate::archive::companion_token::read_or_init(&token_path)
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if !token.matches(&candidate) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let body_bytes = axum::body::to_bytes(request.into_body(), 64 * 1024)
        .await
        .map_err(|_| StatusCode::PAYLOAD_TOO_LARGE)?;
    let body: CompanionHeartbeatBody = serde_json::from_slice(&body_bytes)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let archive_root = crate::archive::default_root();
    let vendor = body.vendor.unwrap_or_default();
    tokio::task::spawn_blocking(move || {
        crate::archive::web::record_heartbeat(
            &archive_root,
            body.extension_version,
            body.user_agent,
            &vendor,
        )
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({"ok": true})))
}
```

- [ ] **Step 3: Routes**

In `src/server/mod.rs::build_router` (alongside `/api/archive/web-conversation`):

```rust
        .route("/api/archive/web-conversations", get(api::api_archive_web_conversations))
        .route("/api/archive/companion-heartbeat", post(api::api_archive_companion_heartbeat))
```

- [ ] **Step 4: Server tests**

3 new tests in `src/server/tests.rs`, all using the existing `HOME_LOCK` mutex pattern from Phase 3a:

- `web_conversations_returns_empty_listing_when_no_captures` — GET `/api/archive/web-conversations` against a fresh tempdir → `{"conversations": [], "heartbeat": null}`.
- `companion_heartbeat_requires_bearer` — POST without bearer → 401.
- `companion_heartbeat_persists_to_disk` — POST with valid bearer + body → 200, file appears at `<HOME>/.heimdall/archive/web/companion-heartbeat.json`.

- [ ] **Step 5: Run + commit**

```
cargo test -p claude-usage-tracker --lib server::
cargo test -p claude-usage-tracker --lib archive::web
cargo clippy --lib -- -D warnings
git add src/archive/web/mod.rs src/server/api.rs src/server/mod.rs src/server/tests.rs
git commit -m "feat(server): web-conversations listing + companion-heartbeat endpoints"
```

---

## Task 2: Extension scaffold (package + tooling)

**Files (all created in `extensions/heimdall-companion/`):**

- `package.json`, `tsconfig.json`, `vitest.config.ts`, `build.mjs`, `manifest.json`, `manifest.firefox.json`, root `README.md`.
- Add `extensions/heimdall-companion/{node_modules,dist}/` to root `.gitignore`.

- [ ] **Step 1: package.json**

```json
{
  "name": "heimdall-companion",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "scripts": {
    "build": "node build.mjs",
    "build:chrome": "node build.mjs chrome",
    "build:firefox": "node build.mjs firefox",
    "typecheck": "tsc --noEmit",
    "test": "vitest run"
  },
  "devDependencies": {
    "@types/chrome": "^0.0.270",
    "@types/firefox-webext-browser": "^120.0.4",
    "@types/node": "^22.0.0",
    "esbuild": "^0.25.0",
    "happy-dom": "^15.0.0",
    "typescript": "^5.6.0",
    "vitest": "^2.1.0"
  },
  "dependencies": {
    "webextension-polyfill": "^0.12.0"
  }
}
```

- [ ] **Step 2: tsconfig.json**

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "noUncheckedIndexedAccess": true,
    "noImplicitAny": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "skipLibCheck": true,
    "resolveJsonModule": true,
    "lib": ["ES2022", "DOM"],
    "types": ["chrome", "node"]
  },
  "include": ["src/**/*", "tests/**/*"]
}
```

- [ ] **Step 3: vitest.config.ts**

```ts
import { defineConfig } from 'vitest/config';
export default defineConfig({
  test: {
    environment: 'happy-dom',
    globals: false,
    include: ['tests/**/*.test.ts'],
  },
});
```

- [ ] **Step 4: manifest.json (Chrome MV3)**

```json
{
  "manifest_version": 3,
  "name": "Heimdall Companion",
  "version": "0.1.0",
  "description": "Capture your claude.ai and chatgpt.com chat history into Heimdall.",
  "permissions": ["alarms", "storage", "activeTab"],
  "host_permissions": [
    "*://claude.ai/*",
    "*://chatgpt.com/*",
    "http://localhost/*",
    "http://127.0.0.1/*"
  ],
  "background": {
    "service_worker": "background.js",
    "type": "module"
  },
  "action": {
    "default_popup": "popup.html",
    "default_title": "Heimdall Companion"
  },
  "options_ui": {
    "page": "options.html",
    "open_in_tab": true
  }
}
```

- [ ] **Step 5: manifest.firefox.json (override)**

```json
{
  "background": {
    "scripts": ["background.js"]
  },
  "browser_specific_settings": {
    "gecko": {
      "id": "heimdall-companion@heimdall.dev",
      "strict_min_version": "115.0"
    }
  }
}
```

- [ ] **Step 6: build.mjs**

```js
import { build } from 'esbuild';
import { mkdir, writeFile, readFile } from 'node:fs/promises';
import { join, dirname } from 'node:path';

const target = process.argv[2] ?? 'chrome';
const out = `dist/${target}`;
await mkdir(out, { recursive: true });

const common = {
  bundle: true,
  format: 'esm',
  target: 'es2022',
  sourcemap: true,
  logLevel: 'info',
};

await build({
  ...common,
  entryPoints: ['src/background.ts', 'src/options/options.ts', 'src/popup/popup.ts'],
  outdir: out,
  entryNames: '[name]',
});

// Merge manifests for Firefox.
const base = JSON.parse(await readFile('manifest.json', 'utf8'));
let manifest = base;
if (target === 'firefox') {
  const fx = JSON.parse(await readFile('manifest.firefox.json', 'utf8'));
  manifest = deepMerge(base, fx);
  delete manifest.background.service_worker;
}
await writeFile(join(out, 'manifest.json'), JSON.stringify(manifest, null, 2));

for (const file of ['src/options/options.html', 'src/popup/popup.html']) {
  const dst = join(out, file.split('/').pop());
  await mkdir(dirname(dst), { recursive: true });
  await writeFile(dst, await readFile(file, 'utf8'));
}

function deepMerge(a, b) {
  if (a === null || typeof a !== 'object') return b;
  if (b === null || typeof b !== 'object') return a;
  const out = Array.isArray(a) ? [...a] : { ...a };
  for (const k of Object.keys(b)) out[k] = deepMerge(a[k], b[k]);
  return out;
}
```

- [ ] **Step 7: README + .gitignore + commit**

```
# .gitignore (append)
extensions/heimdall-companion/node_modules/
extensions/heimdall-companion/dist/
```

`extensions/heimdall-companion/README.md` documents:
- Sideload steps for Chrome (Developer mode → Load unpacked → `dist/chrome/`).
- Sideload for Firefox (about:debugging → Load Temporary Add-on → `dist/firefox/manifest.json`).
- Pair-with-Heimdall flow: run `heimdall companion-token show`, paste the hex into the extension's options page.
- Disclaimer: uses claude.ai / chatgpt.com private endpoints; you own your account data; your session credentials never leave your browser.

```
cd extensions/heimdall-companion && npm install
npm run typecheck
git add extensions/heimdall-companion/{package.json,package-lock.json,tsconfig.json,vitest.config.ts,build.mjs,manifest.json,manifest.firefox.json,README.md} .gitignore
git commit -m "chore(extension): scaffold heimdall-companion (TS + esbuild + Vitest)"
```

---

## Task 3: Extension storage + heimdall client + types

**Files:**
- Create: `extensions/heimdall-companion/src/storage.ts`
- Create: `extensions/heimdall-companion/src/heimdall.ts`
- Create: `extensions/heimdall-companion/src/types.ts`
- Create: `extensions/heimdall-companion/tests/storage.test.ts`
- Create: `extensions/heimdall-companion/tests/heimdall.test.ts`

- [ ] **Step 1: types.ts**

```ts
export interface VendorState {
  enabled: boolean;
  lastSyncAt: string | null;
  lastSeenUpdatedAt: Record<string, string>;
}

export interface ExtensionConfig {
  version: 1;
  heimdallUrl: string;
  companionToken: string | null;
  syncIntervalMinutes: number;
  vendors: Record<string, VendorState>;
  telemetry: { totalCaptures: number; totalErrors: number };
}

export interface WebConversation {
  vendor: string;
  conversation_id: string;
  captured_at: string;
  schema_fingerprint: string;
  payload: unknown;
}

export const DEFAULT_VENDORS = ['claude.ai', 'chatgpt.com'] as const;

export const DEFAULT_CONFIG: ExtensionConfig = {
  version: 1,
  heimdallUrl: 'http://localhost:8080',
  companionToken: null,
  syncIntervalMinutes: 360,
  vendors: Object.fromEntries(DEFAULT_VENDORS.map(v => [v, {
    enabled: true,
    lastSyncAt: null,
    lastSeenUpdatedAt: {},
  }])),
  telemetry: { totalCaptures: 0, totalErrors: 0 },
};
```

- [ ] **Step 2: storage.ts**

```ts
import type { ExtensionConfig, VendorState } from './types';
import { DEFAULT_CONFIG } from './types';

export async function loadConfig(): Promise<ExtensionConfig> {
  const stored = await chrome.storage.local.get(['config']);
  return mergeWithDefaults(stored.config);
}

export async function saveConfig(c: ExtensionConfig): Promise<void> {
  await chrome.storage.local.set({ config: c });
}

export function mergeWithDefaults(raw: unknown): ExtensionConfig {
  if (raw === null || typeof raw !== 'object') return structuredClone(DEFAULT_CONFIG);
  const obj = raw as Partial<ExtensionConfig>;
  const merged: ExtensionConfig = structuredClone(DEFAULT_CONFIG);
  if (typeof obj.heimdallUrl === 'string') merged.heimdallUrl = obj.heimdallUrl;
  if (typeof obj.companionToken === 'string') merged.companionToken = obj.companionToken;
  if (typeof obj.syncIntervalMinutes === 'number' && obj.syncIntervalMinutes > 0) {
    merged.syncIntervalMinutes = obj.syncIntervalMinutes;
  }
  if (obj.vendors && typeof obj.vendors === 'object') {
    for (const [vendor, state] of Object.entries(obj.vendors)) {
      const v = state as Partial<VendorState>;
      merged.vendors[vendor] = {
        enabled: v?.enabled ?? true,
        lastSyncAt: v?.lastSyncAt ?? null,
        lastSeenUpdatedAt: { ...(v?.lastSeenUpdatedAt ?? {}) },
      };
    }
  }
  if (obj.telemetry) {
    merged.telemetry = {
      totalCaptures: obj.telemetry.totalCaptures ?? 0,
      totalErrors: obj.telemetry.totalErrors ?? 0,
    };
  }
  return merged;
}
```

- [ ] **Step 3: heimdall.ts**

```ts
import type { ExtensionConfig, WebConversation } from './types';

export async function postConversation(
  cfg: ExtensionConfig,
  conv: WebConversation,
): Promise<{ saved: boolean; unchanged: boolean }> {
  if (!cfg.companionToken) throw new Error('companion token not paired');
  const url = `${cfg.heimdallUrl.replace(/\/$/, '')}/api/archive/web-conversation`;
  const resp = await fetch(url, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${cfg.companionToken}`,
    },
    body: JSON.stringify(conv),
  });
  if (resp.status === 401) throw new Error('401: companion token invalid (re-pair in options page)');
  if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
  const body = await resp.json() as { saved?: boolean; unchanged?: boolean };
  return {
    saved: body.saved === true,
    unchanged: body.unchanged === true,
  };
}

export async function postHeartbeat(
  cfg: ExtensionConfig,
  vendor: string,
): Promise<void> {
  if (!cfg.companionToken) return;
  const url = `${cfg.heimdallUrl.replace(/\/$/, '')}/api/archive/companion-heartbeat`;
  const body = {
    extension_version: chrome.runtime.getManifest().version,
    user_agent: navigator.userAgent,
    vendor,
  };
  await fetch(url, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${cfg.companionToken}`,
    },
    body: JSON.stringify(body),
  });
}
```

- [ ] **Step 4: storage.test.ts**

```ts
import { describe, expect, it } from 'vitest';
import { mergeWithDefaults } from '../src/storage';
import { DEFAULT_CONFIG } from '../src/types';

describe('mergeWithDefaults', () => {
  it('returns defaults for null', () => {
    expect(mergeWithDefaults(null)).toEqual(DEFAULT_CONFIG);
  });

  it('preserves stored token', () => {
    const merged = mergeWithDefaults({ companionToken: 'abc'.repeat(22) + 'ab' });
    expect(merged.companionToken).toBe('abc'.repeat(22) + 'ab');
  });

  it('rejects negative intervals', () => {
    const merged = mergeWithDefaults({ syncIntervalMinutes: -5 });
    expect(merged.syncIntervalMinutes).toBe(DEFAULT_CONFIG.syncIntervalMinutes);
  });

  it('preserves per-vendor lastSeen state', () => {
    const merged = mergeWithDefaults({
      vendors: { 'claude.ai': { enabled: false, lastSyncAt: '2026-01-01', lastSeenUpdatedAt: { c1: '2026' } } },
    });
    expect(merged.vendors['claude.ai'].enabled).toBe(false);
    expect(merged.vendors['claude.ai'].lastSeenUpdatedAt.c1).toBe('2026');
  });
});
```

- [ ] **Step 5: heimdall.test.ts**

```ts
import { afterEach, describe, expect, it, vi } from 'vitest';
import { postConversation } from '../src/heimdall';
import { DEFAULT_CONFIG } from '../src/types';

afterEach(() => vi.restoreAllMocks());

describe('postConversation', () => {
  it('attaches the bearer header and returns saved=true', async () => {
    const fetchMock = vi.spyOn(globalThis, 'fetch' as never).mockImplementation(async (...args: unknown[]) => {
      const init = args[1] as RequestInit;
      expect((init.headers as Record<string, string>)['Authorization']).toBe('Bearer abc');
      return new Response(JSON.stringify({ saved: true }), { status: 200 });
    });
    const cfg = { ...DEFAULT_CONFIG, companionToken: 'abc' };
    const conv = { vendor: 'claude.ai', conversation_id: 'x', captured_at: 't', schema_fingerprint: 'f', payload: {} };
    const out = await postConversation(cfg, conv);
    expect(out).toEqual({ saved: true, unchanged: false });
    expect(fetchMock).toHaveBeenCalledOnce();
  });

  it('throws on 401 with re-pair hint', async () => {
    vi.spyOn(globalThis, 'fetch' as never).mockResolvedValue(new Response('', { status: 401 }) as never);
    const cfg = { ...DEFAULT_CONFIG, companionToken: 'abc' };
    const conv = { vendor: 'x', conversation_id: 'y', captured_at: 't', schema_fingerprint: 'f', payload: {} };
    await expect(postConversation(cfg, conv)).rejects.toThrow(/re-pair/);
  });

  it('throws when no token paired', async () => {
    const cfg = { ...DEFAULT_CONFIG, companionToken: null };
    const conv = { vendor: 'x', conversation_id: 'y', captured_at: 't', schema_fingerprint: 'f', payload: {} };
    await expect(postConversation(cfg, conv)).rejects.toThrow(/not paired/);
  });
});
```

`chrome.runtime.getManifest()` is referenced in `heimdall.ts` for the heartbeat. Tests don't exercise that path directly (`postHeartbeat` is covered by integration); if Vitest complains about `chrome` global, add a small mock at the top of heimdall.test.ts: `(globalThis as any).chrome = { runtime: { getManifest: () => ({ version: '0.1.0' }) } };`.

- [ ] **Step 6: Run + commit**

```
cd extensions/heimdall-companion
npm run typecheck
npm test
git add src/storage.ts src/heimdall.ts src/types.ts tests/storage.test.ts tests/heimdall.test.ts
git commit -m "feat(extension): storage + heimdall client + types"
```

---

## Task 4: Vendor fetchers (claude + chatgpt)

**Files:**
- Create: `extensions/heimdall-companion/src/vendors/claude.ts`
- Create: `extensions/heimdall-companion/src/vendors/chatgpt.ts`

The fetchers run inside the user's logged-in tab (the background sw uses `chrome.scripting.executeScript` to invoke them). They issue same-origin fetches against `claude.ai` and `chatgpt.com` private APIs.

- [ ] **Step 1: claude.ts**

```ts
//! Runs in claude.ai tab context. Same-origin fetch against /api/.

export interface ClaudeOrg { uuid: string; name?: string }
export interface ClaudeConvSummary { uuid: string; name?: string; updated_at?: string }

export async function listOrgs(): Promise<ClaudeOrg[]> {
  const r = await fetch('/api/organizations', { credentials: 'include' });
  if (!r.ok) throw new Error(`claude orgs: HTTP ${r.status}`);
  return r.json();
}

export async function listConversations(orgId: string): Promise<ClaudeConvSummary[]> {
  const r = await fetch(`/api/organizations/${orgId}/chat_conversations`, { credentials: 'include' });
  if (!r.ok) throw new Error(`claude conversations: HTTP ${r.status}`);
  return r.json();
}

export async function fetchConversation(orgId: string, convId: string): Promise<unknown> {
  const r = await fetch(`/api/organizations/${orgId}/chat_conversations/${convId}`, { credentials: 'include' });
  if (!r.ok) throw new Error(`claude conv ${convId}: HTTP ${r.status}`);
  return r.json();
}
```

- [ ] **Step 2: chatgpt.ts**

```ts
//! Runs in chatgpt.com tab context. Bearer auth obtained from /api/auth/session.

export interface ChatgptConvItem { id: string; title?: string; update_time?: number }

async function getAccessToken(): Promise<string> {
  const r = await fetch('/api/auth/session', { credentials: 'include' });
  if (!r.ok) throw new Error(`chatgpt auth/session: HTTP ${r.status}`);
  const body = await r.json() as { accessToken?: string };
  if (!body.accessToken) throw new Error('chatgpt: no accessToken in session response');
  return body.accessToken;
}

export async function listConversations(pageSize = 28): Promise<ChatgptConvItem[]> {
  const token = await getAccessToken();
  const all: ChatgptConvItem[] = [];
  let offset = 0;
  for (;;) {
    const r = await fetch(`/backend-api/conversations?offset=${offset}&limit=${pageSize}&order=updated`, {
      credentials: 'include',
      headers: { Authorization: `Bearer ${token}` },
    });
    if (!r.ok) throw new Error(`chatgpt conversations: HTTP ${r.status}`);
    const body = await r.json() as { items?: ChatgptConvItem[]; total?: number };
    const items = body.items ?? [];
    all.push(...items);
    offset += items.length;
    if (items.length < pageSize) break;
    if (typeof body.total === 'number' && offset >= body.total) break;
  }
  return all;
}

export async function fetchConversation(convId: string): Promise<unknown> {
  const token = await getAccessToken();
  const r = await fetch(`/backend-api/conversation/${convId}`, {
    credentials: 'include',
    headers: { Authorization: `Bearer ${token}` },
  });
  if (!r.ok) throw new Error(`chatgpt conv ${convId}: HTTP ${r.status}`);
  return r.json();
}
```

- [ ] **Step 3: tsc + commit**

```
cd extensions/heimdall-companion
npm run typecheck
git add src/vendors/
git commit -m "feat(extension): in-page fetchers for claude.ai + chatgpt.com"
```

---

## Task 5: Sync orchestrator + background service worker

**Files:**
- Create: `extensions/heimdall-companion/src/sync.ts`
- Create: `extensions/heimdall-companion/src/background.ts`
- Create: `extensions/heimdall-companion/tests/sync.test.ts`

- [ ] **Step 1: sync.ts**

```ts
import type { ExtensionConfig, WebConversation } from './types';
import { postConversation, postHeartbeat } from './heimdall';
import { saveConfig } from './storage';

export interface SyncResult {
  vendor: string;
  listed: number;
  written: number;
  unchanged: number;
  errors: string[];
}

/** Pure function: given current state and a list of (id, updated_at), return ids to fetch. */
export function pickChanged(
  lastSeen: Record<string, string>,
  observed: Array<{ id: string; updated_at?: string }>,
): string[] {
  const out: string[] = [];
  for (const item of observed) {
    const seen = lastSeen[item.id];
    if (!item.updated_at) {
      // No timestamp from vendor → only fetch if we've never seen this id.
      if (!seen) out.push(item.id);
      continue;
    }
    if (!seen || item.updated_at > seen) out.push(item.id);
  }
  return out;
}

/** SHA-256 hex of the sorted, newline-separated keys of a top-level object. */
export async function schemaFingerprint(value: unknown): Promise<string> {
  if (value === null || typeof value !== 'object') return '';
  const keys = Object.keys(value as Record<string, unknown>).sort().join('\n');
  const buf = new TextEncoder().encode(keys);
  const hash = await crypto.subtle.digest('SHA-256', buf);
  return [...new Uint8Array(hash)].map(b => b.toString(16).padStart(2, '0')).join('');
}

export interface VendorAdapter {
  vendor: string;
  list(): Promise<Array<{ id: string; updated_at?: string }>>;
  fetch(id: string): Promise<unknown>;
}

export async function syncVendor(
  cfg: ExtensionConfig,
  adapter: VendorAdapter,
): Promise<SyncResult> {
  const result: SyncResult = {
    vendor: adapter.vendor, listed: 0, written: 0, unchanged: 0, errors: [],
  };
  const state = cfg.vendors[adapter.vendor];
  if (!state || !state.enabled) return result;

  let observed: Array<{ id: string; updated_at?: string }> = [];
  try {
    observed = await adapter.list();
  } catch (e) {
    result.errors.push(`list: ${(e as Error).message}`);
    return result;
  }
  result.listed = observed.length;
  const changed = pickChanged(state.lastSeenUpdatedAt, observed);

  for (const id of changed) {
    try {
      const payload = await adapter.fetch(id);
      const fingerprint = await schemaFingerprint(payload);
      const conv: WebConversation = {
        vendor: adapter.vendor,
        conversation_id: id,
        captured_at: new Date().toISOString(),
        schema_fingerprint: fingerprint,
        payload,
      };
      const { saved, unchanged } = await postConversation(cfg, conv);
      if (saved) result.written++;
      if (unchanged) result.unchanged++;
      const observedItem = observed.find(o => o.id === id);
      state.lastSeenUpdatedAt[id] = observedItem?.updated_at ?? new Date().toISOString();
    } catch (e) {
      result.errors.push(`${id}: ${(e as Error).message}`);
      cfg.telemetry.totalErrors++;
    }
  }
  state.lastSyncAt = new Date().toISOString();
  cfg.telemetry.totalCaptures += result.written;

  await postHeartbeat(cfg, adapter.vendor).catch(() => {});
  await saveConfig(cfg);
  return result;
}
```

- [ ] **Step 2: background.ts**

```ts
import { loadConfig, saveConfig } from './storage';
import { syncVendor, type VendorAdapter } from './sync';

const ALARM_NAME = 'heimdall-sync';

chrome.runtime.onInstalled.addListener(async () => {
  const cfg = await loadConfig();
  await scheduleAlarm(cfg.syncIntervalMinutes);
});

chrome.runtime.onStartup.addListener(async () => {
  const cfg = await loadConfig();
  await scheduleAlarm(cfg.syncIntervalMinutes);
});

chrome.alarms.onAlarm.addListener(async (alarm) => {
  if (alarm.name !== ALARM_NAME) return;
  await runSyncAll();
});

chrome.runtime.onMessage.addListener((msg, _sender, send) => {
  if (msg?.type === 'syncNow') {
    runSyncAll().then(send).catch(err => send({ error: String(err) }));
    return true;
  }
  return false;
});

async function scheduleAlarm(minutes: number): Promise<void> {
  const period = Math.max(15, minutes); // chrome.alarms minimum 1 min, we keep 15
  await chrome.alarms.clear(ALARM_NAME);
  chrome.alarms.create(ALARM_NAME, { periodInMinutes: period });
}

async function runSyncAll(): Promise<{ results: unknown[] }> {
  const cfg = await loadConfig();
  const results = [];
  for (const vendor of Object.keys(cfg.vendors)) {
    const adapter = await adapterFor(vendor);
    if (!adapter) continue;
    const r = await syncVendor(cfg, adapter);
    results.push(r);
  }
  await saveConfig(cfg);
  return { results };
}

async function adapterFor(vendor: string): Promise<VendorAdapter | null> {
  // Find a tab on the vendor's origin and run the fetcher there.
  const origin = vendorOrigin(vendor);
  if (!origin) return null;
  const tabs = await chrome.tabs.query({ url: `${origin}/*` });
  const tab = tabs[0];
  if (!tab?.id) return null;
  return {
    vendor,
    async list() {
      const [r] = await chrome.scripting.executeScript({
        target: { tabId: tab.id! },
        func: vendor === 'claude.ai' ? listClaude : listChatgpt,
      });
      return r.result as Array<{ id: string; updated_at?: string }>;
    },
    async fetch(id: string) {
      const [r] = await chrome.scripting.executeScript({
        target: { tabId: tab.id! },
        func: vendor === 'claude.ai' ? fetchClaudeConv : fetchChatgptConv,
        args: [id],
      });
      return r.result;
    },
  };
}

function vendorOrigin(vendor: string): string | null {
  if (vendor === 'claude.ai') return 'https://claude.ai';
  if (vendor === 'chatgpt.com') return 'https://chatgpt.com';
  return null;
}

// In-page functions — must be self-contained (no imports), since
// chrome.scripting.executeScript serializes the function source.

function listClaude(): Promise<Array<{ id: string; updated_at?: string }>> {
  return (async () => {
    const orgs = await fetch('/api/organizations', { credentials: 'include' }).then(r => r.json());
    const out: Array<{ id: string; updated_at?: string }> = [];
    for (const o of orgs as Array<{ uuid: string }>) {
      const convs = await fetch(`/api/organizations/${o.uuid}/chat_conversations`, { credentials: 'include' }).then(r => r.json());
      for (const c of convs as Array<{ uuid: string; updated_at?: string }>) {
        out.push({ id: `${o.uuid}/${c.uuid}`, updated_at: c.updated_at });
      }
    }
    return out;
  })();
}

function fetchClaudeConv(combinedId: string): Promise<unknown> {
  return (async () => {
    const [orgId, convId] = combinedId.split('/', 2);
    const r = await fetch(`/api/organizations/${orgId}/chat_conversations/${convId}`, { credentials: 'include' });
    return r.json();
  })();
}

function listChatgpt(): Promise<Array<{ id: string; updated_at?: string }>> {
  return (async () => {
    const session = await fetch('/api/auth/session', { credentials: 'include' }).then(r => r.json());
    const token = (session as { accessToken?: string }).accessToken;
    const out: Array<{ id: string; updated_at?: string }> = [];
    let offset = 0;
    for (;;) {
      const r = await fetch(`/backend-api/conversations?offset=${offset}&limit=28&order=updated`, {
        credentials: 'include',
        headers: { Authorization: `Bearer ${token}` },
      });
      const body = await r.json() as { items?: Array<{ id: string; update_time?: number }>; total?: number };
      const items = body.items ?? [];
      for (const it of items) out.push({ id: it.id, updated_at: it.update_time?.toString() });
      offset += items.length;
      if (items.length < 28) break;
      if (typeof body.total === 'number' && offset >= body.total) break;
    }
    return out;
  })();
}

function fetchChatgptConv(convId: string): Promise<unknown> {
  return (async () => {
    const session = await fetch('/api/auth/session', { credentials: 'include' }).then(r => r.json());
    const token = (session as { accessToken?: string }).accessToken;
    const r = await fetch(`/backend-api/conversation/${convId}`, {
      credentials: 'include',
      headers: { Authorization: `Bearer ${token}` },
    });
    return r.json();
  })();
}
```

- [ ] **Step 3: sync.test.ts**

```ts
import { describe, expect, it, vi } from 'vitest';
import { pickChanged, schemaFingerprint, syncVendor, type VendorAdapter } from '../src/sync';
import { DEFAULT_CONFIG } from '../src/types';

describe('pickChanged', () => {
  it('returns ids never seen', () => {
    expect(pickChanged({}, [{ id: 'a', updated_at: 't' }])).toEqual(['a']);
  });
  it('returns ids whose updated_at advanced', () => {
    expect(pickChanged({ a: '1' }, [{ id: 'a', updated_at: '2' }])).toEqual(['a']);
  });
  it('skips ids whose updated_at is unchanged', () => {
    expect(pickChanged({ a: '2' }, [{ id: 'a', updated_at: '2' }])).toEqual([]);
  });
});

describe('schemaFingerprint', () => {
  it('is stable across key order', async () => {
    const a = await schemaFingerprint({ x: 1, y: 2 });
    const b = await schemaFingerprint({ y: 2, x: 1 });
    expect(a).toBe(b);
  });
  it('returns empty string for non-objects', async () => {
    expect(await schemaFingerprint(null)).toBe('');
    expect(await schemaFingerprint(42)).toBe('');
  });
});

describe('syncVendor', () => {
  it('lists, diffs, fetches, posts, and updates lastSeen', async () => {
    (globalThis as any).chrome = {
      storage: { local: { get: async () => ({}), set: async () => undefined } },
      runtime: { getManifest: () => ({ version: '0.1.0' }) },
    };
    const fetchMock = vi.spyOn(globalThis, 'fetch' as never).mockImplementation(
      async () => new Response(JSON.stringify({ saved: true }), { status: 200 }) as never,
    );
    const adapter: VendorAdapter = {
      vendor: 'claude.ai',
      list: async () => [{ id: 'c1', updated_at: 't1' }],
      fetch: async () => ({ payload: 1 }),
    };
    const cfg = structuredClone(DEFAULT_CONFIG);
    cfg.companionToken = 'tok';
    const result = await syncVendor(cfg, adapter);
    expect(result.listed).toBe(1);
    expect(result.written).toBe(1);
    expect(cfg.vendors['claude.ai'].lastSeenUpdatedAt.c1).toBe('t1');
    expect(fetchMock).toHaveBeenCalled();
  });

  it('skips disabled vendors', async () => {
    const adapter: VendorAdapter = { vendor: 'claude.ai', list: async () => [], fetch: async () => ({}) };
    const cfg = structuredClone(DEFAULT_CONFIG);
    cfg.vendors['claude.ai'].enabled = false;
    const result = await syncVendor(cfg, adapter);
    expect(result.listed).toBe(0);
  });
});
```

- [ ] **Step 4: typecheck + test + commit**

```
cd extensions/heimdall-companion
npm run typecheck
npm test
git add src/sync.ts src/background.ts tests/sync.test.ts
git commit -m "feat(extension): sync orchestrator + background service worker"
```

---

## Task 6: Options page + popup

**Files:**
- Create: `extensions/heimdall-companion/src/options/options.html` + `options.ts`
- Create: `extensions/heimdall-companion/src/popup/popup.html` + `popup.ts`

- [ ] **Step 1: options.html**

```html
<!doctype html>
<html><head><meta charset="utf-8"><title>Heimdall Companion — Options</title>
<style>
  body { font: 14px system-ui, sans-serif; max-width: 560px; margin: 24px auto; padding: 0 16px; }
  h1 { margin-bottom: 4px; }
  label { display: block; margin: 12px 0 4px; font-weight: 600; }
  input[type=text], input[type=number] { width: 100%; padding: 8px; box-sizing: border-box; }
  button { padding: 8px 16px; cursor: pointer; }
  .row { display: flex; gap: 8px; align-items: center; margin: 8px 0; }
  .status { padding: 8px; background: #f3f4f6; border-radius: 4px; font-family: monospace; min-height: 20px; }
  fieldset { margin-top: 16px; }
</style>
</head>
<body>
  <h1>Heimdall Companion</h1>
  <p>Captures your <strong>claude.ai</strong> and <strong>chatgpt.com</strong> chat
  history into your local Heimdall installation. Session credentials never
  leave your browser; conversation payloads only travel to localhost.</p>

  <label for="heimdallUrl">Heimdall URL</label>
  <input id="heimdallUrl" type="text" placeholder="http://localhost:8080">

  <label for="companionToken">Companion token</label>
  <input id="companionToken" type="text" placeholder="run: heimdall companion-token show">

  <label for="syncIntervalMinutes">Sync interval (minutes)</label>
  <input id="syncIntervalMinutes" type="number" min="15" step="5">

  <fieldset>
    <legend>Vendors</legend>
    <div class="row"><input type="checkbox" id="vendor-claude.ai"> <label for="vendor-claude.ai" style="margin:0">claude.ai</label></div>
    <div class="row"><input type="checkbox" id="vendor-chatgpt.com"> <label for="vendor-chatgpt.com" style="margin:0">chatgpt.com</label></div>
  </fieldset>

  <div class="row" style="margin-top:24px">
    <button id="save">Save</button>
    <button id="syncNow">Sync now</button>
  </div>
  <div id="status" class="status"></div>

  <script type="module" src="options.js"></script>
</body></html>
```

- [ ] **Step 2: options.ts**

```ts
import { loadConfig, saveConfig } from '../storage';

const $ = (id: string) => document.getElementById(id) as HTMLInputElement;
const status = document.getElementById('status') as HTMLDivElement;

async function render() {
  const cfg = await loadConfig();
  $('heimdallUrl').value = cfg.heimdallUrl;
  $('companionToken').value = cfg.companionToken ?? '';
  $('syncIntervalMinutes').value = String(cfg.syncIntervalMinutes);
  $('vendor-claude.ai').checked = cfg.vendors['claude.ai']?.enabled ?? true;
  $('vendor-chatgpt.com').checked = cfg.vendors['chatgpt.com']?.enabled ?? true;
}

document.getElementById('save')!.addEventListener('click', async () => {
  const cfg = await loadConfig();
  cfg.heimdallUrl = $('heimdallUrl').value.trim() || cfg.heimdallUrl;
  const tok = $('companionToken').value.trim();
  cfg.companionToken = tok.length === 64 ? tok : null;
  const n = parseInt($('syncIntervalMinutes').value, 10);
  if (Number.isFinite(n) && n >= 15) cfg.syncIntervalMinutes = n;
  cfg.vendors['claude.ai'].enabled = $('vendor-claude.ai').checked;
  cfg.vendors['chatgpt.com'].enabled = $('vendor-chatgpt.com').checked;
  await saveConfig(cfg);
  status.textContent = '[SAVED]';
  setTimeout(() => (status.textContent = ''), 2000);
});

document.getElementById('syncNow')!.addEventListener('click', async () => {
  status.textContent = 'syncing...';
  try {
    const r = await chrome.runtime.sendMessage({ type: 'syncNow' });
    status.textContent = JSON.stringify(r, null, 2);
  } catch (e) {
    status.textContent = `[ERROR: ${(e as Error).message}]`;
  }
});

void render();
```

- [ ] **Step 3: popup.html / popup.ts**

```html
<!doctype html>
<html><head><meta charset="utf-8"><title>Heimdall</title>
<style>
  body { font: 13px system-ui, sans-serif; min-width: 240px; margin: 12px; }
  h1 { font-size: 14px; margin: 0 0 8px; }
  .row { display: flex; justify-content: space-between; padding: 4px 0; }
  .ok { color: #059669; }
  .warn { color: #b91c1c; }
  button { width: 100%; padding: 6px 12px; margin-top: 8px; cursor: pointer; }
</style>
</head>
<body>
  <h1>Heimdall Companion</h1>
  <div class="row"><span>Paired</span><span id="paired"></span></div>
  <div class="row"><span>Last sync</span><span id="lastSync"></span></div>
  <div class="row"><span>Captures</span><span id="captures"></span></div>
  <button id="syncNow">Sync now</button>
  <button id="options">Options...</button>
  <script type="module" src="popup.js"></script>
</body></html>
```

```ts
import { loadConfig } from '../storage';

async function render() {
  const cfg = await loadConfig();
  document.getElementById('paired')!.textContent = cfg.companionToken ? 'yes' : 'no';
  document.getElementById('paired')!.className = cfg.companionToken ? 'ok' : 'warn';
  const lastSyncs = Object.values(cfg.vendors)
    .map(v => v.lastSyncAt)
    .filter((s): s is string => !!s)
    .sort();
  document.getElementById('lastSync')!.textContent = lastSyncs[lastSyncs.length - 1] ?? '—';
  document.getElementById('captures')!.textContent = String(cfg.telemetry.totalCaptures);
}

document.getElementById('syncNow')!.addEventListener('click', () => {
  chrome.runtime.sendMessage({ type: 'syncNow' });
});
document.getElementById('options')!.addEventListener('click', () => {
  chrome.runtime.openOptionsPage();
});

void render();
```

- [ ] **Step 4: build + commit**

```
cd extensions/heimdall-companion
npm run build
npm run typecheck
git add src/options/ src/popup/
git commit -m "feat(extension): options page + toolbar popup"
```

---

## Task 7: Dashboard "Web captures" panel

**Files:**
- Create: `src/ui/components/WebCapturesPanel.tsx` + `.test.tsx`
- Modify: `src/ui/state/store.ts`, `src/ui/index.html`, `src/ui/app.tsx`.
- Re-generate: `src/ui/app.js`, `src/ui/style.css`.

- [ ] **Step 1: store additions**

In `src/ui/state/store.ts`:

```ts
export interface WebConversationSummary {
  vendor: string;
  conversation_id: string;
  captured_at: string;
  history_count: number;
}

export interface CompanionHeartbeat {
  last_seen_at: string;
  extension_version: string | null;
  user_agent: string | null;
  vendors_seen: string[];
}

export const webConversations = signal<WebConversationSummary[]>([]);
export const companionHeartbeat = signal<CompanionHeartbeat | null>(null);
```

- [ ] **Step 2: WebCapturesPanel.tsx**

Mirror the existing `BackupPanel.tsx`/`ImportsPanel.tsx` no-hooks pattern. Render:
- Heartbeat strip: "Companion: connected (Claude + ChatGPT) · last seen 14m ago" or "Companion: not paired — install extension at extensions/heimdall-companion/" with a link to the dashboard's pair-token reveal.
- Counts per vendor.
- Table: Vendor / Conversation ID / Captured / History versions.
- Refresh button calling `onReload`.

(Full code mirrors ImportsPanel — keep it concise; use `esc()` from `../lib/format` and the existing `data-table` class.)

- [ ] **Step 3: WebCapturesPanel.test.tsx**

3 tests modeled on ImportsPanel's tests: empty state, rows-per-capture, heartbeat strip rendering when present.

- [ ] **Step 4: index.html mount**

Add `<div id="web-captures-panel" class="bento-full"></div>` next to the existing `imports-panel` and `backup-panel` mounts.

- [ ] **Step 5: app.tsx wiring**

```ts
async function loadWebConversations(): Promise<void> {
  try {
    const r = await fetch('/api/archive/web-conversations');
    if (!r.ok) throw new Error(`HTTP ${r.status}`);
    const body = await r.json() as { conversations: WebConversationSummary[]; heartbeat: CompanionHeartbeat | null };
    webConversations.value = body.conversations;
    companionHeartbeat.value = body.heartbeat;
  } catch (err) {
    console.error('failed to load web captures:', err);
  }
}

const webMount = document.getElementById('web-captures-panel');
if (webMount) {
  render(<WebCapturesPanel onReload={loadWebConversations} />, webMount);
  void loadWebConversations();
}
```

- [ ] **Step 6: build + commit**

```
./node_modules/.bin/tsc --noEmit
npx vitest run
npm run build:ui
git add src/ui/state/store.ts src/ui/components/WebCapturesPanel.tsx src/ui/components/WebCapturesPanel.test.tsx \
        src/ui/index.html src/ui/app.tsx src/ui/app.js src/ui/style.css
git commit -m "feat(ui): web captures panel with heartbeat strip + per-vendor table"
```

---

## Task 8: Final integration check

Identical to Phase 1/2/3a's final task: full Rust + TS suite, clippy/fmt clean, CLI smoke (`heimdall companion-token show` still works, `/api/archive/web-conversations` registered, `extensions/heimdall-companion` typechecks + tests green), single tidy commit only if drift was found, and a `git log --oneline 44c3361..HEAD` summary.

---

## Self-Review

**Spec coverage:**

| Spec section | Plan task |
|---|---|
| §7a browser extension (TS / Manifest V3 / Chrome+Firefox) | Tasks 2–6 |
| §7a in-page fetchers using `credentials: 'include'` | Task 4 |
| §7a sync strategy (diff against `lastSeenUpdatedAt`, periodic alarm) | Task 5 |
| §7a options page (pair token, interval, sync now) | Task 6 |
| §7 storage layout under `<archive_root>/web/` | Phase 3a (reused) |
| §7 ingest endpoint accepts captures from extension | Phase 3a (reused) + heartbeat in Task 1 |
| §8 dashboard "Web captures" panel + heartbeat | Task 7 |
| §9 companion-token bearer (extension paired via hex paste) | Phase 3a (reused) |

**Out of scope:** store distribution, ChatGPT macOS Keychain reader, Cowork session import, conversation diff/restore back into vendor.

**Type consistency:** `WebConversation`, `WebConversationSummary`, `CompanionHeartbeat`, `ExtensionConfig`, `VendorState`, `SyncResult`, `VendorAdapter` referenced consistently across Rust + TypeScript.

---

## Execution Handoff

Plan complete. Execute via subagent-driven-development on `main` (continuing the existing direct-to-main posture).
