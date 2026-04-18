/// Context-window resolution: hook payload → transcript fallback.
///
/// Phase 5 addition: when the hook JSON lacks `context_window` fields, parse
/// the JSONL transcript to extract the last assistant usage block and infer the
/// context size from the model name.
use std::io::{BufRead, BufReader};
use std::path::Path;

use anyhow::{Context, Result};

use crate::statusline::input::HookInput;

// ── Public type ───────────────────────────────────────────────────────────────

/// Resolved context-window snapshot: both fields guaranteed non-zero.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContextWindow {
    pub total_input_tokens: i64,
    pub context_window_size: i64,
}

impl ContextWindow {
    /// Fractional fill: 0.0 = empty, 1.0 = full, >1.0 = over limit.
    pub fn pct(&self) -> f64 {
        if self.context_window_size <= 0 {
            0.0
        } else {
            self.total_input_tokens as f64 / self.context_window_size as f64
        }
    }
}

// ── Resolvers ─────────────────────────────────────────────────────────────────

/// Read from the hook JSON payload when both `total_input_tokens` and
/// `context_window_size` are present and `context_window_size > 0`.
pub fn from_hook(input: &HookInput) -> Option<ContextWindow> {
    let cw = input.context_window.as_ref()?;
    let tokens = cw.total_input_tokens?;
    let size = cw.context_window_size?;
    if size <= 0 {
        return None;
    }
    Some(ContextWindow {
        total_input_tokens: tokens,
        context_window_size: size,
    })
}

/// Transcript fallback: parse the JSONL at `path`, find the most recent
/// assistant message with a `usage` block, derive the context fill and infer
/// the context-window size from the model name.
///
/// `total_input_tokens = input_tokens + cache_read_input_tokens + cache_creation_input_tokens`
/// (reasoning tokens excluded per spec).
pub fn from_transcript(path: &Path) -> Result<ContextWindow> {
    let file = std::fs::File::open(path)
        .with_context(|| format!("cannot open transcript: {}", path.display()))?;
    let reader = BufReader::new(file);

    let mut last: Option<ContextWindow> = None;

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let v: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if v.get("type").and_then(|t| t.as_str()) != Some("assistant") {
            continue;
        }

        let msg = match v.get("message") {
            Some(m) => m,
            None => continue,
        };

        let usage = match msg.get("usage") {
            Some(u) => u,
            None => continue,
        };

        let input_tokens = usage
            .get("input_tokens")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let cache_read = usage
            .get("cache_read_input_tokens")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let cache_creation = usage
            .get("cache_creation_input_tokens")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        let total_input_tokens = input_tokens + cache_read + cache_creation;

        let model = msg
            .get("model")
            .and_then(|m| m.as_str())
            .unwrap_or("")
            .to_owned();

        let context_window_size = context_size_for_model(&model);

        last = Some(ContextWindow {
            total_input_tokens,
            context_window_size,
        });
    }

    last.ok_or_else(|| anyhow::anyhow!("no assistant usage block found in transcript: {}", path.display()))
}

/// Combined resolver: prefer hook data, fall back to transcript, return `None`
/// if neither yields data.
pub fn resolve(input: &HookInput) -> Option<ContextWindow> {
    if let Some(cw) = from_hook(input) {
        return Some(cw);
    }
    let path = Path::new(&input.transcript_path);
    from_transcript(path).ok()
}

// ── Model size table ──────────────────────────────────────────────────────────

/// Return the context-window size (in tokens) for a given model name.
///
/// GPT/O-series: 128k. Claude Haiku 3 (pre-3.5): 100k.
/// All other Claude families and unknown models: 200k.
pub fn context_size_for_model(model: &str) -> i64 {
    let lower = model.to_ascii_lowercase();
    // GPT and O-series (OpenAI via Codex) — 128k
    if lower.starts_with("gpt-") || lower.starts_with("o1-") || lower.starts_with("o3-") {
        return 128_000;
    }
    // Claude Haiku 3 (pre-3.5) — 100k; Haiku 3.5+ and 4.x are 200k
    if lower.contains("haiku-3-") && !lower.contains("3-5") {
        return 100_000;
    }
    // Claude family default (Sonnet / Opus / Haiku 3.5+ / 4.x) — 200k
    if lower.contains("claude") {
        return 200_000;
    }
    // Ultimate fallback
    200_000
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::statusline::input::{ContextWindow as HookCW, HookInput};
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn make_input(tokens: Option<i64>, size: Option<i64>) -> HookInput {
        HookInput {
            session_id: "s1".to_string(),
            transcript_path: "/tmp/t.jsonl".to_string(),
            model: Some("claude-sonnet-4-6".to_string()),
            cost: None,
            context_window: Some(HookCW {
                total_input_tokens: tokens,
                context_window_size: size,
            }),
        }
    }

    fn make_input_no_cw() -> HookInput {
        HookInput {
            session_id: "s1".to_string(),
            transcript_path: "/tmp/t.jsonl".to_string(),
            model: Some("claude-sonnet-4-6".to_string()),
            cost: None,
            context_window: None,
        }
    }

    // ── from_hook ─────────────────────────────────────────────────────────────

    #[test]
    fn from_hook_present() {
        let input = make_input(Some(45231), Some(200_000));
        let cw = from_hook(&input).expect("should be Some");
        assert_eq!(cw.total_input_tokens, 45231);
        assert_eq!(cw.context_window_size, 200_000);
    }

    #[test]
    fn from_hook_missing_tokens_returns_none() {
        let input = make_input(None, Some(200_000));
        assert!(from_hook(&input).is_none());
    }

    #[test]
    fn from_hook_zero_size_returns_none() {
        let input = make_input(Some(1000), Some(0));
        assert!(from_hook(&input).is_none());
    }

    #[test]
    fn from_hook_no_context_window_returns_none() {
        let input = make_input_no_cw();
        assert!(from_hook(&input).is_none());
    }

    // ── from_transcript ───────────────────────────────────────────────────────

    fn write_transcript(lines: &[serde_json::Value]) -> NamedTempFile {
        let mut f = NamedTempFile::new().unwrap();
        for line in lines {
            writeln!(f, "{}", line).unwrap();
        }
        f
    }

    fn assistant_line(
        input_tokens: i64,
        cache_read: i64,
        cache_creation: i64,
        model: &str,
    ) -> serde_json::Value {
        serde_json::json!({
            "type": "assistant",
            "message": {
                "model": model,
                "usage": {
                    "input_tokens": input_tokens,
                    "output_tokens": 100,
                    "cache_read_input_tokens": cache_read,
                    "cache_creation_input_tokens": cache_creation,
                }
            }
        })
    }

    #[test]
    fn from_transcript_happy_path() {
        let f = write_transcript(&[assistant_line(1000, 200, 50, "claude-sonnet-4-6")]);
        let cw = from_transcript(f.path()).expect("should parse");
        assert_eq!(cw.total_input_tokens, 1250); // 1000+200+50
        assert_eq!(cw.context_window_size, 200_000);
    }

    #[test]
    fn from_transcript_latest_wins() {
        // Three assistant messages; result should be the last one.
        let f = write_transcript(&[
            assistant_line(1000, 0, 0, "claude-sonnet-4-6"),
            serde_json::json!({"type": "user"}),
            assistant_line(5000, 100, 0, "claude-sonnet-4-6"),
            assistant_line(9000, 0, 500, "claude-sonnet-4-6"),
        ]);
        let cw = from_transcript(f.path()).expect("should parse");
        assert_eq!(cw.total_input_tokens, 9500); // 9000+0+500
    }

    #[test]
    fn from_transcript_missing_usage_returns_err() {
        let f = write_transcript(&[serde_json::json!({
            "type": "assistant",
            "message": { "model": "claude-sonnet-4-6" }
            // no "usage"
        })]);
        assert!(from_transcript(f.path()).is_err());
    }

    #[test]
    fn from_transcript_nonexistent_returns_err() {
        let result = from_transcript(Path::new("/nonexistent/path/transcript.jsonl"));
        assert!(result.is_err());
    }

    #[test]
    fn from_transcript_invalid_json_line_skipped() {
        let mut f = NamedTempFile::new().unwrap();
        writeln!(f, "not valid json {{{{").unwrap();
        writeln!(f, "{}", assistant_line(2000, 0, 0, "claude-sonnet-4-6")).unwrap();
        let cw = from_transcript(f.path()).expect("should fall through to valid line");
        assert_eq!(cw.total_input_tokens, 2000);
    }

    // ── context_size_for_model ────────────────────────────────────────────────

    #[test]
    fn context_size_claude_sonnet() {
        assert_eq!(context_size_for_model("claude-sonnet-4-6"), 200_000);
    }

    #[test]
    fn context_size_claude_opus() {
        assert_eq!(context_size_for_model("claude-opus-4"), 200_000);
    }

    #[test]
    fn context_size_gpt_family_128k() {
        assert_eq!(context_size_for_model("gpt-4o"), 128_000);
        assert_eq!(context_size_for_model("gpt-5-turbo"), 128_000);
    }

    #[test]
    fn context_size_o_series_128k() {
        assert_eq!(context_size_for_model("o1-mini"), 128_000);
        assert_eq!(context_size_for_model("o3-preview"), 128_000);
    }

    #[test]
    fn context_size_haiku_3_legacy_100k() {
        // "haiku-3-" without "3-5" → old 100k window
        assert_eq!(context_size_for_model("claude-haiku-3-0"), 100_000);
    }

    #[test]
    fn context_size_haiku_35_is_200k() {
        // haiku 3.5 should NOT match the 100k branch
        assert_eq!(context_size_for_model("claude-haiku-3-5"), 200_000);
    }

    #[test]
    fn context_size_unknown_fallback() {
        assert_eq!(context_size_for_model("some-unknown-model"), 200_000);
    }

    // ── pct ───────────────────────────────────────────────────────────────────

    #[test]
    fn pct_calculation() {
        let cw = ContextWindow {
            total_input_tokens: 100_000,
            context_window_size: 200_000,
        };
        assert!((cw.pct() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn pct_zero_size_returns_zero() {
        let cw = ContextWindow {
            total_input_tokens: 1000,
            context_window_size: 0,
        };
        assert_eq!(cw.pct(), 0.0);
    }
}
