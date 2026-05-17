# Heimdall Companion

Browser extension that captures your claude.ai and chatgpt.com chat history and ships it to a local Heimdall instance for archival and analysis.

The runtime fetch-and-POST loop is fully implemented: a background service worker syncs on an alarm schedule (default every 6 hours) and on manual trigger, posting each changed conversation to the local heimdall daemon at `POST /api/archive/web-conversation`.

## Requirements

- Node.js 20+ (build only)
- Heimdall >= the version that ships `/api/archive/companion-*` endpoints (Phase 3b; `companion-token` subcommand must be available)

## Sideload — Chrome

1. Run `npm install && npm run build:chrome` from this directory.
2. Open Chrome and navigate to `chrome://extensions`.
3. Enable **Developer mode** (toggle, top-right).
4. Click **Load unpacked** and select the `dist/chrome/` folder.
5. The Heimdall Companion icon appears in the toolbar.

## Sideload — Firefox

1. Run `npm install && npm run build:firefox` from this directory.
2. Open Firefox and navigate to `about:debugging#/runtime/this-firefox`.
3. Click **Load Temporary Add-on**.
4. Select `dist/firefox/manifest.json`.
5. The extension reloads on every browser restart; repeat steps 3-4 each session (or use a signed build for permanent install).

## Pair with Heimdall

1. Start the Heimdall dashboard: `heimdall dashboard`.
2. Print your companion token: `heimdall companion-token show`. Copy the 64-character hex string that is printed.
3. Click the Heimdall Companion icon → **Options**.
4. Set **Heimdall URL** to `http://127.0.0.1:8787` (or the port you passed to `heimdall dashboard --port`).
5. Paste the token into **Companion token** and click **Save**.
6. Click **Sync now** to run an immediate capture.

## Privacy

- The extension reads claude.ai and chatgpt.com pages using the permissions declared in `manifest.json`.
- All captured data is sent only to your local Heimdall instance.
- Your session credentials (cookies, tokens) never leave your browser.
- You own your account data. Heimdall stores conversations under `~/.heimdall/archive/web/`.

## Development

```bash
npm install           # install deps
npm run typecheck     # tsc --noEmit
npm run build:chrome  # esbuild -> dist/chrome/
npm run build:firefox # esbuild + firefox manifest -> dist/firefox/
npm test              # vitest — unit + HAR-replay tests
```
