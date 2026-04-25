---
name: release-checklist
description: Run pre-tag verification and walk through the heimdall release pipeline. Mirrors .github/RELEASING.md so a release cannot silently skip a step. Use when the user explicitly asks to cut, prepare, or verify a heimdall release.
disable-model-invocation: true
allowed-tools: [Read, Bash, Edit]
---

# heimdall release checklist

A guided release run. The skill is **user-invoked only** — releases are intentional, network-visible actions that must not happen as a side effect of unrelated work.

The canonical source is [`.github/RELEASING.md`](../../../.github/RELEASING.md). This skill is the executable companion.

## Inputs

Ask the user for:
- The new version (`v0.x.y` or `v0.x.y-rc.N` for pre-release)
- Whether to run the smoke workflow before tagging (default: yes for any release-sensitive change)

## Phase 1 — pre-flight (local)

Run these in order. Stop on the first failure and surface the output.

```bash
# Working tree must be clean
git status --porcelain

# Must be on main and up to date with origin
git rev-parse --abbrev-ref HEAD
git fetch origin main
git status -uno

# Full Rust verification
cargo fmt --check
cargo clippy -- -D warnings
cargo test

# Dashboard verification (only if src/ui or input.css changed since last release)
./node_modules/.bin/tsc --noEmit
npm run build:ui
git diff --exit-code src/ui/app.js src/ui/style.css   # must be clean — committed artifacts must already match source
```

If `git diff` flags `app.js` or `style.css`, the user forgot to rebuild before committing. Stage and commit them as a fix-up before continuing.

## Phase 2 — version bump

1. Edit `Cargo.toml` `[package].version` to the new value.
2. Run `cargo build` to refresh `Cargo.lock`.
3. Confirm the diff is **only** `Cargo.toml` and `Cargo.lock` (and only the `version`/checksum lines).
4. Commit with the conventional format used by recent releases:
   ```bash
   git commit Cargo.toml Cargo.lock -m "chore(release): bump version to <version>"
   git push origin main
   ```

## Phase 3 — CI + smoke

Wait for required checks on the bump commit:

```bash
gh pr checks --watch    # if pushed via PR
# or
gh run list --branch main --limit 1
gh run watch
```

Required green checks (per `.github/RELEASING.md`): `rust-stable`, `rust-msrv`, `ui`, `dependency-review`, `native-smoke`.

If the user opted into the smoke workflow:

```bash
gh workflow run release-smoke.yml
gh run watch --workflow=release-smoke.yml
```

## Phase 4 — tag and publish

```bash
git tag -a <version> -m "Release <version>"
git push origin <version>

gh run list --workflow=release.yml --limit 1
gh run watch --workflow=release.yml
```

Remind the user that the publish job uses the `release` environment and **requires manual approval** in the GitHub UI before assets are published. Expected total wall time: 10–20 minutes.

## Phase 5 — post-release verification

Walk through the post-release checklist from `.github/RELEASING.md`:

- All five platform archives present on the Releases page.
- `heimdallbar-<version>-macos-app.zip` present and contains `HeimdallBar.app`, `bin/heimdallbar`, `Frameworks/HeimdallBarShared.framework`.
- `SHA256SUMS.txt` downloads and verifies locally.
- `claude-usage-tracker --version` and `claude-usage-tracker today` succeed against the downloaded binary.
- `echo '{}' | ./heimdall-hook` returns `{}` and exit 0.
- For macOS app: `spctl -a -vv <app>`; if signing was configured, `xcrun stapler validate <app>`.
- `gh attestation verify <asset> --repo po4yka/heimdall` for at least one asset.
- For pre-release tags (`-rc.`, `-beta.`), the GitHub Release is marked as a pre-release.

## Notes

- The Homebrew cask skeleton at `packaging/homebrew/heimdall.rb` is NOT auto-published. It is copied into the separately-maintained `heimdall/homebrew-tap` repo manually.
- `crates.io` publishing is not automated for this crate.
- Never push a tag without first confirming the bump commit's CI is green.
