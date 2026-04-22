Run a comprehensive dependency audit combining multiple tools.

## Steps

1. **Vulnerability scan** (`cargo audit`):
   - Run `cargo audit` to check for known RUSTSEC advisories
   - If not installed, note: `cargo install cargo-audit`

2. **Policy enforcement** (`cargo deny check`):
   - If `deny.toml` exists, run `cargo deny check`
   - If not installed, note: `cargo install cargo-deny`
   - If `deny.toml` doesn't exist, report that policy enforcement could not be run; do not create one unless explicitly asked.

3. **Unused dependencies** (`cargo machete`):
   - Run `cargo machete` to find deps declared but not used
   - If not installed, note: `cargo install cargo-machete`

4. **Summary**: Report total vulnerabilities, license violations, unused deps, and actionable fixes.

## Notes
- This is report-first; don't auto-fix Cargo.toml, lockfiles, or policy files without confirmation
- For vulnerabilities with patches available, suggest the version bump
- For license violations, explain which dep and which license
