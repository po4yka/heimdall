# Features

Detailed feature catalogue. See the [README](../README.md) for a high-level overview.

## Core

- **Multi-provider analytics** — Claude Code, Codex, Cursor, OpenCode, Pi, Copilot, Xcode CodingAssistant, Cowork, and Amp share one SQLite database and dashboard.
- **Incremental scanning** — only processes new/changed JSONL or SQLite sources; cache-invalidated by mtime+size where applicable.
- **Streaming deduplication** — provider-specific dedup keys (`message.id`, `turn_id` + cumulative tokens, `responseId`, `session_id:message_id`, `amp:<thread>:<event_id>`, etc.).
- **Interactive dashboard** — industrial monochrome UI (dark + light themes) with ApexCharts, sortable tables, CSV export, URL-persistent filters, keep-previous-data refresh.
- **Cost estimation** — single source of truth in Rust, volume discounts, integer-nanos precision, 4-way `CostBreakdown` (input / output / cache-read / cache-write), 5-tier fallback with hardcoded Claude/GPT-5 priority + LiteLLM pass-through for long-tail models.
- **Task categorization** — 13-category deterministic classifier (Coding, Debugging, FeatureDev, Testing, Git, Docs, Research, Refactor, DevOps, Config, Planning, Review, Other). Zero LLM calls.
- **One-shot rate tracking** — detects Edit→Bash→Edit retry cycles as a proxy for first-try success rate.
- **Cross-platform** — macOS, Linux.
- **Zero runtime dependencies** — single binary, no Python/Node/npm required at runtime.

## Real-time

- **`heimdall-hook`** — stdin-driven PreToolUse hook binary writes per-tool cost straight into SQLite (~50ms p99). Bypass mode (`--dangerously-skip-permissions`) short-circuits automatically. Captures Anthropic's hook-reported cost into `live_events.hook_reported_cost_nanos` alongside context-window fill. Install with `claude-usage-tracker hook install`.
- **`statusline` command** — PostToolUse hook that emits a single compact line for Claude Code's status bar: `MODEL | $SESSION / $TODAY / $BLOCK (Xh Ym left) | $/hr [TIER] | N tokens (XX%)`. Hybrid time + transcript-mtime cache with PID semaphore; warm-cache p99 under 5ms. Exits 0 in every path so Claude Code's status bar never breaks on error.
- **Visual burn-rate tier** — `--visual-burn-rate=<off|bracket|emoji|both>` maps tokens/min to Normal / Moderate / High with `[NORMAL]` / `[WARN]` / `[CRIT]` bracketed labels (default) or optional emoji.
- **Context-window tracking** — reads `context_window.total_input_tokens / context_window_size` from the hook payload, falls back to parsing the transcript JSONL for the most recent assistant turn. Configurable severity thresholds surface `[WARN]` at 50% and `[CRIT]` at 80% so users know when to compact.
- **Dual cost source reconciliation** — `--cost-source=both` displays Anthropic's hook-reported cost alongside Heimdall's local estimate (`$0.12 hook / $0.14 local`) with an inline `[WARN: cost drift]` when divergence exceeds 10%.
- **File-watcher auto-refresh** — `dashboard --watch` enables a `notify`-backed watcher with 2s debounce that drives in-process rescans plus an `/api/stream` SSE channel.
- **Usage-limits file source** — parses `~/.claude/**/*-usage-limits` files into `rate_window_history`; provides OAuth-free rate-window data.

## Real-time monitoring (via OAuth)

- **Rate window tracking** — 5-hour session, 7-day weekly, and per-model (Opus/Sonnet) quotas with progress bars and reset countdowns.
- **Plan detection** — automatically identifies Max/Pro/Team/Enterprise from Claude credentials.
- **Monthly budget tracking** — spend vs limit progress bar from OAuth API.
- **Session depletion alerts** — inline `[ERROR: ...]` status next to the rate-window cards when quota runs out or restores.
- **Auto token refresh** — refreshes expired OAuth tokens automatically.

The full OAuth handshake is documented in [auth.md](auth.md).

## Analytics

- **5-hour billing blocks with burn rate + projection** — `blocks` subcommand models Claude's actual billing window; the active block shows elapsed/remaining time, tokens/min burn rate, cost/hour, and a linear projection of end-of-block cost. Single-entry blocks omit the projection (matches ccusage semantics). Per-provider session-length defaults via `[blocks.session_length_by_provider]` for Codex (1h), Amp (24h), etc.
- **Token quota tracking** — `blocks --token-limit=<N|max>` injects live `REMAINING` and `PROJECTED` rows under the active block with green/warn/danger severity markers; API + dashboard card render the same quota progress bar (red only at danger).
- **Gap block visualization** — inactive billing windows render as `(Nh Mm gap)` pseudo-rows so the time axis stays continuous. Suppressible with `--no-gaps`.
- **Weekly aggregation** — `weekly` subcommand groups by ISO calendar week via SQLite `strftime('%Y-%W', ...)`; `--start-of-week=<monday|sunday|...>` is configurable. Dashboard toggles Day/Week bucket via FilterBar.
- **Cost reconciliation panel** — new `/api/cost-reconciliation` endpoint + dashboard panel shows hook-reported vs locally calculated totals over a rolling day/week/month window, with a per-day breakdown table. Red accent only when divergence > 10%.
- **7×24 activity heatmap** — CSS-grid heatmap with monochrome opacity ladder, timezone-aware bucketing.
- **Active-period averaging** — `avg / active day` divided by days with non-zero spend (not calendar days); tooltip documents the divisor.
- **Cache efficiency card** — cache hit-rate percentage with industrial progress bar; formula `cache_read / (cache_read + input_tokens)`.
- **Version distribution donut** — CC-version breakdown with URL-persistent cost / calls / tokens metric switcher.
- **Tool-event cost attribution** — each call's cost is split evenly across its tool invocations; per-MCP and per-file cost queries become tractable.
- **Codex local log support** — scans archived Codex session JSONL and estimates cost from OpenAI API pricing.
- **Amp credit tracking** — new provider at `~/.local/share/amp/threads/*.json` populates `turns.credits` (nullable, non-USD); dashboard tables show a conditional CREDITS column only when the filtered view contains Amp rows, and a `Total Credits` stat card appears alongside `Est. Cost` whenever the active filter has a non-zero credit total.
- **Estimation confidence tiers** — distinguishes exact pricing matches from fallback/unknown model estimates.
- **OpenAI org reconciliation** — optional Codex comparison against official OpenAI organization usage buckets.
- **Subagent session linking** — tracks parent vs subagent token usage with breakdown panel.
- **Entrypoint breakdown** — usage split by CLI, VS Code, JetBrains.
- **Service tier tracking** — inference region and service tier visibility.
- **Cowork label resolution** — walks `local-agent-mode-sessions/<slug>/audit.jsonl` to replace procedurally-generated session slugs with the first user message as a human-readable project label.
- **Project aliases** — map mangled directory-hash slugs to friendly names via `[project_aliases]` config or repeatable `--project-alias SLUG=Name` flag; URL filter still uses the raw slug so bookmarks survive alias changes.
- **Per-model breakdown** — `today --breakdown` and `stats --breakdown` group per-model rows under each provider total with `└─` indented sub-rows.
- **Currency conversion** — display-only conversion to 162 currencies via Frankfurter with 24h disk cache and hardcoded fallback; USD nanos remain the storage representation.
- **Locale-aware dates** — `--locale=<BCP-47>` (ja-JP, de-DE, ...) localizes date columns; defaults to `$LANG` or `en-US`. SQL / JSON / CSV date columns remain ISO for scriptability.
- **Compact CLI mode** — `--compact` drops cache columns and condenses model names to fit within 80 cols. Heimdall auto-detects narrow terminals and hints once to stderr.
- **Cost trend sparkline** — 7-day mini chart.
- **Project search/filter** — text search across projects with URL persistence; searches both raw slug and display-name alias.
- **Paginated sessions** — 25 per page with prev/next navigation.

## MCP server (Model Context Protocol)

Heimdall exposes 9 tools to Claude / Claude Desktop / Cursor at inference time via stdio or HTTP transport. Gated behind the default-on `mcp` cargo feature so `cargo build --no-default-features` excludes the subcommand.

- **Tools:** `heimdall_today`, `heimdall_stats`, `heimdall_weekly`, `heimdall_sessions`, `heimdall_blocks_active`, `heimdall_optimize_grade`, `heimdall_rate_windows`, `heimdall_context_window`, `heimdall_quota`, `heimdall_cost_reconciliation`.
- **Install:** `claude-usage-tracker mcp install --client=<claude-code|claude-desktop|cursor>` writes a tagged entry with a `_heimdall_mcp_version` sentinel so uninstall only removes Heimdall's own entry (user customizations preserved).
- **Transports:** stdio (default; used by most MCP clients) and HTTP-SSE on an axum subrouter.
- **Ingest-time safe:** SQLite access goes through `tokio::task::spawn_blocking`; tracing goes to stderr so stdout stays pure JSON-RPC.

## `optimize` waste detector

`claude-usage-tracker optimize [--format=text|json]` runs five detectors and produces an A–F health grade:

- `ClaudeMdBloatDetector` — estimates tokens `× session count × input rate`.
- `UnusedMcpDetector` — MCP servers in `~/.claude/settings.json` never invoked in recorded sessions.
- `GhostAgentDetector` — agent definitions in `~/.claude/agents/*.md` never referenced.
- `RereadDetector` — same file read ≥3× per session.
- `BashNoiseDetector` — trivial commands (`ls`, `pwd`, `git status`, …) repeated ≥5× per session.

## Agent status monitoring

- **Upstream provider health** — polls `status.claude.com` and `status.openai.com` on every `/api/agent-status` request (cached 60 s). Displays an **Agent Status** card in the dashboard alongside rate-window cards.
- **Dashboard card** — two rows (Claude, OpenAI/Codex); monochrome dot at three opacity levels; red only on `major`/`critical`. Expand/collapse per-component table and active incident list. URL-persistent via `?agent_status_expanded=1`.
- **Rolling uptime** — 30-day and 7-day uptime percentages per component, computed from Heimdall's own history (no external scraping). Requires ≥10 samples in the window before a value appears; `under_maintenance` counts as not-up for SLA-style semantics.
- **Webhook alerts** — fires `agent_status_degraded` / `agent_status_restored` on severity-threshold crossings. Alert floor is **Major** (minor degradations render on the dashboard but do not page).
- **ETag support** — conditional GET (`If-None-Match`) for Claude so unchanged status returns 304 with no body. OpenAI two-call flow polls cold.

## Community signal (opt-in, via StatusGator)

- **Crowdsourced leading indicator** — polls StatusGator's free-tier API v3 for Downdetector-adjacent community reports on Claude and OpenAI services. Off by default; opt in with `[status_aggregator] enabled = true` and a `STATUSGATOR_API_KEY` env var.
- **Clearly labeled as crowdsourced** — renders as a separate "COMMUNITY SIGNAL (VIA STATUSGATOR)" section inside the `Agent Status` card's expanded view so users don't confuse it with official infrastructure telemetry.
- **Divergence-only webhook** — `community_signal_spike` fires ONLY when the crowd reports a spike AND the official `status.*.com` indicator is still `none`/`minor`. Captures the leading-indicator value without duplicating the existing `agent_status_degraded` webhook once the official page catches up.
- **Trait-based backend** — `StatusAggregatorBackend` trait in place; StatusGator is the only backend in v1; IsDown is future-pluggable without touching call sites.
- **Legal/ToS alignment** — Heimdall deliberately does NOT scrape Downdetector (their Fair Use ToS prohibits it) and does NOT use their $2,083/mo Enterprise API. StatusGator legitimately aggregates the same crowd signal and exposes a free-tier documented API.

## Extensibility

- **Config file** — `~/.claude/usage-tracker.{json,toml}` for all settings. JSON ships a `$schema` for IDE autocomplete; generate via `claude-usage-tracker config schema`. Dual-path resolver adds `$HEIMDALL_CONFIG` and `~/.config/heimdall/config.{json,toml}`. JSON is preferred at each path when both exist.
- **Per-command overrides** — `commands.blocks.token_limit`, `commands.statusline.context_low_threshold`, etc. win over the flat config. CLI flags still win over everything.
- **Custom pricing overrides** — per-model rate customization in config.
- **Webhook notifications** — POST to URL on session depletion, cost threshold, agent status transition, or community-signal spike divergence.
- **JSON API** — all dashboard data available via REST endpoints, incl. SSE stream. See [api.md](api.md).
- **MCP** — 9 tools exposed over stdio + HTTP; same data as REST with AI-consumable schemas.
- **Provider plugin pattern** — add a new scanner provider in a single file under `src/scanner/providers/`; see [AGENTS.md](../AGENTS.md).
- **Detector plugin pattern** — add a waste detector in a single file under `src/optimizer/`.

## Codex project assets

The repository ships Codex-native project assets for contributors who use Codex:

- **Custom subagents** live in `.codex/agents/`:
  - `heimdall_explorer` for read-only codebase mapping
  - `heimdall_reviewer` for read-only regression review
  - `heimdall_provider_worker` for scanner/provider implementation
  - `heimdall_dashboard_worker` for dashboard implementation
- **Repo-scoped skills** live in `.agents/skills/`:
  - `heimdall-rust-test-runner` for targeted Rust and UI verification
  - `heimdall-pr-review` for findings-first branch and diff review
  - `heimdall-fix-unwraps` for removing production Rust `.unwrap()` paths safely
  - `heimdall-rust-dependency-audit` for dependency and security audit passes
  - `heimdall-rust-binary-audit` for release-size and bloat analysis
  - `heimdall-scanner-provider` for new or changed providers
  - `heimdall-schema-evolution` for additive schema/data-flow changes
  - `heimdall-dashboard` for `src/ui/` work and committed UI artifacts

Claude Code users have matching repo-local prompts under `.claude/agents/` and `.claude/commands/`.

For repo-specific `desloppify` setup, excludes, validation commands, and ready-to-paste prompts, see [desloppify.md](desloppify.md).
