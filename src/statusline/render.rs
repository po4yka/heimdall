//! Compose the output status line from computed stats.
//!
//! # Char budget
//!
//! Claude Code's status bar is length-capped by the medium (terminal width
//! and Claude Code's status-bar UI). The full output is bounded by
//! [`STATUSLINE_MAX_CHARS`] (180), which is enforced by
//! `tests::render_fits_budget_worst_case` against a fixture exercising every
//! optional segment simultaneously: long model name, dual-cost session
//! segment with drift warn, active block with `Both`-style burn-rate tier
//! (emoji + bracket), and a CRIT context window.
//!
//! Defense-in-depth: the model name is also runtime-truncated to 24 chars
//! (Unicode scalar values, not bytes — see `model_truncation_unicode_no_panic`)
//! so a runaway model alias cannot single-handedly bust the budget. The 180
//! ceiling is the second line of defense.
//!
//! Pattern borrowed from talk-normal's `prompt-chatgpt.md` budget convention:
//! when output overflows, either compress the content or bump the budget
//! here with rationale. Never silently truncate the rendered line at
//! runtime — segments downstream of a truncation point lose information
//! the user paid for.

use crate::analytics::burn_rate::{self, BurnRateConfig, BurnRateTier};
use crate::analytics::quota::Severity;
use crate::pricing::{fmt_cost, fmt_tokens};
use crate::statusline::VisualBurnRate;
use crate::statusline::compute::{ComputedStats, CostSource};

/// Maximum chars in the rendered statusline (Unicode scalar values).
///
/// Sized to accommodate the worst realistic combination of segments:
/// model (24-char truncation cap) + dual-cost session with drift warn +
/// active block with emoji+bracket tier + CRIT context window. Asserted in
/// `tests::render_fits_budget_worst_case`.
pub const STATUSLINE_MAX_CHARS: usize = 180;

/// Options controlling how the status line is rendered.
pub struct RenderOpts {
    pub context_low_threshold: f64,
    pub context_medium_threshold: f64,
    pub burn_rate: BurnRateConfig,
    pub visual_burn_rate: VisualBurnRate,
    /// Phase 8: determines whether to render dual cost sources.
    pub cost_source: CostSource,
}

impl Default for RenderOpts {
    fn default() -> Self {
        Self {
            context_low_threshold: 0.5,
            context_medium_threshold: 0.8,
            burn_rate: BurnRateConfig::default(),
            visual_burn_rate: VisualBurnRate::Bracket,
            cost_source: CostSource::Auto,
        }
    }
}

/// Render a single-line status string using default options.
///
/// Layout:
///   MODEL | $SESSION / $TODAY / $BLOCK (Xh Ym left) | $COST/hr [TIER] | N tokens (XX%)
///
/// When no active block:
///   MODEL | $SESSION / $TODAY | N tokens (XX%)
///
/// When no context window: omit the last segment.
/// Severity markers appended when context fill crosses thresholds:
///   < low  → (no marker)
///   >= low and < medium → [WARN]
///   >= medium → [CRIT]
pub fn render_status_line(stats: &ComputedStats) -> String {
    render_status_line_with_opts(stats, &RenderOpts::default())
}

/// Like `render_status_line` but with configurable severity thresholds.
///
/// Kept for backwards compatibility with callers that only need context thresholds.
/// Pins `visual_burn_rate` to `Off` to preserve pre-Phase-7 output — callers that
/// want the tier badge should use [`render_status_line_with_opts`] directly.
pub fn render_status_line_with_thresholds(
    stats: &ComputedStats,
    context_low_threshold: f64,
    context_medium_threshold: f64,
) -> String {
    render_status_line_with_opts(
        stats,
        &RenderOpts {
            context_low_threshold,
            context_medium_threshold,
            burn_rate: BurnRateConfig::default(),
            visual_burn_rate: VisualBurnRate::Off,
            cost_source: CostSource::Auto,
        },
    )
}

/// Full render with all options.
pub fn render_status_line_with_opts(stats: &ComputedStats, opts: &RenderOpts) -> String {
    // Truncate model name to 24 Unicode scalar values (chars), not bytes,
    // to avoid a panic on multibyte UTF-8 codepoints.
    let model: &str = if stats.model.chars().count() > 24 {
        stats
            .model
            .char_indices()
            .nth(24)
            .map(|(i, _)| &stats.model[..i])
            .unwrap_or(&stats.model)
    } else {
        &stats.model
    };

    let today = fmt_cost(stats.today_cost_nanos as f64 / 1_000_000_000.0);

    // Phase 8: build session cost segment — dual in Both mode, single otherwise.
    let (session_segment, drift_warn) = build_session_segment(stats, opts.cost_source);

    let mut parts: Vec<String> = Vec::new();
    parts.push(model.to_string());

    match &stats.active_block {
        Some(block) => {
            let block_cost = fmt_cost(block.cost_nanos as f64 / 1_000_000_000.0);
            let remaining = format_remaining(block.block_end);
            let mut cost_segment = format!(
                "{} / {} / {} ({})",
                session_segment, today, block_cost, remaining
            );
            if let Some(warn) = &drift_warn {
                cost_segment.push(' ');
                cost_segment.push_str(warn);
            }
            parts.push(cost_segment);

            if let Some(burn) = block.burn_rate {
                let per_hour = fmt_cost(burn.cost_per_hour_nanos as f64 / 1_000_000_000.0);
                let tier = burn_rate::tier(burn.tokens_per_min, &opts.burn_rate);
                let tag = burn_rate_tag(tier, opts.visual_burn_rate);
                if tag.is_empty() {
                    parts.push(format!("{}/hr", per_hour));
                } else {
                    parts.push(format!("{}/hr {}", per_hour, tag));
                }
            }
        }
        None => {
            let mut seg = format!("{} / {}", session_segment, today);
            if let Some(warn) = &drift_warn {
                seg.push(' ');
                seg.push_str(warn);
            }
            parts.push(seg);
        }
    }

    if let (Some(tokens), Some(size)) = (stats.context_tokens, stats.context_size)
        && size > 0
    {
        let fill = tokens as f64 / size as f64;
        let pct = (fill * 100.0).round() as u64;
        let tok_str = fmt_tokens(tokens);
        let severity = context_severity(
            fill,
            opts.context_low_threshold,
            opts.context_medium_threshold,
        );
        let marker = match severity {
            Severity::Ok => String::new(),
            Severity::Warn => " [WARN]".to_string(),
            Severity::Danger => " [CRIT]".to_string(),
        };
        parts.push(format!("{} tokens ({}%){}", tok_str, pct, marker));
    }

    parts.join(" | ")
}

/// Build the session-cost text segment and an optional drift-warning string.
///
/// Returns `(session_text, Some("[WARN: cost drift]"))` when `Both` mode is
/// active, both values are present, and divergence exceeds 10%.
/// Returns `(session_text, None)` in all other cases.
fn build_session_segment(
    stats: &ComputedStats,
    cost_source: CostSource,
) -> (String, Option<String>) {
    if cost_source == CostSource::Both
        && let Some(hook_nanos) = stats.hook_session_cost_nanos
    {
        let local_nanos = stats.local_session_cost_nanos;
        let hook_str = fmt_cost(hook_nanos as f64 / 1_000_000_000.0);
        let local_str = fmt_cost(local_nanos as f64 / 1_000_000_000.0);
        let segment = format!("({} hook / {} local)", hook_str, local_str);

        // Divergence check: skip entirely when local is below the noise floor
        // ($0.001 = 1_000_000 nanos). A zero or near-zero local with any hook
        // cost would produce a vacuously huge divergence ratio and false WARNs.
        const LOCAL_NOISE_FLOOR_NANOS: i64 = 1_000_000; // $0.001
        let warn = if local_nanos >= LOCAL_NOISE_FLOOR_NANOS {
            let denom = local_nanos as f64;
            let divergence = ((hook_nanos - local_nanos).abs() as f64) / denom;
            if divergence > 0.10 {
                Some("[WARN: cost drift]".to_string())
            } else {
                None
            }
        } else {
            None
        };
        return (segment, warn);
        // Both mode but hook absent → fall through to single-value render
    }
    // Auto / Local / Hook — or Both with no hook data
    let session = fmt_cost(stats.session_cost_nanos as f64 / 1_000_000_000.0);
    (session, None)
}

/// Return the tier indicator string for the given style.
///
/// Returns `""` when `style == Off`.
pub fn burn_rate_tag(tier: BurnRateTier, style: VisualBurnRate) -> &'static str {
    match style {
        VisualBurnRate::Off => "",
        VisualBurnRate::Bracket => match tier {
            BurnRateTier::Normal => "[NORMAL]",
            BurnRateTier::Moderate => "[WARN]",
            BurnRateTier::High => "[CRIT]",
        },
        VisualBurnRate::Emoji => match tier {
            BurnRateTier::Normal => "🟢",
            BurnRateTier::Moderate => "⚠️",
            BurnRateTier::High => "🚨",
        },
        VisualBurnRate::Both => match tier {
            BurnRateTier::Normal => "🟢 [NORMAL]",
            BurnRateTier::Moderate => "⚠️ [WARN]",
            BurnRateTier::High => "🚨 [CRIT]",
        },
    }
}

/// Derive severity for the context-window fill using configurable thresholds.
///
/// - `fill < low_threshold` → Ok
/// - `low_threshold <= fill < medium_threshold` → Warn
/// - `fill >= medium_threshold` → Danger
fn context_severity(fill: f64, low: f64, medium: f64) -> Severity {
    // Delegate to the canonical severity_for_pct when thresholds match defaults,
    // but honour caller-supplied thresholds for config overrides.
    if !fill.is_finite() || fill < low {
        Severity::Ok
    } else if fill < medium {
        Severity::Warn
    } else {
        Severity::Danger
    }
}

/// Format the time remaining until `block_end` relative to now.
/// Returns e.g. "3h 12m left" or "45m left".
fn format_remaining(block_end: chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let remaining = block_end.signed_duration_since(now);
    let total_mins = remaining.num_minutes().max(0);
    let hours = total_mins / 60;
    let mins = total_mins % 60;
    if hours > 0 {
        format!("{}h {:02}m left", hours, mins)
    } else {
        format!("{}m left", mins)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analytics::blocks::BurnRate;
    use crate::statusline::compute::{ActiveBlockInfo, ComputedStats};
    use chrono::Utc;

    fn base_stats() -> ComputedStats {
        ComputedStats {
            model: "claude-sonnet-4-6".to_string(),
            local_session_cost_nanos: 120_000_000,
            hook_session_cost_nanos: None,
            session_cost_nanos: 120_000_000, // $0.12
            today_cost_nanos: 2_340_000_000, // $2.34
            active_block: None,
            context_tokens: None,
            context_size: None,
        }
    }

    #[test]
    fn without_block_without_context() {
        let stats = base_stats();
        let line = render_status_line(&stats);
        assert_eq!(line, "claude-sonnet-4-6 | $0.1200 / $2.3400");
    }

    #[test]
    fn with_context_no_block() {
        let mut stats = base_stats();
        stats.context_tokens = Some(45231);
        stats.context_size = Some(200000);
        let line = render_status_line(&stats);
        assert!(line.contains("45.2K tokens (23%)"), "got: {}", line);
    }

    #[test]
    fn with_active_block_and_burn_rate() {
        let block_end = Utc::now() + chrono::Duration::minutes(192); // 3h 12m
        let mut stats = base_stats();
        stats.active_block = Some(ActiveBlockInfo {
            cost_nanos: 450_000_000, // $0.45
            block_end,
            burn_rate: Some(BurnRate {
                tokens_per_min: 100.0,
                cost_per_hour_nanos: 180_000_000, // $0.18/hr
            }),
        });
        let line = render_status_line(&stats);
        assert!(line.contains("$0.1200"), "missing session cost: {}", line);
        assert!(line.contains("$2.3400"), "missing today cost: {}", line);
        assert!(line.contains("$0.4500"), "missing block cost: {}", line);
        assert!(line.contains("left"), "missing remaining: {}", line);
        assert!(line.contains("$0.1800/hr"), "missing burn rate: {}", line);
        // Default style=Bracket, 100 tokens/min is Normal.
        assert!(line.contains("[NORMAL]"), "missing tier tag: {}", line);
    }

    #[test]
    fn with_block_no_burn_rate() {
        let block_end = Utc::now() + chrono::Duration::minutes(60);
        let mut stats = base_stats();
        stats.active_block = Some(ActiveBlockInfo {
            cost_nanos: 100_000_000,
            block_end,
            burn_rate: None,
        });
        let line = render_status_line(&stats);
        // No burn rate segment.
        assert!(!line.contains("/hr"), "unexpected /hr: {}", line);
    }

    #[test]
    fn model_truncated_to_24_chars() {
        let mut stats = base_stats();
        stats.model = "a".repeat(30);
        let line = render_status_line(&stats);
        // Model segment is the first part before " | ".
        let model_part = line.split(" | ").next().unwrap();
        assert_eq!(model_part.len(), 24);
    }

    #[test]
    fn model_truncation_unicode_no_panic() {
        // Each 'ク' is 3 bytes; 30 chars = 90 bytes. Byte-slicing at 24 would
        // land in the middle of a multibyte sequence and panic. Char-based
        // truncation must yield exactly 24 chars without panic.
        let mut stats = base_stats();
        stats.model = "クロード-sonnet-very-long-model-name-xyz".to_string();
        let line = render_status_line(&stats);
        let model_part = line.split(" | ").next().unwrap();
        assert_eq!(
            model_part.chars().count(),
            24,
            "expected 24 chars, got: {:?}",
            model_part
        );
    }

    #[test]
    fn no_embedded_newlines() {
        let block_end = Utc::now() + chrono::Duration::minutes(60);
        let mut stats = base_stats();
        stats.active_block = Some(ActiveBlockInfo {
            cost_nanos: 100_000_000,
            block_end,
            burn_rate: Some(BurnRate {
                tokens_per_min: 50.0,
                cost_per_hour_nanos: 100_000_000,
            }),
        });
        stats.context_tokens = Some(10000);
        stats.context_size = Some(100000);
        let line = render_status_line(&stats);
        assert!(!line.contains('\n'), "must be single line: {:?}", line);
    }

    #[test]
    fn context_zero_size_omits_segment() {
        let mut stats = base_stats();
        stats.context_tokens = Some(1000);
        stats.context_size = Some(0); // division by zero guard
        let line = render_status_line(&stats);
        assert!(
            !line.contains("tokens"),
            "unexpected token segment: {}",
            line
        );
    }

    // ── Severity-band tests ───────────────────────────────────────────────────

    // ── Char-budget test (see module docblock § "Char budget") ───────────────

    /// Worst-realistic statusline output fits STATUSLINE_MAX_CHARS.
    ///
    /// Builds a fixture exercising every optional segment simultaneously:
    /// long model name (truncated to 24 chars), dual-cost session segment
    /// in `Both` mode with cost-drift WARN, active block with high-tier
    /// burn rate rendered as emoji+bracket, and a CRIT context window.
    /// If a future segment is added without budget consideration, this
    /// test fails before the change ships.
    #[test]
    fn render_fits_budget_worst_case() {
        let block_end = Utc::now() + chrono::Duration::minutes(192);
        let stats = ComputedStats {
            // Long alias — runtime-truncated to 24 chars by render.
            model: "claude-opus-4-5-with-very-long-versioned-suffix".to_string(),
            local_session_cost_nanos: 9_999_900_000, // $9.9999
            // Hook is 15% above local → triggers `[WARN: cost drift]`.
            hook_session_cost_nanos: Some(11_499_885_000),
            session_cost_nanos: 9_999_900_000,
            today_cost_nanos: 99_999_900_000, // $99.9999
            active_block: Some(ActiveBlockInfo {
                cost_nanos: 99_999_900_000, // $99.9999
                block_end,
                burn_rate: Some(BurnRate {
                    // RenderOpts::default() inherits BurnRateConfig::default()
                    // (normal_max=4_000, moderate_max=10_000). 20_000 sits well
                    // above moderate_max → High tier (🚨/[CRIT]).
                    tokens_per_min: 20_000.0,
                    cost_per_hour_nanos: 9_999_900_000,
                }),
            }),
            // ~100% fill → CRIT context marker (worst-case render width).
            context_tokens: Some(199_900),
            context_size: Some(200_000),
        };

        let opts = RenderOpts {
            cost_source: CostSource::Both,
            visual_burn_rate: VisualBurnRate::Both,
            ..RenderOpts::default()
        };
        let line = render_status_line_with_opts(&stats, &opts);

        // Sanity: every load-bearing segment is present (otherwise the
        // budget assertion below would be testing a degraded fixture).
        assert!(
            line.contains("[WARN: cost drift]"),
            "fixture missing drift warn: {line:?}"
        );
        assert!(
            line.contains("🚨"),
            "fixture missing high-tier emoji: {line:?}"
        );
        assert!(
            line.contains("[CRIT]"),
            "fixture missing CRIT marker: {line:?}"
        );

        let len = line.chars().count();
        assert!(
            len <= STATUSLINE_MAX_CHARS,
            "statusline overflowed budget: {len} chars > {STATUSLINE_MAX_CHARS}\n  line: {line:?}"
        );
    }

    /// Below low threshold (22% fill) → no severity marker.
    #[test]
    fn context_severity_below_low_no_marker() {
        let mut stats = base_stats();
        // 22% fill: below default 0.5 low threshold
        stats.context_tokens = Some(44_000);
        stats.context_size = Some(200_000);
        let line = render_status_line(&stats);
        assert!(line.contains("tokens"), "expected token segment: {}", line);
        assert!(!line.contains("[WARN]"), "unexpected WARN: {}", line);
        assert!(!line.contains("[CRIT]"), "unexpected CRIT: {}", line);
    }

    /// Between low (0.5) and medium (0.8) threshold → [WARN] marker.
    #[test]
    fn context_severity_warn_band() {
        let mut stats = base_stats();
        // 55% fill: between 0.5 and 0.8
        stats.context_tokens = Some(110_000);
        stats.context_size = Some(200_000);
        let line = render_status_line(&stats);
        assert!(line.contains("[WARN]"), "expected WARN: {}", line);
        assert!(!line.contains("[CRIT]"), "unexpected CRIT: {}", line);
    }

    /// At or above medium threshold (0.8) → [CRIT] marker.
    #[test]
    fn context_severity_crit_band() {
        let mut stats = base_stats();
        // 90% fill: above 0.8 medium threshold
        stats.context_tokens = Some(180_000);
        stats.context_size = Some(200_000);
        let line = render_status_line(&stats);
        assert!(line.contains("[CRIT]"), "expected CRIT: {}", line);
        assert!(!line.contains("[WARN]"), "unexpected WARN: {}", line);
    }

    /// Config threshold override: low=0.2 means 25% fill triggers [WARN].
    /// Under default thresholds (0.5/0.8) 25% would be Ok — this proves the
    /// threshold is actually plumbed end-to-end and not hard-coded.
    #[test]
    fn custom_low_threshold_triggers_warn_at_25_pct() {
        let mut stats = base_stats();
        // 25% fill — above custom low (0.2) but below custom medium (0.5)
        stats.context_tokens = Some(50_000);
        stats.context_size = Some(200_000);
        // Default thresholds: no marker expected
        let default_line = render_status_line(&stats);
        assert!(
            !default_line.contains("[WARN]"),
            "default thresholds should not WARN at 25%: {}",
            default_line
        );
        // Custom low=0.2: WARN expected
        let custom_line = render_status_line_with_thresholds(&stats, 0.2, 0.5);
        assert!(
            custom_line.contains("[WARN]"),
            "custom low=0.2 should WARN at 25%: {}",
            custom_line
        );
        assert!(
            !custom_line.contains("[CRIT]"),
            "should not be CRIT at 25% with medium=0.5: {}",
            custom_line
        );
    }

    // ── Burn-rate tier render tests ───────────────────────────────────────────

    fn stats_with_burn(tokens_per_min: f64) -> ComputedStats {
        let block_end = Utc::now() + chrono::Duration::minutes(120);
        let mut s = base_stats();
        s.active_block = Some(ActiveBlockInfo {
            cost_nanos: 100_000_000,
            block_end,
            burn_rate: Some(BurnRate {
                tokens_per_min,
                cost_per_hour_nanos: 100_000_000,
            }),
        });
        s
    }

    fn opts_with_style(style: VisualBurnRate) -> RenderOpts {
        RenderOpts {
            context_low_threshold: 0.5,
            context_medium_threshold: 0.8,
            burn_rate: BurnRateConfig::default(),
            visual_burn_rate: style,
            cost_source: CostSource::Auto,
        }
    }

    /// Normal tier (100 tokens/min) under Bracket style → [NORMAL].
    #[test]
    fn bracket_normal_tier() {
        let stats = stats_with_burn(100.0);
        let line = render_status_line_with_opts(&stats, &opts_with_style(VisualBurnRate::Bracket));
        assert!(line.contains("[NORMAL]"), "expected [NORMAL]: {}", line);
        assert!(!line.contains("[WARN]"), "unexpected [WARN]: {}", line);
        assert!(!line.contains("[CRIT]"), "unexpected [CRIT]: {}", line);
    }

    /// Moderate tier (5000 tokens/min) under Bracket style → [WARN].
    #[test]
    fn bracket_moderate_tier() {
        let stats = stats_with_burn(5000.0);
        let line = render_status_line_with_opts(&stats, &opts_with_style(VisualBurnRate::Bracket));
        assert!(line.contains("[WARN]"), "expected [WARN]: {}", line);
        assert!(!line.contains("[NORMAL]"), "unexpected [NORMAL]: {}", line);
        assert!(!line.contains("[CRIT]"), "unexpected [CRIT]: {}", line);
    }

    /// High tier (15000 tokens/min) under Bracket style → [CRIT].
    #[test]
    fn bracket_high_tier() {
        let stats = stats_with_burn(15000.0);
        let line = render_status_line_with_opts(&stats, &opts_with_style(VisualBurnRate::Bracket));
        assert!(line.contains("[CRIT]"), "expected [CRIT]: {}", line);
        assert!(!line.contains("[WARN]"), "unexpected [WARN]: {}", line);
        assert!(!line.contains("[NORMAL]"), "unexpected [NORMAL]: {}", line);
    }

    /// Off style strips all tier markers.
    #[test]
    fn off_style_strips_tier() {
        let stats = stats_with_burn(15000.0); // High tier
        let line = render_status_line_with_opts(&stats, &opts_with_style(VisualBurnRate::Off));
        assert!(!line.contains("[NORMAL]"), "unexpected [NORMAL]: {}", line);
        assert!(!line.contains("[WARN]"), "unexpected [WARN]: {}", line);
        assert!(!line.contains("[CRIT]"), "unexpected [CRIT]: {}", line);
        // Burn rate $/hr is still present.
        assert!(line.contains("/hr"), "expected /hr: {}", line);
    }

    // ── Phase 8: Both mode render tests ──────────────────────────────────────

    fn both_opts() -> RenderOpts {
        RenderOpts {
            cost_source: CostSource::Both,
            visual_burn_rate: VisualBurnRate::Off,
            ..RenderOpts::default()
        }
    }

    fn stats_with_dual(hook_nanos: Option<i64>, local_nanos: i64) -> ComputedStats {
        ComputedStats {
            model: "claude-sonnet-4-6".to_string(),
            local_session_cost_nanos: local_nanos,
            hook_session_cost_nanos: hook_nanos,
            // effective = hook when present else local
            session_cost_nanos: hook_nanos.unwrap_or(local_nanos),
            today_cost_nanos: 2_340_000_000,
            active_block: None,
            context_tokens: None,
            context_size: None,
        }
    }

    /// Both mode, both values present, divergence < 10% — no WARN.
    #[test]
    fn both_mode_both_present_no_drift() {
        // hook=$0.12, local=$0.125 → ~4% divergence
        let stats = stats_with_dual(Some(120_000_000), 125_000_000);
        let line = render_status_line_with_opts(&stats, &both_opts());
        assert!(line.contains("hook"), "expected 'hook': {}", line);
        assert!(line.contains("local"), "expected 'local': {}", line);
        assert!(
            !line.contains("[WARN: cost drift]"),
            "unexpected drift WARN: {}",
            line
        );
    }

    /// Both mode, both values present, divergence > 10% — WARN appears.
    #[test]
    fn both_mode_both_present_with_drift() {
        // hook=$0.14, local=$0.10 → 40% divergence; local above noise floor
        let stats = stats_with_dual(Some(140_000_000), 100_000_000);
        let line = render_status_line_with_opts(&stats, &both_opts());
        assert!(line.contains("hook"), "expected 'hook': {}", line);
        assert!(line.contains("local"), "expected 'local': {}", line);
        assert!(
            line.contains("[WARN: cost drift]"),
            "expected drift WARN: {}",
            line
        );
    }

    /// Both mode, hook absent — falls back to single-value rendering (no "hook/local").
    #[test]
    fn both_mode_hook_absent_falls_back_to_single() {
        let stats = stats_with_dual(None, 120_000_000);
        let line = render_status_line_with_opts(&stats, &both_opts());
        assert!(!line.contains("hook"), "unexpected 'hook': {}", line);
        assert!(!line.contains("local"), "unexpected 'local': {}", line);
        // Renders the local cost as single value
        assert!(line.contains("$0.1200"), "expected $0.1200: {}", line);
    }

    /// Both mode, hook has cost but local=0 (below noise floor) — no WARN.
    #[test]
    fn both_mode_empty_local_no_warn() {
        // hook=$0.10 (100_000_000 nanos), local=0 → below noise floor → skip drift check
        let stats = stats_with_dual(Some(100_000_000), 0);
        let line = render_status_line_with_opts(&stats, &both_opts());
        assert!(line.contains("hook"), "expected 'hook': {}", line);
        assert!(line.contains("local"), "expected 'local': {}", line);
        assert!(
            !line.contains("[WARN: cost drift]"),
            "must NOT warn when local is below noise floor: {}",
            line
        );
    }

    /// Both mode, exact 10% divergence — no WARN (threshold is strictly > 10%).
    #[test]
    fn both_mode_exactly_10_pct_no_drift_warn() {
        // hook=110, local=100 → exactly 10%
        let stats = stats_with_dual(Some(110_000_000), 100_000_000);
        let line = render_status_line_with_opts(&stats, &both_opts());
        assert!(
            !line.contains("[WARN: cost drift]"),
            "no WARN at exactly 10%: {}",
            line
        );
    }

    /// Hook mode — uses hook cost as single value, no "hook/local" labels.
    #[test]
    fn hook_mode_single_value() {
        let opts = RenderOpts {
            cost_source: CostSource::Hook,
            visual_burn_rate: VisualBurnRate::Off,
            ..RenderOpts::default()
        };
        let stats = stats_with_dual(Some(120_000_000), 999_000_000);
        let line = render_status_line_with_opts(&stats, &opts);
        assert!(!line.contains("hook"), "unexpected 'hook': {}", line);
        assert!(!line.contains("local"), "unexpected 'local': {}", line);
        // session_cost_nanos=120_000_000 (set by compute; here we simulate)
        // The stats_with_dual helper sets session_cost_nanos=hook when present.
        assert!(line.contains("$0.1200"), "expected $0.1200: {}", line);
    }

    /// Local mode — uses local cost as single value.
    #[test]
    fn local_mode_single_value() {
        let opts = RenderOpts {
            cost_source: CostSource::Local,
            visual_burn_rate: VisualBurnRate::Off,
            ..RenderOpts::default()
        };
        // session_cost_nanos mirrors local in Local mode
        let mut stats = stats_with_dual(Some(999_000_000), 120_000_000);
        stats.session_cost_nanos = stats.local_session_cost_nanos; // simulate Local mode
        let line = render_status_line_with_opts(&stats, &opts);
        assert!(!line.contains("hook"), "unexpected 'hook': {}", line);
        assert!(!line.contains("local"), "unexpected 'local': {}", line);
        assert!(line.contains("$0.1200"), "expected $0.1200: {}", line);
    }
}
