# Data sources

Heimdall auto-discovers sessions from nine local tools. Each provider is implemented in `src/scanner/providers/` and registered in `providers::all()`.

| Tool | Path |
|---|---|
| Claude Code CLI | `~/.claude/projects/` |
| Claude Code subagents | `~/.claude/projects/<slug>/subagents/` |
| Claude Desktop Cowork | `~/.claude/local-agent-mode-sessions/<slug>/audit.jsonl` (label resolution) |
| Xcode CodingAssistant (macOS) | `~/Library/Developer/Xcode/CodingAssistant/ClaudeAgentConfig/projects/` |
| Codex archived sessions | `~/.codex/archived_sessions/` |
| Codex live sessions (JSONL) | `~/.codex/sessions/` |
| Cursor | `~/Library/Application Support/Cursor/User/workspaceStorage/*/state.vscdb` (macOS), `~/.config/Cursor/...` (Linux) |
| OpenCode | `~/Library/Application Support/opencode/*.db` (macOS) and platform equivalents |
| Pi | `~/.pi/sessions/*.jsonl` |
| GitHub Copilot | `~/Library/Application Support/Code/User/globalStorage/github.copilot-chat/` (VS Code), JetBrains paths (best-effort probe) |
| Amp (Sourcegraph) | `~/.local/share/amp/threads/*.json` (override with `$AMP_DATA_DIR`) |
| Claude usage-limits snapshots | `~/.claude/**/*-usage-limits` |
| Custom | `--projects-dir <PATH>` or config file |

## Adding a new provider

See [AGENTS.md § Adding a scanner provider](../AGENTS.md). Each provider implements the `Provider` trait in `src/scanner/provider.rs` and is appended to `providers::all()`.
