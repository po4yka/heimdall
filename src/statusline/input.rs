/// Stdin JSON parsing for the `statusline` PostToolUse hook.
///
/// Mirrors ccusage's `statuslineHookJsonSchema` field layout.
use std::io::Read;
use std::sync::mpsc;
use std::time::Duration;

use anyhow::{Context, Result};

#[derive(Debug, Clone, serde::Deserialize)]
pub struct HookInput {
    pub session_id: String,
    pub transcript_path: String,
    /// Model may be a plain string OR an object `{"id": "...", "display_name": "..."}`.
    /// We accept both via a custom deserializer.
    #[serde(default, deserialize_with = "deserialize_model_field")]
    pub model: Option<String>,
    /// Anthropic-reported session cost (USD). Present when Claude Code runs
    /// against the Anthropic API directly.
    /// Accepts both `{"total_cost_usd": 0.12}` (ccusage schema) and a bare
    /// number `0.12` (legacy/simplified payloads) via a custom deserializer.
    #[serde(default, deserialize_with = "deserialize_cost_field")]
    pub cost: Option<HookCost>,
    #[serde(default)]
    pub context_window: Option<ContextWindow>,
}

/// Deserialize the `model` field: accepts either a plain string or an object
/// `{"id": "...", "display_name": "..."}`.  Returns the `id` field when given
/// an object, or the string as-is when given a plain string.
fn deserialize_model_field<'de, D>(deserializer: D) -> std::result::Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let v = Option::<serde_json::Value>::deserialize(deserializer)?;
    match v {
        None => Ok(None),
        Some(serde_json::Value::String(s)) => Ok(Some(s)),
        Some(serde_json::Value::Object(map)) => {
            // Prefer "id" key; fall back to "display_name".
            let id = map
                .get("id")
                .or_else(|| map.get("display_name"))
                .and_then(|v| v.as_str())
                .map(str::to_owned);
            Ok(id)
        }
        _ => Ok(None),
    }
}

/// Deserialize the `cost` field: accepts either a plain number (bare f64) or
/// an object `{"total_cost_usd": ..., ...}`.
fn deserialize_cost_field<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<HookCost>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let v = Option::<serde_json::Value>::deserialize(deserializer)?;
    match v {
        None => Ok(None),
        Some(serde_json::Value::Number(n)) => {
            let usd = n.as_f64().unwrap_or(0.0);
            Ok(Some(HookCost {
                total_cost_usd: usd,
                total_duration_ms: None,
                total_api_duration_ms: None,
            }))
        }
        Some(obj @ serde_json::Value::Object(_)) => serde_json::from_value::<HookCost>(obj)
            .map(Some)
            .map_err(serde::de::Error::custom),
        _ => Ok(None),
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct HookCost {
    pub total_cost_usd: f64,
    #[serde(default)]
    pub total_duration_ms: Option<f64>,
    #[serde(default)]
    pub total_api_duration_ms: Option<f64>,
}

#[derive(Debug, Clone, Copy, serde::Deserialize)]
pub struct ContextWindow {
    #[serde(default)]
    pub total_input_tokens: Option<i64>,
    #[serde(default)]
    pub context_window_size: Option<i64>,
}

/// Read all of stdin with a hard timeout.
/// Returns `Err` on timeout or parse failure.
///
/// Implementation note: a reader thread is spawned because `stdin().lock()`
/// blocks until EOF.  Claude Code closes stdin after writing, so the thread
/// exits quickly in the normal case.  If stdin is never closed (pathological),
/// the thread outlives this function call but is reaped when the statusline
/// subcommand process exits (which happens within ~100 ms after the timeout).
/// This is intentional and safe for a short-lived per-invocation binary.
pub fn parse_stdin_with_timeout(timeout: Duration) -> Result<HookInput> {
    let (tx, rx) = mpsc::channel::<String>();

    // Named thread for easier debugging if it appears in profiler output.
    std::thread::Builder::new()
        .name("stdin-reader".to_owned())
        .spawn(move || {
            let mut buf = String::new();
            let mut stdin = std::io::stdin().lock();
            let _ = stdin.read_to_string(&mut buf);
            // tx drop on send failure is harmless — receiver timed out.
            let _ = tx.send(buf);
        })
        .context("failed to spawn stdin-reader thread")?;

    let raw = rx
        .recv_timeout(timeout)
        .context("timed out reading stdin")?;

    if raw.trim().is_empty() {
        anyhow::bail!("empty stdin");
    }

    serde_json::from_str(raw.trim()).context("failed to parse hook JSON from stdin")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_payload_object_cost_deserializes() {
        let json = r#"{
            "session_id": "abc123",
            "transcript_path": "/tmp/transcript.jsonl",
            "model": "claude-sonnet-4-6",
            "cost": { "total_cost_usd": 0.12 },
            "context_window": {
                "total_input_tokens": 45231,
                "context_window_size": 200000
            }
        }"#;
        let input: HookInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.session_id, "abc123");
        assert_eq!(input.model.as_deref(), Some("claude-sonnet-4-6"));
        assert!(input.cost.is_some());
        assert!((input.cost.unwrap().total_cost_usd - 0.12).abs() < 1e-10);
        let cw = input.context_window.unwrap();
        assert_eq!(cw.total_input_tokens, Some(45231));
        assert_eq!(cw.context_window_size, Some(200000));
    }

    /// Smoke-test: bare number in `cost` (simplified payload) must parse and
    /// yield the same cost value.
    #[test]
    fn bare_number_cost_deserializes() {
        let json = r#"{
            "session_id": "test",
            "transcript_path": "/tmp/x",
            "model": "claude-sonnet-4-6",
            "cost": 0.12,
            "context_window": {
                "total_input_tokens": 45231,
                "context_window_size": 200000
            }
        }"#;
        let input: HookInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.session_id, "test");
        let cost = input.cost.expect("cost should be Some");
        assert!((cost.total_cost_usd - 0.12).abs() < 1e-10);
        assert_eq!(input.model.as_deref(), Some("claude-sonnet-4-6"));
        let cw = input.context_window.unwrap();
        assert_eq!(cw.total_input_tokens, Some(45231));
        assert_eq!(cw.context_window_size, Some(200000));
    }

    /// Model as object `{"id": "...", "display_name": "..."}` (ccusage schema).
    #[test]
    fn model_object_field_deserializes() {
        let json = r#"{
            "session_id": "s1",
            "transcript_path": "/tmp/t.jsonl",
            "model": {"id": "claude-sonnet-4-6", "display_name": "Claude Sonnet 4.6"}
        }"#;
        let input: HookInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.model.as_deref(), Some("claude-sonnet-4-6"));
    }

    #[test]
    fn missing_optionals_deserializes() {
        let json = r#"{
            "session_id": "s1",
            "transcript_path": "/tmp/t.jsonl"
        }"#;
        let input: HookInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.session_id, "s1");
        assert!(input.model.is_none());
        assert!(input.cost.is_none());
        assert!(input.context_window.is_none());
    }

    #[test]
    fn malformed_json_returns_error() {
        let result: Result<HookInput, _> = serde_json::from_str("not json");
        assert!(result.is_err());
    }

    #[test]
    fn missing_required_fields_returns_error() {
        // Missing session_id
        let result: Result<HookInput, _> = serde_json::from_str(r#"{"transcript_path":"/t"}"#);
        assert!(result.is_err());
    }
}
