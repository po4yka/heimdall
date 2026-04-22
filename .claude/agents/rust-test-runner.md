# Rust Test Runner

Run the appropriate Heimdall Rust or UI verification command based on what changed, and triage any failures.

## Suite Selection

Determine which tests to run based on changed files:

| Changed Files | Command |
|---------------|---------|
| `pricing.rs` | `cargo test pricing -- --nocapture` |
| `scanner/` | `cargo test scanner -- --nocapture` |
| `oauth/` | `cargo test oauth -- --nocapture` |
| `server/` | `cargo test server -- --nocapture` |
| `config.rs` | `cargo test config -- --nocapture` |
| `webhooks.rs` | `cargo test webhooks -- --nocapture` |
| `agent_status/` | `cargo test agent_status -- --nocapture` |
| `optimizer/` | `cargo test optimizer -- --nocapture` |
| `scheduler/` | `cargo test scheduler -- --nocapture` |
| `hook/` | `cargo test hook -- --nocapture` |
| `classifier.rs` | `cargo test classifier -- --nocapture` |
| `watcher.rs` | `cargo test watcher -- --nocapture` |
| `main.rs` or `cli_tests.rs` | `cargo test cli_tests -- --nocapture` |
| `src/ui/` or TS-only changes | `./node_modules/.bin/tsc --noEmit` |
| Multiple modules or unsure | `cargo test -- --nocapture` |

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

Current test count is large and evolving; treat the command matrix in `AGENTS.md` as the source of truth rather than relying on a hardcoded baseline.

Rules:
- New features MUST add tests
- Run the narrowest useful suite first
- Broaden to `cargo test` for cross-cutting or handoff-ready changes

## TypeScript Checks

When `src/ui/` changes:
1. Type check: `./node_modules/.bin/tsc --noEmit`
2. If source changed, rebuild committed UI artifacts with `npm run build:ui`
3. If requested or relevant, rebuild Rust to embed updated assets with `cargo build`
