---
name: heimdall-fix-unwraps
description: Use when removing `.unwrap()` calls from Heimdall production Rust code. Covers non-test library code in scanner, server, oauth, pricing, config, webhooks, and models, with replacement strategies and verification expectations.
---

# Heimdall Fix Unwraps

Use this skill when the task is to remove panic-prone `.unwrap()` calls from production Rust code.

## Scope

- Search `src/` for `.unwrap()`.
- Exclude tests, `main.rs`, and other entry-point-only unwraps unless the user asks otherwise.
- Focus on library code in `scanner/`, `server/`, `oauth/`, `pricing.rs`, `config.rs`, `webhooks.rs`, and `models.rs`.

## Replacement rules

1. In a function returning `Result`, prefer `?`.
2. In a non-`Result` function, decide whether to:
   - change the signature and propagate the error
   - use an explicit fallback such as `unwrap_or_default` or `unwrap_or_else`
   - keep an actually infallible path only with a concise justification comment
3. Leave `.expect(...)` only when the message clearly explains the invariant.

## Workflow

1. Enumerate the remaining production `.unwrap()` sites in scope.
2. Replace each one with the narrowest safe alternative.
3. Avoid drive-by refactors outside the touched failure path.
4. Run the relevant Rust tests after the edits.

## Special case: `unsafe` paths

When removing `.unwrap()` from a code path that contains or calls into an `unsafe` block:

1. Check what SAFETY invariant the `unsafe` block relies on.
2. The new error path (using `?` or a fallback) must preserve that invariant — verify that returning early does not leave invariant-protected state in a partially-initialized condition.
3. If the SAFETY comment says "this call cannot fail", and you're removing the `.unwrap()` that asserted that, update the SAFETY comment to explain how failure is now handled.
4. Add or update the `// SAFETY:` comment to document what happens if an error is returned.

Example: replacing `.unwrap()` on a libc return value — the new `?` path must still close any open fds or release any locks the unsafe code acquired before the failure point.

## Output expectations

- Report how many `.unwrap()` sites were found in scope.
- State which ones were fixed.
- If any were intentionally left, justify each one.
