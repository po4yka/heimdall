# Claude Usage Tracker -- Technical Specification

## Overview

Local analytics tool that parses Claude Code JSONL transcripts and presents usage data via CLI and web dashboard. Single Rust binary, no runtime dependencies.

## JSONL Transcript Format

**Location:** `~/.claude/projects/<project-slug>/<session-uuid>.jsonl`
Subagent sessions: `~/.claude/projects/<project-slug>/subagents/<session-uuid>.jsonl`

### Record Types

Each line is a JSON object with a `type` field:

| Type | Description | Usage-relevant |
|------|-------------|----------------|
| `assistant` | Model response with token usage | Yes (primary) |
| `user` | User message | Yes (session metadata) |
| `system` | System messages | No |
| `progress` | Streaming progress chunks | No |
| `file-history-snapshot` | File backup metadata | No |
| `custom-title` | Session title | Metadata only |
| `agent-name` | Agent identifier | Metadata only |
| `queue-operation` | Queue state | No |
| `last-prompt` | Last user prompt | No |

### Key Fields (assistant records)

```jsonc
{
  "type": "assistant",
  "sessionId": "uuid",
  "timestamp": "2026-04-08T10:00:00Z",
  "cwd": "/home/user/project",
  "gitBranch": "main",
  "version": "1.0.0",
  "entrypoint": "cli",        // "cli", "vscode", "jetbrains"
  "slug": "project-slug",
  "message": {
    "id": "msg_xxx",           // used for streaming dedup
    "model": "claude-sonnet-4-6",
    "usage": {
      "input_tokens": 1500,
      "output_tokens": 800,
      "cache_read_input_tokens": 5000,
      "cache_creation_input_tokens": 200,
      // Extended fields (may not always be present):
      "cache_creation": {
        "ephemeral_5m_input_tokens": 100,
        "ephemeral_1h_input_tokens": 50
      },
      "service_tier": "standard",
      "inference_geo": "us"
    },
    "content": [
      { "type": "text", "text": "..." },
      { "type": "tool_use", "name": "Read", "id": "toolu_xxx" }
    ]
  }
}
```

### Streaming Deduplication

Claude Code logs multiple JSONL records per API response (streaming chunks), all sharing the same `message.id`. **Last record per message.id wins** -- it contains the final usage tallies.

Records without a `message.id` are kept as-is (legacy format).

## Database Schema (SQLite)

**Location:** `~/.claude/usage.db`

### Tables

```sql
CREATE TABLE sessions (
    session_id          TEXT PRIMARY KEY,
    project_name        TEXT,
    project_slug        TEXT,
    first_timestamp     TEXT,
    last_timestamp      TEXT,
    git_branch          TEXT,
    model               TEXT,
    entrypoint          TEXT,         -- cli, vscode, jetbrains
    total_input_tokens  INTEGER DEFAULT 0,
    total_output_tokens INTEGER DEFAULT 0,
    total_cache_read    INTEGER DEFAULT 0,
    total_cache_creation INTEGER DEFAULT 0,
    turn_count          INTEGER DEFAULT 0
);

CREATE TABLE turns (
    id                      INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id              TEXT NOT NULL,
    timestamp               TEXT,
    model                   TEXT,
    input_tokens            INTEGER DEFAULT 0,
    output_tokens           INTEGER DEFAULT 0,
    cache_read_tokens       INTEGER DEFAULT 0,
    cache_creation_tokens   INTEGER DEFAULT 0,
    tool_name               TEXT,
    cwd                     TEXT,
    message_id              TEXT,
    service_tier            TEXT,
    inference_geo           TEXT
);

CREATE TABLE processed_files (
    path    TEXT PRIMARY KEY,
    mtime   REAL,
    lines   INTEGER
);

-- Indexes
CREATE INDEX idx_turns_session ON turns(session_id);
CREATE INDEX idx_turns_timestamp ON turns(timestamp);
CREATE INDEX idx_sessions_first ON sessions(first_timestamp);
CREATE UNIQUE INDEX idx_turns_message_id
    ON turns(message_id) WHERE message_id IS NOT NULL AND message_id != '';
```

### Incremental Scanning Strategy

1. For each `*.jsonl` file, compare current `mtime` against `processed_files` table
2. If unchanged (within 10ms tolerance), skip
3. If new file: full parse, record line count
4. If updated file: read from disk, skip first N lines (already processed), parse remainder
5. After all inserts: recompute session totals from turns table (handles dedup corrections)

## Pricing

Hardcoded pricing table with fallback matching:

1. Exact model name match (e.g., `claude-sonnet-4-6`)
2. Prefix match (e.g., `claude-sonnet-4-6-20260401`)
3. Substring fallback (`opus`/`sonnet`/`haiku` keywords, case-insensitive)
4. Unknown models: cost = 0, display "n/a"

### Current Rates (Anthropic API, verify at anthropic.com/pricing)

| Model Family | Input $/MTok | Output $/MTok | Cache Write $/MTok | Cache Read $/MTok |
|-------------|-------------|--------------|-------------------|------------------|
| Opus 4.x | 15.00 | 75.00 | 18.75 | 1.50 |
| Sonnet 4.x | 3.00 | 15.00 | 3.75 | 0.30 |
| Haiku 4.x | 1.00 | 5.00 | 1.25 | 0.10 |

**Cost formula:**
```
cost = input * input_rate / 1M
     + output * output_rate / 1M
     + cache_read * cache_read_rate / 1M
     + cache_creation * cache_write_rate / 1M
```

## CLI Interface

```
claude-usage-tracker <COMMAND>

Commands:
  scan        Scan JSONL files and update the database
  today       Show today's usage summary
  stats       Show all-time statistics
  dashboard   Scan + start web dashboard

Options:
  --projects-dir <PATH>   Custom transcript directory
  --db-path <PATH>        Custom database path (default: ~/.claude/usage.db)
  --host <HOST>           Dashboard bind address (default: localhost)
  --port <PORT>           Dashboard port (default: 8080)
  -v, --verbose           Verbose output
  -h, --help              Print help
  -V, --version           Print version
```

## HTTP API

**Server:** axum on tokio, single-threaded runtime sufficient for local use.

### Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/` | Serve dashboard HTML |
| GET | `/api/data` | All dashboard data as JSON |
| POST | `/api/rescan` | Delete DB + full rescan, return stats |
| GET | `/api/health` | Health check |

### GET /api/data Response

```jsonc
{
  "all_models": ["claude-sonnet-4-6", "claude-haiku-4-5"],
  "daily_by_model": [
    {
      "day": "2026-04-08",
      "model": "claude-sonnet-4-6",
      "input": 15000,
      "output": 8000,
      "cache_read": 50000,
      "cache_creation": 2000,
      "turns": 25
    }
  ],
  "sessions_all": [
    {
      "session_id": "abc12345",
      "project": "user/myproject",
      "last": "2026-04-08 10:30",
      "last_date": "2026-04-08",
      "duration_min": 45.2,
      "model": "claude-sonnet-4-6",
      "turns": 15,
      "input": 12000,
      "output": 6000,
      "cache_read": 40000,
      "cache_creation": 1500
    }
  ],
  "generated_at": "2026-04-08 14:30:00"
}
```

## Dashboard UI

### Requirements

- Dark theme (GitHub-style dark palette)
- Responsive layout (desktop-first, mobile-friendly)
- All filtering client-side (model checkboxes, date range: 7d/30d/90d/all)
- URL persistence for filters via query params
- Auto-refresh every 30 seconds
- No build step: HTML/CSS/JS embedded in the Rust binary via `include_str!`

### Components

1. **Summary cards** -- sessions, turns, tokens (in/out/cache), estimated cost
2. **Daily token chart** -- stacked bar (input/output/cache_read/cache_creation)
3. **Model distribution** -- doughnut chart
4. **Top projects** -- horizontal bar chart
5. **Cost by model table** -- sortable columns
6. **Sessions table** -- sortable, with CSV export
7. **Cost by project table** -- sortable, with CSV export
8. **Rescan button** -- triggers full DB rebuild

### Charting

Use ApexCharts 4.x loaded from CDN (with graceful degradation note for offline).
Charts are themed through CSS custom properties in `src/ui/lib/charts.ts` so they
follow the industrial-design palette and respond to the `data-theme` toggle.

## Rust Crate Dependencies

| Crate | Purpose |
|-------|---------|
| `clap` | CLI argument parsing |
| `axum` | HTTP server |
| `tokio` | Async runtime |
| `rusqlite` (bundled) | SQLite with bundled libsqlite3 |
| `serde` / `serde_json` | JSON parsing and serialization |
| `chrono` | Timestamp handling |
| `glob` / `walkdir` | File discovery |
| `tracing` | Structured logging |
| `open` | Open browser |

## Non-Goals

- User authentication (localhost only)
- Historical pricing changes (single hardcoded table)
- Real-time streaming (polling is sufficient)
- Cowork session tracking (server-side only, no local JSONL)
- Desktop app / Tauri wrapper (web dashboard is sufficient)

## Testing Strategy

- Unit tests for parser, dedup logic, cost calculation, project name inference
- Integration tests for scan pipeline (temp dirs + temp DBs)
- HTTP endpoint tests using axum's built-in test utilities
- Pricing parity tests (single source of truth in Rust, no duplication)
