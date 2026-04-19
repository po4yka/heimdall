# Heimdall -- Claude Usage Tracker

A fast, local analytics platform for Claude Code, Codex, Cursor, OpenCode, Pi, Copilot, Xcode CodingAssistant, and Cowork sessions. Built in Rust.

Reads local transcripts written by every supported tool, then presents an interactive dashboard with cost estimates, cache efficiency, task categorization, activity heatmap, provider-aware filtering, waste-detection grade, and rate-limit tracking -- all running entirely on your machine. Two binaries ship together: `claude-usage-tracker` for CLI + dashboard, `heimdall-hook` for sub-second real-time ingest.

## Features

### Core

- **Multi-provider analytics** -- Claude Code, Codex, Cursor, OpenCode, Pi, Copilot, Xcode CodingAssistant, and Cowork share one SQLite database and dashboard.
- **Incremental scanning** -- only processes new/changed JSONL or SQLite sources; cache-invalidated by mtime+size where applicable.
- **Streaming deduplication** -- provider-specific dedup keys (`message.id`, `turn_id` + cumulative tokens, `responseId`, `session_id:message_id`, etc.).
- **Interactive dashboard** -- industrial monochrome UI (dark + light themes) with ApexCharts, sortable tables, CSV export, URL-persistent filters, keep-previous-data refresh.
- **Cost estimation** -- single source of truth in Rust, volume discounts, integer-nanos precision, 4-way CostBreakdown (input / output / cache-read / cache-write), 5-tier fallback with hardcoded Claude/GPT-5 priority + LiteLLM pass-through for long-tail models.
- **Task categorization** -- 13-category deterministic classifier (Coding, Debugging, FeatureDev, Testing, Git, Docs, Research, Refactor, DevOps, Config, Planning, Review, Other). Zero LLM calls.
- **One-shot rate tracking** -- detects Edit->Bash->Edit retry cycles as a proxy for first-try success rate.
- **Cross-platform** -- macOS, Linux, Windows.
- **Zero runtime dependencies** -- single binary, no Python/Node/npm required at runtime.

### Real-time

- **`heimdall-hook`** -- stdin-driven PreToolUse hook binary writes per-tool cost straight into SQLite (~50ms p99). Bypass mode (`--dangerously-skip-permissions`) short-circuits automatically. Install with `claude-usage-tracker hook install`.
- **File-watcher auto-refresh** -- `dashboard --watch` enables a `notify`-backed watcher with 2s debounce that drives in-process rescans plus an `/api/stream` SSE channel.
- **Usage-limits file source** -- parses `~/.claude/**/*-usage-limits` files into `rate_window_history`; provides OAuth-free rate-window data.

### Real-time monitoring (via OAuth)

- **Rate window tracking** -- 5-hour session, 7-day weekly, and per-model (Opus/Sonnet) quotas with progress bars and reset countdowns.
- **Plan detection** -- automatically identifies Max/Pro/Team/Enterprise from Claude credentials.
- **Monthly budget tracking** -- spend vs limit progress bar from OAuth API.
- **Session depletion alerts** -- inline `[ERROR: ...]` status next to the rate-window cards when quota runs out or restores.
- **Auto token refresh** -- refreshes expired OAuth tokens automatically.

### Analytics

- **7×24 activity heatmap** -- CSS-grid heatmap with monochrome opacity ladder, timezone-aware bucketing.
- **Active-period averaging** -- `avg / active day` divided by days with non-zero spend (not calendar days); tooltip documents the divisor.
- **Cache efficiency card** -- cache hit-rate percentage with industrial progress bar; formula `cache_read / (cache_read + input_tokens)`.
- **Version distribution donut** -- CC-version breakdown with URL-persistent cost / calls / tokens metric switcher.
- **Tool-event cost attribution** -- each call's cost is split evenly across its tool invocations; per-MCP and per-file cost queries become tractable.
- **Codex local log support** -- scans archived Codex session JSONL and estimates cost from OpenAI API pricing.
- **Estimation confidence tiers** -- distinguishes exact pricing matches from fallback/unknown model estimates.
- **OpenAI org reconciliation** -- optional Codex comparison against official OpenAI organization usage buckets.
- **Subagent session linking** -- tracks parent vs subagent token usage with breakdown panel.
- **Entrypoint breakdown** -- usage split by CLI, VS Code, JetBrains.
- **Service tier tracking** -- inference region and service tier visibility.
- **Cowork label resolution** -- walks `local-agent-mode-sessions/<slug>/audit.jsonl` to replace procedurally-generated session slugs with the first user message as a human-readable project label.
- **Currency conversion** -- display-only conversion to 162 currencies via Frankfurter with 24h disk cache and hardcoded fallback; USD nanos remain the storage representation.
- **Cost trend sparkline** -- 7-day mini chart.
- **Project search/filter** -- text search across projects with URL persistence.
- **Paginated sessions** -- 25 per page with prev/next navigation.

### `optimize` waste detector

`claude-usage-tracker optimize [--format=text|json]` runs five detectors and produces an A–F health grade:

- `ClaudeMdBloatDetector` -- estimates tokens `× session count × input rate`.
- `UnusedMcpDetector` -- MCP servers in `~/.claude/settings.json` never invoked in recorded sessions.
- `GhostAgentDetector` -- agent definitions in `~/.claude/agents/*.md` never referenced.
- `RereadDetector` -- same file read ≥3× per session.
- `BashNoiseDetector` -- trivial commands (`ls`, `pwd`, `git status`, ...) repeated ≥5× per session.

### CLI subcommands

- `scan`, `today`, `stats`, `dashboard`, `dashboard --watch`
- `export --format=<csv|json|jsonl> --period=<today|week|month|year|all> --output=<path>` (optional `--provider`, `--project`)
- `optimize --format=<text|json>`
- `scheduler install|uninstall|status [--interval=<hourly|daily>]` (platform-native via launchd / cron / schtasks)
- `daemon install|uninstall|status` (macOS-only always-on dashboard via launchd with `KeepAlive: true`)
- `hook install|uninstall|status` (wires `heimdall-hook` into `~/.claude/settings.json` PreToolUse)
- `db reset [--yes]` (TTY-guarded destructive wipe — type `rebuild` interactively, or pass `--yes` in non-TTY)
- `menubar` (SwiftBar-formatted output for macOS menu-bar widgets)
- `pricing refresh` (fetch LiteLLM catalogue into `~/.cache/heimdall/litellm_pricing.json`)

### Agent status monitoring

- **Upstream provider health** -- polls `status.claude.com` and `status.openai.com` on every `/api/agent-status` request (cached 60 s). Displays an **Agent Status** card in the dashboard alongside rate-window cards.
- **Dashboard card** -- two rows (Claude, OpenAI/Codex); monochrome dot at three opacity levels; red only on `major`/`critical`. Expand/collapse per-component table and active incident list. URL-persistent via `?agent_status_expanded=1`.
- **Rolling uptime** -- 30-day and 7-day uptime percentages per component, computed from Heimdall's own history (no external scraping). Requires ≥10 samples in the window before a value appears; `under_maintenance` counts as not-up for SLA-style semantics.
- **Webhook alerts** -- fires `agent_status_degraded` / `agent_status_restored` on severity-threshold crossings. Alert floor is **Major** (minor degradations render on the dashboard but do not page).
- **ETag support** -- conditional GET (`If-None-Match`) for Claude so unchanged status returns 304 with no body. OpenAI two-call flow polls cold.

### Community signal (opt-in, via StatusGator)

- **Crowdsourced leading indicator** -- polls StatusGator's free-tier API v3 for Downdetector-adjacent community reports on Claude and OpenAI services. Off by default; opt in with `[status_aggregator] enabled = true` and a `STATUSGATOR_API_KEY` env var.
- **Clearly labeled as crowdsourced** -- renders as a separate "COMMUNITY SIGNAL (VIA STATUSGATOR)" section inside the `Agent Status` card's expanded view so users don't confuse it with official infrastructure telemetry.
- **Divergence-only webhook** -- `community_signal_spike` fires ONLY when the crowd reports a spike AND the official `status.*.com` indicator is still `none`/`minor`. Captures the leading-indicator value without duplicating the existing `agent_status_degraded` webhook once the official page catches up.
- **Trait-based backend** -- `StatusAggregatorBackend` trait in place; StatusGator is the only backend in v1; IsDown is future-pluggable without touching call sites.
- **Legal/ToS alignment** -- Heimdall deliberately does NOT scrape Downdetector (their Fair Use ToS prohibits it) and does NOT use their $2,083/mo Enterprise API. StatusGator legitimately aggregates the same crowd signal and exposes a free-tier documented API.

### Extensibility

- **Config file** -- `~/.claude/usage-tracker.{json,toml}` for all settings (JSON ships a `$schema` for IDE autocomplete). Dual-path resolver adds `$HEIMDALL_CONFIG` and `~/.config/heimdall/config.{json,toml}`. JSON is preferred at each path when both exist.
- **Custom pricing overrides** -- per-model rate customization in config.
- **Webhook notifications** -- POST to URL on session depletion, cost threshold, agent status transition, or community-signal spike divergence.
- **JSON API** -- all dashboard data available via REST endpoints, incl. SSE stream.
- **Provider plugin pattern** -- add a new scanner provider in a single file under `src/scanner/providers/`; see [AGENTS.md](AGENTS.md).
- **Detector plugin pattern** -- add a waste detector in a single file under `src/optimizer/`.

## Install

### Prebuilt binary (recommended)

Download the tarball for your platform from the [GitHub Releases](https://github.com/po4yka/heimdall/releases) page, extract it, and move both binaries to `/usr/local/bin`.

**macOS (recommended): universal binary — runs natively on Apple Silicon and Intel**

```bash
# macOS universal binary (arm64 + x86_64 in one file, lipo-merged at release time)
VERSION=$(curl -fsSL https://api.github.com/repos/po4yka/heimdall/releases/latest | jq -r '.tag_name')
curl -fsSL "https://github.com/po4yka/heimdall/releases/download/${VERSION}/heimdall-${VERSION}-universal-apple-darwin.tar.gz" \
  | tar xz --strip-components=1 -C /usr/local/bin
```

**All platforms one-liner (requires curl, jq, tar):**

```bash
# Replace PLATFORM with your target (see table below)
PLATFORM="aarch64-apple-darwin"
VERSION=$(curl -fsSL https://api.github.com/repos/po4yka/heimdall/releases/latest | jq -r '.tag_name')
curl -fsSL "https://github.com/po4yka/heimdall/releases/download/${VERSION}/heimdall-${VERSION}-${PLATFORM}.tar.gz" \
  | tar xz --strip-components=1 -C /usr/local/bin
```

Supported platforms:

| Platform | Archive |
|----------|---------|
| macOS (universal — Apple Silicon + Intel) | `heimdall-<version>-universal-apple-darwin.tar.gz` |
| macOS (Apple Silicon only) | `heimdall-<version>-aarch64-apple-darwin.tar.gz` |
| macOS (Intel only) | `heimdall-<version>-x86_64-apple-darwin.tar.gz` |
| Linux x86\_64 | `heimdall-<version>-x86_64-unknown-linux-gnu.tar.gz` |
| Linux ARM64 | `heimdall-<version>-aarch64-unknown-linux-gnu.tar.gz` |
| Windows x86\_64 | `heimdall-<version>-x86_64-pc-windows-msvc.zip` |

Verify the download against the published checksums:

```bash
curl -fsSL "https://github.com/po4yka/heimdall/releases/download/${VERSION}/SHA256SUMS.txt" | sha256sum --check --ignore-missing
```

### Homebrew (macOS)

```bash
brew tap heimdall/tap
brew install heimdall/tap/heimdall
```

_The tap repository (`heimdall/homebrew-tap`) must be created and published by the maintainer before this works. The cask formula skeleton lives at `packaging/homebrew/heimdall.rb` in this repo._

### Daemon mode (macOS only)

Run the dashboard as a persistent background service that starts automatically at login:

```bash
claude-usage-tracker daemon install
claude-usage-tracker daemon status
claude-usage-tracker daemon uninstall
```

The daemon runs `claude-usage-tracker dashboard --host localhost --port 8080 --watch` under launchd with `KeepAlive: true`. Logs are written to `~/Library/Logs/heimdall/`. Linux systemd and Windows Service support is deferred to a future release.

### Scheduler (cross-platform)

For just periodic ingest (not a persistent dashboard), use the `scheduler` subcommand:

```bash
claude-usage-tracker scheduler install --interval=hourly
claude-usage-tracker scheduler status
claude-usage-tracker scheduler uninstall
```

It writes a native schedule entry: a launchd plist on macOS, a tagged `# heimdall-scheduler:v1` crontab line on Linux, or a `HeimdallScan` task on Windows. Runs at minute `:17` to avoid scheduler pile-up.

### Real-time hook

Install the PreToolUse hook so Claude Code reports every tool invocation's cost in real time:

```bash
claude-usage-tracker hook install
claude-usage-tracker hook status
claude-usage-tracker hook uninstall
```

This appends a tagged hook entry to `~/.claude/settings.json` that runs `heimdall-hook` on each tool call. A `settings.json.heimdall-bak` backup is written before every modification. The hook binary is fire-and-forget: ~50ms p99, never blocks Claude Code, and automatically respects bypass mode.

### From source

```bash
cargo install --git https://github.com/po4yka/heimdall

# Or build locally
git clone https://github.com/po4yka/heimdall
cd heimdall
cargo build --release
sudo cp target/release/claude-usage-tracker target/release/heimdall-hook /usr/local/bin/
```

## Usage

```bash
# Scan transcripts and open the dashboard
claude-usage-tracker dashboard

# Dashboard with live auto-refresh (file-watcher + SSE)
claude-usage-tracker dashboard --watch

# Quick terminal summary of today's usage
claude-usage-tracker today
claude-usage-tracker today --json

# All-time statistics
claude-usage-tracker stats
claude-usage-tracker stats --json

# Scan only (update database without UI)
claude-usage-tracker scan

# Custom transcript directory
claude-usage-tracker scan --projects-dir /path/to/projects

# Custom host/port
claude-usage-tracker dashboard --host 0.0.0.0 --port 9090

# Export aggregated usage
claude-usage-tracker export --format=csv --period=month --output=usage.csv
claude-usage-tracker export --format=json --period=all --output=all.json --provider=claude

# Run the waste detector
claude-usage-tracker optimize               # human-readable text
claude-usage-tracker optimize --format=json

# Refresh long-tail model pricing
claude-usage-tracker pricing refresh

# SwiftBar menu-bar widget (macOS)
claude-usage-tracker menubar
```

### Filter output with `--jq`

Every report command accepts `--jq <filter>` for in-tool post-processing
(implies `--json`). No system `jq` needed.

```bash
claude-usage-tracker today --jq '.total_estimated_cost'
claude-usage-tracker stats --jq '.by_model[] | select(.provider == "claude") | .model'
claude-usage-tracker weekly --jq '.weeks | length'
claude-usage-tracker blocks --jq '.[0].estimated_cost'
claude-usage-tracker optimize --jq '.grade'
claude-usage-tracker export --format=jsonl --jq '.model' --output=-
```

Filter errors exit with status 2. Empty results (null or no match) produce no output and exit 0.

## Configuration

Create `~/.claude/usage-tracker.toml` or `~/.claude/usage-tracker.json` (all fields optional). The dual-path resolver also checks `$HEIMDALL_CONFIG` and `~/.config/heimdall/config.{json,toml}`. When both formats exist at the same location, JSON takes precedence.

### JSON format with IDE autocomplete

Heimdall also accepts JSON config (`~/.claude/usage-tracker.json`). Add a `$schema` key
for VS Code / IntelliJ autocomplete:

```json
{
  "$schema": "https://raw.githubusercontent.com/po4yka/heimdall/main/schemas/heimdall.config.schema.json",
  "blocks": { "token_limit": 1000000 },
  "statusline": { "context_low_threshold": 0.5, "context_medium_threshold": 0.8 }
}
```

Per-command overrides nest under `commands.<name>` and win over flat defaults:

```json
{
  "blocks": { "token_limit": 500000 },
  "commands": { "blocks": { "token_limit": 1000000 } }
}
```

The `commands.blocks.token_limit` wins.

Generate/refresh the schema locally:

```bash
claude-usage-tracker config schema > schemas/heimdall.config.schema.json
```

```toml
# Custom project directories (overrides platform defaults)
projects_dirs = ["/home/user/projects"]

# Database location (default: ~/.claude/usage.db)
db_path = "/custom/path/usage.db"

# Dashboard server
host = "0.0.0.0"
port = 9090

# Display preferences (currency conversion is display-only; USD nanos remain in storage)
[display]
currency = "EUR"   # ISO 4217 code; default "USD"

# OAuth settings (reads ~/.claude/.credentials.json)
[oauth]
enabled = true
refresh_interval = 60

# Optional OpenAI organization usage reconciliation for Codex API-backed usage
[openai]
enabled = true
admin_key_env = "OPENAI_ADMIN_KEY"
refresh_interval = 300
lookback_days = 30

# Custom pricing overrides ($/MTok)
[pricing.my-custom-model]
input = 2.0
output = 8.0
cache_write = 2.5    # optional, defaults to input * 1.25
cache_read = 0.2     # optional, defaults to input * 0.10

# LiteLLM pricing source (pulls model_prices_and_context_window.json into a disk cache)
[pricing_source]
source = "litellm"      # or "static" (default)
refresh_hours = 24

# Webhook notifications
[webhooks]
url = "https://hooks.example.com/notify"
cost_threshold = 50.0
session_depleted = true
agent_status = true   # fire on major/critical provider transitions (default: true)

# Upstream coding-agent status monitoring
[agent_status]
enabled = true
refresh_interval = 60    # seconds between polls
claude_enabled = true
openai_enabled = true
alert_min_severity = "major"  # "minor" | "major" | "critical"

# Community signal via StatusGator — OFF by default.
# Opt in by setting enabled=true and exporting STATUSGATOR_API_KEY in your shell.
[status_aggregator]
enabled = false
provider = "statusgator"                    # trait-based; future backends pluggable
api_key_env = "STATUSGATOR_API_KEY"         # env var name; never store the key in TOML
refresh_interval = 300                      # seconds; 5-min cadence is friendly to the free tier
claude_services = ["claude-ai", "claude"]   # StatusGator service slugs
openai_services = ["openai", "chatgpt"]
spike_webhook = true                        # fire `community_signal_spike` on leading-indicator
```

### Project aliases

Map mangled Claude Code project slugs to human-readable names. Aliases apply
to CLI tables, dashboard labels, and CSV export — storage keeps the raw slug
for scriptability.

```toml
[project_aliases]
"-Users-po4yka-GitRep-heimdall" = "Heimdall"
"-Users-po4yka-GitRep-ccusage" = "ccusage"
```

CLI override (repeatable, wins over config):

```
heimdall today --project-alias="-Users-po4yka-GitRep-heimdall=Heimdall"
heimdall stats --project-alias="-Users-po4yka-GitRep-heimdall=Heimdall" \
               --project-alias="-Users-po4yka-GitRep-ccusage=ccusage"
```

## Data Sources

Automatically discovers sessions from:

| Tool | Path |
|------|------|
| Claude Code CLI | `~/.claude/projects/` |
| Claude Code subagents | `~/.claude/projects/<slug>/subagents/` |
| Claude Desktop Cowork | `~/.claude/local-agent-mode-sessions/<slug>/audit.jsonl` (for label resolution) |
| Xcode CodingAssistant (macOS) | `~/Library/Developer/Xcode/CodingAssistant/ClaudeAgentConfig/projects/` |
| Codex archived sessions | `~/.codex/archived_sessions/` |
| Codex live sessions (JSONL) | `~/.codex/sessions/` |
| Cursor | `~/Library/Application Support/Cursor/User/workspaceStorage/*/state.vscdb` (macOS), `~/.config/Cursor/...` (Linux), `%APPDATA%/Cursor/...` (Windows) |
| OpenCode | `~/Library/Application Support/opencode/*.db` (macOS) and platform equivalents |
| Pi | `~/.pi/sessions/*.jsonl` |
| GitHub Copilot | `~/Library/Application Support/Code/User/globalStorage/github.copilot-chat/` (VS Code), JetBrains paths (best-effort probe) |
| Claude usage-limits snapshots | `~/.claude/**/*-usage-limits` |
| Custom | `--projects-dir <PATH>` or config file |

## How It Works

1. **Scan** -- walks provider-specific filesystem paths for session logs (JSONL / SQLite / mixed-format).
2. **Parse** -- extracts provider-aware session metadata, per-turn token usage, subagent flags, service tier, tool invocations with captured arguments (file paths, bash commands) where present.
3. **Classify** -- 13-category regex classifier assigns each turn a task category using tool names + first user message heuristics.
4. **Estimate** -- computes turn-level API-equivalent cost snapshots with pricing version + confidence metadata; breaks down into input / output / cache-read / cache-write components that sum exactly.
5. **Attribute** -- splits each turn's cost evenly across its tool invocations (remainder to the first event) into `tool_events` so per-tool cost queries are tractable.
6. **Deduplicate** -- streaming events sharing the same provider-specific dedup key are collapsed (last record wins).
7. **Store** -- upserts into a local SQLite database at `~/.claude/usage.db`.
8. **Serve** -- axum HTTP server delivers the dashboard UI and JSON API.
9. **Reconcile** -- optionally compares Codex local estimates to OpenAI organization usage buckets.
10. **Monitor** -- polls Claude OAuth API for real-time rate windows (optional), parses usage-limits files as a fallback source.
11. **Watch & push** -- with `dashboard --watch`, a `notify`-backed file watcher triggers in-process rescans and broadcasts via `/api/stream` SSE; `heimdall-hook` writes live per-tool-call events directly into the DB.

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/` | Dashboard HTML |
| GET | `/api/data` | All dashboard data (models, sessions, daily, subagent, entrypoints, service tiers, cache efficiency, version summary) |
| GET | `/api/data?tz_offset_min=N&week_starts_on=N` | Timezone-aware bucketing for day-grouped metrics |
| GET | `/api/heatmap?period=<period>&tz_offset_min=N` | 7×24 cell grid + active-period averaging summary |
| GET | `/api/usage-windows` | Real-time rate windows, budget, identity (cached 60s) |
| GET | `/api/agent-status` | Upstream provider health: Claude (status.claude.com) + OpenAI (status.openai.com). Cached `refresh_interval` seconds; ETag conditional GET for Claude. Returns `AgentStatusSnapshot` JSON with rolling uptime. |
| GET | `/api/community-signal` | StatusGator-backed crowdsourced leading indicator. Returns `{"enabled": false}` when off; full `CommunitySignal` JSON when on. |
| POST | `/api/rescan` | Atomic full rescan |
| GET | `/api/stream` | Server-Sent Events broadcasting `scan_completed` from the file-watcher |
| GET | `/api/health` | Health check |

## Architecture

```
src/
  lib.rs               -- Library root shared between both binaries
  main.rs              -- Primary CLI (clap): scan/today/stats/dashboard/export/
                          optimize/scheduler/daemon/hook/db/menubar/pricing
  config.rs            -- TOML config file loading + dual-path resolver
  models.rs            -- Shared data types (Session, Turn, ToolEvent, CacheEfficiency, ...)
  pricing.rs           -- Single pricing source, 4-way CostBreakdown, 5-tier fallback
  currency.rs          -- Frankfurter USD->N conversion + 24h disk cache
  litellm.rs           -- LiteLLM catalogue fetch + cache
  tz.rs                -- TzParams for timezone-aware SQL bucketing
  export.rs            -- `export` subcommand
  menubar.rs           -- SwiftBar widget renderer + injection sanitizer
  db.rs                -- TTY-guarded `db reset` command
  webhooks.rs          -- Webhook notification system
  openai.rs            -- OpenAI organization usage reconciliation client
  agent_status/        -- Upstream provider health (status.claude.com + status.openai.com)
                          with rolling uptime computed from agent_status_history
  status_aggregator/   -- StatusGator community signal (opt-in, off by default)
  oauth/               -- Claude OAuth (credentials, refresh, API, models)
  scanner/
    classifier.rs      -- 13-category task classifier
    oneshot.rs         -- Edit->Bash->Edit retry detection
    cowork.rs          -- Ephemeral Cowork label resolution
    usage_limits.rs    -- Usage-limits file parser
    watcher.rs         -- `notify`-backed file watcher (--watch flag)
    provider.rs        -- Provider trait + SessionSource
    providers/         -- claude, codex, xcode, cursor, opencode, pi, copilot
  hook/                -- heimdall-hook binary (bypass, ingest, install)
  optimizer/           -- 5 waste detectors + A–F grade
  scheduler/           -- Cross-platform scheduler (launchd, cron, schtasks) + daemon
  server/              -- axum server, API endpoints, SSE stream, embedded assets
  ui/                  -- Preact + Tailwind v4 dashboard (compiled JS/CSS committed)
```

See [CLAUDE.md](CLAUDE.md) for the expanded architecture tree and [AGENTS.md](AGENTS.md) for development conventions and extension playbooks.

## Development

```bash
cargo build                      # build both binaries
cargo test                       # full suite (616+ tests across 4 suites)
cargo clippy -- -D warnings      # lint
cargo fmt --check                # format check
./node_modules/.bin/tsc --noEmit # TypeScript type check
npm run build:ui                 # recompile dashboard bundle
```

Release pipeline: `.github/workflows/release.yml` builds all 5 targets on `v*.*.*` tag push and produces a consolidated `SHA256SUMS.txt`. The universal macOS artifact is produced by a post-matrix `lipo` job. See [.github/RELEASING.md](.github/RELEASING.md) for the release-cutting playbook.

## Prior Art & Acknowledgements

Heimdall harvests patterns from three sibling projects in the local-AI-observability space:

- **[Codeburn](https://github.com/AgentSeal/codeburn)** (TypeScript CLI) — upstream session parser, 13-category classifier, provider plugin pattern, `optimize` waste detector concept, SwiftBar menubar, currency conversion.
  - Source last reviewed: 2026-04-18 (upstream commit `69268a9` — "docs: remove .claudeignore references from README", 2026-04-17).
- **[Third-Eye](https://github.com/fien-atone/third-eye)** (TypeScript web) — tool-event cost attribution, 7×24 heatmap, client-sent timezone handling, active-period averaging, cross-platform scheduler, CC-version tracking.
  - Source last reviewed: 2026-04-18 (upstream commit `2076cb3` — "v1.1.1: localize date-range preset buttons", 2026-04-15).
- **[Claude-Guardian](https://github.com/anshaneja5/Claude-Guardian)** (Swift + Python, macOS) — real-time PreToolUse cost injection, file-watcher auto-refresh, usage-limits file parsing, cache-token breakdown, Homebrew cask + LaunchAgent + universal-binary distribution stack.
  - Source last reviewed: 2026-04-18 (upstream commit `c3cc925` — "Fix SHA256 for v2.3.1 (GitHub repackaged zip)", 2026-03-27).

Also inspired by:

- **[phuryn/claude-usage](https://github.com/phuryn/claude-usage)** (Python) — local dashboard for Claude Code token usage, costs, and session history; Pro/Max progress bar.
  - Source last reviewed: 2026-04-18 (upstream commit `af507cd` — "docs: add CHANGELOG.md (#36)", 2026-04-09).
- **[CodexBar](https://github.com/steipete/CodexBar)** (macOS menu bar app) — usage stats for OpenAI Codex and Claude Code without requiring login.
  - Source last reviewed: 2026-04-18 (upstream commit `b5f7e73` — "fix: broaden CLI binary lookup to native installer paths for Claude (#731)", 2026-04-18).

See [ROADMAP.md](ROADMAP.md) for the full phased implementation history.

## License

MIT
