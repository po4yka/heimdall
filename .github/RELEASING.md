# Releasing Heimdall

This document describes how to cut a new release and verify the resulting artifacts.

## Prerequisites

- Write access to the `po4yka/heimdall` GitHub repository.
- A local clone on the `main` branch with no uncommitted changes.
- `gh` CLI authenticated (`gh auth status`).
- For signed macOS app releases, configure these GitHub Actions secrets:
  - `HEIMDALL_CODESIGN_IDENTITY`
  - `HEIMDALL_APPLE_CERTIFICATE_BASE64`
  - `HEIMDALL_APPLE_CERTIFICATE_PASSWORD`
  - `HEIMDALL_NOTARY_APPLE_ID`
  - `HEIMDALL_NOTARY_TEAM_ID`
  - `HEIMDALL_NOTARY_APP_PASSWORD`

If the Apple signing secrets are missing, the workflow still builds the macOS
artifact with ad-hoc signatures for structure validation, but notarization is
skipped. Production tags should always have the secrets configured.

---

## Step-by-step: cutting a release

### 1. Bump the version in `Cargo.toml`

Edit the `version` field in the `[package]` table:

```toml
[package]
name = "claude-usage-tracker"
version = "0.x.y"   # <- new version
```

Run `cargo build` to regenerate `Cargo.lock`:

```bash
cargo build
```

### 2. Update `CHANGELOG.md`

Add a section for the new version at the top of the file. Follow the format:

```markdown
## [0.x.y] - YYYY-MM-DD

### Added
- ...

### Fixed
- ...

### Changed
- ...
```

The release workflow extracts this section automatically and uses it as the
GitHub Release body.

### 3. Commit and push

```bash
git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "chore(release): bump version to v0.x.y"
git push origin main
```

### 4. Tag the release

```bash
git tag -a v0.x.y -m "Release v0.x.y"
git push --tags
```

Pushing the tag triggers `.github/workflows/release.yml`.

### 5. Monitor the workflow

```bash
gh run list --workflow=release.yml
gh run watch   # live tail the most recent run
```

The matrix builds five Rust targets in parallel. The release workflow also
builds a separate macOS app artifact for `HeimdallBar.app` plus the bundled
`heimdallbar` CLI, signs it, validates the embedded widget/helper/framework
layout, and notarizes it when Apple credentials are present. The
`consolidate-checksums` job runs after all matrix jobs complete and attaches
the combined `SHA256SUMS.txt`.

Expected total wall-clock time: **10--20 minutes** (Linux arm64 via `cross`
takes the longest).

---

## Post-release checklist

- [ ] Confirm all five platform archives appear on the GitHub Releases page.
- [ ] Confirm the `heimdallbar-<version>-macos-app.zip` artifact appears on
      the GitHub Releases page and contains:
  - `HeimdallBar.app`
  - `bin/heimdallbar`
  - `Frameworks/HeimdallBarShared.framework`
- [ ] Download `SHA256SUMS.txt` and verify at least one archive locally:
  ```bash
  sha256sum --check --ignore-missing SHA256SUMS.txt
  ```
- [ ] Smoke-test the downloaded binary on at least one platform:
  ```bash
  ./claude-usage-tracker --version
  ./claude-usage-tracker today
  ```
- [ ] Confirm `heimdall-hook` binary is present in the archive and executes
  (it reads stdin and exits 0 on empty input):
  ```bash
  echo '{}' | ./heimdall-hook
  ```
- [ ] Smoke-test the native app artifact on macOS:
  ```bash
  unzip heimdallbar-v0.x.y-macos-app.zip
  open heimdallbar-v0.x.y-macos-app/HeimdallBar.app
  DYLD_FRAMEWORK_PATH=heimdallbar-v0.x.y-macos-app/Frameworks \
    ./heimdallbar-v0.x.y-macos-app/bin/heimdallbar config dump
  spctl -a -vv heimdallbar-v0.x.y-macos-app/HeimdallBar.app
  ```
- [ ] If signing secrets were configured, confirm notarization completed and
      stapling succeeded:
  ```bash
  xcrun stapler validate heimdallbar-v0.x.y-macos-app/HeimdallBar.app
  ```
- [ ] Update the README install one-liner if the tarball naming changed.
- [ ] If a pre-release tag (`v0.x.y-rc.1`, `v0.x.y-beta.1`) was used, verify
  the release is marked as a pre-release on GitHub.

---

## Targets matrix

| Target | Runner | Cross-compilation |
|--------|--------|-------------------|
| `aarch64-apple-darwin` | `macos-latest` | Native (macOS ARM) |
| `x86_64-apple-darwin` | `macos-latest` | Native (macOS Intel) |
| `x86_64-unknown-linux-gnu` | `ubuntu-latest` | Native |
| `aarch64-unknown-linux-gnu` | `ubuntu-latest` | `cross` wrapper |
| `x86_64-pc-windows-msvc` | `windows-latest` | Native |

### Known gap: Linux arm64 (`cross`)

The `aarch64-unknown-linux-gnu` target uses [`cross`](https://github.com/cross-rs/cross),
which runs a Docker container on the `ubuntu-latest` runner. If `cross` setup
fails or the Docker layer takes too long, this target may be dropped to keep
the other four working. If that happens:

1. Remove the `aarch64-unknown-linux-gnu` entry from the matrix in
   `.github/workflows/release.yml`.
2. Document the gap here.
3. Revisit in Phase 22 (macOS distribution hardening) which will also address
   Linux ARM packaging.

---

## macOS distribution notes

- The workflow stages the macOS release with
  [script/package_heimdallbar_distribution.sh](/Users/po4yka/GitRep/heimdall/script/package_heimdallbar_distribution.sh:1),
  signs it with
  [script/sign_heimdallbar_distribution.sh](/Users/po4yka/GitRep/heimdall/script/sign_heimdallbar_distribution.sh:1),
  and validates it with
  [script/validate_heimdallbar_distribution.sh](/Users/po4yka/GitRep/heimdall/script/validate_heimdallbar_distribution.sh:1).
- Validation includes `codesign --verify`, helper/widget presence checks, and a
  real `heimdallbar config dump` launch against the packaged framework layout.
- The bundled `heimdallbar` CLI is shipped outside the `.app` bundle, so local
  smoke tests should export `DYLD_FRAMEWORK_PATH=<artifact>/Frameworks` before
  invoking it directly.

## Notes

- **Homebrew distribution** is planned for Phase 22. A cask formula at
  `heimdall/homebrew-tap` will enable `brew install heimdall/tap/heimdall`.
  Until then, users should use the prebuilt binaries or `cargo install`.
- **crates.io publishing** is not automated in this phase (Phase 11). Binary
  distribution only. Add a `cargo publish` step in a future phase if needed.
- Pre-release tags (containing a `-` in the version, e.g. `v0.2.0-rc.1`) are
  automatically marked as GitHub pre-releases by the workflow.
