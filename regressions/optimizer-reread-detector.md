# Regression: reread waste detector

**Heuristic:** flag any file read three or more times within a single session as potential waste; estimate monthly cost via a 500-token-per-read assumption at a rough Sonnet input rate.

**Source code:** `src/optimizer/reread.rs`

- Severity thresholds: `severity_for_reads` (≥3 → Low, ≥6 → Medium, ≥16 → High).
- Cost formula: `waste_nanos` (`rereads * 500 * 20_000_000 / 1_000`, where `rereads = reads - 1`).
- Path filter: `WHERE kind = 'file' AND value LIKE '/%'`.

**Fixture:** `tests/fixtures/regressions/reread-detector/` — create on first measured iteration. Should contain a SQLite snapshot with one session that exhibits 3, 6, and 16 reads of three distinct files (one per severity bucket) plus negative cases (legacy `value = "Read"` rows and 2-read files that must not flag).

## Current locked baseline: v0.1.0 (2026-04-10)

The detector landed in Phase 6 with the formula and thresholds above. The unit tests in `src/optimizer/reread.rs` cover the SQL filter, the severity buckets, and the waste arithmetic, but no measured baseline against a real-session fixture has been captured yet. The first row in the leak-count table below is therefore marked baseline-pending; the next contributor to touch this detector should produce the v0.1.0 measurement at the same time as their change.

Known gaps not yet addressed:

- **Token assumption is constant.** Every flagged read is assumed to be 500 tokens. A 50-line file and a 5 000-line file produce the same waste estimate. Migrating to a real per-read byte count (already available on tool events for some providers) would tighten the high-severity tail.
- **First read is "free" by convention** (`rereads = reads - 1`). If the first read happened in a prior session and was evicted from the context window, the user pays for it again on the next read, but the detector counts that paid read as the free baseline. Cross-session deduplication is out of scope today; flag it before changing the formula.
- **No tool-source distinction.** A path matched by `Read`, `Grep`, and `Glob` all increment the same counter. In code-search-heavy sessions this produces false positives. Filtering on the originating tool is a candidate v0.2 change.
- **Windows paths are silently dropped.** The legacy-row guard `value LIKE '/%'` excludes Windows paths (`C:\...`). Heimdall ships for Windows in the release matrix, so on Windows this detector currently returns zero findings. The fix is either a stricter check on the legacy-row shape (`value NOT IN ('Read', 'Edit', 'Write')`) or splitting the filter by platform.

## Leak count over time

| Version | Heuristic state | False positives | False negatives | Notes |
|---|---|---|---|---|
| v0.1.0 (initial) | ≥3 → Low, ≥6 → Medium, ≥16 → High; 500-token assumption; `value LIKE '/%'` filter | baseline pending | baseline pending | Phase 6 implementation landed |

## How to re-run this measurement

```bash
# Run the detector against the live DB and isolate reread findings.
cargo run -- optimize --format=json \
  | jq '.findings[] | select(.detector == "reread_files")'
```

To produce a measured row in the table:

1. Capture a real session DB snapshot, anonymised under `tests/fixtures/regressions/reread-detector/fixture.db`.
2. Hand-label the expected findings: which `(session_id, file_path)` pairs are real waste and which are false positives (e.g. an editor that reads a file once per keystroke, or a `Grep`-tool re-match on the same path).
3. Run the detector against the fixture (point `db_path` at the fixture via `--db <path>` once that flag exists, or via a temporary test in `src/optimizer/reread.rs`) and diff against the labels.
4. Record the false-positive / false-negative counts in the table above alongside the heuristic state at that version. Append the wrong titles verbatim under "leak excerpts" — those are the receipts that motivate the next iteration.

## v0.1.0 leak excerpts

None captured yet. Append on the first measured run.
