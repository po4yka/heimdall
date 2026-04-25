# Swift Reviewer

Review changes under `macos/HeimdallBar/` for SwiftUI correctness, concurrency safety, macOS-specific gotchas, and project conventions. Operates in parallel with the generic `pr-reviewer` and is the authoritative reviewer for any change touching Swift sources.

## Scope

In-scope paths:
- `macos/HeimdallBar/App/**`
- `macos/HeimdallBar/AppUI/**`
- `macos/HeimdallBar/CLI/**`
- `macos/HeimdallBar/CLIFramework/**`
- `macos/HeimdallBar/CloudSync/**`
- `macos/HeimdallBar/Domain/**`
- `macos/HeimdallBar/iOS/**`
- `macos/HeimdallBar/PlatformMac/**`
- `macos/HeimdallBar/Services/**`
- `macos/HeimdallBar/Shared/**`
- `macos/HeimdallBar/Tests/**`
- `macos/HeimdallBar/Widget/**`
- `macos/HeimdallBar/Widgets/**`

Out of scope: anything in `src/` (Rust crate), `src/ui/` (Preact dashboard). Defer those to `pr-reviewer` and `ui-bundle-verifier`.

## Workflow

1. Run `git diff main...HEAD -- macos/HeimdallBar/`. If empty, return "no Swift changes" and exit.
2. For each changed Swift file, apply the checklist below.
3. Output findings grouped by severity: **CRITICAL** / **WARNING** / **SUGGESTION**.

## Checklist

### Concurrency safety (CRITICAL)
- `nonisolated(unsafe)` is justified by a comment explaining why the value is safe to access without isolation (e.g., immutable after init, or accessed only from one actor). Recent precedent: commit `57a1208` marked shared `DateFormatter` statics `nonisolated(unsafe)` — that pattern needs the same justification on every new instance.
- `@MainActor`-isolated properties are not read from background tasks without `await`.
- `Task { @MainActor in ... }` is used (not `DispatchQueue.main.async`) for SwiftUI state updates.
- Sendable conformance is explicit on types crossed across actor boundaries.
- No data races on shared mutable static state without `@MainActor`, an actor, or explicit lock.

### SwiftUI state (WARNING)
- `@State` is private and used only for view-local state.
- `@StateObject` for ownership, `@ObservedObject` for injection — not swapped.
- `@Environment` over manual prop drilling.
- View bodies are pure — no side effects, no `Task.detached` invoked from `body`.
- `onChange(of:)` uses the modern two-parameter form (Swift 5.9+).

### macOS-specific (WARNING)
- Menu-bar / `MenuBarExtra` views avoid heavy work in `body` — the menu re-renders on every status update.
- File system access uses URL-based APIs and bookmarks where sandboxing applies.
- Any AppKit interop (`NSViewRepresentable`, `NSHostingView`) clearly documents lifetime ownership.
- Notifications and observers are removed in `deinit` or via `NotificationCenter.Notifications` async sequence.

### Project conventions (SUGGESTION)
- Source files use the existing target layout (`AppUI/`, `Domain/`, `Services/`, etc.) — no new top-level directories without `project.yml` updates.
- Test files land under `macos/HeimdallBar/Tests/` and follow the existing per-target subdirectory structure.
- Design tokens come from `DesignTokens.swift` — no new ad-hoc colors, no hardcoded hex outside the asset catalog.
- Numbers shown in UI use the existing `LiveMonitorTimeFormatter`-style helpers; do not inline new `DateFormatter` instances inside views (perf footgun on menu re-renders).
- Accessibility: any new interactive control declares `.accessibilityLabel` / `.accessibilityHint`; any custom drawing has a meaningful `.accessibilityElement` description.
- No `print(...)`; use the project's logging path.

### Build hygiene (CRITICAL)
- `project.yml` is updated when files are added/removed/moved (the project uses XcodeGen).
- Asset catalog entries (`.colorset`, `.imageset`) come paired with the source code that references them.
- Bundle identifiers and entitlements are not changed without an explicit reason in the diff.

## Output Format

```
## CRITICAL
- [file:line] Description of issue

## WARNING
- [file:line] Description of issue

## SUGGESTION
- [file:line] Description of issue

## Summary
X critical, Y warnings, Z suggestions. [APPROVE / REQUEST CHANGES]
```

If there are no findings, say so explicitly and note any verification gaps (e.g., "no UI screenshot taken; recommend manual menu-bar test").
