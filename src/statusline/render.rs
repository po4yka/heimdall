/// Compose the output status line from computed stats.
use crate::analytics::quota::Severity;
use crate::pricing::{fmt_cost, fmt_tokens};
use crate::statusline::compute::ComputedStats;

/// Render a single-line status string.
///
/// Layout:
///   MODEL | $SESSION / $TODAY / $BLOCK (Xh Ym left) | $COST/hr | N tokens (XX%)
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
    render_status_line_with_thresholds(stats, 0.5, 0.8)
}

/// Like `render_status_line` but with configurable severity thresholds.
pub fn render_status_line_with_thresholds(
    stats: &ComputedStats,
    context_low_threshold: f64,
    context_medium_threshold: f64,
) -> String {
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

    let session = fmt_cost(stats.session_cost_nanos as f64 / 1_000_000_000.0);
    let today = fmt_cost(stats.today_cost_nanos as f64 / 1_000_000_000.0);

    let mut parts: Vec<String> = Vec::new();
    parts.push(model.to_string());

    match &stats.active_block {
        Some(block) => {
            let block_cost = fmt_cost(block.cost_nanos as f64 / 1_000_000_000.0);
            let remaining = format_remaining(block.block_end);
            let cost_segment = format!("{} / {} / {} ({})", session, today, block_cost, remaining);
            parts.push(cost_segment);

            if let Some(burn) = block.burn_rate {
                let per_hour = fmt_cost(burn.cost_per_hour_nanos as f64 / 1_000_000_000.0);
                parts.push(format!("{}/hr", per_hour));
            }
        }
        None => {
            parts.push(format!("{} / {}", session, today));
        }
    }

    if let (Some(tokens), Some(size)) = (stats.context_tokens, stats.context_size)
        && size > 0
    {
        let fill = tokens as f64 / size as f64;
        let pct = (fill * 100.0).round() as u64;
        let tok_str = fmt_tokens(tokens);
        let severity = context_severity(fill, context_low_threshold, context_medium_threshold);
        let marker = match severity {
            Severity::Ok => String::new(),
            Severity::Warn => " [WARN]".to_string(),
            Severity::Danger => " [CRIT]".to_string(),
        };
        parts.push(format!("{} tokens ({}%){}", tok_str, pct, marker));
    }

    parts.join(" | ")
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
}
