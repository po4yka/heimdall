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

When you touch a heuristic — a waste detector under `src/optimizer/`, a pricing tier or fallback path in `src/pricing.rs`, or a regex-driven classifier — also append a row to (or create) the matching file in [`regressions/`](regressions/). That directory holds per-heuristic iteration logs with measured leak counts per release; see [`regressions/README.md`](regressions/README.md) for the file format and rationale.

## Receipt-driven contributions

Two kinds of change need explicit evidence on the PR before merge: new waste detectors and new pricing entries. Both surfaces ship heuristics that affect every user's bill display, so PRs without receipts will be asked for them. The pattern is borrowed from [talk-normal's contribution gate](https://github.com/hexiecs/talk-normal/blob/main/CONTRIBUTING.md): a heuristic landed on personal taste accumulates noise; a heuristic landed on a verbatim receipt accumulates signal.

### New waste detector (under `src/optimizer/`)

Required on the PR or in the linked issue:

1. **Anonymised fixture** under `tests/fixtures/regressions/<detector>/` demonstrating the pattern — JSONL, SQLite snapshot, or a minimal programmatic builder. One fixture, one detector. Anonymise paths (`/Users/alice/...` → `/proj/...`), redact tokens.
2. **Prevalence evidence.** Has the pattern shown up in more than one session and more than one user's data, or only on the contributor's machine? Heimdall does not ship single-machine heuristics; cite another user's session, an issue thread, or community samples.
3. **Severity / monthly-waste formula rationale.** Document the thresholds and the cost formula inline at the top of the detector module. Why this token assumption? Why this severity boundary? "Round number" is not a rationale.
4. **Regression-file row.** Add or extend a file under [`regressions/`](regressions/) with the v0.X.0 baseline measurement against the fixture: false positives, false negatives, and — if measurable — precision and recall. See [`regressions/README.md`](regressions/README.md) for the format.
5. **No new false positives on existing fixtures.** If fixtures already exist under `tests/fixtures/regressions/`, run the full detector suite against each and confirm yours does not introduce findings on previously-clean fixtures. At time of writing no fixtures exist; the first contributor to add one is also responsible for the cross-detector test harness.

### New pricing entry (in `src/pricing.rs::PRICING_TABLE`)

Required on the PR:

1. **Source receipt.** A direct link to the provider's pricing page (Anthropic, OpenAI, etc.) with the retrieval date, or an API-invoice excerpt with sensitive details redacted. Hardcoded prices without a sourced rate are rejected — they drift silently and there is no way to audit the original number months later.
2. **Alias coverage.** List every alias the model is referred to in real session data: the full versioned alias (`claude-sonnet-4-7-20260101`), the stripped form (`claude-sonnet-4-7`), and any provider-specific variants. Confirm each alias resolves at the intended tier of the 5-tier fallback (tier 1–4 for Claude/GPT-5, never tier 5 / LiteLLM). Add a unit test in `src/pricing.rs` covering each new alias.
3. **`PRICING_VERSION` decision documented.** Bump `PRICING_VERSION` *only* when an existing model's rate changes. Adding a new model alias at existing-tier rates does *not* require a version bump — the new alias inherits the existing version stamp. State which case applies in the PR description.
4. **Regression-file row.** Append a row to [`regressions/pricing-fallback-claude.md`](regressions/pricing-fallback-claude.md) recording the new hardcoded count and any aliases observed falling through to tier 5 before this fix.

PRs that change the *behaviour* of the 5-tier fallback chain itself — not just the data in the table — are heuristic changes, not data changes. Those follow the waste-detector gate above, plus a unit test that pins the new precedence.

### Reviewer note

Reviewers may close a PR with "needs receipts" without further engineering review if the gate above is unmet. This is not a judgement on the change; it is the same filter talk-normal applies to rule suggestions. Bring the receipts and reopen.

## Reporting security issues

**Do not open a public GitHub issue.** Please follow [SECURITY.md](SECURITY.md) — private vulnerability reporting via GitHub Security Advisories or email to the maintainer.

## License

By contributing, you agree that your contributions will be licensed under the [BSD 3-Clause License](LICENSE) that covers the project. You retain copyright to your contributions.
