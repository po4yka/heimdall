# Security Policy

Heimdall is a local-first observability tool for AI coding assistants. It reads session logs, OAuth credentials, and SQLite databases from the user's home directory, runs an embedded web dashboard on `localhost`, and ingests Claude Code hook events. Because it sits close to credentials and developer machines, we take security reports seriously.

## Supported Versions

Only the latest tagged release on `main` receives security fixes. Older pre-1.0 releases are not patched — please upgrade before reporting.

| Version    | Supported          |
| ---------- | ------------------ |
| `main`     | :white_check_mark: |
| latest tag | :white_check_mark: |
| older      | :x:                |

## Reporting a Vulnerability

**Please do not open a public GitHub issue for security reports.**

Use one of the following private channels:

1. **GitHub Private Vulnerability Reporting** (preferred) — <https://github.com/po4yka/heimdall/security/advisories/new>
2. **Email** — `nikkipochaev@gmail.com` with subject prefix `[heimdall-security]`

Include, where possible:

- A description of the issue and its impact
- Affected version / commit SHA and platform (macOS, Linux, Windows)
- Reproduction steps or a minimal proof-of-concept
- Any logs, stack traces, or screenshots
- Whether the report is already public or under coordinated disclosure elsewhere

You should receive an acknowledgement within **5 business days**. We aim to provide an initial assessment within **10 business days** and a fix or mitigation plan within **30 days** for confirmed high-severity issues.

## Scope

In scope:

- The `claude-usage-tracker` CLI and embedded dashboard (`src/server/`)
- The `heimdall-hook` ingest binary (`src/hook/`)
- The Heimdall SwiftUI menu-bar app (`macos/Heimdall/`)
- OAuth credential handling (`src/oauth/`)
- Pricing, currency, and webhook outbound calls
- Build and release workflows under `.github/workflows/`

Out of scope:

- Vulnerabilities in upstream dependencies — please report those to the upstream project. We track them via Dependabot and `cargo audit`.
- Issues that require an attacker to already control the user's account, shell, or filesystem (e.g. tampering with `~/.claude/.credentials.json` directly).
- Denial-of-service against the local-only dashboard from the same machine.
- Findings against forks or third-party redistributions of Heimdall.

## Disclosure

We follow coordinated disclosure. After a fix lands we will:

1. Publish a GitHub Security Advisory with a CVE where applicable.
2. Credit the reporter in the advisory and release notes (unless anonymity is requested).
3. Reference the advisory from the changelog of the release that contains the fix.

Thank you for helping keep Heimdall and its users safe.
