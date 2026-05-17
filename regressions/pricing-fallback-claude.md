# Regression: pricing fallback for Claude / GPT-5 families

**Heuristic:** 5-tier model-name → price lookup with a hardcoded-only guarantee for Claude and GPT-5 model families. A LiteLLM cache entry must never substitute for the hardcoded fallback chain on these families.

**Source code:** `src/pricing.rs`

- Hardcoded table: `PRICING_TABLE` (initial constant near line 113).
- LiteLLM passthrough: `get_litellm` (near line 109) backed by `LITELLM_MAP: OnceLock<HashMap<String, ModelPricing>>`.
- Per-config override: `set_overrides` / `get_override` (near lines 37–43).
- Version stamp: `PRICING_VERSION` constant (currently `2026-04-10`).

**Fixture:** `tests/fixtures/regressions/pricing-fallback/` — create on first measured iteration. Should contain:

1. A frozen `litellm_pricing.json` snapshot at the cache layout produced by `cargo run -- pricing refresh`.
2. A `models.txt` listing every Claude / GPT-5 model alias the codebase has seen in the wild (read directly from a real `~/.claude/usage.db` via `SELECT DISTINCT model FROM turns`).
3. An `expected.json` mapping each alias to the expected `(input, output, cache_write, cache_read)` tuple at the current `PRICING_VERSION`.

## Current locked baseline: PRICING_VERSION = 2026-04-10

The 5-tier lookup, in order:

1. **Per-config override** — `PRICING_OVERRIDES`, populated from TOML at startup. Wins against everything below it.
2. **Exact hardcoded match** — direct `PRICING_TABLE` lookup by alias.
3. **Prefix hardcoded match** — e.g. `claude-sonnet-4-5-20250929` resolves to the `claude-sonnet-4-5` entry.
4. **Keyword hardcoded match** — anything containing `opus` resolves to the latest Opus entry, etc.
5. **LiteLLM cache** — `get_litellm` consults the `litellm_pricing.json` snapshot on disk.
6. **Unknown** — confidence stamped `low`; cost estimate may be inaccurate.

**Critical invariant.** Claude and GPT-5 family aliases must never reach tier 5. A LiteLLM cache that disagrees with the hardcoded table must be ignored for these families even if a hardcoded miss occurs. The next iteration of this heuristic should add an explicit assertion in `pricing.rs` and a unit test demonstrating that a LiteLLM entry for a hypothetical `claude-sonnet-4-7` does not substitute for or override the hardcoded chain.

Known gaps not yet addressed:

- **No CI assertion that every Claude / GPT-5 alias resolves before tier 5.** A new Claude model that ships without a `PRICING_TABLE` entry silently falls through to LiteLLM, producing a price that may differ from the published rate by 10–30%. The first measured row in the table below should record any aliases observed in real session data that fall through.
- **Override-vs-LiteLLM precedence is implicit.** Overrides are resolved at tier 1 by insertion order, but the LiteLLM map is a separate `OnceLock` — there is no test that an override wins against a LiteLLM entry for the same model.
- **Volume-discount thresholds are hardcoded.** Sonnet 4.5 has a 200K-token threshold. A future model-pricing change that retroactively sets a different threshold requires both a `PRICING_TABLE` edit and a `PRICING_VERSION` bump — record every bump here so the audit trail stays in one place.
- **Prefix-match ordering is undocumented.** If two prefixes both match an alias (e.g. `claude-sonnet-4` vs `claude-sonnet-4-5`), the longer prefix should win. The current implementation relies on table order; a unit test should pin this once a fixture exists.

## Leak count over time

| Version | PRICING_VERSION | Hardcoded Claude/GPT-5 entries | LiteLLM-fallthrough aliases observed | Notes |
|---|---|---|---|---|
| v0.1.0 (initial) | 2026-04-10 | baseline pending | baseline pending | 5-tier fallback shipped; LiteLLM source optional |

## How to re-run this measurement

```bash
# 1. Refresh the LiteLLM cache so tier 5 reflects current upstream data.
cargo run -- pricing refresh

# 2. Dump every distinct model alias seen in the local DB.
sqlite3 ~/.claude/usage.db \
  "SELECT DISTINCT model FROM turns WHERE model IS NOT NULL;" \
  > /tmp/observed-models.txt

# 3. For each alias, classify which tier resolved the price. The
#    cleanest path is a one-off test in src/pricing.rs that calls
#    the lookup with each alias and records the resolution tier;
#    commit only the assertion if the run produces a finding.
```

To produce a measured row in the table:

1. Run steps 1–3 above.
2. Count Claude/GPT-5 aliases that resolved at tier 5 — those are the regressions. Their alias strings go under "leak excerpts" verbatim.
3. For each leaked alias, add the corresponding hardcoded entry to `PRICING_TABLE`.
4. Bump `PRICING_VERSION` only when a rate changes, not when a new alias is added — aliases at the existing rates inherit the existing version stamp.
5. Append a new row to the table above, with the new `PRICING_VERSION`, the updated hardcoded count, and the leak counts before and after the fix.

## v0.1.0 leak excerpts

None captured yet. Append on the first measured run.
