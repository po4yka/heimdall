# PR Reviewer

Review code changes for correctness, safety, and project policy.

## Workflow

1. Run `git diff main...HEAD` to see all changes on this branch
2. If no branch (working on main), run `git diff HEAD~1` for last commit
3. Identify affected modules: scanner, server, oauth, pricing, config, webhooks, UI
4. Apply the checklist below to each changed file
5. Output findings grouped by severity: **CRITICAL** / **WARNING** / **SUGGESTION**

## Checklist

### Safety (CRITICAL – must fix before merge)
- No `.unwrap()` in library code (scanner/, server/, pricing.rs, oauth/, webhooks.rs)
- SAFETY comments on any `unsafe` blocks
- No hardcoded secrets, tokens, or API keys
- Error paths don't panic – use `Result` propagation
- OAuth credentials not logged (check `tracing::info!`/`debug!` calls near token handling)
- SQL queries parameterized (no string interpolation in queries)

### Correctness (WARNING – should fix)
- New public functions have corresponding tests
- SQL queries only in `db.rs` – nowhere else
- Pricing changes only in `pricing.rs` (single source of truth)
- `calc_cost()` uses `calc_cost_nanos()` internally (not direct f64 math)
- Session totals recomputed after turn inserts (dedup correctness)
- Config fields have `#[serde(default)]` for backward compatibility

### Quality (SUGGESTION – nice to have)
- TODO comments include author tags: `TODO(name)`
- No dead code (unused imports, unreachable functions)
- Clippy clean – no `#[allow]` without justification comment
- TypeScript changes: `app.ts` modified AND `app.js` recompiled
- Test count not decreased (baseline: 118)

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
