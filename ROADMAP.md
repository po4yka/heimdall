# Heimdall -- Roadmap

Detailed phased plan for the next evolution of `claude-usage-tracker`. Each phase is self-contained: it lands in `main` behind no feature flags, adds tests, and leaves the tree green.

Status legend: **[ ]** not started · **[~]** in progress · **[x]** done

Completed phases (Phases 0--22 from prior cycles sourced from Codeburn, Third-Eye, and Claude-Guardian) have been removed from this file. See `git log` or README "Prior Art & Acknowledgements" for historical attribution.

---

## Guiding Principles

- **Additive only.** Nothing in this roadmap replaces existing Heimdall logic. The current pricing engine, OAuth rate-window monitor, SQLite scanner, and webhook system stay as-is.
- **Rust-native.** No re-introduction of Node as a runtime. Frontend stays build-time only.
- **Deterministic.** No LLM calls at runtime. Classification, waste detection, and one-shot tracking must be reproducible across runs.
- **Integer-nanos everywhere.** Cost math never uses `f64`; display is the only place floats appear.
- **Zero new runtime deps unless justified.** Each added crate requires a one-line rationale in the PR description.
- **Tests land with code.** Each phase's acceptance gate includes "tests passing and coverage not regressed."

---

## Sources & References

This roadmap cycle harvests patterns from one sibling project. Each phase names a concrete source file for future implementation work.

| Project | Repo | Language | Role |
|---------|------|----------|------|
| **ccusage** | https://github.com/ryoppippi/ccusage | TypeScript pnpm monorepo (CLI + MCP + Amp) | Modern CLI analytics with distinct strengths in billing-window modeling and Claude Code integration. Source of the 5-hour billing-block burn-rate + projection engine, `statusline` PostToolUse hook command, context-window tracking, MCP server for inference-time usage queries, JSON-schema config with per-command overrides, project aliasing, Amp credit tracking, and `--jq` post-processing. Source last reviewed: 2026-04-18 (upstream commit `7258c34`). |

When porting, prefer reading the source file directly over reimplementing from memory. Each "Source:" line points at the canonical file in the referenced repo.

---

## Phase 1 -- 5-Hour Billing Blocks + Burn Rate + End-of-Block Projection **[ ]**

**Motivation:** Claude charges in 5-hour billing windows; Heimdall shows cumulative daily/monthly cost but never answers "am I going to blow my quota before this block resets?" ccusage explicitly models these windows with live burn rate and projected end-of-block cost -- the single highest-value analytics gap.

**Source:** [ccusage/apps/ccusage/src/commands/blocks.ts](https://github.com/ryoppippi/ccusage/blob/main/apps/ccusage/src/commands/blocks.ts), [ccusage/apps/ccusage/src/_session-blocks.ts](https://github.com/ryoppippi/ccusage/blob/main/apps/ccusage/src/_session-blocks.ts) -- `identifySessionBlocks()` groups turn events into 5h windows anchored on first activity; `calculateBurnRate()` returns `tokensPerMinute` + `costPerHour` over elapsed time in the active block; `projectBlockUsage()` linearly extrapolates to block end.

### TODO

1. **Pure analytics module.** `src/analytics/blocks.rs` with:
   ```rust
   pub struct BillingBlock { pub start: DateTime<Utc>, pub end: DateTime<Utc>,
       pub tokens: TokenBreakdown, pub cost_nanos: i64, pub models: Vec<String>,
       pub is_active: bool, pub entry_count: u32 }
   pub struct BurnRate { pub tokens_per_min: f64, pub cost_per_hour_nanos: i64 }
   pub struct Projection { pub projected_cost_nanos: i64, pub projected_tokens: u64 }
   pub fn identify_blocks(turns: &[Turn], session_hours: f64) -> Vec<BillingBlock>;
   pub fn calculate_burn_rate(block: &BillingBlock, now: DateTime<Utc>) -> BurnRate;
   pub fn project_block_usage(block: &BillingBlock, rate: &BurnRate) -> Projection;
   ```
2. **Block anchor semantics.** Mirror ccusage: a block starts at the first turn's `startedAt` floored to the hour, extends `session_hours` (default 5.0), and closes on the first turn that falls outside. Gaps longer than `session_hours` start a new block.
3. **Integer-nanos discipline.** `cost_per_hour_nanos` computed as `cost_nanos * 3600_000_000_000 / elapsed_micros`; no `f64` in cost math (unlike ccusage, which is free to use JS numbers).
4. **SQL helper.** `scanner/db.rs::load_turns_in_range(db, since, until) -> Vec<TurnForBlocks>` returning only the columns needed by the analytics module.
5. **CLI subcommand.** `claude-usage-tracker blocks [--session-length=5] [--active] [--json]`. Table: `BLOCK START | ELAPSED | COST | TOKENS | MODELS | STATUS`.
6. **Tests.** Fixtures: back-to-back activity, mid-block idle gap, cross-day UTC boundary, empty DB.

### Acceptance

- Block grouping matches ccusage output byte-for-byte on a 500-turn fixture (port `_session-blocks.test.ts` cases).
- `calculate_burn_rate` on a half-elapsed block returns the expected tokens/min within ±1 ulp.
- `--json` round-trips via `serde_json`.

### Effort: M (2--3 days)

---

## Phase 2 -- Token Quota Percentage + Inline REMAINING/PROJECTED Rows **[ ]**

**Motivation:** Knowing the burn rate is only half the battle; users also need "how close am I to the quota ceiling?" ccusage's blocks view injects live REMAINING and PROJECTED rows inline under the active block when `--token-limit` is set, with red/warning coloring near or past the limit.

**Source:** [ccusage/apps/ccusage/src/commands/blocks.ts](https://github.com/ryoppippi/ccusage/blob/main/apps/ccusage/src/commands/blocks.ts) lines 352--467 -- `tokenLimit` arg accepts a number or the string `max` (reuses the historical peak), and the renderer emits two pseudo-rows under the active block. Threshold coloring: green <50%, yellow 50--80%, red >80%.

### TODO

1. **CLI flag.** `blocks --token-limit=<N|max>`. When `max`, resolve against `SELECT MAX(tokens) FROM billing_blocks_historical`.
2. **Pseudo-row renderer.** Extend the `blocks` table writer to emit two extra rows under the active block: `→ REMAINING` and `→ PROJECTED`, each with a `%` column.
3. **Threshold coloring.** Add `src/ui_cli/severity.rs` with `severity_for_pct(pct) -> Severity { Ok | Warn | Danger }`; ANSI-style via `owo-colors` (no new dep if already present; else add).
4. **JSON parity.** When `--json` is set, serialize `{"remaining": {...}, "projected": {...}, "quota_pct": 0.73}` as siblings of the block object.
5. **Dashboard card.** `BillingBlocksCard.tsx` adds a `SegmentedProgressBar` for the quota %; red accent only when crossing the danger threshold (matches industrial design rule).

### Acceptance

- With `--token-limit=1000000` on a block at 750K tokens, table shows `75% (WARN)` on REMAINING and a projected % based on burn rate.
- `--token-limit=max` resolves to the historical maximum and never panics on empty history.

### Effort: S (1 day, after Phase 1)

---

## Phase 3 -- Weekly Aggregation Report **[ ]**

**Motivation:** Teams on sprint cadences need ISO-week views; Heimdall ships daily and monthly but no weekly. ccusage has a first-class `weekly` command with configurable start-of-week.

**Source:** [ccusage/apps/ccusage/src/commands/weekly.ts](https://github.com/ryoppippi/ccusage/blob/main/apps/ccusage/src/commands/weekly.ts) -- `--startOfWeek` enum (`sunday|monday|...`), `--instances`, `--breakdown`. Reuses the `_daily-grouping` aggregator with a different key function.

### TODO

1. **SQL bucket.** Extend `scanner/db.rs` with `fn sum_by_week(db, tz: TzParams, start_of_week: Weekday) -> Vec<WeekRow>`. Uses SQLite `strftime('%Y-%W', datetime(...))` with an offset for the chosen start-of-week.
2. **CLI subcommand.** `claude-usage-tracker weekly [--start-of-week=monday] [--json] [--breakdown]`.
3. **Dashboard toggle.** Add a `week` option to the existing period segmented control in `FilterBar.tsx`; SQL reuses the new helper.
4. **API endpoint.** `/api/data?period=week&week_starts_on=1` returns the weekly bucket.

### Acceptance

- ISO-8601 week boundaries correct across the 2027 year-end transition.
- `--start-of-week=sunday` shifts all buckets by one day vs the default.

### Effort: S (1 day)

---

## Phase 4 -- `statusline` PostToolUse Hook Command **[ ]**

**Motivation:** Claude Code's status bar is the highest-visibility surface for live usage feedback. ccusage's `statusline` reads the PostToolUse hook JSON from stdin and emits a single compact line for Claude Code to render. Complements Heimdall's existing PreToolUse `heimdall-hook` (which writes to the DB); `statusline` is the *read* side.

**Source:** [ccusage/apps/ccusage/src/commands/statusline.ts](https://github.com/ryoppippi/ccusage/blob/main/apps/ccusage/src/commands/statusline.ts) (577 lines), [ccusage/apps/ccusage/src/_types.ts](https://github.com/ryoppippi/ccusage/blob/main/apps/ccusage/src/_types.ts) `statuslineHookJsonSchema`. Key pieces: stdin reader with timeout, hybrid time+mtime cache with PID semaphore (avoid redundant recomputation when hook fires per keystroke), layout composer.

### TODO

1. **Subcommand.** `claude-usage-tracker statusline [--refresh-interval=30] [--cost-source=auto] [--offline]`. Reads JSON from stdin, writes one line to stdout, exits 0. Never blocks longer than `--refresh-interval` seconds.
2. **Input schema.** `src/statusline/hook_input.rs` mirrors `statuslineHookJsonSchema`: `session_id`, `transcript_path`, `model`, `cost`, `context_window { total_input_tokens, context_window_size }`.
3. **Output layout.** `MODEL | $SESSION / $TODAY / $BLOCK (Xh Ym left) | $/hr | N tokens (XX%)`. No emoji by default (industrial style); allow `--emoji` opt-in for parity.
4. **Cache.** `~/.cache/heimdall/statusline.json` with `{ session_id, last_computed_at, last_transcript_mtime, payload }`. PID semaphore: write `~/.cache/heimdall/statusline.lock` atomically with `O_EXCL`; stale-lock detection via PID existence check.
5. **Hook installer extension.** `claude-usage-tracker hook install` grows `--statusline` flag that additionally writes the `statusLine` entry to `~/.claude/settings.json` (tagged `# heimdall-statusline:v1` for clean uninstall).

### Acceptance

- On synthetic PostToolUse JSON, output matches the ccusage layout minus emoji.
- Statusline respects `--offline`: no network calls, no LiteLLM fetch, no currency conversion — pure local.
- p99 latency <50ms on a warm cache (matches heimdall-hook's SLO).

### Effort: L (3--4 days)

---

## Phase 5 -- Context-Window Usage Tracking **[ ]**

**Motivation:** The single most user-visible gap in ccusage vs other tools: knowing "how full is my context window?" lets users compact before hitting the limit. ccusage surfaces it in the statusline as `N tokens (XX%)` with green/yellow/red thresholds.

**Source:** [ccusage/apps/ccusage/src/commands/statusline.ts](https://github.com/ryoppippi/ccusage/blob/main/apps/ccusage/src/commands/statusline.ts) lines 460--506. Reads `context_window.total_input_tokens / context_window.context_window_size` from the hook payload; falls back to computing context tokens from the transcript JSONL when the hook doesn't supply it.

### TODO

1. **Parser.** `src/statusline/context_window.rs::from_hook(&HookInput) -> Option<ContextWindow>` and `::from_transcript(&Path) -> Result<ContextWindow>` — the transcript fallback sums `input_tokens + cache_read_tokens` across the most recent assistant turn.
2. **Thresholds.** Config-driven: `statusline.context_low_threshold = 0.50`, `statusline.context_medium_threshold = 0.80`.
3. **Dashboard card.** `ContextWindowCard.tsx` when a currently-open transcript path is available (derived from newest mtime JSONL per project). Shows `45,231 / 200,000 (22%)` with a segmented bar.
4. **Hook payload extension.** `heimdall-hook` persists the `context_window` block into `live_events.context_input_tokens` + `context_window_size` (new columns; schema migration).

### Acceptance

- Given a transcript with known turn tokens, `from_transcript` returns within 1% of the hook-reported value.
- Dashboard card hides gracefully when no transcript is currently open.

### Effort: M (2 days; depends on Phase 4 for payload schema)

---

## Phase 6 -- MCP Server for Inference-Time Usage Queries **[ ]**

**Motivation:** Claude itself can query Heimdall during a session if Heimdall exposes an MCP server — novel integration that no other observability tool in the reference set offers. ccusage ships a standalone `@ccusage/mcp` package with 6 tools over stdio + Streamable HTTP.

**Source:** [ccusage/apps/mcp/src/mcp.ts](https://github.com/ryoppippi/ccusage/blob/main/apps/mcp/src/mcp.ts), [ccusage/apps/mcp/src/ccusage.ts](https://github.com/ryoppippi/ccusage/blob/main/apps/mcp/src/ccusage.ts), [ccusage/apps/mcp/src/codex.ts](https://github.com/ryoppippi/ccusage/blob/main/apps/mcp/src/codex.ts) -- `createMcpHttpApp` mounts a Hono app at `/mcp` with `StreamableHTTPTransport`; `createMcpStdioServer` handles stdio. 6 tools: `daily`, `session`, `monthly`, `blocks`, `codex-daily`, `codex-monthly`.

### TODO

1. **Crate choice.** Use [`rmcp`](https://crates.io/crates/rmcp) (official Rust MCP SDK). Add under a `mcp` cargo feature to keep default binary small.
2. **Transports.** Both stdio (default) and HTTP-SSE via `axum` subrouter at `/mcp` (reuse the existing dashboard server).
3. **Tools.** Mirror ccusage's 6 tools + add Heimdall-specific ones: `today`, `blocks_active`, `optimize_grade`, `rate_window_snapshot`, `weekly`. Each tool's input schema is a Rust struct with `#[derive(JsonSchema)]`.
4. **Handlers.** Thin wrappers around existing `stats::*` / `optimize::*` / analytics functions; reuse integer-nanos types and serialize identically to the REST API.
5. **Subcommand.** `claude-usage-tracker mcp [--transport=stdio|http]` and `claude-usage-tracker mcp install` (writes tagged entry into `~/.claude/.mcp.json` and/or `~/.cursor/mcp.json`).
6. **Tests.** In-process client via `rmcp`'s test harness hitting each tool.

### Acceptance

- `claude mcp add heimdall -- claude-usage-tracker mcp` registers the server and `daily` tool returns JSON matching REST `/api/data?period=day`.
- HTTP transport returns `Content-Type: text/event-stream` and survives a 60s idle without disconnecting.

### Effort: L (4--5 days; new crate territory)

---

## Phase 7 -- Visual Burn-Rate Indicator **[ ]**

**Motivation:** Raw $/hr is less scannable than a visual. ccusage maps burn rate to Normal/Moderate/High tiers with 🟢/⚠️/🚨 glyphs (or bracketed `[Normal]`/`[Moderate]`/`[High]` text) in the statusline.

**Source:** [ccusage/apps/ccusage/src/commands/statusline.ts](https://github.com/ryoppippi/ccusage/blob/main/apps/ccusage/src/commands/statusline.ts) lines 115--447 -- `--visual-burn-rate` enum (`off|emoji|text|emoji-text`), `burnRateTier()` maps `tokensPerMinute` to a tier with configurable thresholds.

### TODO

1. **Function.** `src/analytics/burn_rate.rs::tier(tokens_per_min: f64, cfg: &BurnRateConfig) -> BurnRateTier` returning `Normal | Moderate | High`.
2. **Config.** `[statusline] burn_rate_normal_max = 4000`, `burn_rate_moderate_max = 10000` (tokens/min).
3. **CLI flag.** `statusline --visual-burn-rate=<off|bracket|emoji|both>`. Default `bracket` to match Heimdall's industrial style.
4. **Bracket renderer.** `[NORMAL]` / `[WARN]` / `[CRIT]` — reuses `src/ui_cli/severity.rs` from Phase 2.
5. **Dashboard integration.** Active-block card shows the tier badge next to `$/hr`.

### Acceptance

- Configurable thresholds override defaults.
- `--visual-burn-rate=off` strips the indicator entirely.

### Effort: XS (half day; depends on Phase 1)

---

## Phase 8 -- Dual Cost Source Reconciliation **[ ]**

**Motivation:** When Anthropic's hook-reported `cost` diverges from Heimdall's locally calculated cost, users want to see both side-by-side to diagnose pricing drift. ccusage's `--cost-source both` does exactly this.

**Source:** [ccusage/apps/ccusage/src/commands/statusline.ts](https://github.com/ryoppippi/ccusage/blob/main/apps/ccusage/src/commands/statusline.ts) lines 305--333 -- `costSourceChoices = ['auto', 'ccusage', 'cc', 'both']`. `both` emits `($0.12 cc / $0.14 ccusage)`.

### TODO

1. **Capture hook cost.** `heimdall-hook` already receives the hook payload; extend `live_events` schema with `hook_reported_cost_nanos` (nullable).
2. **CLI flag.** `statusline --cost-source=<auto|local|hook|both>` (rename ccusage's `cc` to `hook` and `ccusage` to `local` for clarity).
3. **Renderer.** In `both` mode output `($X hook / $Y local)`; add a divergence warning bracket when `|hook-local|/local > 0.10`.
4. **Dashboard panel.** `CostReconciliationPanel.tsx` -- daily/monthly bar showing delta between hook-reported and locally calculated totals.
5. **API endpoint.** `/api/cost-reconciliation?period=...` returns `{ hook_total_nanos, local_total_nanos, turn_breakdown: [...] }`.

### Acceptance

- Divergence >10% renders inline `[WARN: cost drift]` in the statusline.
- Panel hidden when hook has never reported costs (all rows null).

### Effort: S (1 day; depends on Phase 4)

---

## Phase 9 -- `--jq` Post-Processing on JSON Output **[ ]**

**Motivation:** Power-user scriptability without shell pipelines. ccusage pipes its JSON output through the system `jq` binary via a `--jq` flag on every report command. Equivalent to `ccusage daily --json | jq '.totals.cost'` but preserves exit codes and avoids pipe setup.

**Source:** [ccusage/apps/ccusage/src/_jq-processor.ts](https://github.com/ryoppippi/ccusage/blob/main/apps/ccusage/src/_jq-processor.ts), shared arg registered in `_shared-args.ts`. Implies `--json`.

### TODO

1. **Shared flag.** Add `--jq=<filter>` to `today`, `stats`, `export`, `blocks`, `weekly`, `optimize` subcommands. Implies `--json`.
2. **Bundled engine.** Use the `jaq` crate (pure-Rust `jq` clone) to avoid a system dependency. Pinned to a release tag; falls back to invoking system `jq` if present and `--jq-external` is set (rare).
3. **Stream mode.** For `export --format=jsonl`, apply the filter per-record rather than loading the whole file.
4. **Error path.** Jq parse errors exit 2 with a clear message; filter that yields `null` emits empty string and exits 0.

### Acceptance

- `today --jq '.total.cost_usd'` emits a single number.
- `export --format=jsonl --jq '.model' --output=-` streams one model name per line.
- No system `jq` required by default.

### Effort: S (1 day)

---

## Phase 10 -- JSON-Schema-Backed Config File + Per-Command Overrides **[ ]**

**Motivation:** Heimdall's TOML config is flat and unvalidated. ccusage ships a `ccusage.json` with `$schema` for IDE autocomplete and a `commands.<name>` section for per-command overrides -- both are big UX improvements for long-lived projects.

**Source:** [ccusage/apps/ccusage/src/_config-loader-tokens.ts](https://github.com/ryoppippi/ccusage/blob/main/apps/ccusage/src/_config-loader-tokens.ts), [ccusage/ccusage.example.json](https://github.com/ryoppippi/ccusage/blob/main/ccusage.example.json). Discovery: CWD `.ccusage/ccusage.json` → Claude config dir → project root.

### TODO

1. **Generate schema.** Add `schemars` to derive JSON Schema from the existing `Config` struct. Emit to `schemas/heimdall.config.schema.json` at build time via a `build.rs` or a `pricing refresh`-style subcommand.
2. **New file format.** Support `~/.claude/usage-tracker.json` (in addition to the existing TOML) with `$schema` pointing at the hosted URL `https://raw.githubusercontent.com/po4yka/heimdall/main/schemas/heimdall.config.schema.json`.
3. **Per-command section.** Parse `commands.blocks.tokenLimit`, `commands.statusline.offline`, `commands.daily.instances` etc. Precedence: CLI flag > `commands.<name>` > `defaults` > hardcoded.
4. **`mergeConfigWithArgs` equivalent.** Refactor CLI arg parsing so every subcommand first resolves its effective config (flag ∪ per-command ∪ defaults) before running.
5. **Publish schema.** GitHub Action publishes the schema to `gh-pages` on each release; README links to it for IDE integration.

### Acceptance

- `heimdall blocks` uses `commands.blocks.tokenLimit` from config when `--token-limit` is absent.
- VSCode autocompletes config keys after `"$schema": "..."` reference.
- Legacy TOML config still loads; users are not forced to migrate.

### Effort: M (2 days)

---

## Phase 11 -- Project Aliases for Human-Readable Names **[ ]**

**Motivation:** Claude Code project slugs are mangled directory hashes ("-Users-po4yka-GitRep-heimdall"). ccusage lets users define display aliases.

**Source:** [ccusage/apps/ccusage/src/commands/daily.ts](https://github.com/ryoppippi/ccusage/blob/main/apps/ccusage/src/commands/daily.ts) lines 54--69, [ccusage/apps/ccusage/src/_project-names.ts](https://github.com/ryoppippi/ccusage/blob/main/apps/ccusage/src/_project-names.ts) -- `--project-aliases "hash=Display Name,other=Other"`.

### TODO

1. **Config section.** `[project_aliases]` in the config file: `"-Users-po4yka-GitRep-heimdall" = "Heimdall"`.
2. **CLI override.** `--project-alias KEY=VALUE` repeatable flag.
3. **Display layer only.** Storage keeps the raw slug (aliases are display-time).
4. **Dashboard wiring.** `projects` API response adds `display_name` alongside `slug`; all components use `display_name || slug`.
5. **URL-persistent filter** continues to use slug; aliases are purely cosmetic.

### Acceptance

- Alias applied to Sessions table, Project Cost table, ProjectChart, heatmap tooltip.
- Unknown slug gracefully falls back to the raw slug.

### Effort: XS (half day)

---

## Phase 12 -- Amp Credit Tracking (Non-USD Billing Unit) **[ ]**

**Motivation:** Amp is Sourcegraph's AI coding tool that bills in "credits" (its own abstract unit), not USD. Heimdall currently ignores Amp credits entirely. ccusage ships a dedicated `@ccusage/amp` package.

**Source:** [ccusage/apps/amp/src/commands/daily.ts](https://github.com/ryoppippi/ccusage/blob/main/apps/amp/src/commands/daily.ts), [monthly.ts](https://github.com/ryoppippi/ccusage/blob/main/apps/amp/src/commands/monthly.ts), [session.ts](https://github.com/ryoppippi/ccusage/blob/main/apps/amp/src/commands/session.ts), [_types.ts](https://github.com/ryoppippi/ccusage/blob/main/apps/amp/src/_types.ts) -- each session emits a `credits: number` field accumulated in parallel with USD.

### TODO

1. **Amp provider.** `src/scanner/providers/amp.rs` implementing the `Provider` trait. Discovery: `~/.amp/sessions/*.jsonl` (verify path against the Amp CLI).
2. **Schema.** Add `credits REAL` column to `turns` + `sessions` tables (nullable; only populated for Amp).
3. **Dashboard column.** New "Credits" column in `ProjectCostTable.tsx` / `SessionsTable.tsx` shown only when the filtered range has non-null credits.
4. **Export.** `export --provider=amp` emits `credits` column in CSV.
5. **Stats.** `today`, `stats`, `weekly`, `monthly`, `export` all surface credits for Amp sessions.

### Acceptance

- Amp session JSONL parses without errors; credits sum correctly across the day.
- Non-Amp providers never display a Credits column.

### Effort: M (2 days)

---

## Phase 13 -- Configurable Session Block Duration **[ ]**

**Motivation:** 5-hour windows are Claude-specific; other providers (Codex API, custom) may use different billing cadences. ccusage's `--session-length` accepts any positive hour value including fractional.

**Source:** [ccusage/apps/ccusage/src/_session-blocks.ts](https://github.com/ryoppippi/ccusage/blob/main/apps/ccusage/src/_session-blocks.ts) -- `identifySessionBlocks(entries, sessionDurationHours)`, default 5.

### TODO

1. **Flag.** `blocks --session-length=<hours>` accepts any `f64 > 0`. Already wired in Phase 1's function signature; just surface the CLI flag and config key `commands.blocks.sessionLength`.
2. **Per-provider defaults.** Config table: `[blocks.session_length_by_provider] claude = 5.0, codex = 1.0, amp = 24.0`. `blocks --provider=codex` picks up the provider default.
3. **Validation.** Reject `≤0`, `>168` (one week) with a clear error.

### Acceptance

- `blocks --session-length=1` yields hourly windows.
- Per-provider default applied when no explicit flag is set.

### Effort: XS (2 hours; depends on Phase 1)

---

## Phase 14 -- Per-Model Breakdown Sub-Rows in CLI Tables **[ ]**

**Motivation:** Terminal users scanning `today` or `weekly` often want to see "which model drove the cost" without switching to `--json`. ccusage's `--breakdown` flag injects indented per-model sub-rows under each aggregation row.

**Source:** ccusage shared arg `breakdown` in [_shared-args.ts](https://github.com/ryoppippi/ccusage/blob/main/apps/ccusage/src/_shared-args.ts); rendering helper `pushBreakdownRows` in `packages/terminal` (monorepo root). Used in `daily.ts`, `weekly.ts`, `monthly.ts`.

### TODO

1. **Shared flag.** Add `--breakdown` to `today`, `weekly`, `monthly`, `export` (CSV only when formatted as human-readable).
2. **Table writer.** Extend `src/ui_cli/table.rs` (create if missing) with a generic `render_with_breakdown(row, sub_rows)` helper. Sub-rows render indented with a leading `└─`.
3. **Respect industrial style.** Plain ASCII indent; no unicode boxes in the default theme.

### Acceptance

- `today --breakdown` shows per-model rows under the daily total.
- Plain `today` unchanged.

### Effort: XS (half day)

---

## Phase 15 -- Gap Block Visualization in `blocks` Output **[ ]**

**Motivation:** Zero-activity billing windows should be visible in the blocks table, not silently omitted. ccusage inserts explicit `(14h gap)` pseudo-rows to keep the time axis continuous.

**Source:** [ccusage/apps/ccusage/src/_session-blocks.ts](https://github.com/ryoppippi/ccusage/blob/main/apps/ccusage/src/_session-blocks.ts) `createGapBlock()`, rendered dim-gray at [blocks.ts](https://github.com/ryoppippi/ccusage/blob/main/apps/ccusage/src/commands/blocks.ts) lines 377--390 as `pc.gray('(inactive)')`.

### TODO

1. **Function.** `identify_blocks` (from Phase 1) gains `include_gaps: bool`. Gap block has `cost_nanos = 0`, `tokens = zero`, `is_gap = true`.
2. **Renderer.** Show gap rows as `(Nh Nm gap)` in dim/gray style. Use `owo-colors` dim modifier.
3. **JSON parity.** Gap blocks serialize with `"kind": "gap"` to distinguish from empty activity blocks.
4. **Dashboard.** Gaps rendered as translucent rows in the activity timeline card (if added in Phase 1).

### Acceptance

- `blocks` on a week of sporadic activity shows continuous time axis.
- `--json` includes gap entries under an explicit `kind`.

### Effort: XS (2 hours; depends on Phase 1)

---

## Phase 16 -- Locale-Aware Date/Time Formatting **[ ]**

**Motivation:** Users outside en-US expect dates in their locale ("4月18日" not "Apr 18"). ccusage threads a `--locale` flag through every formatter.

**Source:** ccusage shared `locale` arg in [_shared-args.ts](https://github.com/ryoppippi/ccusage/blob/main/apps/ccusage/src/_shared-args.ts), used by `formatBlockTime`, `formatDateCompact`, `formatDateLong` throughout commands.

### TODO

1. **Crate.** Add `icu_datetime` (or `time` with `formatting` feature if weight matters). Formatter takes a `LocaleFallbacker`.
2. **Flag + config.** `--locale=ja-JP`, `commands.daily.locale`, global `[display] locale = "ja-JP"`. Default resolves from `$LANG`.
3. **Render site.** All CLI table date columns and dashboard date labels pass through a single `format_date(dt, locale)` helper.
4. **Tests.** Snapshot tests for `en-US`, `ja-JP`, `de-DE`, `ru-RU`.

### Acceptance

- `--locale=ja-JP` yields localized month/day labels in the `today` and `weekly` tables.
- Dashboard respects browser locale (sent via `Accept-Language`) when config is unset.

### Effort: S (1 day)

---

## Phase 17 -- Compact CLI Table Mode for Screenshot Sharing **[ ]**

**Motivation:** Responsive auto-wrap tables look jumbled in narrow terminals and screenshots. ccusage's `--compact` forces a narrow layout by dropping cache columns and condensing model lists.

**Source:** ccusage shared `compact` arg in [_shared-args.ts](https://github.com/ryoppippi/ccusage/blob/main/apps/ccusage/src/_shared-args.ts); `ResponsiveTable` with `forceCompact` in `packages/terminal`.

### TODO

1. **Shared flag.** `--compact` on all table-producing commands. Config key `[display] compact = false`.
2. **Compact renderer.** Column whitelist per subcommand: `today` keeps DATE/COST/TOKENS; drops cache-read/cache-write. Model list condenses to first model + "+N more".
3. **Auto-detect.** When stdout is a TTY with `COLUMNS < 100` and `--compact` is unset, emit a hint: `(narrow terminal detected; try --compact)`.

### Acceptance

- `today --compact` fits within 80 columns.
- Default wide mode unchanged.

### Effort: XS (2 hours)

---

## Deferred / Explicitly Out of Scope

- **TUI dashboard.** Heimdall chose web; duplicating in `ratatui` is a lot of code for marginal value.
- **Cloud sync / hosted dashboard.** Violates the "strictly local" architectural principle.
- **LLM-based classification.** Determinism is a feature; regex classifier stays.
- **Prometheus/OTel exporters.** Webhooks handle the real-time use case.
- **Permission-approval UI / mascot layer.** Orthogonal problem space; Heimdall is observational, not interventional.
- **Blocking hook polling loop.** Heimdall's hooks are fire-and-forget observational writes only.
- **Localhost HTTP IPC between hook and daemon.** Direct SQLite writes from the hook binary are simpler and faster.

---

## Sequencing Summary

| Phase | Feature | Effort | Unlocks |
|-------|---------|--------|---------|
| 1 | 5h billing blocks + burn rate + projection | M | Foundation for Phases 2, 7, 13, 15; "am I over quota" answer |
| 2 | Token quota % + inline REMAINING/PROJECTED rows | S | Live quota pressure visibility |
| 3 | Weekly aggregation report | S | Sprint-cadence teams |
| 4 | `statusline` PostToolUse hook | L | Always-visible usage in Claude Code's own status bar |
| 5 | Context-window usage tracking | M | Compact-before-overflow UX |
| 6 | MCP server | L | Inference-time self-query by Claude/Cursor/Desktop |
| 7 | Visual burn-rate indicator | XS | Scannable tier badge in statusline |
| 8 | Dual cost source reconciliation | S | Diagnose Anthropic vs local cost drift |
| 9 | `--jq` post-processing | S | Power-user scriptability |
| 10 | JSON-schema config + per-command overrides | M | IDE autocomplete, persistent per-project defaults |
| 11 | Project aliases | XS | Readable directory-hash project names |
| 12 | Amp credit tracking | M | First-class non-USD billing unit |
| 13 | Configurable session block duration | XS | Model non-5h providers |
| 14 | Per-model breakdown sub-rows | XS | Terminal model-driver visibility |
| 15 | Gap block visualization | XS | Continuous time axis in blocks |
| 16 | Locale-aware date formatting | S | International users |
| 17 | Compact CLI table mode | XS | Screenshot-friendly output |

All phases in this cycle source from ccusage. Total estimated effort: **~7--8 weeks focused work**.

**Dependency graph:**
- Phase 1 is the foundation for Phases 2 (quota % piggybacks on blocks), 7 (burn-rate tier badge), 13 (configurable duration), 15 (gap blocks).
- Phase 4 precedes Phase 5 (statusline payload schema must land before context-window integration) and Phase 8 (dual cost source is a statusline render mode).
- Phase 10 is orthogonal but simplifies configuration of all later phases; sequence early if dev bandwidth allows.
- Phase 6 (MCP) is standalone but benefits from Phase 1 (exposes `blocks_active` tool) and Phase 3 (exposes `weekly` tool).
- All other phases are independent and parallelizable.

---

## Tracking

Update the status legend at the top of each phase as work progresses. Link PRs inline. When a phase closes, move its full section into a `CHANGELOG.md` under the shipping version.
