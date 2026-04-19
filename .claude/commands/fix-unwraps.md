Find and fix `.unwrap()` calls in non-test production code.

## Scope
- Search `src/` for `.unwrap()` calls
- EXCLUDE: `#[cfg(test)]` modules, `cli_tests.rs`, `main.rs` (entry point is OK)
- INCLUDE: `scanner/`, `server/`, `oauth/`, `pricing.rs`, `config.rs`, `webhooks.rs`, `models.rs`

## Classification
For each `.unwrap()` found:

1. **In a function returning `Result`**: Replace with `?` operator
2. **In a function not returning `Result`**: Evaluate:
   - Can the function signature change to return `Result`? If so, change it and propagate
   - Is `.unwrap_or_default()` or `.unwrap_or_else(|| ...)` appropriate?
   - Is the unwrap truly infallible? (e.g., `Regex::new` with a constant pattern, `OnceLock::get()` after guaranteed init) – add a comment: `// SAFETY: <reason>`
3. **`.expect("message")`**: These are acceptable if the message explains why it can't fail. Leave them.

## Process
1. Run `grep -rn '\.unwrap()' src/ --include='*.rs'` to find all occurrences
2. Filter out test modules (lines inside `#[cfg(test)]` or `mod tests`)
3. For each remaining unwrap, apply the classification above
4. After all changes, run `cargo test` to verify nothing breaks
5. Report: total unwraps found, fixed, left (with justification)
