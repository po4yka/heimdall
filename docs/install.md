# Install

Detailed installation paths for all surfaces. The [README](../README.md) has the quick-start; this doc covers every variant.

## Prebuilt binary (recommended)

Download the tarball for your platform from the [GitHub Releases](https://github.com/po4yka/heimdall/releases) page, extract it, and move both binaries to `/usr/local/bin`.

### macOS — universal binary (Apple Silicon + Intel)

```bash
VERSION=$(curl -fsSL https://api.github.com/repos/po4yka/heimdall/releases/latest | jq -r '.tag_name')
curl -fsSL "https://github.com/po4yka/heimdall/releases/download/${VERSION}/heimdall-${VERSION}-universal-apple-darwin.tar.gz" \
  | tar xz --strip-components=1 -C /usr/local/bin
```

### All platforms one-liner (requires curl, jq, tar)

```bash
PLATFORM="aarch64-apple-darwin"   # see table below
VERSION=$(curl -fsSL https://api.github.com/repos/po4yka/heimdall/releases/latest | jq -r '.tag_name')
curl -fsSL "https://github.com/po4yka/heimdall/releases/download/${VERSION}/heimdall-${VERSION}-${PLATFORM}.tar.gz" \
  | tar xz --strip-components=1 -C /usr/local/bin
```

| Platform | Archive |
|---|---|
| macOS (universal — Apple Silicon + Intel) | `heimdall-<version>-universal-apple-darwin.tar.gz` |
| macOS (Apple Silicon only) | `heimdall-<version>-aarch64-apple-darwin.tar.gz` |
| macOS (Intel only) | `heimdall-<version>-x86_64-apple-darwin.tar.gz` |
| Linux x86\_64 | `heimdall-<version>-x86_64-unknown-linux-gnu.tar.gz` |
| Linux ARM64 | `heimdall-<version>-aarch64-unknown-linux-gnu.tar.gz` |

Verify against published checksums:

```bash
curl -fsSL "https://github.com/po4yka/heimdall/releases/download/${VERSION}/SHA256SUMS.txt" | sha256sum --check --ignore-missing
```

## Homebrew (macOS)

```bash
brew tap heimdall/tap
brew install heimdall/tap/heimdall
```

The tap repository (`heimdall/homebrew-tap`) must be created and published by the maintainer before this works. The cask formula skeleton lives at `packaging/homebrew/heimdall.rb` in this repo.

## From source

```bash
cargo install --git https://github.com/po4yka/heimdall

# Or build locally
git clone https://github.com/po4yka/heimdall
cd heimdall
cargo build --release
sudo cp target/release/claude-usage-tracker target/release/heimdall-hook /usr/local/bin/
```

## HeimdallBar (native macOS menu-bar app)

See [heimdallbar.md](heimdallbar.md).

## Daemon mode (macOS only)

Run the dashboard as a persistent background service that starts automatically at user login:

```bash
claude-usage-tracker daemon install
claude-usage-tracker daemon status
claude-usage-tracker daemon uninstall
```

The daemon runs `claude-usage-tracker dashboard --host localhost --port 8080 --watch --no-open --background-poll` under a per-user LaunchAgent with `KeepAlive: true`. That means the service starts at login, does not open a browser window, and begins warming remote monitoring/data-fetch caches even before the dashboard is opened manually. Logs are written to `~/Library/Logs/heimdall/`. Linux systemd user services with logon-trigger support are deferred to a future release.

## Scheduler

For just periodic ingest (not a persistent dashboard), use the `scheduler` subcommand:

```bash
claude-usage-tracker scheduler install --interval=hourly
claude-usage-tracker scheduler status
claude-usage-tracker scheduler uninstall
```

It writes a native schedule entry: a launchd plist on macOS or a tagged `# heimdall-scheduler:v1` crontab line on Linux. Runs at minute `:17` to avoid scheduler pile-up.

## Real-time hook

Install the PreToolUse hook so Claude Code reports every tool invocation's cost in real time:

```bash
claude-usage-tracker hook install
claude-usage-tracker hook status
claude-usage-tracker hook uninstall
```

This appends a tagged hook entry to `~/.claude/settings.json` that runs `heimdall-hook` on each tool call. A `settings.json.heimdall-bak` backup is written before every modification. The hook binary is fire-and-forget: ~50ms p99, never blocks Claude Code, and automatically respects bypass mode.

## Statusline (Claude Code status bar)

Wire `claude-usage-tracker statusline` into Claude Code's status-bar config (`~/.claude/settings.json::statusLine`). Heimdall streams a single compact line showing the current model, session/today/block costs, burn-rate tier, and context-window fill. Cache + PID lock keep the warm-path under 5 ms.

## MCP server

```bash
# Claude Code
claude-usage-tracker mcp install --client=claude-code

# Cursor / Claude Desktop
claude-usage-tracker mcp install --client=cursor
claude-usage-tracker mcp install --client=claude-desktop

claude-usage-tracker mcp status --client=claude-code
claude-usage-tracker mcp uninstall --client=claude-code
```

Writes a tagged entry to the client's `.mcp.json` with a `_heimdall_mcp_version` sentinel so uninstall only removes Heimdall's own entry. User customizations are preserved.
