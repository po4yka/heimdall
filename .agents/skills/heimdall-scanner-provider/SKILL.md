---
name: heimdall-scanner-provider
description: Use when adding, fixing, or refactoring a Heimdall scanner provider. Covers provider discovery, parser shape, provider tagging, registration, path-based detection, and the required tests for JSONL, SQLite, and mixed-format providers.
---

# Heimdall Scanner Provider

Use this skill for work under `src/scanner/providers/` and the surrounding scanner pipeline.

## Defaults

- Follow the provider guidance in `AGENTS.md` instead of inventing a new pattern.
- Keep provider behavior additive and best-effort. Unknown input formats should usually return an empty result, not crash the scan.
- Preserve stable provider slugs. They flow into `turns.provider`, `sessions.provider`, filters, and summaries.

## Implementation checklist

1. Create or update `src/scanner/providers/<name>.rs`.
2. Implement `Provider` with a stable `name()`, filesystem discovery, and parsing.
3. Choose the right backend pattern:
   - JSONL-backed: reuse the shared parser when the format is Claude-compatible.
   - SQLite-backed: probe schema first and return empty when tables are missing.
   - Mixed-format: treat unrecognized shapes as non-fatal and return `Ok(Vec::new())`.
4. Register the provider in `src/scanner/providers/mod.rs`.
5. Update `provider_for_dir()` in `src/scanner/mod.rs` if `--projects-dir` needs path-based routing for the provider.
6. Ensure turns carry the correct provider tag and source path.
7. Add or update tests in `src/scanner/tests.rs`.

## Test minimums

- `name()` returns the expected slug.
- Fixture-based parse test proves returned turns are tagged with the provider.
- Discovery logic handles missing directories without error.
- If the provider is SQLite or mixed-format, add one test for absent or unsupported schema returning empty rather than failing.

## Guardrails

- Keep all SQL outside provider files unless the provider must read an external SQLite database directly.
- Do not add `unwrap()` in library code.
- Use `tempfile` fixtures for tests; never touch the real user home directory.
