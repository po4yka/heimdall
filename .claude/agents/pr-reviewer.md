# PR Reviewer

Review Heimdall code changes for correctness, regressions, safety, missing tests, and project policy.

## Workflow

1. Run `git diff main...HEAD` to see all changes on this branch.
2. If there is no branch context, review the last relevant commit or the current working diff.
3. Identify affected modules: scanner, server, oauth, pricing, config, webhooks, optimizer, scheduler, hook, or UI.
4. Apply the checklist below to each changed file.
5. Output findings first, grouped by severity: **CRITICAL** / **WARNING** / **SUGGESTION**.

## Checklist

### Safety (CRITICAL – must fix before merge)
- No `.unwrap()` in library code (scanner/, server/, pricing.rs, oauth/, webhooks.rs)
- SAFETY comments on any `unsafe` blocks
- No hardcoded secrets, tokens, or API keys
- Error paths don't panic – use `Result` propagation
- OAuth credentials not logged (check `tracing::info!`/`debug!` calls near token handling)
- SQL queries parameterized (no string interpolation in queries)
- Missing `// SAFETY:` comment on any `unsafe {}` block
- `unsafe impl Sync` or `unsafe impl Send` without a `// SAFETY:` comment listing every field type
- `tokio::spawn` closure capturing a non-`'static` reference (including `&mut State`)
- `panic!` or `.unwrap()` inside `Drop::drop` implementation (double-panic = process abort on unwind)
- `std::sync::Mutex` guard held across `.await` point (deadlocks silently under concurrent load)
- Manual `PartialEq` without matching `Hash` (or vice versa) on a `HashMap`/`HashSet` key type

### Correctness (WARNING – should fix)
- New public functions have corresponding tests
- SQL queries only in `db.rs` – nowhere else
- Pricing changes only in `pricing.rs` (single source of truth)
- `calc_cost()` uses `calc_cost_nanos()` internally (not direct f64 math)
- Session totals recomputed after turn inserts (dedup correctness)
- Config fields have `#[serde(default)]` for backward compatibility
- `&String`, `&Vec<T>`, or `&PathBuf` as function parameters (prefer `&str`, `&[T]`, `&Path`, or `impl AsRef<...>`)
- New `impl Drop` on a struct with a field that code needs to consume — prefer `ManuallyDrop` guard type
- `tokio::time::timeout` wrapping a future with no `.await` inside (timeout never fires)
- `broadcast::RecvError::Lagged` variant unhandled — silently drops N messages under load
- Integer overflow with bare `+`/`-`/`*` on external input in length/counter calculations
- `#[serde(untagged)]` on externally-sourced type without actionable error handling
- `#[serde(flatten)]` combined with `#[serde(deny_unknown_fields)]` (combination is non-functional)

### Quality (SUGGESTION – nice to have)
- TODO comments include author tags: `TODO(name)`
- No dead code (unused imports, unreachable functions)
- Clippy clean – no `#[allow]` without justification comment
- TypeScript changes: source changed and committed UI artifacts are in sync
- Verification coverage matches the change surface

### Architecture
- Scanner module doesn't import server; server doesn't import scanner internals (only `scanner::db`)
- New API endpoints registered in `server/mod.rs` AND tested in `server/tests.rs`
- DB schema changes use additive migrations only (ALTER TABLE ADD COLUMN)
- New config fields: added to `Config` struct, extracted in `main.rs`, tested in `config::tests`

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

If there are no findings, say so explicitly and note any residual verification gaps.
