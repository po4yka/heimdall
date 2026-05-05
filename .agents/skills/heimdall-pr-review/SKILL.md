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

### Correctness

- New behavior has matching tests.
- SQL stays in the intended database modules.
- Pricing logic stays centralized in `pricing.rs`.
- New API routes are wired in `server/mod.rs` and tested.
- Schema changes are additive only.
- New config fields use `#[serde(default)]` and are threaded through the call path.
- `&String`, `&Vec<T>`, or `&PathBuf` as function parameters — prefer `&str`, `&[T]`, `&Path`, or `impl AsRef<...>` (**WARNING**)
- New `impl Drop` on a struct that has a field external code needs to move out — prefer `ManuallyDrop` guard (**WARNING**)

### Quality

- No unjustified `#[allow(...)]`.
- UI source changes do not forget committed build artifacts.
- Architecture boundaries remain intact.

## Output format

- Findings first, ordered by severity.
- Keep summaries brief and secondary.
- If there are no findings, say that explicitly and note any residual test or verification gaps.
