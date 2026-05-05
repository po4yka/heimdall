# Configuration

Create `~/.claude/usage-tracker.toml` or `~/.claude/usage-tracker.json` (all fields optional). The dual-path resolver also checks `$HEIMDALL_CONFIG` and `~/.config/heimdall/config.{json,toml}`. When both formats exist at the same location, JSON takes precedence.

## JSON format with IDE autocomplete

Add a `$schema` key for VS Code / IntelliJ autocomplete:

```json
{
  "$schema": "https://raw.githubusercontent.com/po4yka/heimdall/main/schemas/heimdall.config.schema.json",
  "blocks": { "token_limit": 1000000 },
  "statusline": { "context_low_threshold": 0.5, "context_medium_threshold": 0.8 }
}
```

## Per-command overrides

`commands.<name>` keys win over flat defaults; CLI flags still win over everything.

```json
{
  "blocks": { "token_limit": 500000 },
  "commands": { "blocks": { "token_limit": 1000000 } }
}
```

## Per-provider session block duration

Claude's billing window is 5 hours but other providers differ:

```toml
[blocks]
token_limit = 1000000
session_length_hours = 5.0

[blocks.session_length_by_provider]
claude = 5.0
codex = 1.0
amp = 24.0
```

Precedence: `--session-length` flag > `--provider` lookup > flat default > 5.0.

## Statusline thresholds

```toml
[statusline]
context_low_threshold = 0.5     # below → no marker
context_medium_threshold = 0.8  # 0.5–0.8 → [WARN]; > 0.8 → [CRIT]
burn_rate_normal_max = 4000     # tokens/min at or below → Normal
burn_rate_moderate_max = 10000  # tokens/min at or below → Moderate; above → High
```

## Project aliases

Map mangled Claude Code project slugs to human-readable names:

```toml
[project_aliases]
"-Users-po4yka-GitRep-heimdall" = "Heimdall"
"-Users-po4yka-GitRep-ccusage" = "ccusage"
```

CLI override (repeatable, wins over config):

```bash
heimdall today --project-alias="-Users-po4yka-GitRep-heimdall=Heimdall"
```

## Locale

Dates in CLI tables can be localized. Set `[display] locale = "ja-JP"` in config or pass `--locale=ja-JP` on the command. Default resolves from `$LANG` and falls back to `en-US`. SQL / JSON / CSV date columns stay ISO for scriptability.

## Full TOML reference

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
locale = "ja-JP"   # BCP-47 locale for date formatting; default resolved from $LANG
compact = false    # narrow CLI tables by default

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
agent_status = true
cap_changes = true

# Upstream coding-agent status monitoring
[agent_status]
enabled = true
refresh_interval = 60
claude_enabled = true
openai_enabled = true
alert_min_severity = "major"

# Community signal via StatusGator — OFF by default.
[status_aggregator]
enabled = false
provider = "statusgator"
key_env_var = "STATUSGATOR_API_KEY"
refresh_interval = 300
claude_services = ["claude-ai", "claude"]
openai_services = ["openai", "chatgpt"]
spike_webhook = true
```
