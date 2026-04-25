<!--
Thanks for your contribution! Please fill in the sections below.
Delete any section that is not relevant.
-->

## Summary

<!-- 1–3 sentences. What does this PR change, and why? -->

## Type of change

- [ ] `fix` — bug fix
- [ ] `feat` — new feature
- [ ] `refactor` — internal change with no behaviour difference
- [ ] `perf` — performance improvement
- [ ] `docs` — documentation only
- [ ] `test` — tests only
- [ ] `chore` / `ci` — tooling, dependencies, build, CI

## Related issues

<!-- Use "Closes #123", "Refs #456", or "Part of #789". -->

## Changes

<!-- Bullet list of the user-visible / reviewer-relevant changes. -->

-

## Test plan

<!-- How did you verify this works? Reviewers will look here first. -->

- [ ] `cargo build`
- [ ] `cargo test`
- [ ] `cargo clippy -- -D warnings`
- [ ] `cargo fmt --check`
- [ ] `./node_modules/.bin/tsc --noEmit` (if `src/ui/` changed)
- [ ] `npm run build:ui` and committed regenerated `app.js` / `style.css` (if `src/ui/` changed)
- [ ] Manual UI verification in a browser (if dashboard-visible)
- [ ] Migration tested on an existing `~/.claude/usage.db` (if `scanner/db.rs` changed)

## Screenshots / output

<!-- Optional: dashboard screenshots, CLI output, before/after diffs. -->

## Checklist

- [ ] My commits follow [Conventional Commits](https://www.conventionalcommits.org/).
- [ ] My commit subject lines are under 72 characters.
- [ ] I added tests covering my change (or explained in the PR why none are needed).
- [ ] I updated documentation under [docs/](../docs) and [CLAUDE.md](../CLAUDE.md) if behaviour or architecture changed.
- [ ] I ran the full test suite locally.
- [ ] By submitting this PR I agree my contribution is licensed under [BSD 3-Clause](../LICENSE).
