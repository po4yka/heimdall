Audit the release binary for size and bloat.

## Steps

1. **Build release binary**:
   ```
   cargo build --release
   ```

2. **Report binary size**:
   ```
   ls -lh target/release/claude-usage-tracker
   ```

3. **Bloat analysis** (if cargo-bloat installed):
   - By crate: `cargo bloat --release --crates`
   - Top 20 functions: `cargo bloat --release -n 20`
   - If not installed, note: `cargo install cargo-bloat`

4. **Dependency tree size**:
   ```
   cargo tree --depth 1 | wc -l
   ```
   Report total direct + transitive dependency count.

5. **Analysis**:
   - Flag any single crate contributing >20% of binary size
   - Check for heavy optional features that could be disabled
   - Compare `reqwest` features (rustls-tls vs native-tls, json feature)
   - Check if `rusqlite` bundled SQLite adds significant bloat
   - Note: release profile has `lto = true` and `strip = true`

6. **Suggestions**: List concrete actions to reduce binary size if it exceeds 10 MB:
   - Disable unused reqwest features
   - Consider `miniz_oxide` vs `flate2` if compression is pulled in
   - Evaluate if `chrono` can be replaced with lighter alternatives
   - Check codegen-units = 1 for better LTO

## Notes
- This is report-first; do not change release profile settings or dependency features unless explicitly asked.
- Prefer repo-specific observations over generic binary-size advice.
