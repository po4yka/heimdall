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

- [ ] Source selection is modeled, but not truly enforced end-to-end.
- [ ] Web extras are scaffolded, but not actually scraping/importing real browser sessions.
- [ ] Keychain storage exists, but only as a minimal store, not a full prompt/import policy layer.
- [ ] Menu visuals are functional, but not yet CodexBar-level polished for stale/error/incident/icon behavior.
- [ ] Widgets render snapshot data, but do not yet have parity-grade layouts or refresh behavior.
- [ ] CLI parity exists only for the core commands, not for all output/detail semantics.
- [ ] Packaging builds unsigned artifacts only; notarization/signing and helper embedding are incomplete.
- [ ] Swift unit tests are still missing.

### Not Started

- [ ] Real browser cookie import for Safari / Chrome-family browsers.
- [ ] Hidden `WKWebView` extraction of OpenAI dashboard extras.
- [ ] Claude web fallback logic where parity needs it.
- [ ] Bundling `claude-usage-tracker` inside `HeimdallBar.app/Contents/Helpers`.
- [ ] Release signing, entitlements hardening, notarization.

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

- [ ] Implement a shared Swift source resolver.
  - Inputs:
    - provider config
    - available Rust snapshot sources
    - web extras availability
    - login-required state
  - Outputs:
    - chosen display source
    - source explanation
    - fallback chain
- [ ] Enforce `auto|oauth|web|cli` semantics consistently for:
  - [ ] menu bar app
  - [ ] widget snapshot generation
  - [ ] `heimdallbar usage`
  - [ ] `heimdallbar cost`
- [ ] Add provider-specific “unsupported source” behavior.
  - Example: Claude should not silently claim CLI support if there is no Claude CLI source.
- [ ] Surface source mismatch warnings in the menu and CLI.

**Files**

- [macos/HeimdallBar/Shared/Sources/AppModel.swift](/Users/po4yka/GitRep/heimdall/macos/HeimdallBar/Shared/Sources/AppModel.swift:1)
- [macos/HeimdallBar/Shared/Sources/HeimdallBarModels.swift](/Users/po4yka/GitRep/heimdall/macos/HeimdallBar/Shared/Sources/HeimdallBarModels.swift:1)
- [macos/HeimdallBar/CLI/Sources/main.swift](/Users/po4yka/GitRep/heimdall/macos/HeimdallBar/CLI/Sources/main.swift:1)

**Acceptance**

- [ ] Changing source preference changes app behavior, not just labels.
- [ ] CLI JSON/text output clearly states selected source and why.

---

## Milestone 3: Browser Session Import + Keychain Policy

**Outcome:** HeimdallBar can securely ingest browser-authenticated sessions needed for web-only dashboard extras.

- [ ] Design imported-session model.
  - Required fields:
    - provider
    - browser source
    - imported_at
    - cookie names/domains present
    - login-required status
- [ ] Implement browser discovery/import for:
  - [ ] Safari
  - [ ] Chrome
  - [ ] Arc
  - [ ] Brave
- [ ] Add explicit Keychain policy decisions:
  - [ ] store imported session blob in Keychain
  - [ ] avoid plaintext token/cookie storage on disk
  - [ ] replace/clear existing imported session
- [ ] Add user-facing import/reset actions in settings.
- [ ] Add login-required and expired-session detection.

**Files**

- [macos/HeimdallBar/Shared/Sources/KeychainStore.swift](/Users/po4yka/GitRep/heimdall/macos/HeimdallBar/Shared/Sources/KeychainStore.swift:1)
- [macos/HeimdallBar/Shared/Sources/DashboardAdjunctController.swift](/Users/po4yka/GitRep/heimdall/macos/HeimdallBar/Shared/Sources/DashboardAdjunctController.swift:1)
- [macos/HeimdallBar/App/Sources/SettingsView.swift](/Users/po4yka/GitRep/heimdall/macos/HeimdallBar/App/Sources/SettingsView.swift:1)

**Acceptance**

- [ ] User can import browser session data from supported browsers.
- [ ] Imported sessions are stored only in Keychain-backed secure storage.
- [ ] Menu/settings show valid vs expired vs missing session state.

---

## Milestone 4: Hidden WebKit Dashboard Extras

**Outcome:** optional Codex/Claude web-only extras are actually fetched and displayed.

- [ ] Implement hidden `WKWebView` navigation flow for OpenAI dashboard extras.
- [ ] Add page-load timeout, retry, and auth-expired handling.
- [ ] Extract concrete OpenAI dashboard fields needed for parity.
  - Examples:
    - credits balance details
    - dashboard-only quota lanes
    - dashboard-only reset metadata
- [ ] Implement Claude web fallback only where it adds information not available from OAuth/local data.
- [ ] Cache extracted extras and rate-limit refreshes.
- [ ] Add opt-in battery/privacy note in settings.

**Files**

- [macos/HeimdallBar/Shared/Sources/WebDashboardScraper.swift](/Users/po4yka/GitRep/heimdall/macos/HeimdallBar/Shared/Sources/WebDashboardScraper.swift:1)
- [macos/HeimdallBar/Shared/Sources/DashboardAdjunctController.swift](/Users/po4yka/GitRep/heimdall/macos/HeimdallBar/Shared/Sources/DashboardAdjunctController.swift:1)
- [macos/HeimdallBar/Shared/Sources/AppModel.swift](/Users/po4yka/GitRep/heimdall/macos/HeimdallBar/Shared/Sources/AppModel.swift:1)

**Acceptance**

- [ ] With imported browser session, web-only fields show up in menu/widget/CLI as configured.
- [ ] Without a session, app reports login-required cleanly and stays functional.

---

## Milestone 5: Menu Bar UX Parity

**Outcome:** native menu UI matches the intended CodexBar-like behavior, not just the data model.

- [ ] Refine menu bar icon rendering.
  - [ ] top/bottom lane rendering
  - [ ] stale dimming
  - [ ] error dimming
  - [ ] incident overlay
  - [ ] merged-icon summarization logic
- [ ] Expand Overview tab behavior.
  - [ ] side-by-side provider summaries
  - [ ] combined cost / activity summary
  - [ ] provider switcher behavior
- [ ] Add lane-level pace / reset messaging.
- [ ] Add clear refresh-state UI.
  - [ ] in-flight refresh
  - [ ] last refresh age
  - [ ] failed refresh state
- [ ] Add menu actions for:
  - [ ] refresh selected provider
  - [ ] refresh all
  - [ ] open dashboard
  - [ ] open settings
  - [ ] import/reset web session
- [ ] Align error and degraded state vocabulary with Rust status indicators.

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

- [ ] Replace placeholder widget layouts with parity-grade designs for:
  - [ ] Switcher widget
  - [ ] Usage widget
  - [ ] History widget
  - [ ] Compact widget
- [ ] Make widget content respect source preferences and web extras.
- [ ] Add widget-specific snapshot projection layer.
- [ ] Improve widget refresh policy.
  - [ ] after app refresh
  - [ ] timeline cadence
  - [ ] login-required fallback
- [ ] Add app-group persistence if needed for extension-safe sharing.
- [ ] Add widget tests for snapshot generation and provider selection.

**Files**

- [macos/HeimdallBar/Widget/Sources/HeimdallBarWidget.swift](/Users/po4yka/GitRep/heimdall/macos/HeimdallBar/Widget/Sources/HeimdallBarWidget.swift:1)
- [macos/HeimdallBar/Shared/Sources/WidgetSnapshotStore.swift](/Users/po4yka/GitRep/heimdall/macos/HeimdallBar/Shared/Sources/WidgetSnapshotStore.swift:1)
- [macos/HeimdallBar/Shared/Sources/AppModel.swift](/Users/po4yka/GitRep/heimdall/macos/HeimdallBar/Shared/Sources/AppModel.swift:1)

**Acceptance**

- [ ] Widget views differ meaningfully by family and data purpose.
- [ ] Widgets remain useful when one provider is unavailable.

---

## Milestone 7: CLI Parity Completion

**Outcome:** `heimdallbar` is a serious parity surface, not just a debug helper.

- [ ] Harden CLI argument parser.
  - [ ] consistent invalid-argument errors
  - [ ] command help/usage output
  - [ ] clearer separation of `config` subcommands
- [ ] Add text output parity for:
  - [ ] lane summaries
  - [ ] incident/status display
  - [ ] source explanation
  - [ ] login-required web state
- [ ] Add optional provider-filtered refresh behavior that mirrors the app.
- [ ] Make `--source` affect actual source-resolution behavior, not just output metadata.
- [ ] Add CLI tests for:
  - [ ] `usage`
  - [ ] `cost`
  - [ ] `config validate`
  - [ ] `config dump`
  - [ ] invalid option combinations

**Files**

- [macos/HeimdallBar/CLI/Sources/main.swift](/Users/po4yka/GitRep/heimdall/macos/HeimdallBar/CLI/Sources/main.swift:1)
- [macos/HeimdallBar/Shared/Sources/HeimdallAPIClient.swift](/Users/po4yka/GitRep/heimdall/macos/HeimdallBar/Shared/Sources/HeimdallAPIClient.swift:1)

**Acceptance**

- [ ] `heimdallbar usage` and `heimdallbar cost` are usable without knowing app internals.
- [ ] JSON output is stable enough for scripting.

---

## Milestone 8: Helper Bundling and App Lifecycle

**Outcome:** app owns the Heimdall helper end-to-end instead of relying on PATH.

- [ ] Bundle `claude-usage-tracker` into `HeimdallBar.app/Contents/Helpers`.
- [ ] Update build script / Xcode packaging to copy helper binary into app bundle.
- [ ] Teach `HeimdallHelperController` to prefer bundled helper and only fall back to PATH when developing.
- [ ] Add helper lifecycle rules:
  - [ ] detect already-running loopback server
  - [ ] reuse matching helper when possible
  - [ ] terminate child helper on app exit when owned by app
  - [ ] handle helper upgrade/restart

**Files**

- [macos/HeimdallBar/Shared/Sources/HeimdallHelperController.swift](/Users/po4yka/GitRep/heimdall/macos/HeimdallBar/Shared/Sources/HeimdallHelperController.swift:1)
- [script/build_and_run.sh](/Users/po4yka/GitRep/heimdall/script/build_and_run.sh:1)
- [.github/workflows/release.yml](/Users/po4yka/GitRep/heimdall/.github/workflows/release.yml:1)

**Acceptance**

- [ ] App launches and refreshes correctly without relying on the user’s PATH.

---

## Milestone 9: Testing and Fixtures

**Outcome:** parity work becomes safe to iterate on.

- [ ] Add Swift unit test target(s) for shared logic.
- [ ] Add tests for:
  - [ ] source resolver precedence
  - [ ] menu projection generation
  - [ ] overview projection
  - [ ] widget snapshot projection
  - [ ] CLI formatting
- [ ] Add fixture-driven tests for:
  - [ ] Codex OAuth payloads
  - [ ] Codex RPC payloads
  - [ ] Codex CLI `/status` output
  - [ ] web-extras extraction states
  - [ ] login-required states
- [ ] Keep Rust and Swift fixtures aligned on the same example payloads where possible.

**Acceptance**

- [ ] New parity work is gated by automated tests, not only manual menu checks.

---

## Milestone 10: Release Hardening

**Outcome:** shipped artifacts are usable by real users, not just local development builds.

- [ ] Finalize app entitlements for:
  - [ ] widget extension
  - [ ] app group if used
  - [ ] WebKit/browser access needs
- [ ] Add signing pipeline.
- [ ] Add notarization pipeline.
- [ ] Verify widget extension is embedded and recognized in release build.
- [ ] Verify bundled CLI launches from app artifact.
- [ ] Add post-build validation steps in CI:
  - [ ] `codesign --verify`
  - [ ] bundle structure checks
  - [ ] helper existence
  - [ ] widget embed existence

**Files**

- [macos/HeimdallBar/project.yml](/Users/po4yka/GitRep/heimdall/macos/HeimdallBar/project.yml:1)
- [.github/workflows/release.yml](/Users/po4yka/GitRep/heimdall/.github/workflows/release.yml:1)
- [.github/RELEASING.md](/Users/po4yka/GitRep/heimdall/.github/RELEASING.md:1)

**Acceptance**

- [ ] macOS release artifact is signed/notarized and contains app, widget, helper, and CLI.

---

## Milestone 11: Documentation and User Setup

**Outcome:** users can install and understand HeimdallBar without reading source.

- [ ] Add README section for HeimdallBar:
  - [ ] what it is
  - [ ] how it differs from SwiftBar / statusline
  - [ ] how to launch it
  - [ ] how helper auto-start works
  - [ ] how to enable web extras
- [ ] Document privacy model for browser session import and WebKit scraping.
- [ ] Document troubleshooting for:
  - [ ] helper not reachable
  - [ ] widget not appearing
  - [ ] expired browser session
  - [ ] missing Codex auth.json
- [ ] Document release artifact contents.

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

- [ ] Native app can run without depending on PATH-installed helper binaries.
- [ ] Claude and Codex both have stable live-provider snapshots with documented source selection.
- [ ] Optional web extras use real browser-imported sessions and hidden WebKit extraction.
- [ ] Merged and per-provider menu modes are both fully usable.
- [ ] Widgets are parity-grade and provider-selectable.
- [ ] `heimdallbar` CLI is scriptable and source-aware.
- [ ] CI covers Rust and Swift parity logic.
- [ ] Release artifacts are signed/notarized and bundle app + widget + helper + CLI.
