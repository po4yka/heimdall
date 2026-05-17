# Heimdall regressions

Per-heuristic iteration logs. Each file in this directory tracks one waste detector, pricing-fallback path, or classifier across releases — what the rule looked like in each version, what it caught, and what it leaked.

The pattern is borrowed from [talk-normal's `regressions/` convention](https://github.com/hexiecs/talk-normal/tree/main/regressions). A heuristic exercised only by unit tests with synthetic inputs drifts silently against real session data over time. Versioned tables of measured behaviour against a fixed fixture make that drift visible and force every release that touches the heuristic to either record a new measurement or explain why the heuristic was not retested.

## When to add a file

Create a new file in this directory when any of the following ships:

- A new waste detector under `src/optimizer/` (`Detector` trait impl).
- A new pricing tier or fallback path in `src/pricing.rs`.
- A new regex- or `RegexSet`-driven classifier (e.g. `src/scanner/classifier.rs`).

For an existing heuristic that was iterated, append a new row to the existing file's leak-count table — never create a second file.

## File format

Filename: `<area>-<heuristic>.md`, kebab-case. Examples in this directory:

- `optimizer-reread-detector.md`
- `pricing-fallback-claude.md`

Skeleton:

- **Heuristic** — one-sentence description.
- **Source code** — `src/...` pointer to the implementation.
- **Fixture** — path under `tests/fixtures/regressions/<name>/` (create on first use).
- **Current locked baseline: vX.Y.Z (YYYY-MM-DD)** — short prose: measured behaviour today, known gaps.
- **Leak count over time** — one row per release that touched the heuristic. Columns vary by file but always include version, heuristic state, and at least one measured count.
- **vX.Y.Z leak excerpts** — verbatim wrong output / wrong cost / wrong classification. Receipts, not summary prose.
- **Why vX.Y.Z still leaked** — per-iteration root cause notes that motivate the next change.
- **How to re-run** — reproducible command sequence so any contributor can reproduce the row in the table.

## Fixture data

Fixtures live under `tests/fixtures/regressions/<heuristic>/`. Each fixture should exercise exactly one detector or one pricing path — multi-purpose fixtures are harder to label and produce ambiguous regressions when they break.

- Anonymise paths (`/Users/alice/src/...` → `/proj/src/...`).
- Redact OAuth tokens, API keys, and refresh tokens.
- Prefer minimal SQLite snapshots or trimmed JSONL over full session captures. A 30-line JSONL that triggers exactly one finding is more useful than a 30 000-line dump.
- Large real-world fixtures belong behind a Git LFS attachment, not in this tree.

## Why per-version rows matter

A single accuracy number ("F1 = 0.87") goes stale silently. A table with one row per release surfaces drift the moment a contributor opens the file to add a row, and forces the question "did this release retest the heuristic, and if not, why not?" before merge.
