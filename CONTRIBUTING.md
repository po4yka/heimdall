# Contributing to Heimdall

Thanks for your interest in contributing! Heimdall is a local analytics platform for AI coding assistants, built in Rust with a Preact dashboard and a SwiftUI macOS companion app. This guide covers everything you need to land a change.

## TL;DR

1. Fork + branch from `main`.
2. `cargo build && cargo test` (and `npm run build:ui` if you touched `src/ui/`).
3. Open a PR using the [PR template](.github/PULL_REQUEST_TEMPLATE.md).
4. By submitting, you agree your contribution is licensed under [BSD 3-Clause](LICENSE).

## Code of Conduct

Participation in this project is governed by the [Code of Conduct](CODE_OF_CONDUCT.md). Please read it before contributing.

## Getting set up

### Prerequisites

- **Rust** stable (edition 2024). Run `rustup update`.
- **Node.js** 20+ and **npm** — only needed when editing `src/ui/*.tsx` or `src/ui/input.css`. The compiled `app.js`/`style.css` are committed so plain Rust builds don't need Node.
- **Xcode 26+** — only needed for the macOS HeimdallBar app and iOS surfaces.

### Clone & build

```bash
git clone https://github.com/po4yka/heimdall
cd heimdall
cargo build                         # both binaries (default features incl. mcp + jq)
cargo test                          # full suite (880+ tests)
```

### Optional: dashboard UI

```bash
npm install                         # one-time
npm run build:ui                    # esbuild + Tailwind v4
./node_modules/.bin/tsc --noEmit    # type-check
```

After UI changes, **commit the regenerated `src/ui/app.js` and `src/ui/style.css`** alongside your `.tsx` source so plain `cargo build` still works.

## What to work on

- Browse [open issues](https://github.com/po4yka/heimdall/issues), especially anything labelled `good first issue` or `help wanted`.
- For new features, please open a [feature-request issue](.github/ISSUE_TEMPLATE/feature_request.yml) before writing code so we can agree on scope and shape.
- For bug fixes, a [bug-report issue](.github/ISSUE_TEMPLATE/bug_report.yml) plus a failing test is the fastest path to a merge.

## Branching & commits

- Branch from `main`. Use a short, kebab-case branch name: `fix-cursor-dedup`, `feat-amp-credits-card`, etc.
- Use [Conventional Commits](https://www.conventionalcommits.org/): `feat:`, `fix:`, `docs:`, `refactor:`, `chore:`, `test:`, `perf:`, `ci:`.
- Keep the subject line under **72 characters**. Use the body to explain *why*, not *what*.
- Prefer small, focused commits over one mega-commit. A reviewer should be able to understand each commit in isolation.
- Sign off your commits if you wish (DCO is welcome but not required).

## Code style

### Rust

- `cargo fmt --check` and `cargo clippy -- -D warnings` must pass.
- No `.unwrap()` in library code (`src/scanner/`, `src/server/`, `src/pricing/`, …). It is acceptable in tests and `main.rs`.
- Use `thiserror` for error types, `anyhow` in `main.rs` / CLI glue.
- Prefer `&str` over `String` in function signatures where possible.
- All SQL queries live in `src/scanner/db.rs` — nowhere else.
- Tests use the `tempfile` crate; never touch the user's real `~/.claude/` in tests.
- Logs via `tracing`: `debug!` for per-file progress, `info!` for scan summaries, `warn!` for recoverable errors. Never log to stdout in MCP code paths — stdout is reserved for JSON-RPC.

### TypeScript / Preact (dashboard)

- Sentence-case for UI copy; `ALL CAPS` reserved for `<th>` table headers.
- All dynamic text must go through `esc()` in `src/ui/lib/format.ts` (XSS protection).
- Use Preact signals from `src/ui/state/store.ts` — don't introduce a different state library.
- Keep components under ~300 lines; extract new files into `src/ui/components/` rather than growing existing ones.
- Follow the design system documented at `.agents/skills/industrial-design/SKILL.md`.

### Swift (HeimdallBar)

- The macOS app is split into layered modules: `HeimdallDomain` → `HeimdallServices` → `HeimdallPlatformMac` → `HeimdallAppUI` / `HeimdallWidgets` / `HeimdallCLI`. Don't introduce upward dependencies.
- Run the `Validate Domain Boundaries` and `Validate Services Boundaries` build phases — they enforce the layering.

## Tests

```bash
cargo test                          # full suite
cargo test scanner                  # one module
cargo test -- --nocapture           # see stdout
```

If you add a new feature, please add at least one test that exercises it. New scanner providers must include a fixture-based parser test in `src/scanner/providers/<name>_tests.rs`.

For dashboard changes that can't be unit-tested, manually verify in a browser against the dev server: `cargo run -- dashboard --watch`.

## Submitting a pull request

1. Push your branch and open a PR against `main`.
2. Fill in the [PR template](.github/PULL_REQUEST_TEMPLATE.md) — especially the test plan.
3. Make sure CI passes:
   - `cargo build`, `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`
   - `tsc --noEmit` if `src/ui/` was touched
4. Be ready to iterate. Reviewers may ask for changes; please squash review fixups into the original commit when reasonable, or keep them as fixup commits and let the reviewer squash on merge.

## Adding a scanner provider, waste detector, or analytics view

See [AGENTS.md](AGENTS.md) for the full "Adding X" playbook — including new models, new JSONL fields, new API endpoints, and database schema migrations.

## Reporting security issues

**Do not open a public GitHub issue.** Please follow [SECURITY.md](SECURITY.md) — private vulnerability reporting via GitHub Security Advisories or email to the maintainer.

## License

By contributing, you agree that your contributions will be licensed under the [BSD 3-Clause License](LICENSE) that covers the project. You retain copyright to your contributions.
