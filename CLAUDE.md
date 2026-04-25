# Heimdall (`claude-usage-tracker`) – Development Guide

## Project

Local AI session observability for Claude Code, Codex, Cursor, OpenCode, Pi, Copilot, Xcode CodingAssistant, and Cowork. Two Rust binaries share one crate:

- `claude-usage-tracker` – CLI + embedded web dashboard.
- `heimdall-hook` – lightweight stdin-driven hook binary for real-time PreToolUse ingest.

## Claude Workflow Assets

Claude Code mirrors the Codex Rust workflow skills through repo-local agent and command prompts:

- `.claude/agents/rust-test-runner.md` mirrors `heimdall-rust-test-runner`
- `.claude/agents/pr-reviewer.md` mirrors `heimdall-pr-review`
- `.claude/commands/fix-unwraps.md` mirrors `heimdall-fix-unwraps`
- `.claude/commands/audit-deps.md` mirrors `heimdall-rust-dependency-audit`
- `.claude/commands/binary-audit.md` mirrors `heimdall-rust-binary-audit`

## Build & Run

```bash
# TypeScript (dashboard UI) -- only needed when modifying src/ui/*.tsx
npm install                                    # one-time: install deps
npm run build:ts                               # compile TSX -> JS
npm run build:css                              # compile Tailwind -> CSS
npm run build:ui                               # both JS + CSS

# Rust
cargo build                    # debug build (produces both binaries)
cargo build --release          # release build
cargo run -- dashboard         # scan + start dashboard
cargo run -- dashboard --watch # + live file-watcher auto-refresh
cargo run -- today             # today's usage
cargo run -- today --json      # JSON output for scripting
cargo run -- stats             # all-time stats
cargo run -- stats --json      # JSON output
cargo run -- scan              # scan only
cargo run -- export --format=csv --period=month --output=out.csv
cargo run -- optimize --format=json
cargo run -- scheduler install     # launchd / cron / schtasks
cargo run -- daemon install        # macOS-only always-on dashboard
cargo run -- hook install          # wire heimdall-hook into ~/.claude/settings.json
cargo run -- db reset --yes        # destructive DB wipe (TTY-guarded)
cargo run -- menubar               # SwiftBar-formatted output
cargo run -- pricing refresh       # pull LiteLLM catalogue into the cache
```

The compiled `src/ui/app.js` and `src/ui/style.css` are committed to git so `cargo build` works without Node.js installed. Only re-run `npm run build:ui` after editing `src/ui/*.tsx` or `src/ui/input.css`.

## Test

```bash
cargo test                        # full suite across 4 suites (572+ tests)
cargo test scanner                # scanner module tests
cargo test pricing                # pricing + LiteLLM + cost-breakdown tests
cargo test oauth                  # OAuth module tests
cargo test config                 # config tests
cargo test webhooks               # webhook tests
cargo test cli_tests              # CLI command tests
cargo test optimizer              # 5 waste-detector tests
cargo test scheduler              # launchd / cron / schtasks tests
cargo test hook                   # heimdall-hook pipeline tests
cargo test classifier             # 13-category task classifier tests
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
  lib.rs               -- Library root; both binaries share this
  main.rs              -- Primary CLI (clap): scan/today/stats/dashboard/export/
                          optimize/scheduler/daemon/hook/db/menubar/pricing
  cli_tests.rs         -- CLI command tests
  config.rs            -- TOML config + dual-path resolver
                          ($HEIMDALL_CONFIG -> ~/.config/heimdall -> ~/.claude/usage-tracker.toml)
  models.rs            -- Shared types (Session, Turn, ToolEvent, CacheEfficiency, etc.)
  pricing.rs           -- Pricing table, calc_cost_nanos, volume discounts, 4-way CostBreakdown,
                          5-tier lookup with hardcoded Claude/GPT-5 priority + LiteLLM fallback
  currency.rs          -- Frankfurter USD->N conversion, 24h disk cache, hardcoded fallback
  litellm.rs           -- LiteLLM catalogue fetch + cache + `pricing refresh` entry
  tz.rs                -- TzParams for timezone-aware SQL bucketing
  export.rs            -- `export` subcommand (csv / json / jsonl), period filtering
  menubar.rs           -- SwiftBar widget renderer + injection-hardened sanitizer
  db.rs                -- `db reset` TTY-guarded destructive command
  webhooks.rs          -- Fire-and-forget webhook POSTs on session depletion / cost threshold /
                          agent status transitions (agent_status_degraded / agent_status_restored)
  openai.rs            -- OpenAI organization usage reconciliation client

  agent_status/
    mod.rs             -- poll() orchestrator + poll_with_injection() test seam + AlertDirection
    client.rs          -- Claude HTTP client (ETag/If-None-Match) + OpenAI two-call client
    models.rs          -- AgentStatusSnapshot, ProviderStatus, StatusIndicator, raw wire structs
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
    tests.rs           -- Integration tests for scan pipeline
    classifier.rs      -- 13-category task classifier (pure RegexSet)
    oneshot.rs         -- Edit->Bash->Edit retry-cycle detection
    cowork.rs          -- Ephemeral Cowork label resolution from audit.jsonl
    usage_limits.rs    -- Parses ~/.claude/**/*-usage-limits into rate_window_history
    watcher.rs         -- `notify`-backed file watcher w/ 2s debounce (--watch flag)
    provider.rs        -- `Provider` trait + `SessionSource`
    providers/
      mod.rs           -- Central registry: pub fn all() -> Vec<Box<dyn Provider>>
      claude.rs        -- Claude Code JSONL
      codex.rs         -- Codex archived / live JSONL
      xcode.rs         -- Xcode CodingAssistant (macOS-gated)
      cursor.rs        -- Cursor (SQLite state.vscdb, schema-probing)
      cursor_cache.rs  -- mtime+size sidecar cache for Cursor DB parses
      opencode.rs      -- OpenCode (SQLite)
      pi.rs            -- Pi (JSONL, responseId last-wins)
      copilot.rs       -- GitHub Copilot (mixed-format best-effort probe)

  hook/
    main.rs            -- heimdall-hook binary entry (thin wrapper)
    mod.rs             -- main_impl(): bypass -> stdin -> parse -> SQLite INSERT OR IGNORE
    bypass.rs          -- Ancestor process walk for `--dangerously-skip-permissions`
    ingest.rs          -- Parse Claude Code hook JSON payload into live_events
    install.rs         -- hook install/uninstall/status against ~/.claude/settings.json

  optimizer/
    mod.rs             -- Detector trait, Severity, Finding, OptimizeReport, run_optimize
    grade.rs           -- compute_grade (A..F from finding severities)
    claude_md.rs       -- ClaudeMdBloatDetector
    mcp.rs             -- UnusedMcpDetector
    agents.rs          -- GhostAgentDetector
    reread.rs          -- RereadDetector (same file read >=3x per session)
    bash.rs            -- BashNoiseDetector (repeated trivial commands per session)

  scheduler/
    mod.rs             -- Scheduler trait, Interval, InstallStatus, current() dispatch
    launchd.rs         -- macOS plist generation + launchctl
    cron.rs            -- Linux crontab text transformation (`# heimdall-scheduler:v1` tag)
    schtasks.rs        -- Windows schtasks argv builder
    daemon.rs          -- LaunchdDaemonScheduler (macOS-only always-on dashboard)

  server/
    mod.rs             -- axum router: /, /api/data, /api/rescan, /api/usage-windows,
                          /api/heatmap, /api/health, /api/stream (SSE)
    api.rs             -- Handlers with AppState (db_path, oauth cache via RwLock, webhook config)
    tz.rs              -- Re-export of crate::tz::TzParams
    assets.rs          -- include_str! for HTML/CSS/JS
    tests.rs           -- HTTP endpoint tests

  ui/
    index.html         -- Dashboard HTML shell (embeds compiled CSS + JS)
    input.css          -- Tailwind v4 entry with Apple-Swiss refined tokens
    style.css          -- Generated CSS (committed)
    app.tsx            -- Entry point, data loading, filter logic, heatmap wiring
    app.js             -- Compiled JS (committed, do not edit directly)
    components/
      Header.tsx                  -- Sticky header, theme toggle, rescan button, [REFRESHING]
      FilterBar.tsx               -- Models, range, provider, project search (URL-persistent)
      RateWindowCard.tsx          -- Rate window / budget / unavailable cards
      EstimationMeta.tsx          -- Confidence / billing / pricing cards
      ReconciliationBlock.tsx     -- OpenAI org usage reconciliation
      InlineStatus.tsx            -- Bracketed [OK] / [ERROR: ...] status
      SegmentedProgressBar.tsx    -- Signature segmented progress viz
      StatsCards.tsx              -- Summary cards incl. Avg/Active Day + CacheEfficiencyCard
      CacheEfficiencyCard.tsx     -- Cache hit-rate percentage + monochrome progress bar
      ActivityHeatmap.tsx         -- 7x24 CSS-grid heatmap with monochrome opacity ladder
      SubagentSummary.tsx         -- Subagent breakdown
      EntrypointTable.tsx         -- Entrypoint usage table
      ServiceTiers.tsx            -- Service tiers table
      ToolUsageTable.tsx          -- Tool invocations (inline rank bar)
      McpSummaryTable.tsx         -- MCP server usage (inline rank bar)
      BranchTable.tsx             -- Git branch summary (inline rank bar)
      VersionTable.tsx            -- CLI version summary
      VersionDonut.tsx            -- Version donut w/ cost|calls|tokens metric switcher
      ApexChart.tsx               -- Generic ApexCharts wrapper
      DailyChart.tsx              -- Daily token usage bar chart
      ModelChart.tsx              -- Model distribution donut
      ProjectChart.tsx            -- Top projects horizontal bar
      HourlyChart.tsx             -- Activity by hour of day
      Sparkline.tsx               -- 7-day trend sparkline
      SessionsTable.tsx           -- Sessions table with sort/pagination
      ModelCostTable.tsx          -- Cost by model w/ cache-read + cache-write columns
      ProjectCostTable.tsx        -- Cost by project table
      DataTable.tsx               -- Generic tanstack/table-core wrapper
      Footer.tsx                  -- Static footer
    state/
      types.ts         -- TypeScript interfaces (mirrors Rust models)
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

- **Two binaries, one crate**: `claude-usage-tracker` (CLI + dashboard) and `heimdall-hook` (real-time ingest) share `lib.rs`. The hook binary is fire-and-forget: always exits 0, always prints `{}`, ~50ms p99.
- **Single pricing source**: `pricing.rs` is the only place model prices are defined. The dashboard receives pre-computed costs from the API. No pricing logic in JS.
- **Integer nanos**: `calc_cost_nanos()` computes cost in billionths of a dollar (i64) to avoid f64 drift. `calc_cost()` is a thin wrapper. `CostBreakdown::total_nanos()` sums exactly.
- **5-tier pricing fallback**: exact hardcoded -> prefix hardcoded -> keyword hardcoded -> LiteLLM cache -> unknown. Claude and GPT-5 families NEVER fall through to LiteLLM even if the cache has a conflicting entry.
- **Volume discounts**: `ModelPricing` has optional `threshold_tokens` + above-threshold rates. Sonnet 4.5 has a 200K threshold.
- **Pricing overrides**: Config file can override any model's rates. Applied via `OnceLock<HashMap>` at startup.
- **Embedded assets**: HTML/CSS/JS embedded via `include_str!` at compile time.
- **TypeScript source**: `src/ui/app.tsx` and `src/ui/components/*.tsx` are the source of truth. Compiled via esbuild. Committed so `cargo build` works without Node.js.
- **Incremental scanning**: Track file mtime + line count in `processed_files` table. Skip already-processed lines.
- **Dedup correctness**: After all turn inserts, recompute session totals from turns table via `SELECT SUM(...)`. Each provider defines its own dedup key.
- **Atomic rescan**: Write to temp DB, then atomically rename. No data loss on crash.
- **OAuth caching**: Usage windows cached in `RwLock<Option<(Instant, Data)>>` for configurable interval (default 60s).
- **Subagent tracking**: `isSidechain` + `agentId` from JSONL stored as `is_subagent` + `agent_id` in turns table.
- **Provider pattern**: Every data source implements `Provider`. Registered in `providers::all()`. JSONL providers flow through `parser::parse_jsonl_file` dispatcher; SQLite/mixed-format providers parse directly via `Provider::parse()`.
- **Tool-event cost attribution**: Each turn's cost is split evenly across its tool invocations; remainder goes to the first event. Enables per-MCP and per-file cost queries.
- **Client-sent timezone**: Browser sends `tz_offset_min`; SQL applies `datetime(timestamp, '+N minutes')` shift before `strftime` bucketing. No server TZ config.
- **Embedded version stamps on install surfaces**: every persistent surface heimdall installs (hook entry in `~/.claude/settings.json` via `_heimdall_version` key, statusline entry via `_heimdall_statusline_version` key, cron tag `# heimdall-scheduler:v1 (heimdall X.Y.Z)`, launchd plist `<!-- heimdall X.Y.Z -->` comment) carries `env!("CARGO_PKG_VERSION")` so users can `grep heimdall <surface>` to answer "what version is installed?" without running a status command. Pattern follows talk-normal's `<!-- talk-normal X.Y.Z -->` convention. Ownership detection uses version-independent markers (`HOOK_DESCRIPTION`, `STATUSLINE_VERSION_KEY` presence, `CRON_TAG` substring, plist filename + Label) so newer binaries cleanly uninstall entries written by older ones. New install surfaces (Windows schtasks, macOS daemon) should follow this convention.
- **Replace-in-place install idempotency**: `hook install`, `statusline-hook install`, and `mcp install` are *idempotent in the talk-normal sense* — every re-run reaches the current state, refreshing the binary path, command, and version stamp from improvement (5). First run returns `Installed`; subsequent runs return `Updated` (no `AlreadyInstalled` no-op variant — distinguishing "already current" from "refreshed to current" is more machinery than the user-visible signal warrants). MCP additionally refuses to overwrite a user-customized `heimdall` entry (one without our sentinel) — that safety property is orthogonal to idempotency. Cron and launchd schedulers already self-update via their existing strip-then-append flow.

## Conventions

- Use `thiserror` for error types, `anyhow` in main/CLI.
- Prefer `&str` over `String` in function signatures where possible.
- All SQL queries in `scanner/db.rs`, nowhere else.
- Tests use the `tempfile` crate for temp dirs and DB files; never touch the user's real `~/.claude/` in tests.
- No `.unwrap()` in library code (scanner, server, pricing). OK in tests and main.
- Log with `tracing`: `debug!` for per-file progress, `info!` for scan summaries, `warn!` for recoverable errors.
- Rust edition 2024.

## Common Tasks

See [AGENTS.md](AGENTS.md) for the full "Adding X" playbook covering: new models, new JSONL fields, new API endpoints, new scanner providers (JSONL / SQLite / mixed-format), new waste detectors, database schema migrations, and config file changes.

## Dashboard UI

When editing dashboard files (`src/ui/`), follow the repo-scoped design skill at `.agents/skills/industrial-design/SKILL.md`. The legacy `.claude/skills/industrial-design/SKILL.md` copy remains for Claude compatibility. Key rules:

- **Typography:** Inter for UI, headings, body. Geist Mono for numbers, code, tabular columns. No Space Grotesk, no Space Mono, no Doto.
- **Canvas:** dark `#0A0A0A`, light `#F5F5F5` (warm off-white). Both first-class via CSS variables; never hardcode hex values.
- **Accents:** `--accent-interactive` blue-gray `#4A7FA5` is the primary interactive affordance (links, selected states, primary buttons). `--accent` red `#D71921` is reserved for semantic error / destructive / over-limit only. Status colors (`--success`, `--warning`) unchanged.
- **Hierarchy:** sentence-case throughout. ALL-CAPS monospace is reserved for `<th>` table column headers only — stat card labels, section titles, filter labels, chart titles use 11–12px sentence-case in `--text-secondary`.
- **Concentric radii:** nested shapes share a center point (`inner_radius = outer_radius - padding`). Card-within-card means inner radius follows from outer radius + padding; don't pick independent values.
- **Structure:** no gradients. No shadows on content surfaces — Liquid Glass translucency is acceptable only on the sticky header (navigation-layer chrome). Content surfaces stay flat with 1px border separation. No toast popups — use inline `[SAVED]` / `[ERROR: ...]` bracket status text near the trigger.
- **Data-viz:** smooth progress bars with 2px rounded ends, color-encoded by threshold (not segmented LED-meter geometry). Category differentiation via opacity (100/60/30) or pattern before color. Tabular numerals via `font-feature-settings: "tnum"`.
- **XSS protection:** all dynamic text through `esc()` in `src/ui/lib/format.ts`.
- **Rebuild after changes:** `npm run build:ui`. Commit `app.js` + `style.css` alongside source.
- **Heatmap / card intensity:** opacity on `--color-text-primary`, never a color ramp.

## Release

See [.github/RELEASING.md](.github/RELEASING.md). Tags matching `v*.*.*` trigger a 5-target cross-build plus a `lipo`-merged universal macOS artifact. The Homebrew cask skeleton lives at `packaging/homebrew/heimdall.rb` and is copied into the separately-maintained `heimdall/homebrew-tap` repo at release time.
