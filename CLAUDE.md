# Claude Usage Tracker -- Development Guide

## Project

Local Claude Code usage analytics dashboard. Rust binary with embedded web UI, OAuth-based rate limit monitoring, and webhook notifications.

## Build & Run

```bash
# TypeScript (dashboard UI) -- only needed when modifying src/ui/app.ts
npm install                                    # one-time: install esbuild + typescript
./node_modules/.bin/esbuild src/ui/app.ts \
  --outfile=src/ui/app.js --bundle \
  --format=iife --target=es2020                # compile TS -> JS

# Rust
cargo build                    # debug build
cargo build --release          # release build
cargo run -- dashboard         # scan + start dashboard
cargo run -- today             # today's usage
cargo run -- today --json      # JSON output for scripting
cargo run -- stats             # all-time stats
cargo run -- stats --json      # JSON output
cargo run -- scan              # scan only
```

The compiled `src/ui/app.js` is committed to git so `cargo build` works without Node.js installed. Only re-run esbuild after editing `src/ui/app.ts`.

## Test

```bash
cargo test                        # all 118 tests
cargo test scanner                # scanner module tests
cargo test pricing                # pricing tests
cargo test oauth                  # OAuth module tests
cargo test config                 # config tests
cargo test webhooks               # webhook tests
cargo test cli_tests              # CLI command tests
cargo test -- --nocapture         # with stdout
./node_modules/.bin/tsc --noEmit  # TypeScript type check
```

## Lint

```bash
cargo clippy -- -D warnings
cargo fmt --check
```

## Architecture

```
src/
  main.rs              -- CLI (clap), entry point, cmd_today/cmd_stats
  cli_tests.rs         -- CLI command tests
  config.rs            -- TOML config: projects_dirs, db_path, host, port, oauth, webhooks, pricing
  models.rs            -- Shared types (Session, Turn, ScanResult, DashboardData, SubagentSummary, etc.)
  pricing.rs           -- Pricing table, calc_cost/calc_cost_nanos, volume discounts, overrides via OnceLock
  webhooks.rs          -- Fire-and-forget webhook POSTs on session depletion / cost threshold
  oauth/
    mod.rs             -- poll_usage(): load creds -> refresh if needed -> fetch API -> attach identity
    credentials.rs     -- Read ~/.claude/.credentials.json, token refresh via platform.claude.com
    api.rs             -- GET api.anthropic.com/api/oauth/usage, response building
    models.rs          -- CredentialsFile, OAuthUsageResponse, UsageWindowsResponse, Plan, Identity
  scanner/
    mod.rs             -- scan() orchestration, incremental processing, walkdir
    parser.rs          -- JSONL parsing, streaming dedup by message.id, subagent detection (isSidechain/agentId)
    db.rs              -- SQLite schema, init, migrations, queries, rate_window_history table
    tests.rs           -- Integration tests for scan pipeline
  server/
    mod.rs             -- axum router: /, /api/data, /api/rescan, /api/usage-windows, /api/health
    api.rs             -- Handlers with AppState (db_path, oauth cache via RwLock, webhook config)
    assets.rs          -- include_str! for HTML/CSS/JS
    tests.rs           -- HTTP endpoint tests
  ui/
    index.html         -- Dashboard HTML shell
    style.css          -- Dark theme CSS
    app.ts             -- Dashboard TypeScript source (types, rendering, filtering, charts)
    app.js             -- Compiled JS (committed, do not edit directly)
```

## Key Design Decisions

- **Single pricing source**: pricing.rs is the only place model prices are defined. The dashboard receives pre-computed costs from the API. No pricing logic in JS.
- **Integer nanos**: `calc_cost_nanos()` computes cost in billionths of a dollar (i64) to avoid f64 drift. `calc_cost()` is a thin wrapper.
- **Volume discounts**: `ModelPricing` has optional `threshold_tokens` + above-threshold rates. Sonnet 4.5 has a 200K threshold.
- **Pricing overrides**: Config file can override any model's rates. Applied via `OnceLock<HashMap>` at startup.
- **Embedded assets**: HTML/CSS/JS embedded via `include_str!` at compile time.
- **TypeScript source**: `src/ui/app.ts` is the source of truth. Compiled via esbuild. Committed so `cargo build` works without Node.js.
- **Incremental scanning**: Track file mtime + line count in `processed_files` table. Skip already-processed lines.
- **Dedup correctness**: After all turn inserts, recompute session totals from turns table via `SELECT SUM(...)`.
- **Atomic rescan**: Write to temp DB, then atomically rename. No data loss on crash.
- **OAuth caching**: Usage windows cached in `RwLock<Option<(Instant, Data)>>` for configurable interval (default 60s).
- **Subagent tracking**: `isSidechain` + `agentId` from JSONL stored as `is_subagent` + `agent_id` in turns table.

## Conventions

- Use `thiserror` for error types, `anyhow` in main/CLI
- Prefer `&str` over `String` in function signatures where possible
- All SQL queries in `db.rs`, nowhere else
- Tests use `tempfile` crate for temp dirs and DB files
- No `.unwrap()` in library code (scanner, server, pricing). OK in tests and main.
- Log with `tracing`: `debug!` for per-file progress, `info!` for scan summaries, `warn!` for recoverable errors

## Common Tasks

### Adding a new model to pricing

Edit `pricing.rs` only. Add to `PRICING_TABLE`. Set `threshold_tokens: None` unless it has volume discounts. Tests verify the lookup logic.

### Adding a new JSONL field

1. Add field to the `Turn` or `Session` struct in `models.rs`
2. Parse it in `parser.rs`
3. Add column migration in `db.rs` (ALTER TABLE with try/catch pattern)
4. Expose via API in `api.rs` if needed by the dashboard
5. Update `app.ts` if it should appear in the UI, then recompile

### Adding a new API endpoint

1. Add handler in `server/api.rs`
2. Add route in `server/mod.rs`
3. Add test in `server/tests.rs` (include in `test_app()` router)

### Changing the database schema

Always use additive migrations (ALTER TABLE ADD COLUMN). Check for column existence before adding. Never drop columns or tables in migrations -- only in full rescan.

### Config file changes

1. Add field to the appropriate struct in `config.rs` (with `#[serde(default)]`)
2. Extract in `main.rs` before the match
3. Pass through to where it's needed (server, scanner, etc.)
4. Add test for parsing in `config.rs` tests
