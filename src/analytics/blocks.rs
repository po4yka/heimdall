use chrono::{DateTime, Duration, Timelike, Utc};
use serde::Serialize;

/// Per-token-type breakdown for a billing block.
#[derive(Debug, Clone, Default, Serialize)]
pub struct TokenBreakdown {
    pub input: i64,
    pub output: i64,
    pub cache_read: i64,
    pub cache_creation: i64,
    pub reasoning_output: i64,
}

impl TokenBreakdown {
    pub fn total(&self) -> i64 {
        self.input + self.output + self.cache_read + self.cache_creation + self.reasoning_output
    }
}

/// Lightweight view of a Turn used for block analytics.
/// Built from a SQL query in `scanner::db`; tests may build directly.
#[derive(Debug, Clone)]
pub struct TurnForBlocks {
    pub timestamp: DateTime<Utc>,
    pub model: String,
    pub tokens: TokenBreakdown,
    pub cost_nanos: i64,
}

/// One billing block (typically a 5-hour window).
#[derive(Debug, Clone, Serialize)]
pub struct BillingBlock {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub tokens: TokenBreakdown,
    pub cost_nanos: i64,
    /// Deduped model names, sorted for stable output.
    pub models: Vec<String>,
    pub is_active: bool,
    pub entry_count: u32,
    /// Timestamp of the first turn actually in this block.
    pub first_timestamp: DateTime<Utc>,
    /// Timestamp of the last turn actually in this block.
    pub last_timestamp: DateTime<Utc>,
    /// True when this is a synthetic gap pseudo-row (no real activity).
    pub is_gap: bool,
    /// "block" for real activity blocks, "gap" for synthetic gap rows.
    pub kind: &'static str,
}

/// Instantaneous burn rate for an active block.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct BurnRate {
    pub tokens_per_min: f64,
    pub cost_per_hour_nanos: i64,
}

/// End-of-block projection for an active block.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Projection {
    pub projected_cost_nanos: i64,
    pub projected_tokens: u64,
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn floor_to_hour(dt: DateTime<Utc>) -> DateTime<Utc> {
    // Infallible arithmetic: subtract sub-hour components.
    let offset_secs = i64::from(dt.minute()) * 60 + i64::from(dt.second());
    let offset_nanos = i64::from(dt.nanosecond());
    dt - Duration::seconds(offset_secs) - Duration::nanoseconds(offset_nanos)
}

// ── public API ────────────────────────────────────────────────────────────────

/// Identify billing blocks from a slice of turns.
///
/// Equivalent to `identify_blocks_with_now(turns, session_hours, Utc::now())`.
pub fn identify_blocks(turns: &[TurnForBlocks], session_hours: f64) -> Vec<BillingBlock> {
    identify_blocks_with_now(turns, session_hours, Utc::now())
}

/// Test seam: same as `identify_blocks` but accepts an explicit `now`.
pub fn identify_blocks_with_now(
    turns: &[TurnForBlocks],
    session_hours: f64,
    now: DateTime<Utc>,
) -> Vec<BillingBlock> {
    if turns.is_empty() {
        return vec![];
    }

    let session_dur = Duration::seconds((session_hours * 3600.0) as i64);

    // Sort defensively.
    let mut sorted: Vec<&TurnForBlocks> = turns.iter().collect();
    sorted.sort_by_key(|t| t.timestamp);

    let mut blocks: Vec<BillingBlock> = Vec::new();

    // State for the current open block.
    let mut block_start = floor_to_hour(sorted[0].timestamp);
    let mut block_end = block_start + session_dur;
    let mut block_tokens = TokenBreakdown::default();
    let mut block_cost: i64 = 0;
    let mut block_models: Vec<String> = Vec::new();
    let mut block_count: u32 = 0;
    let mut last_ts = sorted[0].timestamp;
    let mut first_ts = sorted[0].timestamp;

    for turn in &sorted {
        let ts = turn.timestamp;

        // FIX 4: use strict `>` to match ccusage (gap must exceed session_dur, not equal it).
        let new_block_due = ts >= block_end || (ts - last_ts) > session_dur;

        if new_block_due && block_count > 0 {
            // Close current block.
            // FIX 3: use strict `<` to match ccusage (gap equal to session_dur => inactive).
            let is_active = block_end > now && (now - last_ts) < session_dur;
            blocks.push(BillingBlock {
                start: block_start,
                end: block_end,
                tokens: block_tokens,
                cost_nanos: block_cost,
                models: block_models,
                is_active,
                entry_count: block_count,
                first_timestamp: first_ts,
                last_timestamp: last_ts,
                is_gap: false,
                kind: "block",
            });

            // Open new block anchored on this turn.
            block_start = floor_to_hour(ts);
            block_end = block_start + session_dur;
            block_tokens = TokenBreakdown::default();
            block_cost = 0;
            block_models = Vec::new();
            block_count = 0;
            first_ts = ts;
        }

        // Accumulate.
        block_tokens.input += turn.tokens.input;
        block_tokens.output += turn.tokens.output;
        block_tokens.cache_read += turn.tokens.cache_read;
        block_tokens.cache_creation += turn.tokens.cache_creation;
        block_tokens.reasoning_output += turn.tokens.reasoning_output;
        block_cost += turn.cost_nanos;
        if !block_models.contains(&turn.model) {
            block_models.push(turn.model.clone());
            block_models.sort();
        }
        block_count += 1;
        last_ts = ts;
    }

    // Close the last block.
    if block_count > 0 {
        // FIX 3: use strict `<` to match ccusage (gap equal to session_dur => inactive).
        let is_active = block_end > now && (now - last_ts) < session_dur;
        blocks.push(BillingBlock {
            start: block_start,
            end: block_end,
            tokens: block_tokens,
            cost_nanos: block_cost,
            models: block_models,
            is_active,
            entry_count: block_count,
            first_timestamp: first_ts,
            last_timestamp: last_ts,
            is_gap: false,
            kind: "block",
        });
    }

    blocks
}

/// Same as `identify_blocks_with_now` but optionally inserts synthetic gap
/// pseudo-rows between activity blocks separated by more than `session_hours`.
pub fn identify_blocks_with_gaps(
    turns: &[TurnForBlocks],
    session_hours: f64,
    now: DateTime<Utc>,
    include_gaps: bool,
) -> Vec<BillingBlock> {
    let mut blocks = identify_blocks_with_now(turns, session_hours, now);

    if !include_gaps || blocks.len() < 2 {
        return blocks;
    }

    let mut with_gaps: Vec<BillingBlock> = Vec::with_capacity(blocks.len() * 2);

    for i in 0..blocks.len() - 1 {
        with_gaps.push(blocks[i].clone());
        let gap_duration = blocks[i + 1].start - blocks[i].end;
        if gap_duration > Duration::zero() {
            with_gaps.push(BillingBlock {
                start: blocks[i].end,
                end: blocks[i + 1].start,
                first_timestamp: blocks[i].end,
                last_timestamp: blocks[i + 1].start,
                tokens: TokenBreakdown::default(),
                cost_nanos: 0,
                models: vec![],
                is_active: false,
                entry_count: 0,
                is_gap: true,
                kind: "gap",
            });
        }
    }
    // Push the last block.
    if let Some(last) = blocks.pop() {
        with_gaps.push(last);
    }

    with_gaps
}

/// Calculate burn rate for an active block.
///
/// Matches ccusage: elapsed = last_timestamp - first_timestamp.
/// Returns `None` for single-entry blocks or zero-duration blocks.
pub fn calculate_burn_rate(block: &BillingBlock, _now: DateTime<Utc>) -> Option<BurnRate> {
    if block.entry_count < 2 {
        return None;
    }
    let elapsed = block.last_timestamp - block.first_timestamp;
    let elapsed_secs = elapsed.num_seconds();
    if elapsed_secs <= 0 {
        return None;
    }
    let elapsed_minutes = elapsed_secs as f64 / 60.0;

    let tokens_per_min = block.tokens.total() as f64 / elapsed_minutes;

    // i128 intermediate to avoid overflow when cost_nanos is large.
    let cost_per_hour_nanos = ((block.cost_nanos as i128 * 3600) / elapsed_secs as i128) as i64;

    Some(BurnRate {
        tokens_per_min,
        cost_per_hour_nanos,
    })
}

/// Project end-of-block totals for an active block.
///
/// When `rate` is `None` or `block.is_active == false`, returns current totals unchanged.
pub fn project_block_usage(
    block: &BillingBlock,
    rate: Option<BurnRate>,
    now: DateTime<Utc>,
) -> Projection {
    let Some(rate) = rate else {
        return Projection {
            projected_cost_nanos: block.cost_nanos,
            projected_tokens: block.tokens.total() as u64,
        };
    };

    if !block.is_active {
        return Projection {
            projected_cost_nanos: block.cost_nanos,
            projected_tokens: block.tokens.total() as u64,
        };
    }

    let remaining = (block.end - now).max(Duration::zero());
    let remaining_secs = remaining.num_seconds();
    let remaining_minutes = remaining_secs as f64 / 60.0;

    let additional_tokens = (rate.tokens_per_min * remaining_minutes).round() as u64;
    // i128 intermediate to avoid overflow.
    let additional_cost =
        ((rate.cost_per_hour_nanos as i128 * remaining_secs as i128) / 3600) as i64;

    Projection {
        projected_cost_nanos: block.cost_nanos + additional_cost,
        projected_tokens: block.tokens.total() as u64 + additional_tokens,
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn ts(s: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
    }

    fn turn(
        timestamp: DateTime<Utc>,
        model: &str,
        input: i64,
        output: i64,
        cost_nanos: i64,
    ) -> TurnForBlocks {
        TurnForBlocks {
            timestamp,
            model: model.to_string(),
            tokens: TokenBreakdown {
                input,
                output,
                cache_read: 0,
                cache_creation: 0,
                reasoning_output: 0,
            },
            cost_nanos,
        }
    }

    // ── basic cases ───────────────────────────────────────────────────────────

    #[test]
    fn empty_input_returns_empty() {
        let blocks = identify_blocks(&[], 5.0);
        assert!(blocks.is_empty());
    }

    #[test]
    fn single_turn_produces_one_block() {
        let now = ts("2026-01-01T12:00:00Z");
        let t0 = ts("2026-01-01T10:30:00Z"); // 10:30 → floor → 10:00
        let blocks = identify_blocks_with_now(&[turn(t0, "sonnet", 100, 50, 1_000_000)], 5.0, now);
        assert_eq!(blocks.len(), 1);
        // Block start must be floored to hour.
        assert_eq!(blocks[0].start, ts("2026-01-01T10:00:00Z"));
        assert_eq!(blocks[0].end, ts("2026-01-01T15:00:00Z"));
        assert_eq!(blocks[0].entry_count, 1);
        assert!(blocks[0].is_active); // now (12:00) < end (15:00) and gap = now-t0 = 1.5h < 5h
        // first_timestamp and last_timestamp are both the single turn.
        assert_eq!(blocks[0].first_timestamp, t0);
        assert_eq!(blocks[0].last_timestamp, t0);
    }

    #[test]
    fn two_turns_30_min_apart_one_block() {
        let now = ts("2026-01-01T20:00:00Z"); // after block end, inactive
        let t0 = ts("2026-01-01T09:00:00Z");
        let t1 = ts("2026-01-01T09:30:00Z");
        let blocks = identify_blocks_with_now(
            &[
                turn(t0, "sonnet", 100, 50, 1_000),
                turn(t1, "sonnet", 100, 50, 1_000),
            ],
            5.0,
            now,
        );
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].entry_count, 2);
        assert_eq!(blocks[0].tokens.input, 200);
        assert_eq!(blocks[0].cost_nanos, 2_000);
    }

    #[test]
    fn two_turns_6h_apart_session_5h_two_blocks() {
        let now = ts("2026-01-01T20:00:00Z");
        let t0 = ts("2026-01-01T09:00:00Z");
        let t1 = ts("2026-01-01T15:00:00Z"); // 6h later > 5h → new block
        let blocks = identify_blocks_with_now(
            &[
                turn(t0, "sonnet", 100, 50, 1_000),
                turn(t1, "sonnet", 200, 80, 2_000),
            ],
            5.0,
            now,
        );
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].entry_count, 1);
        assert_eq!(blocks[1].entry_count, 1);
        assert_eq!(blocks[1].tokens.input, 200);
    }

    #[test]
    fn turn_at_block_end_starts_new_block() {
        // Block end = 09:00 + 5h = 14:00; turn at exactly 14:00 must start new block.
        let now = ts("2026-01-01T20:00:00Z");
        let t0 = ts("2026-01-01T09:00:00Z");
        let t1 = ts("2026-01-01T14:00:00Z"); // exactly at block.end
        let blocks = identify_blocks_with_now(
            &[
                turn(t0, "sonnet", 100, 50, 1_000),
                turn(t1, "sonnet", 200, 80, 2_000),
            ],
            5.0,
            now,
        );
        assert_eq!(blocks.len(), 2, "turn at block.end must start new block");
    }

    // ── cross-day UTC boundary ────────────────────────────────────────────────

    #[test]
    fn cross_day_utc_boundary_one_block() {
        // Block starts at 23:00, extends to 04:00 next day.
        let now = ts("2026-01-02T10:00:00Z");
        let t0 = ts("2026-01-01T23:10:00Z");
        let t1 = ts("2026-01-02T01:00:00Z"); // 1h 50m after t0, within block
        let t2 = ts("2026-01-02T03:30:00Z"); // 2h 30m after t1, still within block
        let blocks = identify_blocks_with_now(
            &[
                turn(t0, "sonnet", 100, 50, 1_000),
                turn(t1, "haiku", 200, 80, 2_000),
                turn(t2, "sonnet", 150, 60, 1_500),
            ],
            5.0,
            now,
        );
        assert_eq!(
            blocks.len(),
            1,
            "all turns in cross-day span belong to one block"
        );
        assert_eq!(blocks[0].start, ts("2026-01-01T23:00:00Z"));
        assert_eq!(blocks[0].end, ts("2026-01-02T04:00:00Z"));
        assert_eq!(blocks[0].entry_count, 3);
    }

    // ── model dedup and sort ──────────────────────────────────────────────────

    #[test]
    fn models_are_deduped_and_sorted() {
        let now = ts("2026-01-01T20:00:00Z");
        let t0 = ts("2026-01-01T09:00:00Z");
        let t1 = ts("2026-01-01T09:30:00Z");
        let t2 = ts("2026-01-01T10:00:00Z");
        let blocks = identify_blocks_with_now(
            &[
                turn(t0, "claude-sonnet-4-6", 100, 50, 1_000),
                turn(t1, "claude-haiku-3-5", 100, 50, 1_000),
                turn(t2, "claude-sonnet-4-6", 100, 50, 1_000), // duplicate
            ],
            5.0,
            now,
        );
        assert_eq!(blocks.len(), 1);
        assert_eq!(
            blocks[0].models,
            vec!["claude-haiku-3-5", "claude-sonnet-4-6"]
        );
    }

    // ── burn rate ─────────────────────────────────────────────────────────────

    #[test]
    fn burn_rate_no_panic_on_very_short_elapsed() {
        // Two turns at the same timestamp → elapsed = 0 → returns None.
        let now = ts("2026-01-01T09:00:00Z");
        let block = BillingBlock {
            start: now,
            end: now + Duration::hours(5),
            tokens: TokenBreakdown {
                input: 60,
                output: 0,
                cache_read: 0,
                cache_creation: 0,
                reasoning_output: 0,
            },
            cost_nanos: 600_000,
            models: vec![],
            is_active: true,
            entry_count: 2,
            first_timestamp: now,
            last_timestamp: now,
            is_gap: false,
            kind: "block",
        };
        // elapsed = last - first = 0 → None
        assert!(calculate_burn_rate(&block, now).is_none());
    }

    #[test]
    fn burn_rate_single_entry_returns_none() {
        // FIX 1 / FIX 5: single-entry block must return None from calculate_burn_rate.
        let t0 = ts("2026-01-01T10:30:00Z");
        let now = t0 + Duration::minutes(30);
        let blocks =
            identify_blocks_with_now(&[turn(t0, "sonnet", 1_000, 500, 5_000_000)], 5.0, now);
        assert_eq!(blocks.len(), 1);
        let block = &blocks[0];
        assert_eq!(block.entry_count, 1);

        let rate = calculate_burn_rate(block, now);
        assert!(
            rate.is_none(),
            "single-entry block must return None burn rate"
        );

        // project_block_usage with None rate returns current totals.
        let proj = project_block_usage(block, None, now);
        assert_eq!(proj.projected_tokens, block.tokens.total() as u64);
        assert_eq!(proj.projected_cost_nanos, block.cost_nanos);
    }

    // ── is_active boundary (FIX 3) ────────────────────────────────────────────

    #[test]
    fn is_active_false_when_gap_equals_session_length() {
        // Two turns 5h apart, session_length=5h, now = last_turn + exactly 5h.
        // gap == session_dur → with strict `<`, block is inactive.
        let t0 = ts("2026-01-01T09:00:00Z");
        let t1 = ts("2026-01-01T14:00:00Z"); // 5h after t0
        let now = t1 + Duration::hours(5); // exactly 5h after last turn

        let blocks = identify_blocks_with_now(
            &[
                turn(t0, "sonnet", 100, 50, 1_000),
                turn(t1, "sonnet", 100, 50, 1_000),
            ],
            5.0,
            now,
        );
        // The second turn is at exactly block_end (09:00+5h=14:00) → new block.
        // The second block: now = last_ts + 5h = gap equals session_dur → is_active = false.
        assert_eq!(blocks.len(), 2);
        assert!(
            !blocks[1].is_active,
            "gap == session_dur must yield is_active=false"
        );
    }

    // ── projection ────────────────────────────────────────────────────────────

    #[test]
    fn projection_on_inactive_block_returns_current_totals() {
        let now = ts("2026-01-01T20:00:00Z");
        let start = ts("2026-01-01T09:00:00Z");
        let block = BillingBlock {
            start,
            end: ts("2026-01-01T14:00:00Z"),
            tokens: TokenBreakdown {
                input: 1000,
                output: 500,
                cache_read: 0,
                cache_creation: 0,
                reasoning_output: 0,
            },
            cost_nanos: 5_000_000,
            models: vec![],
            is_active: false,
            entry_count: 2,
            first_timestamp: start,
            last_timestamp: start + Duration::minutes(30),
            is_gap: false,
            kind: "block",
        };
        let rate = BurnRate {
            tokens_per_min: 100.0,
            cost_per_hour_nanos: 1_000_000,
        };
        let proj = project_block_usage(&block, Some(rate), now);
        assert_eq!(proj.projected_cost_nanos, block.cost_nanos);
        assert_eq!(proj.projected_tokens, block.tokens.total() as u64);
    }

    // ── gap insertion ─────────────────────────────────────────────────────────

    #[test]
    fn gap_inserted_between_blocks_10h_apart_session_5h() {
        let now = ts("2026-01-02T20:00:00Z");
        let t0 = ts("2026-01-01T09:00:00Z");
        let t1 = ts("2026-01-01T19:00:00Z"); // 10h later > 5h session → separate blocks
        let blocks = identify_blocks_with_gaps(
            &[
                turn(t0, "sonnet", 100, 50, 1_000),
                turn(t1, "sonnet", 200, 80, 2_000),
            ],
            5.0,
            now,
            true,
        );
        // Expect: block, gap, block = 3 entries.
        assert_eq!(blocks.len(), 3, "expected block + gap + block");
        assert!(!blocks[0].is_gap, "first entry must be a real block");
        assert!(blocks[1].is_gap, "middle entry must be a gap");
        assert_eq!(blocks[1].kind, "gap");
        assert_eq!(blocks[1].cost_nanos, 0);
        assert_eq!(blocks[1].entry_count, 0);
        assert!(!blocks[2].is_gap, "last entry must be a real block");
    }

    #[test]
    fn no_gap_when_blocks_only_4h_apart_session_5h() {
        // Two turns that create 2 blocks; gap between blocks < session_hours → no gap inserted.
        // t0 at 09:00, block ends at 14:00. t1 at 14:00 (exactly at boundary) starts block 2
        // ending at 19:00. Gap between block1.end(14:00) and block2.start(14:00) = 0 → no gap.
        let now = ts("2026-01-01T20:00:00Z");
        let t0 = ts("2026-01-01T09:00:00Z");
        let t1 = ts("2026-01-01T14:00:00Z"); // exactly at block boundary → new block
        let blocks = identify_blocks_with_gaps(
            &[
                turn(t0, "sonnet", 100, 50, 1_000),
                turn(t1, "sonnet", 200, 80, 2_000),
            ],
            5.0,
            now,
            true,
        );
        // Gap between blocks = 0s which is <= session_dur → no gap row.
        assert_eq!(blocks.len(), 2, "no gap row when blocks are adjacent");
        assert!(blocks.iter().all(|b| !b.is_gap));
    }

    #[test]
    fn single_block_with_gaps_enabled_returns_one_entry() {
        let now = ts("2026-01-01T12:00:00Z");
        let t0 = ts("2026-01-01T09:00:00Z");
        let blocks =
            identify_blocks_with_gaps(&[turn(t0, "sonnet", 100, 50, 1_000)], 5.0, now, true);
        assert_eq!(blocks.len(), 1);
        assert!(!blocks[0].is_gap);
    }

    #[test]
    fn empty_turns_with_gaps_enabled_returns_empty() {
        let now = ts("2026-01-01T12:00:00Z");
        let blocks = identify_blocks_with_gaps(&[], 5.0, now, true);
        assert!(blocks.is_empty());
    }

    #[test]
    fn gap_block_json_has_kind_and_is_gap_fields() {
        let now = ts("2026-01-02T20:00:00Z");
        let t0 = ts("2026-01-01T09:00:00Z");
        let t1 = ts("2026-01-01T19:00:00Z"); // 10h apart → gap
        let blocks = identify_blocks_with_gaps(
            &[
                turn(t0, "sonnet", 100, 50, 1_000),
                turn(t1, "sonnet", 200, 80, 2_000),
            ],
            5.0,
            now,
            true,
        );
        assert_eq!(blocks.len(), 3);
        let gap = &blocks[1];
        let v = serde_json::to_value(gap).expect("serialization must not fail");
        assert_eq!(v["is_gap"], serde_json::json!(true));
        assert_eq!(v["kind"], serde_json::json!("gap"));
        assert_eq!(v["cost_nanos"], serde_json::json!(0));
        assert_eq!(v["entry_count"], serde_json::json!(0));
    }

    // ── unsorted input ────────────────────────────────────────────────────────

    #[test]
    fn unsorted_input_is_sorted_before_processing() {
        let now = ts("2026-01-01T20:00:00Z");
        let t0 = ts("2026-01-01T09:00:00Z");
        let t1 = ts("2026-01-01T09:30:00Z");
        // Provide in reverse order.
        let blocks = identify_blocks_with_now(
            &[turn(t1, "sonnet", 200, 0, 0), turn(t0, "sonnet", 100, 0, 0)],
            5.0,
            now,
        );
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].tokens.input, 300);
    }

    // ── back-to-back activity ─────────────────────────────────────────────────

    #[test]
    fn back_to_back_activity_same_block() {
        let now = ts("2026-01-01T20:00:00Z");
        let base = ts("2026-01-01T08:00:00Z");
        let turns: Vec<TurnForBlocks> = (0..10)
            .map(|i| turn(base + Duration::minutes(i * 20), "sonnet", 100, 50, 1_000))
            .collect();
        // 10 turns × 20 min = 200 min = 3h 20m, all within 5h block.
        let blocks = identify_blocks_with_now(&turns, 5.0, now);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].entry_count, 10);
    }

    // ── mid-block idle gap (short) ────────────────────────────────────────────

    #[test]
    fn mid_block_idle_gap_shorter_than_session_stays_in_block() {
        // 3-hour gap inside a 5h block: stays in the same block.
        let now = ts("2026-01-01T20:00:00Z");
        let t0 = ts("2026-01-01T08:00:00Z");
        let t1 = ts("2026-01-01T11:00:00Z"); // 3h gap, but still < 5h and block hasn't ended
        let blocks = identify_blocks_with_now(
            &[
                turn(t0, "sonnet", 100, 50, 1_000),
                turn(t1, "sonnet", 200, 80, 2_000),
            ],
            5.0,
            now,
        );
        assert_eq!(blocks.len(), 1);
    }

    // ── token breakdown accumulation ──────────────────────────────────────────

    #[test]
    fn token_fields_accumulated_correctly() {
        let now = ts("2026-01-01T20:00:00Z");
        let t0 = ts("2026-01-01T09:00:00Z");
        let t1 = ts("2026-01-01T09:30:00Z");
        let turns = vec![
            TurnForBlocks {
                timestamp: t0,
                model: "sonnet".to_string(),
                tokens: TokenBreakdown {
                    input: 100,
                    output: 50,
                    cache_read: 200,
                    cache_creation: 300,
                    reasoning_output: 10,
                },
                cost_nanos: 1_000,
            },
            TurnForBlocks {
                timestamp: t1,
                model: "sonnet".to_string(),
                tokens: TokenBreakdown {
                    input: 50,
                    output: 25,
                    cache_read: 100,
                    cache_creation: 150,
                    reasoning_output: 5,
                },
                cost_nanos: 500,
            },
        ];
        let blocks = identify_blocks_with_now(&turns, 5.0, now);
        assert_eq!(blocks.len(), 1);
        let b = &blocks[0];
        assert_eq!(b.tokens.input, 150);
        assert_eq!(b.tokens.output, 75);
        assert_eq!(b.tokens.cache_read, 300);
        assert_eq!(b.tokens.cache_creation, 450);
        assert_eq!(b.tokens.reasoning_output, 15);
        assert_eq!(b.cost_nanos, 1_500);
    }
}
