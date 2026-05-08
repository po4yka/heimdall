---
name: heimdall-pr-review
description: Use when reviewing Heimdall code changes for correctness, regressions, safety, missing tests, and repo policy violations. Covers findings-first review output for Rust, scanner, server, UI, and config changes.
---

# Heimdall PR Review

Use this skill for review requests in the Heimdall repo.

## Trigger guidance

- Use it when the user asks for a review, pre-handoff regression pass, or risk check.
- Prefer findings-first output ordered by severity.
- Do not switch into implementation unless the user asks to fix the findings.

## Review workflow

1. Inspect the branch diff against the appropriate base.
2. Identify affected surfaces such as scanner, server, oauth, pricing, config, webhooks, optimizer, scheduler, or UI.
3. Check the changed code against the repo rules below.
4. Report concrete findings with file and line references.

## Review checklist

### Safety

- No `.unwrap()` in library code under scanner, server, pricing, oauth, config, models, or webhooks.
- No panic-prone error paths when `Result` propagation is available.
- No hardcoded secrets, tokens, or API keys.
- No logged OAuth secrets or tokens.
- SQLite queries stay parameterized.
- Missing `// SAFETY:` comment on any `unsafe {}` block (**CRITICAL**)
- `unsafe impl Sync` or `unsafe impl Send` without a `// SAFETY:` comment listing every field type (**CRITICAL**)
- `tokio::spawn` capturing a non-`'static` reference (including `&mut State`) (**CRITICAL**)
- Panicking inside `Drop::drop` — any `.unwrap()` on cleanup path = double-panic if called during unwind (**CRITICAL**)
- `#[no_mangle]` without `#[unsafe(no_mangle)]` form in edition 2024 (**CRITICAL**)

### Correctness

- New behavior has matching tests.
- SQL stays in the intended database modules.
- Pricing logic stays centralized in `pricing.rs`.
- New API routes are wired in `server/mod.rs` and tested.
- Schema changes are additive only.
- New config fields use `#[serde(default)]` and are threaded through the call path.
- `&String`, `&Vec<T>`, or `&PathBuf` as function parameters — prefer `&str`, `&[T]`, `&Path`, or `impl AsRef<...>` (**WARNING**)
- New `impl Drop` on a struct that has a field external code needs to move out — prefer `ManuallyDrop` guard (**WARNING**)
- `tokio::time::timeout` wrapping a non-yielding future — timeout will never fire (**WARNING**)
- `std::sync::Mutex` guard held across `.await` — deadlocks silently under concurrent load (**CRITICAL**)
- `broadcast::RecvError::Lagged` unhandled — silently drops N messages under load (**WARNING**)
- Manual `PartialEq` without matching `Hash` (or vice versa) on `HashMap`/`HashSet` key types (**CRITICAL**)
- Integer overflow on length or counter arithmetic using bare `+`/`-`/`*` on untrusted input (**WARNING**)

### TypeScript / UI

- `.forEach(async ...)` in `src/ui/` — `.forEach` ignores the returned Promise; errors are silently swallowed and execution order is non-deterministic. Use `for...of` with `await` for sequential execution or `Promise.all(items.map(async ...))` for parallel. (**HIGH**)
- `useSignal(prop)` where the argument is a component prop — the signal captures only the initial value and goes stale on re-renders. Require `useComputed(() => props.value)` instead. (**HIGH**)
- `{signal.value}` in JSX return for read-only display — forces full component re-render; pass the signal object directly (`{signal}`) for DOM-level binding. (**MEDIUM**)
- `onChange` on `<input>` / `<textarea>` / `<select>` without `preact/compat` — fires only on blur in raw Preact, not on every keystroke. Confirm `preact/compat` alias is active in `tsconfig.json` or use `onInput` instead. (**HIGH**)

### Quality

- No unjustified `#[allow(...)]`.
- UI source changes do not forget committed build artifacts.
- Architecture boundaries remain intact.

## Output format

- Findings first, ordered by severity.
- Keep summaries brief and secondary.
- If there are no findings, say that explicitly and note any residual test or verification gaps.
