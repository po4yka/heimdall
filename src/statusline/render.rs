/// Compose the output status line from computed stats.
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
pub fn render_status_line(stats: &ComputedStats) -> String {
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
        let pct = ((tokens as f64 / size as f64) * 100.0).round() as u64;
        let tok_str = fmt_tokens(tokens);
        parts.push(format!("{} tokens ({}%)", tok_str, pct));
    }

    parts.join(" | ")
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
}
