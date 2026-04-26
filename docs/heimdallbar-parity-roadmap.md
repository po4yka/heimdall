# HeimdallBar Full Parity Roadmap

> **For agentic workers:** Use this as the source of truth for HeimdallBar parity work. Keep the checkbox state accurate. Do not mark an item complete until code, tests, and verification for that item are done.

## Goal

Reach practical feature parity with CodexBar for **Claude Code** and **Codex** inside this repository:

- native macOS menu bar app
- merged and per-provider status surfaces
- live quotas and countdowns
- incidents and degraded-state treatment
- optional web-dashboard extras via browser session import + hidden WebKit
- widget support
- bundled parity CLI
- release-ready packaging for app + widget + helper CLI

## Current State

### Landed

- [x] Rust live-provider module exists for Claude + Codex.
- [x] Rust menu/widget-oriented endpoints exist:
  - `/api/live-providers`
  - `/api/live-providers/refresh`
  - `/api/live-providers/history`
- [x] Codex auth parsing, OAuth fetch, CLI RPC path, and PTY-style `/status` fallback exist.
- [x] Native Xcode project scaffold exists with app, widget, shared module, and CLI.
- [x] Menu bar app compiles and supports merged vs per-provider surfaces.
- [x] Widget bundle compiles with four widget variants.
- [x] `heimdallbar` CLI compiles and supports `usage`, `cost`, `config validate`, and `config dump`.
- [x] Release workflow now uploads a separate macOS app artifact in addition to Rust binaries.

### Partial / Stubbed

- [x] Source selection is modeled and enforced across the native app, widgets, and bundled CLI.
- [ ] Web extras are scaffolded, but not actually scraping/importing live dashboard data yet.
- [ ] Menu visuals are functional, but not yet CodexBar-level polished for stale/error/incident/icon behavior.
- [x] Widgets render snapshot data with family-specific layouts, shared projection logic, and explicit refresh behavior.
- [x] CLI parity now includes explicit help, stricter argument handling, source-aware text/JSON output, and shared parser/formatter tests.
- [ ] Packaging/signing/notarization pipeline exists, but signed-notarized release verification still depends on secret-backed CI.
- [ ] Swift coverage exists for widgets, CLI, and helper resolution, but broader parity coverage is still incomplete.

### Not Started

- [x] Hidden `WKWebView` extraction of OpenAI dashboard extras. — Shipped via
  Milestone 4; the line was stale until Phase 13 reconciled it.
- [x] Claude web fallback logic where parity needs it. — Phase 13 reviewed
  the candidate fields (`claude.ai/settings/billing`, `console.anthropic.com`
  credit balance, per-day usage, team activity) and decided to keep the
  documented stub: OAuth + admin API + local DB cover the parity-relevant
  data; no web-only field justifies a WKWebView path today. Rationale block
  lives in
  [WebDashboardScraper.swift](/Users/po4yka/GitRep/heimdall/macos/HeimdallBar/PlatformMac/Sources/WebDashboardScraper.swift:129).
- [x] Bundling `claude-usage-tracker` inside `HeimdallBar.app/Contents/Helpers`.
- [ ] Release signing, entitlements hardening, notarization still need final release-run confirmation.
  - Today `App/HeimdallBar.entitlements` declares only iCloud +
    application-groups; it omits `com.apple.security.app-sandbox`, so the
    app currently runs unsandboxed. Flipping sandbox on (Phase 14) will
    require adding `com.apple.security.network.client` (WKWebView reaches
    chatgpt.com) and either a file-access entitlement or a security-scoped
    bookmark for browser-cookie reads under `~/Library/`. Phase 13 left the
    file unchanged on purpose — modifying it now would alter release
    artifacts ahead of the signing-pipeline work.

---

## Milestone 1: Harden Rust Live Provider Core

**Outcome:** Rust becomes stable and explicit as the live data source for native app, widgets, and CLI.

- [ ] Add explicit per-provider response mode to `/api/live-providers` refresh semantics.
  - Goal: `refresh(provider=claude)` should either return only Claude or clearly document that it refreshes one provider but returns all snapshots.
  - Files:
    - [src/server/api.rs](/Users/po4yka/GitRep/heimdall/src/server/api.rs:359)
    - [src/live_providers/mod.rs](/Users/po4yka/GitRep/heimdall/src/live_providers/mod.rs:19)
- [ ] Add typed compact endpoint payloads for widget/CLI consumers instead of generic `serde_json::Value`.
  - Goal: reduce decoding ambiguity in Swift.
- [ ] Add Codex source provenance fields in the snapshot.
  - Examples: `oauth`, `cli-rpc`, `cli-pty`, `web`, `unavailable`.
- [ ] Add provider refresh/error metadata to clarify fallback order.
  - Examples: last attempted source, fallback source, fetch duration, error summary.
- [ ] Add Rust tests for:
  - [ ] Codex auth path variants (`~/.codex/auth.json`, `$CODEX_HOME/auth.json`, API-key-only mode).
  - [ ] OAuth failure -> RPC fallback.
  - [ ] RPC failure -> PTY fallback.
  - [ ] Provider cost history endpoint contracts.
  - [ ] Refresh cache invalidation behavior.

**Acceptance**

- [ ] Swift app and CLI can decode all live-provider endpoints without custom one-off workarounds.
- [ ] Rust tests cover the fallback tree and endpoint shape.

---

## Milestone 2: Real Source Resolution and Provider Preference Semantics

**Outcome:** provider settings actually control source selection, instead of being mostly cosmetic.

- [x] Implement a shared Swift source resolver.
  - Inputs:
    - provider config
    - available Rust snapshot sources
    - web extras availability
    - login-required state
  - Outputs:
    - chosen display source
    - source explanation
    - fallback chain
- [x] Enforce `auto|oauth|web|cli` semantics consistently for:
  - [x] menu bar app
  - [x] widget snapshot generation
  - [x] `heimdallbar usage`
  - [x] `heimdallbar cost`
- [x] Add provider-specific “unsupported source” behavior.
  - Example: Claude should not silently claim CLI support if there is no Claude CLI source.
- [x] Surface source mismatch warnings in the menu and CLI.

**Files**

- [macos/HeimdallBar/Shared/Sources/AppModel.swift](/Users/po4yka/GitRep/heimdall/macos/HeimdallBar/Shared/Sources/AppModel.swift:1)
- [macos/HeimdallBar/Shared/Sources/HeimdallBarModels.swift](/Users/po4yka/GitRep/heimdall/macos/HeimdallBar/Shared/Sources/HeimdallBarModels.swift:1)
- [macos/HeimdallBar/CLI/Sources/main.swift](/Users/po4yka/GitRep/heimdall/macos/HeimdallBar/CLI/Sources/main.swift:1)

**Acceptance**

- [x] Changing source preference changes app behavior, not just labels.
- [x] CLI JSON/text output clearly states selected source and why.

---

## Milestone 3: Browser Session Import + Keychain Policy

**Outcome:** HeimdallBar can securely ingest browser-authenticated sessions needed for web-only dashboard extras.

- [x] Design imported-session model.
  - Required fields:
    - provider
    - browser source
    - imported_at
    - cookie names/domains present
    - login-required status
- [x] Implement browser discovery/import for:
  - [x] Safari
  - [x] Chrome
  - [x] Arc
  - [x] Brave
- [x] Add explicit Keychain policy decisions:
  - [x] store imported session blob in Keychain
  - [x] avoid plaintext token/cookie storage on disk
  - [x] replace/clear existing imported session
- [x] Add user-facing import/reset actions in settings.
- [x] Add login-required and expired-session detection.

**Files**

- [macos/HeimdallBar/Shared/Sources/KeychainStore.swift](/Users/po4yka/GitRep/heimdall/macos/HeimdallBar/Shared/Sources/KeychainStore.swift:1)
- [macos/HeimdallBar/Shared/Sources/DashboardAdjunctController.swift](/Users/po4yka/GitRep/heimdall/macos/HeimdallBar/Shared/Sources/DashboardAdjunctController.swift:1)
- [macos/HeimdallBar/App/Sources/SettingsView.swift](/Users/po4yka/GitRep/heimdall/macos/HeimdallBar/App/Sources/SettingsView.swift:1)

**Acceptance**

- [x] User can import browser session data from supported browsers.
- [x] Imported sessions are stored only in Keychain-backed secure storage.
- [x] Menu/settings show valid vs expired vs missing session state.

---

## Milestone 4: Hidden WebKit Dashboard Extras

**Outcome:** optional Codex/Claude web-only extras are actually fetched and displayed.

- [x] Implement hidden `WKWebView` navigation flow for OpenAI dashboard extras.
- [x] Add page-load timeout, retry, and auth-expired handling.
- [x] Extract concrete OpenAI dashboard fields needed for parity.
  - Examples:
    - credits balance details
    - dashboard-only quota lanes
    - dashboard-only reset metadata
- [x] Implement Claude web fallback only where it adds information not available from OAuth/local data.
- [x] Cache extracted extras and rate-limit refreshes.
- [x] Add opt-in battery/privacy note in settings.

**Files**

- [macos/HeimdallBar/Shared/Sources/WebDashboardScraper.swift](/Users/po4yka/GitRep/heimdall/macos/HeimdallBar/Shared/Sources/WebDashboardScraper.swift:1)
- [macos/HeimdallBar/Shared/Sources/DashboardAdjunctController.swift](/Users/po4yka/GitRep/heimdall/macos/HeimdallBar/Shared/Sources/DashboardAdjunctController.swift:1)
- [macos/HeimdallBar/Shared/Sources/AppModel.swift](/Users/po4yka/GitRep/heimdall/macos/HeimdallBar/Shared/Sources/AppModel.swift:1)

**Acceptance**

- [x] With imported browser session, web-only fields show up in menu/widget/CLI as configured.
  - Verified by `WebExtrasFlowTests.swift` (Phase 13) which asserts the
    Codex web-source path through `ProviderPresentationState`,
    `MenuProjectionBuilder`, and `WidgetSnapshotBuilder` end to end.
- [x] Without a session, app reports login-required cleanly and stays functional.

---

## Milestone 5: Menu Bar UX Parity

**Outcome:** native menu UI matches the intended CodexBar-like behavior, not just the data model.

- [x] Refine menu bar icon rendering.
  - [x] top/bottom lane rendering
  - [x] stale dimming
  - [x] error dimming
  - [x] incident overlay
  - [x] merged-icon summarization logic
- [x] Expand Overview tab behavior.
  - [x] side-by-side provider summaries
  - [x] combined cost / activity summary
  - [x] provider switcher behavior
- [x] Add lane-level pace / reset messaging.
- [x] Add clear refresh-state UI.
  - [x] in-flight refresh
  - [x] last refresh age
  - [x] failed refresh state
- [x] Add menu actions for:
  - [x] refresh selected provider
  - [x] refresh all
  - [x] open dashboard
  - [x] open settings
  - [x] import/reset web session
- [x] Align error and degraded state vocabulary with Rust status indicators.

**Files**

- [macos/HeimdallBar/App/Sources/MenuBarMeterRenderer.swift](/Users/po4yka/GitRep/heimdall/macos/HeimdallBar/App/Sources/MenuBarMeterRenderer.swift:1)
- [macos/HeimdallBar/App/Sources/RootMenuView.swift](/Users/po4yka/GitRep/heimdall/macos/HeimdallBar/App/Sources/RootMenuView.swift:1)
- [macos/HeimdallBar/App/Sources/HeimdallBarApp.swift](/Users/po4yka/GitRep/heimdall/macos/HeimdallBar/App/Sources/HeimdallBarApp.swift:6)
- [macos/HeimdallBar/Shared/Sources/MenuProjectionBuilder.swift](/Users/po4yka/GitRep/heimdall/macos/HeimdallBar/Shared/Sources/MenuProjectionBuilder.swift:1)

**Acceptance**

- [ ] Merged and split modes both feel complete.
- [ ] Incident/stale/error states are visually distinct and understandable.

---

## Milestone 6: Widget Parity

**Outcome:** widgets are first-class surfaces, not just thin views over stored JSON.

- [x] Replace placeholder widget layouts with parity-grade designs for:
  - [x] Switcher widget
  - [x] Usage widget
  - [x] History widget
  - [x] Compact widget
- [x] Make widget content respect source preferences and web extras.
- [x] Add widget-specific snapshot projection layer.
- [x] Improve widget refresh policy.
  - [x] after app refresh
  - [x] timeline cadence
  - [x] login-required fallback
- [x] Add app-group persistence if needed for extension-safe sharing.
- [x] Add widget tests for snapshot generation and provider selection.

**Files**

- [macos/HeimdallBar/Widget/Sources/HeimdallBarWidget.swift](/Users/po4yka/GitRep/heimdall/macos/HeimdallBar/Widget/Sources/HeimdallBarWidget.swift:1)
- [macos/HeimdallBar/Shared/Sources/WidgetSnapshotStore.swift](/Users/po4yka/GitRep/heimdall/macos/HeimdallBar/Shared/Sources/WidgetSnapshotStore.swift:1)
- [macos/HeimdallBar/Shared/Sources/AppModel.swift](/Users/po4yka/GitRep/heimdall/macos/HeimdallBar/Shared/Sources/AppModel.swift:1)

**Acceptance**

- [x] Widget views differ meaningfully by family and data purpose.
- [x] Widgets remain useful when one provider is unavailable.

---

## Milestone 7: CLI Parity Completion

**Outcome:** `heimdallbar` is a serious parity surface, not just a debug helper.

- [x] Harden CLI argument parser.
  - [x] consistent invalid-argument errors
  - [x] command help/usage output
  - [x] clearer separation of `config` subcommands
- [x] Add text output parity for:
  - [x] lane summaries
  - [x] incident/status display
  - [x] source explanation
  - [x] login-required web state
- [x] Add optional provider-filtered refresh behavior that mirrors the app.
- [x] Make `--source` affect actual source-resolution behavior, not just output metadata.
- [x] Add CLI tests for:
  - [x] `usage`
  - [x] `cost`
  - [x] `config validate`
  - [x] `config dump`
  - [x] invalid option combinations

**Files**

- [macos/HeimdallBar/CLI/Sources/main.swift](/Users/po4yka/GitRep/heimdall/macos/HeimdallBar/CLI/Sources/main.swift:1)
- [macos/HeimdallBar/Shared/Sources/HeimdallAPIClient.swift](/Users/po4yka/GitRep/heimdall/macos/HeimdallBar/Shared/Sources/HeimdallAPIClient.swift:1)

**Acceptance**

- [x] `heimdallbar usage` and `heimdallbar cost` are usable without knowing app internals.
- [x] JSON output is stable enough for scripting.

---

## Milestone 8: Helper Bundling and App Lifecycle

**Outcome:** app owns the Heimdall helper end-to-end instead of relying on PATH.

- [x] Bundle `claude-usage-tracker` into `HeimdallBar.app/Contents/Helpers`.
- [x] Update build script / Xcode packaging to copy helper binary into app bundle.
- [x] Teach `HeimdallHelperController` to prefer bundled helper and only fall back to PATH when developing.
- [x] Add helper lifecycle rules:
  - [x] detect already-running loopback server
  - [x] reuse matching helper when possible
  - [x] terminate child helper on app exit when owned by app
  - [x] handle helper upgrade/restart

**Files**

- [macos/HeimdallBar/Shared/Sources/HeimdallHelperController.swift](/Users/po4yka/GitRep/heimdall/macos/HeimdallBar/Shared/Sources/HeimdallHelperController.swift:1)
- [script/build_and_run.sh](/Users/po4yka/GitRep/heimdall/script/build_and_run.sh:1)
- [.github/workflows/release.yml](/Users/po4yka/GitRep/heimdall/.github/workflows/release.yml:1)

**Acceptance**

- [x] App launches and refreshes correctly without relying on the user’s PATH.

---

## Milestone 9: Testing and Fixtures

**Outcome:** parity work becomes safe to iterate on.

- [x] Add Swift unit test target(s) for shared logic.
- [x] Add tests for:
  - [x] source resolver precedence
  - [x] menu projection generation
  - [x] overview projection
  - [x] widget snapshot projection
  - [x] CLI formatting
- [x] Add fixture-driven tests for:
  - [x] Codex OAuth payloads
  - [x] Codex RPC payloads
  - [x] Codex CLI `/status` output
  - [x] web-extras extraction states
  - [x] login-required states
- [x] Keep Rust and Swift fixtures aligned on the same example payloads where possible.

**Acceptance**

- [x] New parity work is gated by automated tests, not only manual menu checks.

---

## Milestone 10: Release Hardening

**Outcome:** shipped artifacts are usable by real users, not just local development builds.

- [x] Finalize app entitlements for:
  - [x] widget extension
  - [x] app group if used
  - [x] WebKit/browser access needs
- [x] Add signing pipeline.
- [x] Add notarization pipeline.
- [x] Verify widget extension is embedded and recognized in release build.
- [x] Verify bundled CLI launches from app artifact.
- [x] Add post-build validation steps in CI:
  - [x] `codesign --verify`
  - [x] bundle structure checks
  - [x] helper existence
  - [x] widget embed existence

**Files**

- [macos/HeimdallBar/project.yml](/Users/po4yka/GitRep/heimdall/macos/HeimdallBar/project.yml:1)
- [.github/workflows/release.yml](/Users/po4yka/GitRep/heimdall/.github/workflows/release.yml:1)
- [.github/RELEASING.md](/Users/po4yka/GitRep/heimdall/.github/RELEASING.md:1)

**Acceptance**

- [ ] macOS release artifact is signed/notarized and contains app, widget, helper, and CLI.

---

## Milestone 11: Documentation and User Setup

**Outcome:** users can install and understand HeimdallBar without reading source.

- [x] Add README section for HeimdallBar:
  - [x] what it is
  - [x] how it differs from SwiftBar / statusline
  - [x] how to launch it
  - [x] how helper auto-start works
  - [x] how to enable web extras
- [x] Document privacy model for browser session import and WebKit scraping.
- [x] Document troubleshooting for:
  - [x] helper not reachable
  - [x] widget not appearing
  - [x] expired browser session
  - [x] missing Codex auth.json
- [x] Document release artifact contents.

**Acceptance**

- [ ] New user can install and use the app with only repo docs.

---

## Recommended Execution Order

1. Milestone 1: Harden Rust live-provider contracts.
2. Milestone 2: Make source resolution real.
3. Milestone 8: Bundle helper correctly.
4. Milestone 3: Browser session import + Keychain policy.
5. Milestone 4: Hidden WebKit extras.
6. Milestone 5: Menu bar UX parity polish.
7. Milestone 6: Widget parity.
8. Milestone 7: CLI parity completion.
9. Milestone 9: Tests and fixtures.
10. Milestone 10: Release hardening.
11. Milestone 11: Docs.

---

## Definition of Done

HeimdallBar reaches “full parity” only when all of the following are true:

- [x] Native app can run without depending on PATH-installed helper binaries.
- [ ] Claude and Codex both have stable live-provider snapshots with documented source selection.
- [ ] Optional web extras use real browser-imported sessions and hidden WebKit extraction.
- [ ] Merged and per-provider menu modes are both fully usable.
- [ ] Widgets are parity-grade and provider-selectable.
- [ ] `heimdallbar` CLI is scriptable and source-aware.
- [ ] CI covers Rust and Swift parity logic.
- [ ] Release artifacts are signed/notarized and bundle app + widget + helper + CLI.
