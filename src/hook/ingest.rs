/// Hook payload parsing and `live_events` row construction.
///
/// Claude Code delivers a JSON object on stdin for every PreToolUse event.
/// The shape (from Claude-Guardian/hook/pre_tool_use.py) is roughly:
///
/// ```json
/// {
///   "session_id": "abc123",
///   "tool_name": "Edit",
///   "tool_use_id": "tu_xyz",
///   "hook_input": {
///     "cost": { "total_cost_usd": 0.000123 },
///     "usage": { "input_tokens": 500, "output_tokens": 200 }
///   }
/// }
/// ```
///
/// All fields are optional; missing fields map to `None` / 0.
use serde::Deserialize;

/// Extracted fields from a single PreToolUse hook payload.
#[derive(Debug, PartialEq)]
pub struct LiveEvent {
    pub dedup_key: String,
    pub received_at: String,
    pub session_id: Option<String>,
    pub tool_name: Option<String>,
    /// Cost stored as integer nanos (1 USD = 1_000_000_000 nanos).
    pub cost_usd_nanos: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub raw_json: String,
    /// Phase 5: context-window fields (NULL when absent in payload).
    pub context_input_tokens: Option<i64>,
    pub context_window_size: Option<i64>,
    /// Phase 8: hook-reported cost nanos. Some when cost was present in
    /// payload, None when absent (so the DB column stays NULL).
    pub hook_reported_cost_nanos: Option<i64>,
}

// ── Internal deserialization types ──────────────────────────────────────────

#[derive(Deserialize, Default)]
struct HookPayload {
    session_id: Option<String>,
    tool_name: Option<String>,
    tool_use_id: Option<String>,
    hook_input: Option<HookInput>,
}

#[derive(Deserialize, Default)]
struct HookInput {
    #[serde(default, deserialize_with = "deserialize_cost_field")]
    cost: Option<f64>,
    usage: Option<UsageBlock>,
    context_window: Option<ContextWindowBlock>,
}

#[derive(Deserialize, Default)]
struct ContextWindowBlock {
    total_input_tokens: Option<i64>,
    context_window_size: Option<i64>,
}

#[derive(Deserialize, Default)]
struct UsageBlock {
    input_tokens: Option<i64>,
    output_tokens: Option<i64>,
}

/// Deserialize the `cost` field: accepts either a bare number OR an object
/// `{ "total_cost_usd": ... }`. Returns `None` when absent or unrecognised.
fn deserialize_cost_field<'de, D>(deserializer: D) -> std::result::Result<Option<f64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let v = Option::<serde_json::Value>::deserialize(deserializer)?;
    match v {
        None => Ok(None),
        Some(serde_json::Value::Number(n)) => Ok(Some(n.as_f64().unwrap_or(0.0))),
        Some(serde_json::Value::Object(map)) => {
            let usd = map.get("total_cost_usd").and_then(|v| v.as_f64());
            match usd {
                Some(n) => Ok(Some(n)),
                None => Ok(None),
            }
        }
        _ => Ok(None),
    }
}

// ── Public API ───────────────────────────────────────────────────────────────

/// Parse raw JSON bytes from stdin into a `LiveEvent`.
///
/// Returns `None` if `json` is not valid JSON — callers should log this to
/// stderr and exit 0 without writing to the DB.
pub fn parse_hook_payload(json: &str, received_at: &str) -> Option<LiveEvent> {
    let payload: HookPayload = serde_json::from_str(json).ok()?;

    // hook_input.cost is now Option<f64> (already decoded from bare number OR object)
    let hook_cost_usd: Option<f64> = payload
        .hook_input
        .as_ref()
        .and_then(|hi| hi.cost);

    let cost_usd_nanos = hook_cost_usd.map(usd_to_nanos).unwrap_or(0);

    // Phase 8: persist the raw hook cost separately (None = column stays NULL).
    let hook_reported_cost_nanos = hook_cost_usd.map(usd_to_nanos);

    let input_tokens = payload
        .hook_input
        .as_ref()
        .and_then(|hi| hi.usage.as_ref())
        .and_then(|u| u.input_tokens)
        .unwrap_or(0);

    let output_tokens = payload
        .hook_input
        .as_ref()
        .and_then(|hi| hi.usage.as_ref())
        .and_then(|u| u.output_tokens)
        .unwrap_or(0);

    let context_input_tokens = payload
        .hook_input
        .as_ref()
        .and_then(|hi| hi.context_window.as_ref())
        .and_then(|cw| cw.total_input_tokens);

    let context_window_size = payload
        .hook_input
        .as_ref()
        .and_then(|hi| hi.context_window.as_ref())
        .and_then(|cw| cw.context_window_size);

    let dedup_key = build_dedup_key(
        payload.session_id.as_deref(),
        payload.tool_use_id.as_deref(),
        received_at,
    );

    Some(LiveEvent {
        dedup_key,
        received_at: received_at.to_string(),
        session_id: payload.session_id,
        tool_name: payload.tool_name,
        cost_usd_nanos,
        input_tokens,
        output_tokens,
        raw_json: json.to_string(),
        context_input_tokens,
        context_window_size,
        hook_reported_cost_nanos,
    })
}

/// Convert a USD float to integer nanos, clamped to non-negative.
fn usd_to_nanos(usd: f64) -> i64 {
    (usd * 1_000_000_000.0).round().max(0.0) as i64
}

/// Build the dedup key: `"{session_id}:{tool_use_id}"` when tool_use_id is
/// present, else `"{session_id}:{received_at}"`.
fn build_dedup_key(
    session_id: Option<&str>,
    tool_use_id: Option<&str>,
    received_at: &str,
) -> String {
    let sid = session_id.unwrap_or("unknown");
    match tool_use_id {
        Some(id) if !id.is_empty() => format!("{sid}:{id}"),
        _ => format!("{sid}:{received_at}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const REALISTIC_PAYLOAD: &str = r#"{
        "session_id": "session-abc123",
        "tool_name": "Edit",
        "tool_use_id": "tu_xyz789",
        "hook_input": {
            "cost": { "total_cost_usd": 0.001234 },
            "usage": { "input_tokens": 1500, "output_tokens": 300 }
        }
    }"#;

    #[test]
    fn parse_realistic_payload_extracts_all_fields() {
        let event = parse_hook_payload(REALISTIC_PAYLOAD, "2024-01-15T10:30:00.123Z")
            .expect("should parse");

        assert_eq!(event.session_id.as_deref(), Some("session-abc123"));
        assert_eq!(event.tool_name.as_deref(), Some("Edit"));
        assert_eq!(event.dedup_key, "session-abc123:tu_xyz789");
        // 0.001234 USD * 1e9 rounded = 1_234_000
        assert_eq!(event.cost_usd_nanos, 1_234_000);
        assert_eq!(event.input_tokens, 1500);
        assert_eq!(event.output_tokens, 300);
    }

    #[test]
    fn parse_malformed_json_returns_none() {
        let result = parse_hook_payload("not valid json {{{", "2024-01-15T10:30:00Z");
        assert!(result.is_none());
    }

    #[test]
    fn parse_empty_object_uses_defaults() {
        let event = parse_hook_payload("{}", "2024-01-15T10:30:00Z").expect("should parse");
        assert!(event.session_id.is_none());
        assert!(event.tool_name.is_none());
        assert_eq!(event.cost_usd_nanos, 0);
        assert_eq!(event.input_tokens, 0);
        assert_eq!(event.output_tokens, 0);
        // dedup_key falls back to received_at
        assert!(event.dedup_key.contains("unknown:"));
    }

    #[test]
    fn parse_missing_tool_use_id_uses_received_at_in_dedup_key() {
        let json = r#"{"session_id": "ses1"}"#;
        let ts = "2024-01-15T10:30:00.999Z";
        let event = parse_hook_payload(json, ts).expect("should parse");
        assert_eq!(event.dedup_key, format!("ses1:{ts}"));
    }

    #[test]
    fn parse_missing_cost_block_gives_zero_nanos() {
        let json = r#"{"session_id": "s1", "tool_name": "Bash", "tool_use_id": "t1"}"#;
        let event = parse_hook_payload(json, "ts").expect("should parse");
        assert_eq!(event.cost_usd_nanos, 0);
    }

    #[test]
    fn usd_to_nanos_rounds_correctly() {
        assert_eq!(usd_to_nanos(0.001), 1_000_000);
        assert_eq!(usd_to_nanos(1.0), 1_000_000_000);
        // Negative values clamp to zero
        assert_eq!(usd_to_nanos(-1.0), 0);
    }

    #[test]
    fn dedup_key_uses_tool_use_id_when_present() {
        assert_eq!(build_dedup_key(Some("ses"), Some("tu1"), "ts"), "ses:tu1");
    }

    #[test]
    fn dedup_key_falls_back_to_received_at_for_empty_tool_use_id() {
        assert_eq!(build_dedup_key(Some("ses"), Some(""), "ts123"), "ses:ts123");
    }

    /// Phase 5: context_window fields are extracted when present in hook_input.
    #[test]
    fn parse_context_window_fields_extracted() {
        let json = r#"{
            "session_id": "s1",
            "tool_name": "Edit",
            "tool_use_id": "tu1",
            "hook_input": {
                "cost": { "total_cost_usd": 0.001 },
                "usage": { "input_tokens": 500, "output_tokens": 100 },
                "context_window": {
                    "total_input_tokens": 45231,
                    "context_window_size": 200000
                }
            }
        }"#;
        let event = parse_hook_payload(json, "2026-04-18T10:00:00Z").expect("should parse");
        assert_eq!(event.context_input_tokens, Some(45231));
        assert_eq!(event.context_window_size, Some(200_000));
    }

    /// Phase 5: context_window absent → both fields are None.
    #[test]
    fn parse_context_window_absent_gives_none() {
        let json = r#"{
            "session_id": "s1",
            "tool_use_id": "tu2",
            "hook_input": { "cost": { "total_cost_usd": 0.0 } }
        }"#;
        let event = parse_hook_payload(json, "ts").expect("should parse");
        assert!(event.context_input_tokens.is_none());
        assert!(event.context_window_size.is_none());
    }

    // ── Phase 8: hook_reported_cost_nanos tests ──────────────────────────────

    /// Cost object shape → hook_reported_cost_nanos is Some.
    #[test]
    fn parse_cost_object_gives_hook_reported_cost_nanos() {
        let json = r#"{
            "session_id": "s1",
            "tool_use_id": "tu1",
            "hook_input": { "cost": { "total_cost_usd": 0.14 } }
        }"#;
        let event = parse_hook_payload(json, "ts").expect("should parse");
        assert_eq!(event.hook_reported_cost_nanos, Some(140_000_000));
        assert_eq!(event.cost_usd_nanos, 140_000_000);
    }

    /// Bare-number cost shape → hook_reported_cost_nanos is Some.
    #[test]
    fn parse_bare_number_cost_gives_hook_reported_cost_nanos() {
        let json = r#"{
            "session_id": "s1",
            "tool_use_id": "tu2",
            "hook_input": { "cost": 0.05 }
        }"#;
        let event = parse_hook_payload(json, "ts").expect("should parse");
        assert_eq!(event.hook_reported_cost_nanos, Some(50_000_000));
        assert_eq!(event.cost_usd_nanos, 50_000_000);
    }

    /// `{ "total_cost_usd": null }` → hook_reported_cost_nanos is None (not Some(0)).
    #[test]
    fn parse_cost_object_with_null_total_cost_usd_gives_none() {
        let json = r#"{
            "session_id": "s1",
            "tool_use_id": "tu_null",
            "hook_input": { "cost": { "total_cost_usd": null } }
        }"#;
        let event = parse_hook_payload(json, "ts").expect("should parse");
        assert_eq!(event.hook_reported_cost_nanos, None);
        assert_eq!(event.cost_usd_nanos, 0);
    }

    /// No cost field → hook_reported_cost_nanos is None.
    #[test]
    fn parse_absent_cost_gives_hook_reported_cost_nanos_none() {
        let json = r#"{"session_id": "s1", "tool_use_id": "tu3", "hook_input": {}}"#;
        let event = parse_hook_payload(json, "ts").expect("should parse");
        assert_eq!(event.hook_reported_cost_nanos, None);
        assert_eq!(event.cost_usd_nanos, 0);
    }
}
