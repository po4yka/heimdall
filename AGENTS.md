# Heimdall (`claude-usage-tracker`) – Development Guide

## Project

Local AI session observability for Claude Code, Codex, and adjacent tooling. Two Rust binaries share one crate:

- `claude-usage-tracker` – CLI + embedded web dashboard.
- `heimdall-hook` – lightweight stdin-driven hook binary for real-time PreToolUse ingest.

Persistent storage is a single SQLite database. UI is Preact + Tailwind v4 compiled to a single JS/CSS bundle via esbuild; both compiled artifacts live in git so `cargo build` never requires Node.js.

## Build & Run

```bash
# TypeScript (dashboard UI) -- only needed when modifying src/ui/*.tsx
npm install                                    # one-time: install deps
npm run build:ts                               # compile TSX -> JS
npm run build:css                              # compile Tailwind -> CSS
npm run build:ui                               # both JS + CSS
./node_modules/.bin/tsc --noEmit               # type-check only

# Rust — default dashboard-binary
cargo build                    # debug build
cargo build --release          # release build (both binaries)
cargo run -- dashboard         # scan + start dashboard
cargo run -- today             # today's usage
cargo run -- today --json      # JSON output for scripting
cargo run -- stats             # all-time stats
cargo run -- stats --json      # JSON output
cargo run -- scan              # scan only
cargo run -- export --format=csv --period=month --output=out.csv
cargo run -- optimize --format=json
cargo run -- scheduler install  # launchd / cron / schtasks
cargo run -- daemon install     # macOS-only always-on dashboard
cargo run -- hook install       # wire up heimdall-hook into ~/.claude/settings.json
cargo run -- db reset --yes     # destructive DB wipe (TTY-guarded)
cargo run -- menubar            # SwiftBar-formatted snapshot
cargo run -- pricing refresh    # pull LiteLLM catalogue into the cache

# Hook binary — reads stdin, exits ~50ms, always prints "{}"
cargo run --bin heimdall-hook
```

The compiled `src/ui/app.js` and `src/ui/style.css` are committed to git so `cargo build` works without Node.js installed. Only re-run `npm run build:ui` after editing `src/ui/*.tsx` or `src/ui/input.css`.

## Test

```bash
cargo test                        # full Rust suite (538+ tests across 4 suites)
cargo test scanner                # scanner module tests
cargo test pricing                # pricing + LiteLLM + cost-breakdown tests
cargo test oauth                  # OAuth module tests
cargo test config                 # config tests
cargo test webhooks               # webhook tests
cargo test cli_tests              # CLI command tests
cargo test optimizer              # waste-detector tests
cargo test scheduler              # launchd / cron / schtasks tests
cargo test hook                   # heimdall-hook pipeline tests
cargo test classifier             # 13-category classifier tests
cargo test watcher                # file-watcher debounce + shutdown tests
cargo test -- --nocapture         # with stdout
./node_modules/.bin/tsc --noEmit  # TypeScript type check
```

The `--watch` flag on `dashboard` spawns the file-watcher (Phase 20); its tests are timing-sensitive and run with a 3-second debounce bound.

## Lint

```bash
cargo clippy -- -D warnings
cargo fmt --check
```

CI runs both gates on every PR. The release workflow (`.github/workflows/release.yml`) also cross-builds both binaries on 5 targets when a `v*.*.*` tag is pushed.

## Architecture

```
src/
  lib.rs               -- Library root; both binaries depend on it
  main.rs              -- Primary CLI (clap): scan/today/stats/dashboard/export/
                          optimize/scheduler/daemon/hook/db/menubar/pricing
  cli_tests.rs         -- CLI command tests
  config.rs            -- TOML config loader w/ dual-path resolver
                          ($HEIMDALL_CONFIG -> ~/.config/heimdall -> ~/.claude/usage-tracker.toml)
  models.rs            -- Shared types (Session, Turn, ToolEvent, CacheEfficiency, ...)
  pricing.rs           -- Pricing table, calc_cost_nanos, volume discounts,
                          CostBreakdown 4-way split, LiteLlm 5th-tier fallback
  currency.rs          -- Frankfurter USD->N conversion with 24h disk cache + hardcoded fallback
  litellm.rs           -- LiteLLM catalogue fetch + cache (~/.cache/heimdall/litellm_pricing.json)
  tz.rs                -- TzParams shared between server and db for timezone-aware bucketing
  export.rs            -- `export` subcommand (csv / json / jsonl) with period filtering
  menubar.rs           -- SwiftBar widget renderer + injection-hardened sanitizer
  db.rs                -- `db reset` command with TTY confirmation guard (pure `should_proceed`)
  webhooks.rs          -- Fire-and-forget webhook POSTs on session depletion / cost threshold /
                          agent status transitions (agent_status_degraded / agent_status_restored)
  openai.rs            -- OpenAI organization usage reconciliation client

  agent_status/
    mod.rs             -- poll() orchestrator + poll_with_injection() test seam + AlertDirection enum
    client.rs          -- Claude HTTP client (ETag/If-None-Match) + OpenAI two-call client
    models.rs          -- AgentStatusSnapshot, ProviderStatus, ComponentStatus, IncidentSummary,
                          StatusIndicator enum, raw Statuspage wire structs
    filter.rs          -- Hardcoded component allowlists (Claude IDs, OpenAI names)

  oauth/
    mod.rs             -- poll_usage(): load creds -> refresh if needed -> fetch API -> attach identity
    credentials.rs     -- Read ~/.claude/.credentials.json, token refresh via platform.claude.com
    api.rs             -- GET api.anthropic.com/api/oauth/usage, response building
    models.rs          -- CredentialsFile, OAuthUsageResponse, UsageWindowsResponse, Plan, Identity

  scanner/
    mod.rs             -- scan() orchestration, incremental processing, walkdir
    parser.rs          -- JSONL parsing, streaming dedup by message.id, tool_inputs capture
    db.rs              -- SQLite schema, queries, migrations; all SQL lives here
    tests.rs           -- Integration tests for the full scan pipeline
    classifier.rs      -- 13-category task classifier (regex RegexSet, pure function)
    oneshot.rs         -- Edit->Bash->Edit retry-cycle detection
    cowork.rs          -- Ephemeral Cowork label resolution from audit.jsonl
    usage_limits.rs    -- Parses ~/.claude/**/*-usage-limits files into rate_window_history
    watcher.rs         -- `notify`-backed file watcher w/ 2s debounce (--watch flag)
    provider.rs        -- `Provider` trait + `SessionSource`
    providers/
      mod.rs           -- Central registry: pub fn all() -> Vec<Box<dyn Provider>>
      claude.rs        -- Claude Code JSONL
      codex.rs         -- Codex archived / live JSONL
      xcode.rs         -- Xcode CodingAssistant (macOS-gated)
      cursor.rs        -- Cursor (SQLite-backed via state.vscdb)
      cursor_cache.rs  -- mtime+size sidecar cache for Cursor DB parses
      opencode.rs      -- OpenCode (SQLite; schema-probing)
      pi.rs            -- Pi (JSONL; responseId last-wins dedup)
      copilot.rs       -- GitHub Copilot (mixed-format best-effort probe)

  hook/
    main.rs            -- heimdall-hook binary entry (thin wrapper)
    mod.rs             -- main_impl(): bypass check -> stdin -> parse -> SQLite INSERT OR IGNORE
    bypass.rs          -- Ancestor process walk for `--dangerously-skip-permissions`
    ingest.rs          -- Parse Claude Code hook JSON payload into live_events
    install.rs         -- hook install/uninstall/status against ~/.claude/settings.json

  optimizer/
    mod.rs             -- Detector trait, Severity, Finding, OptimizeReport, run_optimize
    grade.rs           -- compute_grade (A..F from finding severities)
    claude_md.rs       -- ClaudeMdBloatDetector (file_size * session_count waste)
    mcp.rs             -- UnusedMcpDetector (configured but never invoked)
    agents.rs          -- GhostAgentDetector (~/.claude/agents/*.md vs turns.agent_id)
    reread.rs          -- RereadDetector (same file read >=3x per session)
    bash.rs            -- BashNoiseDetector (trivial-command repetition per session)

  scheduler/
    mod.rs             -- Scheduler trait, Interval, InstallStatus, current() dispatch
    launchd.rs         -- macOS plist generation + launchctl
    cron.rs            -- Linux crontab text transformation (tagged via `# heimdall-scheduler:v1`)
    schtasks.rs        -- Windows schtasks argv builder
    daemon.rs          -- LaunchdDaemonScheduler (macOS-only always-on dashboard)

  server/
    mod.rs             -- axum router: /, /api/data, /api/rescan, /api/usage-windows,
                          /api/heatmap, /api/health, /api/stream (SSE)
    api.rs             -- Handlers with AppState (db_path, oauth cache, webhook config, scan_tx)
    tz.rs              -- Re-export of crate::tz::TzParams (flattened into axum Query structs)
    assets.rs          -- include_str! for HTML/CSS/JS
    tests.rs           -- HTTP endpoint tests

  ui/
    index.html         -- Dashboard HTML shell with mount points (incl. #activity-heatmap)
    input.css          -- Tailwind v4 entry with industrial monochrome tokens
    style.css          -- Generated CSS (committed)
    app.tsx            -- Entry point, data loading, filter logic, heatmap wiring
    app.js             -- Compiled JS (committed, do not edit directly)
    components/
      Header.tsx            -- Sticky header, theme toggle, rescan button, [REFRESHING] status
      FilterBar.tsx         -- Models, range, provider, project search (URL-persistent)
      RateWindowCard.tsx    -- Rate window / budget / unavailable cards
      EstimationMeta.tsx    -- Confidence / billing / pricing cards
      ReconciliationBlock.tsx -- OpenAI org usage reconciliation
      InlineStatus.tsx      -- Bracketed [OK] / [ERROR: ...] status
      SegmentedProgressBar.tsx -- Signature segmented progress viz
      StatsCards.tsx        -- Summary stat cards incl. Avg/Active Day, CacheEfficiencyCard
      CacheEfficiencyCard.tsx -- Cache hit-rate percentage + monochrome progress bar (Phase 21)
      ActivityHeatmap.tsx   -- 7x24 CSS-grid heatmap with monochrome opacity ladder (Phase 13)
      SubagentSummary.tsx   -- Subagent breakdown
      EntrypointTable.tsx   -- Entrypoint usage table
      ServiceTiers.tsx      -- Service tiers table
      ToolUsageTable.tsx    -- Tool invocations with inline rank bar
      McpSummaryTable.tsx   -- MCP server usage with inline rank bar
      BranchTable.tsx       -- Git branch summary with inline rank bar
      VersionTable.tsx      -- CLI version summary
      VersionDonut.tsx      -- Version distribution donut with cost/calls/tokens switcher (Phase 16)
      ApexChart.tsx         -- Generic ApexCharts wrapper
      DailyChart.tsx        -- Daily token usage bar chart
      ModelChart.tsx        -- Model distribution donut
      ProjectChart.tsx      -- Top projects horizontal bar
      HourlyChart.tsx       -- Activity by hour of day (complements the heatmap)
      Sparkline.tsx         -- 7-day trend sparkline
      SessionsTable.tsx     -- Sessions table with sort/pagination
      ModelCostTable.tsx    -- Cost by model table w/ cache-read + cache-write columns
      ProjectCostTable.tsx  -- Cost by project table
      DataTable.tsx         -- Generic tanstack/table-core wrapper
      Footer.tsx            -- Static footer
    state/
      types.ts         -- TypeScript interfaces mirroring the Rust models
      store.ts         -- Preact signals (filters, chrome, loadState, versionDonutMetric)
    lib/
      format.ts        -- Number/cost formatting utilities (incl. esc() for XSS)
      csv.ts           -- CSV export utilities
      charts.ts        -- Industrial chart options factory, monochrome color ladders
      status.ts        -- Inline [STATUS] helper (setStatus / clearStatus)
      theme.ts         -- Theme detection + apply
      rescan.ts        -- Rescan trigger factory
```

## Key Design Decisions

- **Single pricing source**: `pricing.rs` is the only place model prices are defined. The dashboard receives pre-computed costs from the API. No pricing logic in JS.
- **Integer nanos**: `calc_cost_nanos()` computes cost in billionths of a dollar (i64) to avoid f64 drift. `calc_cost()` is a thin wrapper. `CostBreakdown::total_nanos()` sums exactly.
- **Volume discounts**: `ModelPricing` has optional `threshold_tokens` + above-threshold rates. Sonnet 4.5 has a 200K threshold.
- **Pricing overrides & LiteLLM fallback**: Config can override any model's rates; the `pricing refresh` subcommand pulls the LiteLLM catalogue into `~/.cache/heimdall/litellm_pricing.json`. The 5-tier lookup preserves hardcoded Claude/GPT-5 prices — LiteLLM is consulted only after all hardcoded tiers miss.
- **Embedded assets**: HTML/CSS/JS embedded via `include_str!` at compile time.
- **TypeScript source**: `src/ui/app.tsx` and `src/ui/components/*.tsx` are the source of truth. Compiled via esbuild. Committed so `cargo build` works without Node.js.
- **Incremental scanning**: Track file mtime + line count in `processed_files` table. Skip already-processed lines.
- **Dedup correctness**: After all turn inserts, recompute session totals from turns table via `SELECT SUM(...)`. Each provider defines its own dedup key.
- **Atomic rescan**: Write to temp DB, then atomically rename. No data loss on crash.
- **OAuth caching**: Usage windows cached in `RwLock<Option<(Instant, Data)>>` for configurable interval (default 60s).
- **Subagent tracking**: `isSidechain` + `agentId` from JSONL stored as `is_subagent` + `agent_id` in turns table.
- **Provider pattern**: Every data source implements `Provider`. Registered in `providers::all()`. JSONL providers flow through `parser::parse_jsonl_file` dispatcher; SQLite/mixed-format providers bypass the dispatcher and parse directly via `Provider::parse()`.
- **Tool-event cost attribution**: Each turn's cost is split evenly across its tool invocations (remainder goes to the first event) so per-MCP / per-file cost queries are tractable. Integer-nanos math preserves the sum exactly.
- **Real-time hook**: `heimdall-hook` binary is fire-and-forget — always exits 0, always prints `{}`, ~50ms p99. It never blocks Claude Code. Bypass mode (ancestor process has `--dangerously-skip-permissions`) short-circuits the DB write.
- **Client-sent timezone**: `TzParams` flows from browser fetch -> axum handler -> SQL `datetime(timestamp, '+N minutes')` shift. One source of truth, no server TZ config needed.
- **Dual-config resolution**: `HEIMDALL_CONFIG` env -> `~/.config/heimdall/config.toml` -> `~/.claude/usage-tracker.toml` -> bundled defaults. Shared between both binaries.

## Conventions

- Use `thiserror` for error types, `anyhow` in main/CLI.
- Prefer `&str` over `String` in function signatures where possible.
- All SQL queries in `db.rs`, nowhere else.
- Tests use the `tempfile` crate for temp dirs and DB files; never touch the user's real `~/.claude/` in tests.
- No `.unwrap()` in library code (scanner, server, pricing). OK in tests and main.
- Log with `tracing`: `debug!` for per-file progress, `info!` for scan summaries, `warn!` for recoverable errors.
- No `#[allow(dead_code)]` drive-bys — every allow must name the intent in a comment.
- Rust edition 2024; MSRV matches CI matrix.

## Common Tasks

### Adding a new model to pricing

Edit `pricing.rs` only. Add to `PRICING_TABLE`. Set `threshold_tokens: None` unless it has volume discounts. Tests verify the lookup logic. If you want the model auto-updated from LiteLLM for long-tail pricing, add it to the LiteLLM passthrough list rather than hardcoding — but Claude and GPT-5 families MUST stay hardcoded.

### Adding a new JSONL field

1. Add the field to the `Turn` or `Session` struct in `models.rs`.
2. Parse it in `parser.rs` (Claude path) and/or the relevant provider module.
3. Add a column migration in `scanner/db.rs` (ALTER TABLE with `has_column` guard).
4. Update `insert_turns` / `upsert_sessions` to persist it.
5. Expose via API in `server/api.rs` if needed by the dashboard.
6. Update `src/ui/state/types.ts` + the relevant `.tsx` if it should appear in the UI, then `npm run build:ui`.

### Adding a new API endpoint

1. Add handler in `server/api.rs`.
2. Add route in `server/mod.rs`.
3. Add test in `server/tests.rs` (include the route in `test_app()`).
4. If the endpoint does time-bucket aggregation, accept `TzParams` via `Query` (flatten — axum's `Query` doesn't compose `#[serde(flatten)]` cleanly).

### Adding a new scanner provider

Providers live at `src/scanner/providers/<name>.rs` and implement `crate::scanner::provider::Provider`:

```rust
pub trait Provider: Send + Sync {
    fn name(&self) -> &'static str;
    fn discover_sessions(&self) -> anyhow::Result<Vec<SessionSource>>;
    fn parse(&self, path: &Path) -> anyhow::Result<Vec<Turn>>;
}
```

Steps:

1. Create `src/scanner/providers/<name>.rs` with a struct (`FooProvider`) and a trivial `new()` that resolves default discovery directories (e.g. from `HOME`).
2. Implement `name()` returning a stable slug like `"foo"`. This slug is stored in `turns.provider` and `sessions.provider` and surfaces in the dashboard's Provider filter.
3. Implement `discover_sessions()` by walking the appropriate filesystem path and returning one `SessionSource` per session file.
4. Implement `parse()` — either delegate to `parser::parse_claude_jsonl_file` if the format is Claude-compatible (the dispatcher does the retag) or write a dedicated parser in the provider module.
5. Register the provider in `src/scanner/providers/mod.rs` inside `pub fn all()`. Platform-gate via `#[cfg(target_os = "...")]` on the `push` call only – never on the whole function.
6. Tests go in `src/scanner/tests.rs`. Minimum coverage: `name()` returns the expected slug, and a fixture-based parse test asserts returned `Turn`s carry the provider tag.

The explicit `--projects-dir` CLI override routes through `provider_for_dir()` in `src/scanner/mod.rs` — update that helper if the new provider needs path-based detection from that override surface.

Three provider backend categories exist — choose the matching template:

- **JSONL-backed** (Claude, Codex, Xcode, Pi): one record per line; `parse_jsonl_file` in `parser.rs` dispatches to the per-provider parser. Pi uses `responseId` dedup (last-wins); Claude/Xcode use `message.id` dedup; Codex uses cumulative token cross-check keyed on `turn_id`.
- **SQLite-backed** (Cursor, OpenCode): open the DB read-only via `rusqlite`; probe for tables before querying; return empty on missing schema. The `parse_jsonl_file` dispatcher is bypassed — call `Provider::parse()` directly. See `cursor.rs` and `opencode.rs`. Use `cursor_cache.rs` for mtime+size sidecar invalidation.
- **Mixed-format / best-effort probe** (Copilot): format varies by IDE and is not publicly documented; probe for JSON or JSONL files and look for recognizable usage fields; always return `Ok(Vec::new())` when the format is unrecognized — never error or panic. Document the uncertainty in the module header.

### Adding a waste detector

Detectors live at `src/optimizer/<name>.rs` and implement `Detector`:

```rust
pub trait Detector {
    fn name(&self) -> &'static str;
    fn run(&self, conn: &Connection) -> anyhow::Result<Vec<Finding>>;
}
```

Register in `optimizer/mod.rs::run_optimize_with_overrides`. Severity thresholds and monthly-waste estimates are detector-specific – document the formula inline. Grades fold via `grade::compute_grade`.

### Changing the database schema

Always use additive migrations (ALTER TABLE ADD COLUMN). Check for column existence with `has_column` before adding. Never drop columns or tables in migrations – only in full rescan. If you introduce a new default for existing rows, add an idempotent `UPDATE ... WHERE column IS NULL OR column = ''` after the ADD COLUMN. Covers both freshly-created DBs and mid-upgrade ones.

### Config file changes

1. Add field to the appropriate struct in `config.rs` (with `#[serde(default)]`).
2. Extract in `main.rs` before the match.
3. Pass through to where it's needed (server, scanner, etc.).
4. Add test for parsing in `config.rs` tests.
5. If the field lives under a new TOML section, prefer a name that doesn't collide with existing key-valued sections (e.g. `[pricing]` is already a table of overrides; the LiteLLM toggle uses `[pricing_source]`).

## Dashboard UI

When editing dashboard files (`src/ui/`), follow the design skill at `.claude/skills/industrial-design/SKILL.md`. Key rules:

- Monochrome canvas; single red accent (`#D71921`) per screen for urgent/destructive only.
- Numbers in Space Mono (tabular numerals); body in Space Grotesk; Doto for hero display.
- No gradients, no shadows, no toast popups — use inline `[SAVED]` / `[ERROR: ...]` status.
- Dark (OLED `#000`) and light (warm off-white `#F5F5F5`) both first-class via CSS variables.
- XSS protection: all dynamic text through `esc()` in `src/ui/lib/format.ts`.
- Recompile after changes: `npm run build:ui`. Commit `app.js` + `style.css` alongside the source.
- Heatmap and card intensity use opacity on `--color-text-primary`, never a color ramp.
- Rank bars on summary tables: background `<div>` at `~12%` opacity, text layered above via `z-index`.

## Release

See [.github/RELEASING.md](.github/RELEASING.md). Tags matching `v*.*.*` trigger a 5-target cross-build plus a `lipo`-merged universal macOS artifact. The Homebrew cask skeleton lives at `packaging/homebrew/heimdall.rb` and is copied into the separately-maintained tap repository at release time.
