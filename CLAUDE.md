# Claude Usage Tracker -- Development Guide

## Project

Local Claude Code usage analytics dashboard. Rust binary with embedded web UI, OAuth-based rate limit monitoring, and webhook notifications.

## Build & Run

```bash
# TypeScript (dashboard UI) -- only needed when modifying src/ui/*.tsx
npm install                                    # one-time: install deps
npm run build:ts                               # compile TSX -> JS
npm run build:css                              # compile Tailwind -> CSS
npm run build:ui                               # both JS + CSS

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

The compiled `src/ui/app.js` and `src/ui/style.css` are committed to git so `cargo build` works without Node.js installed. Only re-run the build after editing `src/ui/*.tsx` or `src/ui/input.css`.

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
    input.css          -- Tailwind v4 entry (source)
    style.css          -- Generated CSS (committed)
    app.tsx            -- Entry point, data loading, filter logic
    app.js             -- Compiled JS (committed, do not edit directly)
    components/
      Header.tsx            -- Sticky header, theme toggle, rescan button
      FilterBar.tsx         -- Models, range, provider, project search
      RateWindowCard.tsx    -- Rate window / budget / unavailable cards
      EstimationMeta.tsx    -- Confidence / billing / pricing cards
      ReconciliationBlock.tsx -- OpenAI org usage reconciliation
      InlineStatus.tsx      -- Bracketed [OK] / [ERROR: ...] status
      SegmentedProgressBar.tsx -- Signature segmented progress viz
      StatsCards.tsx        -- Summary stat cards (Doto hero on total cost)
      SubagentSummary.tsx   -- Subagent breakdown
      EntrypointTable.tsx   -- Entrypoint usage table
      ServiceTiers.tsx      -- Service tiers table
      ToolUsageTable.tsx    -- Tool invocations
      McpSummaryTable.tsx   -- MCP server usage
      BranchTable.tsx       -- Git branch summary
      VersionTable.tsx      -- CLI version summary
      ApexChart.tsx         -- Generic ApexCharts wrapper
      DailyChart.tsx        -- Daily token usage bar chart
      ModelChart.tsx        -- Model distribution donut
      ProjectChart.tsx      -- Top projects horizontal bar
      HourlyChart.tsx       -- Activity by hour of day
      Sparkline.tsx         -- 7-day trend sparkline
      SessionsTable.tsx     -- Sessions table with sort/pagination
      ModelCostTable.tsx    -- Cost by model table (with share bar)
      ProjectCostTable.tsx  -- Cost by project table
      DataTable.tsx         -- Generic tanstack/table-core wrapper
      Footer.tsx            -- Static footer
    state/
      types.ts         -- TypeScript interfaces
      store.ts         -- Preact signals state (filters, chrome, status)
    lib/
      format.ts        -- Number/cost formatting utilities
      csv.ts           -- CSV export utilities
      charts.ts        -- Industrial chart options factory, color ladders
      status.ts        -- Inline [STATUS] helper (setStatus / clearStatus)
      theme.ts         -- Theme detection + apply
      rescan.ts        -- Rescan trigger factory
```

## Key Design Decisions

- **Single pricing source**: pricing.rs is the only place model prices are defined. The dashboard receives pre-computed costs from the API. No pricing logic in JS.
- **Integer nanos**: `calc_cost_nanos()` computes cost in billionths of a dollar (i64) to avoid f64 drift. `calc_cost()` is a thin wrapper.
- **Volume discounts**: `ModelPricing` has optional `threshold_tokens` + above-threshold rates. Sonnet 4.5 has a 200K threshold.
- **Pricing overrides**: Config file can override any model's rates. Applied via `OnceLock<HashMap>` at startup.
- **Embedded assets**: HTML/CSS/JS embedded via `include_str!` at compile time.
- **TypeScript source**: `src/ui/app.tsx` and `src/ui/components/*.tsx` are the source of truth. Compiled via esbuild. Committed so `cargo build` works without Node.js.
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
5. Update the relevant `.tsx` files in `src/ui/` if it should appear in the UI, then run `npm run build:ui`

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

## Dashboard UI

When editing dashboard files (`src/ui/`), follow the design skill at `.claude/skills/industrial-design/SKILL.md`. Key rules:
- Monochrome canvas; single red accent (`#D71921`) per screen for urgent/destructive only
- Numbers in Space Mono (tabular numerals); body in Space Grotesk; Doto for hero display
- No gradients, no shadows, no toast popups — use inline `[SAVED]` / `[ERROR: ...]` status
- Dark (OLED `#000`) and light (warm off-white `#F5F5F5`) both first-class via CSS variables
- XSS protection: all dynamic text through `esc()` in `src/ui/lib/format.ts`
- Recompile after changes: `npm run build:ui`
- Note: existing `src/ui/` still uses the legacy indigo/Inter palette; align during the next UI refactor
