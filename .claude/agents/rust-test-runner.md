# Rust Test Runner

Run the appropriate test suite based on what changed, and triage any failures.

## Suite Selection

Determine which tests to run based on changed files:

| Changed Files | Command |
|---------------|---------|
| `pricing.rs` | `cargo test pricing` |
| `scanner/` | `cargo test scanner` |
| `oauth/` | `cargo test oauth` |
| `server/` | `cargo test server` |
| `config.rs` | `cargo test config` |
| `webhooks.rs` | `cargo test webhooks` |
| `agent_status/` | `cargo test agent_status` |
| `main.rs` or `cli_tests.rs` | `cargo test cli_tests` |
| `app.ts` | `./node_modules/.bin/tsc --noEmit` |
| Multiple modules or unsure | `cargo test` |

Always run the targeted suite first for fast feedback, then `cargo test` for full verification.

## Running Tests

```bash
# Targeted (fast feedback)
cargo test <module> -- --nocapture

# Full suite
cargo test

# With output for debugging
cargo test -- --nocapture 2>&1
```

## Failure Triage Protocol

When tests fail:

1. **Read the failure output** – identify test name, expected vs actual values
2. **Classify the failure**:
   - Compilation error → fix the code, not the test
   - Assertion failure → compare expected vs actual, check if test or code is wrong
   - Timeout → check for deadlocks or infinite loops
   - Flaky → run the test 3 times; if it passes sometimes, investigate race conditions
3. **Check recent changes** – `git diff` to see what changed
4. **Fix and re-run** – make the minimal fix, run only the affected test first

## Coverage Baseline

Current test count: **572+ tests** (across 4 suites: lib, main, heimdall-hook, doc-tests)

Rules:
- New features MUST add tests
- Test count should not decrease
- Run `cargo test 2>&1 | grep "test result"` to check count

## TypeScript Checks

When `app.ts` changes:
1. Type check: `./node_modules/.bin/tsc --noEmit`
2. Compile: `./node_modules/.bin/esbuild src/ui/app.ts --outfile=src/ui/app.js --bundle --format=iife --target=es2020`
3. Rebuild Rust to embed new JS: `cargo build`
