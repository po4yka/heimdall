//! Amp (Sourcegraph) AI coding provider — reads session data from Amp thread files.
//!
//! # Backend
//!
//! JSON-backed. Amp writes one JSON file per thread in
//! `~/.local/share/amp/threads/` (or `$AMP_DATA_DIR/threads/` if the env var
//! is set). Each file is a complete thread object, not JSONL.
//!
//! # Default path
//!
//! `~/.local/share/amp/threads/` — all platforms (macOS, Linux, Windows).
//! Override by setting `AMP_DATA_DIR` environment variable.
//!
//! # Thread file schema
//!
//! ```json
//! {
//!   "id": "T-<uuid>",
//!   "title": "...",
//!   "created": <epoch_ms>,
//!   "messages": [
//!     { "role": "assistant", "messageId": 1,
//!       "usage": { "cacheCreationInputTokens": 500, "cacheReadInputTokens": 200, ... } }
//!   ],
//!   "usageLedger": {
//!     "events": [
//!       { "id": "event-1", "timestamp": "...", "model": "claude-...",
//!         "credits": 1.5, "tokens": { "input": 100, "output": 50 },
//!         "operationType": "inference",
//!         "fromMessageId": 0, "toMessageId": 1 }
//!     ]
//!   }
//! }
//! ```
//!
//! # Billing model
//!
//! Amp bills in abstract "credits", not USD.  `estimated_cost_nanos` is set to 0
//! and `credits` carries the per-event value.  The scanner aggregates credits
//! into the `sessions.total_credits` column.
//!
//! # Dedup strategy
//!
//! Each event has a unique `id` field.  The `message_id` stored in turns is
//! `amp:<thread_id>:<event_id>` which satisfies the `(provider, message_id)`
//! unique index in the turns table.
//!
//! # Session ID derivation
//!
//! `amp:<thread_id>` — e.g. `amp:T-abc123`.

use std::io::BufReader;
use std::path::{Path, PathBuf};

use anyhow::Result;
use tracing::warn;

use crate::models::Turn;
use crate::scanner::provider::{Provider, SessionSource};

/// Provider slug stored in `turns.provider`.
pub const PROVIDER_AMP: &str = "amp";

/// Environment variable name for custom Amp data directory.
const AMP_DATA_DIR_ENV: &str = "AMP_DATA_DIR";

// ---------------------------------------------------------------------------
// Provider struct
// ---------------------------------------------------------------------------

pub struct AmpProvider {
    pub dirs: Vec<PathBuf>,
}

impl AmpProvider {
    /// Construct with the platform-default Amp threads directory.
    pub fn new() -> Self {
        let dir = if let Ok(env_path) = std::env::var(AMP_DATA_DIR_ENV) {
            let p = PathBuf::from(env_path);
            p.join("threads")
        } else {
            let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
            home.join(".local")
                .join("share")
                .join("amp")
                .join("threads")
        };
        Self { dirs: vec![dir] }
    }

    /// Construct with explicit discovery directories (used in tests).
    #[cfg(test)]
    pub fn new_with_dirs(dirs: Vec<PathBuf>) -> Self {
        Self { dirs }
    }
}

impl Default for AmpProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for AmpProvider {
    fn name(&self) -> &'static str {
        PROVIDER_AMP
    }

    fn discover_sessions(&self) -> Result<Vec<SessionSource>> {
        let mut sources = Vec::new();
        for dir in &self.dirs {
            if !dir.exists() {
                continue;
            }
            for entry in walkdir::WalkDir::new(dir)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "json") {
                    sources.push(SessionSource {
                        path: path.to_path_buf(),
                    });
                }
            }
        }
        Ok(sources)
    }

    fn parse(&self, path: &Path) -> Result<Vec<Turn>> {
        Ok(parse_amp_thread_file(path))
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Parse an Amp thread JSON file into `Turn` records.
///
/// Each `usageLedger.events` entry becomes one Turn. Cache tokens are resolved
/// from the `messages` array by matching `toMessageId`. Malformed files are
/// silently skipped. Dedup is handled by the `(provider, message_id)` unique
/// index on insert; within a single file we deduplicate by event `id`.
pub(crate) fn parse_amp_thread_file(path: &Path) -> Vec<Turn> {
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) => {
            warn!("amp: cannot open {}: {}", path.display(), e);
            return Vec::new();
        }
    };

    // Thread files can be large; read the whole file at once.
    let mut reader = BufReader::new(file);
    let mut content = String::new();
    if let Err(e) = std::io::Read::read_to_string(&mut reader, &mut content) {
        warn!("amp: cannot read {}: {}", path.display(), e);
        return Vec::new();
    }

    let thread: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            warn!("amp: cannot parse {}: {}", path.display(), e);
            return Vec::new();
        }
    };

    let thread_id = match thread.get("id").and_then(|v| v.as_str()) {
        Some(id) if !id.is_empty() => id.to_string(),
        _ => {
            warn!("amp: missing thread id in {}", path.display());
            return Vec::new();
        }
    };

    let session_id = format!("amp:{thread_id}");
    let source_path = path.to_string_lossy().to_string();

    // Build a lookup: message_id (integer) → (cache_creation, cache_read)
    // from the assistant messages array.
    let messages = thread.get("messages").and_then(|v| v.as_array());
    let mut cache_map: std::collections::HashMap<i64, (i64, i64)> =
        std::collections::HashMap::new();
    if let Some(msgs) = messages {
        for msg in msgs {
            if msg.get("role").and_then(|v| v.as_str()) != Some("assistant") {
                continue;
            }
            let mid = match msg.get("messageId").and_then(|v| v.as_i64()) {
                Some(id) => id,
                None => continue,
            };
            if let Some(usage) = msg.get("usage") {
                let creation = usage
                    .get("cacheCreationInputTokens")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                let cache_read = usage
                    .get("cacheReadInputTokens")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                cache_map.insert(mid, (creation, cache_read));
            }
        }
    }

    let events = match thread
        .get("usageLedger")
        .and_then(|v| v.get("events"))
        .and_then(|v| v.as_array())
    {
        Some(arr) => arr,
        None => return Vec::new(),
    };

    let mut seen_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut turns: Vec<Turn> = Vec::new();

    for event in events {
        let event_id = match event.get("id").and_then(|v| v.as_str()) {
            Some(id) if !id.is_empty() => id.to_string(),
            _ => continue,
        };

        // Dedup within the file — in case the same event appears twice.
        if !seen_ids.insert(event_id.clone()) {
            continue;
        }

        let timestamp = event
            .get("timestamp")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let model = event
            .get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("amp")
            .to_string();

        let credits: Option<f64> = event.get("credits").and_then(|v| v.as_f64());

        let input_tokens = event
            .get("tokens")
            .and_then(|v| v.get("input"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let output_tokens = event
            .get("tokens")
            .and_then(|v| v.get("output"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        // Resolve cache tokens from messages via toMessageId.
        let to_msg_id = event.get("toMessageId").and_then(|v| v.as_i64());
        let (cache_creation_tokens, cache_read_tokens) = to_msg_id
            .and_then(|id| cache_map.get(&id).copied())
            .unwrap_or((0, 0));

        // Amp bills in credits, not USD — cost_nanos is 0 for Amp turns.
        // The `credits` field carries the billable amount.
        let message_id = format!("amp:{thread_id}:{event_id}");

        let turn = Turn {
            session_id: session_id.clone(),
            provider: PROVIDER_AMP.to_string(),
            timestamp,
            model,
            input_tokens,
            output_tokens,
            cache_read_tokens,
            cache_creation_tokens,
            reasoning_output_tokens: 0,
            estimated_cost_nanos: 0,
            tool_name: None,
            cwd: String::new(),
            message_id,
            service_tier: None,
            inference_geo: None,
            is_subagent: false,
            agent_id: None,
            source_path: source_path.clone(),
            version: None,
            pricing_version: String::new(),
            pricing_model: String::new(),
            // credits-only: mark billing mode explicitly so the dashboard can
            // suppress cost columns for Amp rows.
            billing_mode: "credits".to_string(),
            cost_confidence: "low".to_string(),
            category: String::new(),
            all_tools: Vec::new(),
            tool_use_ids: Vec::new(),
            tool_inputs: Vec::new(),
            credits,
        };

        turns.push(turn);
    }

    turns.sort_by(|a, b| {
        a.timestamp
            .cmp(&b.timestamp)
            .then_with(|| a.message_id.cmp(&b.message_id))
    });

    turns
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn write_thread_file(dir: &TempDir, name: &str, thread: &serde_json::Value) -> PathBuf {
        let path = dir.path().join(name);
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "{}", thread).unwrap();
        path
    }

    fn simple_thread(thread_id: &str, events: &[serde_json::Value]) -> serde_json::Value {
        serde_json::json!({
            "id": thread_id,
            "title": "Test Thread",
            "usageLedger": {
                "events": events
            }
        })
    }

    fn simple_event(event_id: &str, credits: f64, input: i64, output: i64) -> serde_json::Value {
        serde_json::json!({
            "id": event_id,
            "timestamp": "2026-04-18T09:00:00Z",
            "model": "claude-sonnet-4-6",
            "credits": credits,
            "tokens": { "input": input, "output": output },
            "operationType": "inference"
        })
    }

    // -----------------------------------------------------------------------
    // name()
    // -----------------------------------------------------------------------

    #[test]
    fn amp_provider_name() {
        assert_eq!(AmpProvider::new_with_dirs(vec![]).name(), "amp");
    }

    // -----------------------------------------------------------------------
    // parse: credits are extracted
    // -----------------------------------------------------------------------

    #[test]
    fn amp_parse_credits_populated() {
        let dir = TempDir::new().unwrap();
        let thread = simple_thread("T-abc", &[simple_event("ev-1", 12.5, 1000, 500)]);
        let path = write_thread_file(&dir, "T-abc.json", &thread);

        let provider = AmpProvider::new_with_dirs(vec![dir.path().to_path_buf()]);
        let turns = provider.parse(&path).unwrap();
        assert_eq!(turns.len(), 1);
        assert_eq!(turns[0].credits, Some(12.5));
        assert_eq!(turns[0].input_tokens, 1000);
        assert_eq!(turns[0].output_tokens, 500);
        assert_eq!(turns[0].estimated_cost_nanos, 0);
        assert_eq!(turns[0].provider, "amp");
    }

    // -----------------------------------------------------------------------
    // parse: session_id derived from thread id
    // -----------------------------------------------------------------------

    #[test]
    fn amp_parse_session_id_prefix() {
        let dir = TempDir::new().unwrap();
        let thread = simple_thread("T-xyz", &[simple_event("ev-1", 1.0, 100, 50)]);
        let path = write_thread_file(&dir, "T-xyz.json", &thread);

        let provider = AmpProvider::new_with_dirs(vec![dir.path().to_path_buf()]);
        let turns = provider.parse(&path).unwrap();
        assert_eq!(turns[0].session_id, "amp:T-xyz");
    }

    // -----------------------------------------------------------------------
    // parse: two events from same thread -> two turns
    // -----------------------------------------------------------------------

    #[test]
    fn amp_parse_two_events_two_turns() {
        let dir = TempDir::new().unwrap();
        let thread = simple_thread(
            "T-multi",
            &[
                simple_event("ev-1", 3.2, 200, 100),
                simple_event("ev-2", 7.8, 400, 200),
            ],
        );
        let path = write_thread_file(&dir, "T-multi.json", &thread);

        let provider = AmpProvider::new_with_dirs(vec![dir.path().to_path_buf()]);
        let turns = provider.parse(&path).unwrap();
        assert_eq!(turns.len(), 2);
        let total_credits: f64 = turns.iter().filter_map(|t| t.credits).sum();
        assert!((total_credits - 11.0).abs() < 1e-9);
    }

    // -----------------------------------------------------------------------
    // parse: duplicate event id -> deduped to 1 turn
    // -----------------------------------------------------------------------

    #[test]
    fn amp_parse_dedup_same_event_id() {
        let dir = TempDir::new().unwrap();
        let ev = simple_event("ev-dup", 5.0, 100, 50);
        let thread = simple_thread("T-dedup", &[ev.clone(), ev]);
        let path = write_thread_file(&dir, "T-dedup.json", &thread);

        let provider = AmpProvider::new_with_dirs(vec![dir.path().to_path_buf()]);
        let turns = provider.parse(&path).unwrap();
        assert_eq!(turns.len(), 1, "duplicate event ids must be deduplicated");
    }

    // -----------------------------------------------------------------------
    // parse: cache tokens resolved from messages
    // -----------------------------------------------------------------------

    #[test]
    fn amp_parse_cache_tokens_from_messages() {
        let dir = TempDir::new().unwrap();
        let thread = serde_json::json!({
            "id": "T-cache",
            "messages": [
                {
                    "role": "assistant",
                    "messageId": 1,
                    "usage": {
                        "cacheCreationInputTokens": 500,
                        "cacheReadInputTokens": 200
                    }
                }
            ],
            "usageLedger": {
                "events": [
                    {
                        "id": "ev-1",
                        "timestamp": "2026-04-18T09:00:00Z",
                        "model": "claude-sonnet-4-6",
                        "credits": 1.5,
                        "tokens": { "input": 100, "output": 50 },
                        "operationType": "inference",
                        "fromMessageId": 0,
                        "toMessageId": 1
                    }
                ]
            }
        });
        let path = write_thread_file(&dir, "T-cache.json", &thread);

        let provider = AmpProvider::new_with_dirs(vec![dir.path().to_path_buf()]);
        let turns = provider.parse(&path).unwrap();
        assert_eq!(turns.len(), 1);
        assert_eq!(turns[0].cache_creation_tokens, 500);
        assert_eq!(turns[0].cache_read_tokens, 200);
    }

    // -----------------------------------------------------------------------
    // parse: malformed JSON file is skipped gracefully
    // -----------------------------------------------------------------------

    #[test]
    fn amp_parse_malformed_json_skipped() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("bad.json");
        std::fs::write(&path, b"not valid json").unwrap();

        let provider = AmpProvider::new_with_dirs(vec![dir.path().to_path_buf()]);
        let turns = provider.parse(&path).unwrap();
        assert!(turns.is_empty());
    }

    // -----------------------------------------------------------------------
    // parse: thread with no usageLedger events -> empty turns
    // -----------------------------------------------------------------------

    #[test]
    fn amp_parse_no_events_returns_empty() {
        let dir = TempDir::new().unwrap();
        let thread = serde_json::json!({ "id": "T-empty", "title": "Empty" });
        let path = write_thread_file(&dir, "T-empty.json", &thread);

        let provider = AmpProvider::new_with_dirs(vec![dir.path().to_path_buf()]);
        let turns = provider.parse(&path).unwrap();
        assert!(turns.is_empty());
    }

    // -----------------------------------------------------------------------
    // discover_sessions
    // -----------------------------------------------------------------------

    #[test]
    fn amp_discover_sessions_finds_json_files() {
        let dir = TempDir::new().unwrap();
        let thread = simple_thread("T-disc", &[simple_event("ev-1", 1.0, 100, 50)]);
        let path = write_thread_file(&dir, "T-disc.json", &thread);

        let provider = AmpProvider::new_with_dirs(vec![dir.path().to_path_buf()]);
        let sources = provider.discover_sessions().unwrap();
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].path, path);
        assert_eq!(sources[0].path, path);
    }

    #[test]
    fn amp_discover_sessions_nonexistent_dir_returns_empty() {
        let provider =
            AmpProvider::new_with_dirs(vec![PathBuf::from("/nonexistent/amp/threads/xyz")]);
        let sources = provider.discover_sessions().unwrap();
        assert!(sources.is_empty());
    }

    // -----------------------------------------------------------------------
    // billing_mode is "credits" for Amp turns
    // -----------------------------------------------------------------------

    #[test]
    fn amp_parse_billing_mode_is_credits() {
        let dir = TempDir::new().unwrap();
        let thread = simple_thread("T-bm", &[simple_event("ev-1", 2.0, 100, 50)]);
        let path = write_thread_file(&dir, "T-bm.json", &thread);

        let provider = AmpProvider::new_with_dirs(vec![dir.path().to_path_buf()]);
        let turns = provider.parse(&path).unwrap();
        assert_eq!(turns[0].billing_mode, "credits");
    }
}
