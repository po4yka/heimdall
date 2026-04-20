//! Pi AI provider — reads session logs from Pi's JSONL session files.
//!
//! # Backend
//!
//! JSONL-backed. Pi writes one JSON record per line in session log files.
//!
//! # Default path
//!
//! `~/.pi/sessions/` (primary). Files are expected to be named with a
//! recognizable pattern such as `session-<uuid>.jsonl` or `<uuid>.jsonl`.
//!
//! # Record shape
//!
//! Each line is a JSON object carrying fields that may include:
//! ```json
//! {
//!   "responseId": "<uuid>",
//!   "model": "inflection-3",
//!   "usage": {
//!     "input_tokens": 100,
//!     "output_tokens": 50,
//!     "cached_input_tokens": 10
//!   }
//! }
//! ```
//!
//! # Dedup strategy
//!
//! Within each file, records are deduped by `responseId` — last record wins
//! when a `responseId` appears multiple times (streaming update pattern).
//! Lines without a `responseId` are silently skipped.
//!
//! # Session ID derivation
//!
//! The `session_id` is derived from the file's stem:
//! - `session-<uuid>.jsonl` → `pi:<uuid>`
//! - `<uuid>.jsonl` → `pi:<uuid>`
//! - Any other stem → `pi:<stem>`
//!
//! Source: Codeburn's `src/providers/pi.ts` reference implementation.

use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use anyhow::Result;
use tracing::warn;

use crate::models::Turn;
use crate::pricing;
use crate::scanner::provider::{Provider, SessionSource};

/// Provider slug stored in `turns.provider`.
pub const PROVIDER_PI: &str = "pi";

// ---------------------------------------------------------------------------
// Provider struct
// ---------------------------------------------------------------------------

pub struct PiProvider {
    pub dirs: Vec<PathBuf>,
}

impl PiProvider {
    /// Construct with the platform-default Pi session directories.
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self {
            dirs: vec![home.join(".pi").join("sessions")],
        }
    }

    /// Construct with explicit discovery directories (used in tests).
    #[cfg(test)]
    pub fn new_with_dirs(dirs: Vec<PathBuf>) -> Self {
        Self { dirs }
    }
}

impl Default for PiProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for PiProvider {
    fn name(&self) -> &'static str {
        PROVIDER_PI
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
                if path.extension().is_some_and(|ext| ext == "jsonl") {
                    sources.push(SessionSource {
                        path: path.to_path_buf(),
                    });
                }
            }
        }
        Ok(sources)
    }

    fn parse(&self, path: &Path) -> Result<Vec<Turn>> {
        Ok(parse_pi_jsonl_file(path))
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Derive `pi:<id>` session key from a file path.
///
/// Strips the common `session-` prefix so that `session-abc123.jsonl`
/// becomes `pi:abc123` rather than `pi:session-abc123`.
fn session_id_from_path(path: &Path) -> String {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    let id = stem.strip_prefix("session-").unwrap_or(stem);
    format!("pi:{id}")
}

/// Parse a Pi JSONL session file into `Turn` records.
///
/// Malformed lines are silently skipped. Dedup by `responseId` (last wins).
pub(crate) fn parse_pi_jsonl_file(path: &Path) -> Vec<Turn> {
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) => {
            warn!("pi: cannot open {}: {}", path.display(), e);
            return Vec::new();
        }
    };

    let session_id = session_id_from_path(path);
    let source_path = path.to_string_lossy().to_string();

    // response_id -> Turn (last-wins dedup)
    let mut seen: HashMap<String, Turn> = HashMap::new();

    let reader = BufReader::new(file);
    for line_result in reader.lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(_) => continue,
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let record: serde_json::Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(_) => continue, // skip malformed lines
        };

        let response_id = match record.get("responseId").and_then(|v| v.as_str()) {
            Some(id) if !id.is_empty() => id.to_string(),
            _ => continue, // skip records without a responseId
        };

        let model = record
            .get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let usage = record.get("usage");
        let input_tokens = usage
            .and_then(|u| u.get("input_tokens"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let output_tokens = usage
            .and_then(|u| u.get("output_tokens"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let cache_read_tokens = usage
            .and_then(|u| u.get("cached_input_tokens"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        // Skip records with zero tokens (e.g. metadata-only lines).
        if input_tokens == 0 && output_tokens == 0 && cache_read_tokens == 0 {
            continue;
        }

        let timestamp = record
            .get("timestamp")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let estimate =
            pricing::estimate_cost(&model, input_tokens, output_tokens, cache_read_tokens, 0);

        let turn = Turn {
            session_id: session_id.clone(),
            provider: PROVIDER_PI.to_string(),
            timestamp,
            model,
            input_tokens,
            output_tokens,
            cache_read_tokens,
            cache_creation_tokens: 0,
            reasoning_output_tokens: 0,
            estimated_cost_nanos: estimate.estimated_cost_nanos,
            tool_name: None,
            cwd: String::new(),
            message_id: response_id.clone(),
            service_tier: None,
            inference_geo: None,
            is_subagent: false,
            agent_id: None,
            source_path: source_path.clone(),
            version: None,
            pricing_version: estimate.pricing_version,
            pricing_model: estimate.pricing_model,
            billing_mode: "estimated_local".to_string(),
            cost_confidence: estimate.cost_confidence,
            category: String::new(),
            all_tools: Vec::new(),
            tool_use_ids: Vec::new(),
            tool_inputs: Vec::new(),
            credits: None,
        };

        // Last-wins: overwrite any previous record with the same responseId.
        seen.insert(response_id, turn);
    }

    let mut turns: Vec<Turn> = seen.into_values().collect();
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

    fn write_jsonl(dir: &TempDir, name: &str, lines: &[serde_json::Value]) -> PathBuf {
        let path = dir.path().join(name);
        let mut f = std::fs::File::create(&path).unwrap();
        for line in lines {
            writeln!(f, "{}", line).unwrap();
        }
        path
    }

    fn pi_record(response_id: &str, model: &str, input: i64, output: i64) -> serde_json::Value {
        serde_json::json!({
            "responseId": response_id,
            "model": model,
            "usage": {
                "input_tokens": input,
                "output_tokens": output,
                "cached_input_tokens": 0
            },
            "timestamp": "2026-04-17T10:00:00Z"
        })
    }

    // -----------------------------------------------------------------------
    // name()
    // -----------------------------------------------------------------------

    #[test]
    fn pi_provider_name() {
        assert_eq!(PiProvider::new_with_dirs(vec![]).name(), "pi");
    }

    // -----------------------------------------------------------------------
    // parse: same responseId twice -> 1 turn, last-wins tokens
    // -----------------------------------------------------------------------

    #[test]
    fn pi_parse_dedup_last_wins() {
        let dir = TempDir::new().unwrap();
        let path = write_jsonl(
            &dir,
            "session-abc123.jsonl",
            &[
                serde_json::json!({
                    "responseId": "resp-1",
                    "model": "inflection-3",
                    "usage": {"input_tokens": 50, "output_tokens": 20, "cached_input_tokens": 0},
                    "timestamp": "2026-04-17T10:00:00Z"
                }),
                serde_json::json!({
                    "responseId": "resp-1",
                    "model": "inflection-3",
                    "usage": {"input_tokens": 100, "output_tokens": 60, "cached_input_tokens": 5},
                    "timestamp": "2026-04-17T10:00:01Z"
                }),
            ],
        );

        let provider = PiProvider::new_with_dirs(vec![dir.path().to_path_buf()]);
        let turns = provider.parse(&path).unwrap();
        assert_eq!(turns.len(), 1, "same responseId must dedup to one Turn");
        assert_eq!(turns[0].input_tokens, 100, "last-wins: should use 100");
        assert_eq!(turns[0].output_tokens, 60, "last-wins: should use 60");
        assert_eq!(turns[0].cache_read_tokens, 5);
    }

    // -----------------------------------------------------------------------
    // parse: 3 distinct responseIds -> 3 turns, each with correct metadata
    // -----------------------------------------------------------------------

    #[test]
    fn pi_parse_three_distinct_response_ids() {
        let dir = TempDir::new().unwrap();
        let path = write_jsonl(
            &dir,
            "session-xyz789.jsonl",
            &[
                pi_record("resp-a", "inflection-3", 100, 50),
                pi_record("resp-b", "inflection-3", 200, 80),
                pi_record("resp-c", "inflection-3", 150, 60),
            ],
        );

        let provider = PiProvider::new_with_dirs(vec![dir.path().to_path_buf()]);
        let turns = provider.parse(&path).unwrap();
        assert_eq!(turns.len(), 3);
        for t in &turns {
            assert_eq!(t.provider, "pi");
            assert!(
                t.session_id.starts_with("pi:"),
                "session_id must be prefixed with 'pi:'"
            );
        }
    }

    // -----------------------------------------------------------------------
    // parse: malformed lines are skipped, no panic
    // -----------------------------------------------------------------------

    #[test]
    fn pi_parse_malformed_lines_skipped() {
        let dir = TempDir::new().unwrap();
        let path = {
            let p = dir.path().join("session-bad.jsonl");
            let mut f = std::fs::File::create(&p).unwrap();
            writeln!(f, "{{not valid json}}").unwrap();
            writeln!(f, "{}", pi_record("resp-ok", "inflection-3", 100, 50)).unwrap();
            writeln!(f, "also not json").unwrap();
            p
        };

        let provider = PiProvider::new_with_dirs(vec![dir.path().to_path_buf()]);
        let turns = provider.parse(&path).unwrap();
        // Only the valid record should produce a Turn.
        assert_eq!(turns.len(), 1);
        assert_eq!(turns[0].message_id, "resp-ok");
    }

    // -----------------------------------------------------------------------
    // session_id_from_path
    // -----------------------------------------------------------------------

    #[test]
    fn pi_session_id_strips_session_prefix() {
        let path = PathBuf::from("/home/user/.pi/sessions/session-uuid-1234.jsonl");
        assert_eq!(session_id_from_path(&path), "pi:uuid-1234");
    }

    #[test]
    fn pi_session_id_plain_stem() {
        let path = PathBuf::from("/home/user/.pi/sessions/uuid-5678.jsonl");
        assert_eq!(session_id_from_path(&path), "pi:uuid-5678");
    }

    // -----------------------------------------------------------------------
    // discover_sessions
    // -----------------------------------------------------------------------

    #[test]
    fn pi_discover_sessions_finds_jsonl() {
        let dir = TempDir::new().unwrap();
        let path = write_jsonl(&dir, "session-test.jsonl", &[pi_record("r1", "m1", 1, 1)]);

        let provider = PiProvider::new_with_dirs(vec![dir.path().to_path_buf()]);
        let sources = provider.discover_sessions().unwrap();
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].path, path);
        assert_eq!(sources[0].path, path);
    }

    #[test]
    fn pi_discover_sessions_nonexistent_dir_returns_empty() {
        let provider =
            PiProvider::new_with_dirs(vec![PathBuf::from("/nonexistent/pi/sessions/xyz")]);
        let sources = provider.discover_sessions().unwrap();
        assert!(sources.is_empty());
    }
}
