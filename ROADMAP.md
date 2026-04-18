# Heimdall -- Roadmap

Detailed phased plan for evolving `claude-usage-tracker` into a broader local AI observability platform. Each phase is self-contained: it lands in `main` behind no feature flags, adds tests, and leaves the tree green.

Status legend: **[ ]** not started · **[~]** in progress · **[x]** done

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

Heimdall harvests patterns from two sibling projects in the local-AI-observability space. Each phase below that ports logic names a concrete source file for future implementation work.

| Project | Repo | Language | Role |
|---------|------|----------|------|
| **Codeburn** | https://github.com/AgentSeal/codeburn | TypeScript CLI | Upstream session parser and provider plugin pattern; widest provider coverage (Cursor, OpenCode, Pi, Copilot). Source of the 13-category classifier, `optimize` waste detector, SwiftBar menubar, and currency conversion. |
| **Third-Eye** | https://github.com/fien-atone/third-eye | TypeScript web (Express + React) | Richest web-dashboard implementation of the same problem space. Vendors Codeburn's parser. Source of tool-event cost attribution, 7×24 activity heatmap, client-sent timezone handling, active-period averaging, cross-platform scheduler, and CC-version tracking. |
| **Claude-Guardian** | https://github.com/anshaneja5/Claude-Guardian | Swift + Python (macOS-only) | Primarily a permission-approval UI (SwiftUI mascot + Python hook bridge), not an observability tool -- but its analytics subsystem and macOS packaging engineering are directly transferable. Source of the real-time PreToolUse cost injection pattern, file-watcher auto-refresh, usage-limits file parsing, cache-token breakdown, and the Homebrew cask + LaunchAgent + universal-binary distribution stack. |

When porting, prefer reading the source file directly over reimplementing from memory. Each "Source:" line points at the canonical file in the referenced repo.

---

## Phase 0 -- Foundation: Formalize `Provider` Trait **[x]**

**Motivation:** Scanner logic is currently spread across `src/scanner/` with Claude and Codex paths interleaved. A `Provider` trait makes new sources a one-file contribution and mirrors Codeburn's plugin pattern.

**Source:** [codeburn/src/providers/](https://github.com/AgentSeal/codeburn/tree/main/src/providers) -- `types.ts` (trait shape), `index.ts` (registry), `codex.ts` (reference impl).

### Deliverables

- `src/scanner/provider.rs` -- new trait:
  ```rust
  pub trait Provider: Send + Sync {
      fn name(&self) -> &'static str;
      fn discover_sessions(&self) -> Result<Vec<SessionSource>>;
      fn parse(&self, path: &Path) -> Result<Vec<Turn>>;
  }
  ```
- Extract existing Claude logic into `src/scanner/providers/claude.rs`.
- Extract existing Codex logic into `src/scanner/providers/codex.rs`.
- Extract Xcode CodingAssistant path into `src/scanner/providers/xcode.rs`.
- Central registry: `src/scanner/providers/mod.rs` with `pub fn all() -> Vec<Box<dyn Provider>>`.
- SQLite schema: add `provider` column to sessions table with migration + backfill.
- Update `AGENTS.md` with a "Adding a Provider" section (template file, trait signature, test fixture layout).

### Acceptance

- `cargo test` green (no behavior change; the baseline 128 existing tests still pass, plus the new trait + backfill + xcode-retag coverage added in this phase).
- `clippy -D warnings` clean.
- Schema migration runs idempotently on an existing DB.
- Dashboard "Provider" filter still works.

### Effort: M (1--2 days)

---

## Phase 1 -- CSV/JSON Multi-Period Export **[x]**

**Motivation:** Users need spreadsheet-friendly output for expense reports and manager dashboards. Heimdall currently has `--json` on `today` and `stats` only.

**Source:** [codeburn/src/export.ts](https://github.com/AgentSeal/codeburn/blob/main/src/export.ts) -- column set, period logic, multi-format dispatch.

### Deliverables

- New subcommand:
  ```
  claude-usage-tracker export --format=<csv|json|jsonl> \
                              --period=<today|week|month|year|all> \
                              --output=<path> \
                              [--provider=<name>] [--project=<slug>]
  ```
- `src/export.rs` -- format-agnostic dispatch.
- Add `csv` crate dependency (~30 KB).
- Column set: `date, provider, project, model, input_tokens, output_tokens, cache_read, cache_write, cost_usd_nanos, cost_usd_display`.
- Integration tests with tempfile + round-trip parse.

### Acceptance

- `export --format=csv --period=month` produces a file that round-trips via `csv::Reader` into identical records.
- Nanos preserved as integers in CSV; float only in `cost_usd_display`.
- Documented in README "Export" section.

### Effort: XS (half day)

---

## Phase 2 -- Task Classifier (13 Categories) **[x]**

**Motivation:** Tokens alone don't explain behavior. Knowing "60% of spend was debugging" is the single most requested insight from Codeburn.

**Source:** [codeburn/src/classifier.ts](https://github.com/AgentSeal/codeburn/blob/main/src/classifier.ts) (regex tables) + [codeburn/tests/classifier.test.ts](https://github.com/AgentSeal/codeburn/blob/main/tests/classifier.test.ts) (golden vectors). [third-eye/server/lib/classifier.ts](https://github.com/fien-atone/third-eye/tree/main/server/lib) vendors the same 13 categories and confirms the taxonomy.

### Deliverables

- `src/scanner/classifier.rs`:
  ```rust
  pub enum TaskCategory {
      Coding, Debugging, FeatureDev, Testing, Git, Docs,
      Research, Refactor, DevOps, Config, Planning, Review, Other,
  }
  ```
- Use `regex::RegexSet` for batch matching against tool names + first user message.
- Run classifier during scan, store `category TEXT` on `turns` table (or `session_classifications` for session-level).
- Aggregate query: `SELECT category, SUM(cost_nanos) FROM turns GROUP BY category`.
- Dashboard: new `TaskBreakdownChart.tsx` donut + per-category drilldown.
- CLI: `claude-usage-tracker stats --by=category`.
- Port Codeburn's `tests/classifier.test.ts` fixtures to Rust test vectors.

### Acceptance

- ~500 Codeburn fixture cases reproduce identical categorization (port as golden tests).
- Classifier is pure: no I/O, no mutable state, safe across threads.
- `--by=category` JSON output stable and documented.

### Effort: S (1--2 days)

---

## Phase 3 -- One-Shot Rate Tracking **[x]**

**Motivation:** A compact, viral metric for "how good is the model at this task." Detects Edit→Bash→Edit retry cycles as proxy for rework.

**Source:** [codeburn/src/parser.ts](https://github.com/AgentSeal/codeburn/blob/main/src/parser.ts) -- retry detection heuristic lives alongside the main parse pass.

### Deliverables

- Post-processing pass in scanner: walk turns chronologically per session, flag retry loops.
- Heuristic: same file edited, then Bash (typically build/test), then same file edited again within N turns → not one-shot.
- Add `one_shot BOOLEAN` to `sessions` table (nullable for sessions without edits).
- Metric: `SELECT AVG(CASE WHEN one_shot THEN 1.0 ELSE 0.0 END) FROM sessions WHERE one_shot IS NOT NULL`.
- Dashboard: KPI card next to cost totals.
- CLI: surface in `stats` output.

### Acceptance

- Fixture sessions hand-labeled for ground truth; classifier agrees ≥90%.
- Heuristic constants (`N turns`, tool patterns) named consts with rationale comments.

### Effort: S (1 day)

---

## Phase 4 -- Currency Conversion **[x]**

**Motivation:** Non-USD users currently read dollar amounts mentally. Codeburn's 162-currency Frankfurter integration ships this cleanly.

**Source:** [codeburn/src/currency.ts](https://github.com/AgentSeal/codeburn/blob/main/src/currency.ts) -- Frankfurter client, 24h cache, 162-currency list.

### Deliverables

- `src/currency.rs` using existing `reqwest`; cache at `~/.cache/heimdall/fx.json` with 24h TTL.
- Frankfurter API (ECB data, free, no auth).
- Config:
  ```toml
  [display]
  currency = "EUR"   # ISO 4217; USD is default
  ```
- Applied only in display layer (`pricing::format_cost`), never in storage.
- Dashboard: currency picker in header, URL-persistent alongside existing filters.
- Graceful offline fallback: if cache stale and network fails, display USD with warning annotation.

### Acceptance

- Storage remains USD nanos; verified by DB inspection test.
- FX cache self-invalidates at 24h.
- Offline mode shows `1.23 USD (FX unavailable)` rather than failing.

### Effort: S (1 day)

---

## Phase 5 -- Cursor Provider **[x]**

**Motivation:** Cursor is the largest unpenetrated user base for Heimdall. Codeburn already proved the SQLite parsing pattern.

**Source:** [codeburn/src/providers/cursor.ts](https://github.com/AgentSeal/codeburn/blob/main/src/providers/cursor.ts) + [codeburn/src/cursor-cache.ts](https://github.com/AgentSeal/codeburn/blob/main/src/cursor-cache.ts) -- DB paths per OS, cache invalidation on mtime/size.

### Deliverables

- `src/scanner/providers/cursor.rs` using existing `rusqlite` (no new optional dep; Heimdall already bundles it).
- Read Cursor's session DB at platform-appropriate path:
  - macOS: `~/Library/Application Support/Cursor/User/workspaceStorage/*/state.vscdb`
  - Linux: `~/.config/Cursor/User/workspaceStorage/*/state.vscdb`
  - Windows: `%APPDATA%/Cursor/User/workspaceStorage/*/state.vscdb`
- File-based result cache at `~/.cache/heimdall/cursor/` keyed on source DB mtime+size.
- Fixture-based tests (port Codeburn's `tests/providers/cursor.test.ts` fixtures).

### Acceptance

- Cursor sessions appear in dashboard with correct cost attribution.
- Cache invalidates automatically when source DB changes.
- Tests run without network and without a real Cursor install.

### Effort: M (2--3 days)

---

## Phase 6 -- `optimize` Waste Detector **[~]** (3 of 5 detectors landed)

**Motivation:** Highest-value differentiator from Codeburn. Users learn what to delete/fix to spend less next session.

**Source:** [codeburn/src/optimize.ts](https://github.com/AgentSeal/codeburn/blob/main/src/optimize.ts) -- 41 KB file containing all detectors, findings shape, and A--F grade calculation. **Prerequisite:** Phase 12 (tool-event cost attribution) supplies the data model that makes per-MCP / per-file waste queries tractable.

### Deliverables

- New subcommand: `claude-usage-tracker optimize [--format=<text|json>]`.
- New module tree under `src/optimizer/`:
  - `reread.rs` -- same file read >2x per session
  - `claude_md.rs` -- token cost of `~/.claude/CLAUDE.md` multiplied by session count
  - `mcp.rs` -- MCP servers in `~/.claude/settings.json` never invoked in DB history
  - `agents.rs` -- agents in `~/.claude/agents/` never referenced
  - `bash.rs` -- repeated trivial commands (`ls`, `pwd`, `git status`) in single session
  - `grade.rs` -- weight individual findings to produce A--F grade
- Each detector is a trait impl returning `Vec<Finding>` with `{severity, title, detail, estimated_monthly_waste_nanos}`.
- Dashboard: `/optimize` route, `OptimizationPanel.tsx` with collapsible findings.
- Webhook: optional "weekly optimization digest" trigger.

### Acceptance

- `optimize --format=json` schema documented in SPEC.md.
- Grade is stable across runs when inputs unchanged (deterministic).
- Each detector has a dedicated test with a crafted fixture DB/config.

### Effort: L (4--6 days)

---

## Phase 7 -- OpenCode, Pi, Copilot Providers **[x]**

**Motivation:** Round out provider coverage parity with Codeburn.

**Source:** [codeburn/src/providers/opencode.ts](https://github.com/AgentSeal/codeburn/blob/main/src/providers/opencode.ts), [codeburn/src/providers/pi.ts](https://github.com/AgentSeal/codeburn/blob/main/src/providers/pi.ts), [codeburn/src/providers/copilot.ts](https://github.com/AgentSeal/codeburn/blob/main/src/providers/copilot.ts) -- each file documents its dedup key strategy in the header.

### Deliverables (one sub-PR each)

- `src/scanner/providers/opencode.rs` -- SQLite-backed, session + message ID dedup.
- `src/scanner/providers/pi.rs` -- JSONL, `responseId` dedup.
- `src/scanner/providers/copilot.rs` -- format depends on Copilot's local state; research first.
- Per-provider fixtures in `tests/fixtures/<provider>/`.
- Update dashboard provider filter + README provider table.

### Acceptance

- Each provider gated behind its own capability check (return empty if files absent; no error, no panic).
- Dedup strategies documented in each module header.

### Effort: M per provider (2 days each, ~1 week total)

---

## Phase 8 -- SwiftBar Menubar Widget + Security **[x]**

**Motivation:** macOS users want at-a-glance today-cost in the menu bar. Codeburn has a working SwiftBar plugin generator.

**Source:** [codeburn/src/menubar.ts](https://github.com/AgentSeal/codeburn/blob/main/src/menubar.ts) + [codeburn/tests/security/menubar-injection.test.ts](https://github.com/AgentSeal/codeburn/blob/main/tests/security/menubar-injection.test.ts) -- **port the security test verbatim**; the injection vectors (pipe, newline, `bash=`, `href=`) are the acceptance contract.

### Deliverables

- New subcommand: `claude-usage-tracker menubar` printing SwiftBar-formatted stdout.
- `src/menubar.rs` -- format today's cost, session count, one-shot rate.
- Submenu: drill into per-provider split, "open dashboard" action.
- **Mandatory security test:** `tests/security/menubar_injection.rs` -- project slugs, session titles, and any user-controlled strings passed through a strict sanitizer. Pipe character, newlines, and SwiftBar control sequences (`|`, `bash=`, `href=`) must be stripped or escaped.
- Installation doc snippet: symlink command for `~/Library/Application Support/SwiftBar/Plugins/`.

### Acceptance

- Running `menubar` against a fixture DB with malicious project names produces no shell-executable output.
- Injection test covers pipe, newline, URL schemes, and SwiftBar param keywords.

### Effort: S (1 day feature + 0.5 day security test)

---

## Phase 9 -- LiteLLM Pricing Refresh **[x]** **[x]**

**Motivation:** Claude/GPT-5 already covered by Heimdall's hardcoded table; this handles long-tail providers (Gemini, Mistral, Groq) without release cuts.

**Source:** [codeburn/src/models.ts](https://github.com/AgentSeal/codeburn/blob/main/src/models.ts) (LiteLLM fetch + hardcoded fallback). Third-Eye implements the same three-tier (live → 24h cache → hardcoded) pattern at [third-eye/server/lib/models.ts](https://github.com/fien-atone/third-eye/tree/main/server/lib) -- use as secondary reference.

### Deliverables

- New subcommand: `claude-usage-tracker pricing refresh` fetches LiteLLM `model_prices_and_context_window.json`, caches at `~/.cache/heimdall/pricing.json` with 24h TTL.
- `PricingSource` enum:
  ```rust
  enum PricingSource {
      Static,                    // current default
      LiteLlm { cache_path: PathBuf },
  }
  ```
- Lookup order preserved: exact hardcoded match wins over LiteLLM fuzzy match for Claude/GPT-5 families. Port Codeburn's lesson: never fuzzy-match critical model families.
- Config:
  ```toml
  [pricing]
  source = "litellm"  # or "static" (default)
  refresh_hours = 24
  ```

### Acceptance

- Existing pricing tests unchanged and still pass.
- New test: fuzzy-match Claude-family input returns hardcoded price, not LiteLLM price, even when both present.
- Offline: falls back to cache or static without error.

### Effort: S (1 day)

---

## Phase 10 -- UI Palette Migration (Industrial Design) **[x]**

**Motivation:** Pre-existing gap called out in `CLAUDE.md:183`. `src/ui/` still uses the legacy indigo/Inter palette while the rest of the design system specifies industrial monochrome (OLED dark + warm-off-white light).

### Deliverables

- Audit every component under `src/ui/components/` against `DESIGN.md`.
- Replace Tailwind classes: indigo → monochrome scale, Inter → system mono + display stack per DESIGN spec.
- Regenerate `src/ui/style.css` and `src/ui/app.js` with `npm run build:ui`; commit artifacts.
- Screenshot regression suite (Playwright or manual gallery in `docs/ui-gallery/`).

### Acceptance

- `rg "indigo|bg-blue|text-blue" src/ui/` returns zero matches.
- Side-by-side screenshots reviewed for every panel type.
- CLAUDE.md line flagging the gap is removed.

### Effort: M (2--3 days)

---

## Phase 11 -- Release Automation **[x]**

**Motivation:** No binary distribution today. Users `cargo install --git` or build from source.

### Deliverables

- `.github/workflows/release.yml` triggered on `v*.*.*` tag.
- Matrix build: macOS arm64, macOS x86_64, Linux x86_64, Linux arm64, Windows x86_64.
- Publish to GitHub Releases with SHA256 checksums.
- Optional: `cargo-dist` for a managed setup.
- Homebrew tap formula for macOS one-liner install.
- Update README install section with prebuilt-binary path as default, source build as fallback.

### Acceptance

- Tagging `v0.x.y` produces downloadable artifacts within 15 min.
- Checksums published alongside artifacts.
- Homebrew `brew install heimdall/tap/heimdall` works.

### Effort: M (2 days)

---

## Phase 12 -- Tool-Event Cost Attribution **[x]**

**Motivation:** Third-Eye's single most valuable data-model innovation. Every API call today is attributed to a session and model but not to the specific *tool invocations* inside it. Splitting `cost_nanos / tool_event_count` across each tool event unlocks queries Heimdall currently cannot answer: "which MCP server costs most", "which files cost most to edit", "which bash commands cost most time-money". This is also the prerequisite data model that makes Phase 6's `optimize` detectors tractable.

**Source:** [third-eye/server/db.ts](https://github.com/fien-atone/third-eye/blob/main/server/db.ts) (`tool_events` table schema) + [third-eye/server/ingest.ts](https://github.com/fien-atone/third-eye/blob/main/server/ingest.ts) (cost-splitting logic `costPer = cost / events.length`).

### Deliverables

- New SQLite table `tool_events`:
  ```sql
  CREATE TABLE tool_events (
      dedup_key     TEXT PRIMARY KEY,
      ts_epoch      INTEGER NOT NULL,
      session_id    TEXT NOT NULL,
      project       TEXT NOT NULL,
      kind          TEXT NOT NULL,   -- subagent|skill|mcp|bash|file
      value         TEXT NOT NULL,   -- tool name, file path, bash command, etc.
      cost_nanos    INTEGER NOT NULL
  );
  CREATE INDEX idx_tool_events_kind ON tool_events(kind, ts_epoch);
  ```
- Ingest update: during parse, emit one `tool_events` row per tool invocation within each `assistant` message. Divide the call's `cost_nanos` evenly across events.
- New API endpoint `/api/tool-costs?kind=<kind>&period=<period>` returning ranked aggregates.
- Dashboard: extend `ToolUsageTable.tsx` and `McpSummaryTable.tsx` to show cost columns.
- New dashboard panel: "File hotspots" -- top files by touch count × cost (third-eye has an exemplar).

### Acceptance

- `SELECT SUM(cost_nanos) FROM tool_events WHERE session_id = ?` equals the session's total within floor-division error (±N nanos where N = event count).
- Cost totals in the existing `/api/data` endpoint unchanged.
- Tests: round-trip invariant that summed tool-event costs equal parent call costs.

### Effort: M (2--3 days)

---

## Phase 13 -- 7×24 Activity Heatmap + Active-Period Averaging **[x]**

**Motivation:** Two small analytical upgrades from Third-Eye with disproportionate UX impact. The heatmap answers "when do I code" at a glance (the day/hour cell visual is distinctive and screenshot-worthy). Active-period averaging fixes a silent measurement bug: dividing cost by 30 calendar days when the user only worked 12 of them underreports real daily cost.

**Source:** [third-eye/server/index.ts](https://github.com/fien-atone/third-eye/blob/main/server/index.ts) -- `SELECT strftime('%w', ts), strftime('%H', ts)` query for heatmap; active-period divisor logic documented inline.

### Deliverables

- Heatmap SQL query returning `(dow, hour, cost_nanos, call_count)` for selected date range.
- New API endpoint `/api/heatmap?period=<period>`.
- Dashboard: new `ActivityHeatmap.tsx` -- 7×24 CSS grid, intensity via opacity (per industrial-design: single accent only; use the monochrome ladder).
- Replace existing "avg cost/day" calculation in `StatsCards.tsx` with active-period average; add a tooltip explaining the divisor ("7 active days of 30 calendar days").
- Document the formula in `DESIGN.md`.

### Acceptance

- Heatmap respects current filter state (model, provider, project, date range).
- Active-period average never divides by zero; empty range shows `--`.
- Screenshot regression: add heatmap to UI gallery.

### Effort: S (1 day)

---

## Phase 14 -- Client-Sent Timezone Handling **[x]**

**Motivation:** Today's dashboard bucket boundaries use UTC (or server-local time depending on code path), causing confusing "today" views for non-UTC users. Third-Eye solved this cleanly: the client sends `tzOffsetMin` and `weekStartsOn` on every request; the server applies `datetime(ts, '+N minutes')` in SQL before bucketing. Eliminates server TZ config entirely.

**Source:** [third-eye/server/index.ts](https://github.com/fien-atone/third-eye/blob/main/server/index.ts) -- search for `tzOffsetMin` and `weekStartsOn` query parameter handling.

### Deliverables

- Add `tz_offset_min: Option<i32>` and `week_starts_on: Option<u8>` to all aggregation query handlers.
- SQL helpers shift timestamps: `datetime(ts_epoch, 'unixepoch', ? || ' minutes')`.
- Client: send `new Date().getTimezoneOffset() * -1` and locale-derived `weekStartsOn` on every fetch.
- Fallback: missing params default to UTC (preserves current behavior).

### Acceptance

- Same DB produces correct "today" buckets for any client timezone without server restart.
- Fixture test with 3 timezones (UTC, +9, -5) confirms bucketing.

### Effort: S (1 day)

---

## Phase 15 -- Cross-Platform Scheduler Subcommand **[x]**

**Motivation:** Users currently run `claude-usage-tracker scan` manually or build their own cron entry. A single command that installs a platform-native scheduled job is a killer DX affordance.

**Source:** [third-eye/server/schedule.ts](https://github.com/fien-atone/third-eye/blob/main/server/schedule.ts) -- platform detection, minute offset to avoid `:00` pile-ups, absolute-path resolution for nvm/Homebrew-style installs.

### Deliverables

- New subcommand tree:
  ```
  claude-usage-tracker scheduler install    [--interval=hourly|daily]
  claude-usage-tracker scheduler uninstall
  claude-usage-tracker scheduler status
  ```
- Platform dispatch in `src/scheduler/`:
  - `launchd.rs` -- write `~/Library/LaunchAgents/dev.heimdall.scan.plist`, `launchctl load`
  - `cron.rs` -- append line to user crontab via `crontab -l | ... | crontab -`
  - `schtasks.rs` -- `schtasks /create /sc hourly ...` on Windows
- Minute offset `:17` (not `:00`) to avoid scheduler pile-up.
- Resolve absolute path to the running binary via `std::env::current_exe()`.

### Acceptance

- `scheduler install` followed by `scheduler status` shows "installed, next run at HH:17".
- `scheduler uninstall` cleanly removes the entry (no stale config).
- Integration test per platform gated by `cfg(target_os)`.

### Effort: M (2 days)

---

## Phase 16 -- Claude Code Version Tracking + Distribution **[x]**

**Motivation:** Heimdall already captures `cc_version` in the data model (`VersionTable.tsx` exists). Third-Eye takes it further: a donut chart + metric switcher (cost / calls / tokens) that answers "did upgrading Claude Code change my spend." Low-cost enhancement that produces a talkable screenshot.

**Source:** [third-eye/client/src/App.tsx](https://github.com/fien-atone/third-eye/blob/main/client/src/App.tsx) -- search for `cc_version` distribution donut and metric switcher.

### Deliverables

- New component `VersionDonut.tsx` wrapping `ApexChart.tsx`.
- Metric switcher: cost / calls / tokens (radio pill buttons, URL-persistent).
- Tooltip on hover shows all three metrics together.
- Graceful handling of `cc_version = NULL` (legacy sessions without the field).

### Acceptance

- Donut sums match `VersionTable.tsx` totals exactly.
- Switching metrics preserves selected version segment if any.

### Effort: XS (half day)

---

## Phase 17 -- Cowork Ephemeral Label Resolution **[x]**

**Motivation:** Claude Desktop Cowork sessions live in paths like `wizardly-charming-thompson/` -- procedurally generated slugs that are meaningless to humans. Third-Eye resolves these by walking `local-agent-mode-sessions/audit.jsonl` and extracting the first user message as the display label. If Heimdall adds Cowork support (implied by the broader provider plan), this is the only path to readable project names.

**Source:** [third-eye/server/lib/providers/](https://github.com/fien-atone/third-eye/tree/main/server/lib/providers) -- Cowork-specific file, label derivation from `audit.jsonl`.

### Deliverables

- `src/scanner/providers/cowork.rs` (or extend `claude.rs` if Cowork sessions live under `~/.claude/`).
- Label resolver: walk `local-agent-mode-sessions/<slug>/audit.jsonl`, parse first user message, truncate to 80 chars.
- Cache resolved labels in SQLite `projects` table.

### Acceptance

- Fixture with synthetic Cowork slug resolves to human-readable title.
- Missing `audit.jsonl` falls back to the slug without error.

### Effort: S (1 day)

---

## Phase 18 -- UI Polish Pack

**Motivation:** Three small UX upgrades that each take hours but compound. From Third-Eye's dashboard.

**Source:** [third-eye/client/src/App.tsx](https://github.com/fien-atone/third-eye/blob/main/client/src/App.tsx) (inline bars, TanStack Query `keepPreviousData`) and [third-eye/server/ingest.ts](https://github.com/fien-atone/third-eye/blob/main/server/ingest.ts) (TTY rebuild guard).

### Deliverables

1. **Inline proportional bars for ranked lists.** Replace the per-row count number in `ToolUsageTable`, `McpSummaryTable`, `BranchTable` with a background div at `width: (count / max) * 100%`. Zero new deps; pure CSS. Per industrial-design: monochrome fill at 15% opacity.
2. **Preserve-previous-data on filter change.** Signals-based equivalent of TanStack's `keepPreviousData`: when filters mutate, keep rendering the old data until the new fetch resolves. Prevents mid-interaction blank flashes.
3. **TTY-guarded destructive rebuild.** New subcommand `claude-usage-tracker db reset` requires interactively typing `"rebuild"` to confirm. In non-TTY contexts (CI, pipes) refuse with exit 1 unless `--yes` is passed.

### Acceptance

- Inline-bar rows pass contrast-ratio check in both themes.
- Switching filters triggers a network request with visible [LOADING] status but no UI flash.
- `echo "rebuild" | claude-usage-tracker db reset` rejects (not a TTY); `db reset --yes` succeeds non-interactively.

### Effort: S (1 day total, split across the three)

---

## Phase 19 -- Real-Time PreToolUse Hook Ingest **[x]**

**Motivation:** Heimdall currently only sees usage after Claude Code flushes JSONL to disk. Claude-Guardian proves that `hook_input["cost"]["total_cost_usd"]` arrives on *every* tool invocation via the PreToolUse hook -- true sub-second cost visibility with zero parsing overhead. Turns Heimdall from a periodic-scan tool into a live observer.

**Source:** [Claude-Guardian/hook/pre_tool_use.py](https://github.com/anshaneja5/Claude-Guardian/blob/main/hook/pre_tool_use.py) -- cost extraction at `pre_tool_use.py:228`, bypass-mode ancestor walk at lines 197--213. [Claude-Guardian/hook/permission_request.py](https://github.com/anshaneja5/Claude-Guardian/blob/main/hook/permission_request.py) -- same cost field at line 145.

### Deliverables

- New binary target in `Cargo.toml`:
  ```toml
  [[bin]]
  name = "heimdall-hook"
  path = "src/hook/main.rs"
  ```
- Read JSONL event from stdin, extract `session_id`, `tool_name`, `cost.total_cost_usd`, `input_tokens`, `output_tokens`.
- Write directly to SQLite `live_events` table (no IPC, no HTTP -- simpler than Guardian's `NWListener` server).
- **Bypass detection**: walk `ps` process tree for `--dangerously-skip-permissions`; if found, exit 0 without logging.
- **Dual-config resolution**: `~/.config/heimdall/config.toml` > `~/.claude/usage-tracker.toml` > bundled default. Port Guardian's `_find_config_path()` pattern.
- New subcommand: `claude-usage-tracker hook install` -- writes hook entry into `~/.claude/settings.json` with absolute path to `heimdall-hook` binary. `hook uninstall` removes it cleanly.
- Fire-and-forget by design: hook *always* outputs `{}` (no permission decisions) and exits within ~50 ms.
- Fallback safety: on any error, exit 0 silently -- never block Claude Code's flow.

### Acceptance

- Hook install + one Claude Code session produces `live_events` rows within 1 second of tool invocations.
- `hook uninstall` leaves `settings.json` byte-identical to pre-install state (modulo the hook entry).
- Timing test: hook p99 latency < 100 ms on cold invocation.
- Bypass-mode integration test: crafted `ps` ancestry with `--dangerously-skip-permissions` results in zero `live_events` rows.

### Effort: M (2--3 days)

---

## Phase 20 -- File-Watcher Auto-Refresh + Usage-Limits Source **[x]**

**Motivation:** Two live-update patterns from Guardian that compose naturally. The 30-second dashboard polling loop gets replaced by a file watcher on `~/.claude/`, and a new data source (`*-usage-limits` files) adds a rate-window signal that works without OAuth credentials.

**Source:** [Claude-Guardian/app/ClaudeGuardian/Sources/ClaudeAnalytics.swift](https://github.com/anshaneja5/Claude-Guardian/blob/main/app/ClaudeGuardian/Sources/ClaudeAnalytics.swift) -- `AnalyticsFileWatcher` class (lines 593--660, 2s debounce) and usage-limits reader (lines 404--420).

### Deliverables

- Add `notify = "6"` crate dependency (Rust equivalent of `DispatchSource.makeFileSystemObjectSource`).
- `src/scanner/watcher.rs`:
  - Watch `~/.claude/projects/` recursively for `Write`/`Create` events.
  - Debounce 2 s to coalesce bursty writes.
  - On event: enqueue incremental scan (not full rescan).
- Server push: WebSocket or SSE endpoint (`/api/stream`) emitting `scan_completed` events for dashboard live reload.
- New provider-adjacent parser: `src/scanner/usage_limits.rs` reading `~/.claude/projects/**/*-usage-limits` into `rate_window_history` table.
- Store parsed fields: `five_hour_pct`, `seven_day_pct`, `five_hour_reset_ts`, `seven_day_reset_ts`.
- Dashboard: `RateWindowCard.tsx` gains a "file-derived" source indicator as fallback when OAuth unavailable.

### Acceptance

- Touching a JSONL file triggers dashboard refresh in < 3 s end-to-end.
- Usage-limits ingestion reconciles against OAuth response when both present (±1 % tolerance).
- File watcher survives symlinks and recovers from `~/.claude/` directory being temporarily missing.

### Effort: M (2 days)

---

## Phase 21 -- Cache Token Breakdown + Hit Rate Metric

**Motivation:** Heimdall already tracks the four token types (input / output / cache-write / cache-read), but treats them as implementation detail. Guardian's analytics surface them as first-class columns with per-type cost. The derived "cache hit rate" metric is viral: users immediately want to optimize it.

**Source:** [Claude-Guardian/app/ClaudeGuardian/Sources/ClaudeAnalytics.swift](https://github.com/anshaneja5/Claude-Guardian/blob/main/app/ClaudeGuardian/Sources/ClaudeAnalytics.swift) -- four-way cost computation at lines 213--218 (Sonnet/Opus/Haiku pricing tables).

### Deliverables

- New API fields in `/api/data` response: `{input_tokens, output_tokens, cache_write_tokens, cache_read_tokens, cache_write_cost_nanos, cache_read_cost_nanos, cache_hit_rate}`.
- Cache hit rate formula: `cache_read / (cache_read + input_tokens)` -- document the denominator choice in `DESIGN.md`.
- Dashboard: new `CacheEfficiencyCard.tsx` -- percentage with industrial-monochrome progress bar, tooltip showing absolute savings vs. no-cache baseline.
- `ModelCostTable.tsx`: add cache-read and cache-write columns with share-of-cost micro-bars.

### Acceptance

- Per-type costs sum exactly to total cost within integer-nanos precision.
- Cache hit rate is 0 when no cache was used (no div-by-zero).
- Hit rate displays as `--` when both cache fields are zero to distinguish from "0 % hit rate".

### Effort: S (1 day)

---

## Phase 22 -- macOS Distribution Hardening **[x]**

**Motivation:** Heimdall ships as `cargo install --git` today. Guardian's distribution stack (Homebrew cask with lifecycle hooks + LaunchAgent for autostart + universal arm64/x86_64 binary via `lipo`) is the gold standard for macOS developer tools. Extends Phase 11 with macOS-specific polish.

**Source:** [Claude-Guardian/homebrew/claudeguardian.rb](https://github.com/anshaneja5/Claude-Guardian/blob/main/homebrew/claudeguardian.rb) (cask formula), [Claude-Guardian/post-install.sh](https://github.com/anshaneja5/Claude-Guardian/blob/main/post-install.sh) (LaunchAgent plist generation at lines 146--167), [Claude-Guardian/build-app.sh](https://github.com/anshaneja5/Claude-Guardian/blob/main/build-app.sh) (universal binary via `lipo` at lines 25--51).

### Deliverables

1. **Universal macOS binary.** Release workflow builds both `aarch64-apple-darwin` and `x86_64-apple-darwin`, then merges with `lipo -create -output heimdall`. Distribute a single binary that runs natively on both Apple Silicon and Intel.
2. **Homebrew tap + cask formula.** New repo `heimdall/homebrew-tap` with `Casks/heimdall.rb`:
   - `postflight` runs `xattr -cr` (quarantine strip) and an optional `heimdall scheduler install --interval=hourly` (from Phase 15).
   - `preflight` runs `heimdall scheduler uninstall` cleanly on upgrades.
   - `zap` stanza documents all user data locations: `~/.claude/usage-tracker.toml`, `~/.cache/heimdall/`, `~/Library/LaunchAgents/dev.heimdall.*.plist`, SQLite DB path.
3. **LaunchAgent plist generator.** Optional `heimdall daemon install` writes `~/Library/LaunchAgents/dev.heimdall.daemon.plist` with `RunAtLoad: true`, `StartInterval` or `KeepAlive` mode, logs to `~/Library/Logs/heimdall/`.
4. **README macOS install path.** Add one-liner: `brew install heimdall/tap/heimdall`.

### Acceptance

- Single binary runs on M-series and Intel without rebuild; verified via CI matrix or `lipo -info`.
- `brew install` + `brew uninstall` round-trip leaves no stray files; `brew zap` cleanly removes user data with explicit confirmation.
- LaunchAgent daemon survives logout/login; logs rotate sensibly.

### Effort: M (2 days; overlaps Phase 11)

---

## Deferred / Explicitly Out of Scope

- **TUI dashboard.** Heimdall chose web; duplicating in `ratatui` is a lot of code for marginal value.
- **Cloud sync / hosted dashboard.** Violates the "strictly local" architectural principle.
- **LLM-based classification.** Determinism is a feature; regex classifier stays.
- **Prometheus/OTel exporters.** Webhooks handle the real-time use case.
- **Permission-approval UI / mascot layer** (from Claude-Guardian). Orthogonal problem space; Heimdall is observational, not interventional.
- **Blocking hook polling loop** (from Claude-Guardian). Guardian needs approval round-trips; Heimdall's hooks are fire-and-forget observational writes only.
- **Localhost HTTP IPC between hook and daemon** (from Claude-Guardian). Unnecessary layer for Heimdall -- direct SQLite writes from the hook binary are simpler and faster.

---

## Sequencing Summary

| Phase | Feature | Source | Effort | Unlocks |
|-------|---------|--------|--------|---------|
| 0 | `Provider` trait | Codeburn | M | Phases 5, 7, 17 |
| 1 | CSV/JSON export | Codeburn | XS | Enterprise users |
| 2 | Task classifier | Codeburn + Third-Eye | S | Phase 6 drilldowns |
| 3 | One-shot rate | Codeburn | S | Compelling KPI |
| 4 | Currency conversion | Codeburn | S | International users |
| 5 | Cursor provider | Codeburn | M | Largest untapped user base |
| 6 | `optimize` command | Codeburn | L | Primary differentiator |
| 7 | OpenCode/Pi/Copilot | Codeburn | M each | Coverage parity |
| 8 | SwiftBar + security | Codeburn | S | macOS polish |
| 9 | LiteLLM pricing refresh | Codeburn + Third-Eye | S | Long-tail models |
| 10 | UI palette migration | Heimdall-native | M | Closes known gap |
| 11 | Release automation | Heimdall-native | M | Distribution |
| 12 | Tool-event cost attribution | Third-Eye | M | Phase 6 queries; per-tool insights |
| 13 | Heatmap + active-period avg | Third-Eye | S | Screenshot-worthy UX |
| 14 | Client-sent timezone | Third-Eye | S | Correct buckets for all users |
| 15 | Cross-platform scheduler | Third-Eye | M | Hands-off ingest |
| 16 | CC version donut | Third-Eye | XS | Upgrade-cost visibility |
| 17 | Cowork label resolution | Third-Eye | S | Readable Cowork projects |
| 18 | UI polish pack | Third-Eye | S | Feels-snappier UX |
| 19 | Real-time PreToolUse hook ingest | Claude-Guardian | M | Sub-second cost visibility |
| 20 | File-watcher + usage-limits source | Claude-Guardian | M | Live dashboard; OAuth-free rate data |
| 21 | Cache token breakdown + hit rate | Claude-Guardian | S | Cache-efficiency insight |
| 22 | macOS distribution hardening | Claude-Guardian | M | Homebrew + universal binary + LaunchAgent |

Total estimated effort: **~9--10 weeks focused work**.

**Dependency graph:**
- Phase 0 blocks Phases 5, 7, 17.
- Phase 2 blocks the category-dimension drilldowns in Phase 6.
- Phase 12 blocks the per-tool/per-file detectors in Phase 6 (both can start independently; Phase 6's richer detectors land after 12).
- Phase 14 should precede Phase 13 (heatmap is TZ-sensitive).
- Phase 19 is a prerequisite for fully exercising Phase 20's file-watcher (both can land independently; combined they close the real-time loop).
- Phase 22 extends Phase 11 -- sequence Phase 11 first for cross-platform release, then Phase 22 layers macOS polish.
- All other phases are independent and parallelizable.

---

## Tracking

Update the status legend at the top of each phase as work progresses. Link PRs inline. When a phase closes, move its full section into a `CHANGELOG.md` under the shipping version.
