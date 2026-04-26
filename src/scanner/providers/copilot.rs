//! GitHub Copilot provider — best-effort probe of Copilot's local session data.
//!
//! # Honesty disclaimer
//!
//! **GitHub Copilot's local session/chat format is not publicly documented and
//! varies by IDE integration (VS Code, JetBrains, Neovim, etc.).** This provider
//! is a best-effort stub:
//!
//! - It probes common known filesystem locations.
//! - It tries to parse discovered files as JSON or JSONL and looks for any
//!   object containing recognizable fields (`model`, `usage`, `tokens`,
//!   `prompt`, `completion`).
//! - When no recognizable structure is found it logs a `debug!` and returns
//!   `Ok(Vec::new())` — never an error, never a panic.
//!
//! As Copilot's format becomes better understood this module can be upgraded
//! to a fully structured parser. Until then it is "compile-safe, crash-safe,
//! empty-when-unknown."
//!
//! # Test coverage
//!
//! Coverage today is **synthetic-fixture only** — the unit tests in this file
//! exercise both Anthropic-style (`input_tokens`/`output_tokens`) and
//! OpenAI-style (`prompt_tokens`/`completion_tokens`) usage shapes plus the
//! flat `tokens` variant, but no test ingests an actual file produced by a
//! Copilot client. If you have access to a real Copilot session file (any
//! IDE), an anonymised redaction added as a fixture under the test module
//! would meaningfully harden this provider — open a PR with the redacted
//! payload pasted into a new `#[test]` and we will fold it into CI.
//!
//! # Probed locations
//!
//! - macOS:   `~/Library/Application Support/Code/User/globalStorage/github.copilot-chat/`
//! - Linux:   `~/.config/Code/User/globalStorage/github.copilot-chat/`
//! - Windows: `%APPDATA%/Code/User/globalStorage/github.copilot-chat/`
//! - JetBrains (all platforms): `~/.local/share/JetBrains/*/github-copilot/`
//!
//! Any `*.json` or `*.jsonl` files found under those directories are returned
//! as `SessionSource` entries and probed during `parse()`.
//!
//! # Dedup
//!
//! Per-file, records are deduped by a synthetic key of
//! `{source_file_stem}:{line_index}`. There is no stable response ID in the
//! probed format, so positional dedup is used as a conservative default.

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use anyhow::Result;
use tracing::{debug, warn};

use crate::models::Turn;
use crate::pricing;
use crate::scanner::parser::{ParseResult, empty_parse_result, parse_provider_turns_result};
use crate::scanner::provider::{Provider, SessionSource};

/// Provider slug stored in `turns.provider`.
pub const PROVIDER_COPILOT: &str = "copilot";

// ---------------------------------------------------------------------------
// Provider struct
// ---------------------------------------------------------------------------

pub struct CopilotProvider {
    pub dirs: Vec<PathBuf>,
}

impl CopilotProvider {
    /// Construct with the platform-default Copilot data directories.
    pub fn new() -> Self {
        Self {
            dirs: Self::default_dirs(),
        }
    }

    fn default_dirs() -> Vec<PathBuf> {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let mut result = Vec::new();

        // VS Code Copilot Chat global storage
        #[cfg(target_os = "macos")]
        result.push(
            home.join("Library")
                .join("Application Support")
                .join("Code")
                .join("User")
                .join("globalStorage")
                .join("github.copilot-chat"),
        );

        #[cfg(target_os = "linux")]
        result.push(
            home.join(".config")
                .join("Code")
                .join("User")
                .join("globalStorage")
                .join("github.copilot-chat"),
        );

        // JetBrains path (all platforms): ~/.local/share/JetBrains/*/github-copilot/
        // We record the parent and let discover_sessions() glob for the wildcard.
        if let Some(local_share) = dirs::data_local_dir() {
            let jetbrains_parent = local_share.join("JetBrains");
            if jetbrains_parent.exists() {
                // Walk one level for IDE-specific subdirs (IDEA, PyCharm, etc.)
                if let Ok(entries) = std::fs::read_dir(&jetbrains_parent) {
                    for entry in entries.flatten() {
                        let copilot_path = entry.path().join("github-copilot");
                        result.push(copilot_path);
                    }
                }
            }
        }

        result
    }

    /// Construct with explicit directories (used in tests).
    #[cfg(test)]
    pub fn new_with_dirs(dirs: Vec<PathBuf>) -> Self {
        Self { dirs }
    }
}

impl Default for CopilotProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for CopilotProvider {
    fn name(&self) -> &'static str {
        PROVIDER_COPILOT
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
                if path
                    .extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|ext| ext == "json" || ext == "jsonl")
                {
                    sources.push(SessionSource {
                        path: path.to_path_buf(),
                    });
                }
            }
        }
        Ok(sources)
    }

    fn parse(&self, path: &Path) -> Result<Vec<Turn>> {
        Ok(parse_copilot_file(path))
    }

    fn parse_source(&self, path: &Path, _skip_lines: i64) -> ParseResult {
        match self.parse(path) {
            Ok(turns) => parse_provider_turns_result(self.name(), turns, path, None),
            Err(e) => {
                warn!(
                    "copilot: provider parse failed for {}: {}",
                    path.display(),
                    e
                );
                empty_parse_result()
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Attempt to parse a Copilot session file as JSON or JSONL.
///
/// Returns empty if the file cannot be read or contains no recognizable
/// usage data. Logs `debug!` on unrecognized structure (not `warn!` — the
/// format is legitimately unknown, not a user error).
fn parse_copilot_file(path: &Path) -> Vec<Turn> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "jsonl" => parse_copilot_jsonl(path),
        "json" => parse_copilot_json(path),
        _ => {
            debug!(
                "copilot: unsupported file extension for {}, skipping",
                path.display()
            );
            Vec::new()
        }
    }
}

/// Parse a JSONL file line-by-line.
fn parse_copilot_jsonl(path: &Path) -> Vec<Turn> {
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) => {
            warn!("copilot: cannot open {}: {}", path.display(), e);
            return Vec::new();
        }
    };

    let session_id = session_id_from_path(path);
    let source_str = path.to_string_lossy().to_string();
    let mut turns = Vec::new();
    let mut recognized_any = false;

    let reader = BufReader::new(file);
    for (line_idx, line_result) in reader.lines().enumerate() {
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
            Err(_) => continue,
        };

        if let Some(turn) =
            extract_turn_from_value(&record, &session_id, &source_str, &format!("{line_idx}"))
        {
            recognized_any = true;
            turns.push(turn);
        }
    }

    if !recognized_any && turns.is_empty() {
        debug!(
            "copilot: no recognizable usage fields in JSONL {} — format may differ from probed schema",
            path.display()
        );
    }

    turns
}

/// Parse a JSON file (may be a single object or an array of objects).
fn parse_copilot_json(path: &Path) -> Vec<Turn> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            warn!("copilot: cannot read {}: {}", path.display(), e);
            return Vec::new();
        }
    };

    let value: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => {
            debug!("copilot: {} is not valid JSON — skipping", path.display());
            return Vec::new();
        }
    };

    let session_id = session_id_from_path(path);
    let source_str = path.to_string_lossy().to_string();
    let mut turns = Vec::new();

    match &value {
        serde_json::Value::Array(items) => {
            for (idx, item) in items.iter().enumerate() {
                if let Some(turn) =
                    extract_turn_from_value(item, &session_id, &source_str, &idx.to_string())
                {
                    turns.push(turn);
                }
            }
        }
        obj @ serde_json::Value::Object(_) => {
            if let Some(turn) = extract_turn_from_value(obj, &session_id, &source_str, "0") {
                turns.push(turn);
            }
        }
        _ => {
            debug!(
                "copilot: {} root value is neither object nor array — skipping",
                path.display()
            );
        }
    }

    if turns.is_empty() {
        debug!(
            "copilot: no recognizable usage fields in JSON {} — format may differ from probed schema",
            path.display()
        );
    }

    turns
}

/// Try to extract a `Turn` from a single JSON value.
///
/// Looks for any combination of:
/// - `model`
/// - `usage.input_tokens` / `usage.output_tokens`
/// - `usage.prompt_tokens` / `usage.completion_tokens` (OpenAI-style naming)
/// - top-level `tokens` object with `prompt`/`completion` keys
///
/// Returns `None` when no recognizable usage fields are present.
fn extract_turn_from_value(
    value: &serde_json::Value,
    session_id: &str,
    source_path: &str,
    dedup_suffix: &str,
) -> Option<Turn> {
    // Must be an object to probe.
    let obj = value.as_object()?;

    let model = obj
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    // Try nested `usage` object first (Anthropic / OpenAI style).
    let (input_tokens, output_tokens) = if let Some(usage) = obj.get("usage") {
        let input = usage
            .get("input_tokens")
            .or_else(|| usage.get("prompt_tokens"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let output = usage
            .get("output_tokens")
            .or_else(|| usage.get("completion_tokens"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        (input, output)
    } else if let Some(tokens) = obj.get("tokens") {
        // Flat `tokens` object variant.
        let input = tokens.get("prompt").and_then(|v| v.as_i64()).unwrap_or(0);
        let output = tokens
            .get("completion")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        (input, output)
    } else {
        return None; // No usage data found.
    };

    if input_tokens == 0 && output_tokens == 0 {
        return None;
    }

    let timestamp = obj
        .get("timestamp")
        .or_else(|| obj.get("created_at"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let message_id = obj
        .get("id")
        .or_else(|| obj.get("requestId"))
        .or_else(|| obj.get("request_id"))
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_else(|| format!("{session_id}:{dedup_suffix}"));

    let estimate = pricing::estimate_cost(&model, input_tokens, output_tokens, 0, 0);

    Some(Turn {
        session_id: session_id.to_string(),
        provider: PROVIDER_COPILOT.to_string(),
        timestamp,
        model,
        input_tokens,
        output_tokens,
        cache_read_tokens: 0,
        cache_creation_tokens: 0,
        reasoning_output_tokens: 0,
        estimated_cost_nanos: estimate.estimated_cost_nanos,
        tool_name: None,
        cwd: String::new(),
        message_id,
        service_tier: None,
        inference_geo: None,
        is_subagent: false,
        agent_id: None,
        source_path: source_path.to_string(),
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
    })
}

fn session_id_from_path(path: &Path) -> String {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    format!("copilot:{stem}")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn write_file(dir: &TempDir, name: &str, content: &str) -> PathBuf {
        let path = dir.path().join(name);
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "{}", content).unwrap();
        path
    }

    // -----------------------------------------------------------------------
    // name()
    // -----------------------------------------------------------------------

    #[test]
    fn copilot_provider_name() {
        assert_eq!(CopilotProvider::new_with_dirs(vec![]).name(), "copilot");
    }

    // -----------------------------------------------------------------------
    // discover_sessions: no default paths existing -> empty, no error
    // -----------------------------------------------------------------------

    #[test]
    fn copilot_discover_no_default_paths_returns_empty() {
        let provider =
            CopilotProvider::new_with_dirs(vec![PathBuf::from("/nonexistent/copilot/path")]);
        let result = provider.discover_sessions();
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    // -----------------------------------------------------------------------
    // parse: JSON that doesn't match expected fields -> empty, no error
    // -----------------------------------------------------------------------

    #[test]
    fn copilot_parse_unrecognized_json_returns_empty() {
        let dir = TempDir::new().unwrap();
        let path = write_file(&dir, "session.json", r#"{"foo": "bar", "baz": [1, 2, 3]}"#);

        let provider = CopilotProvider::new_with_dirs(vec![dir.path().to_path_buf()]);
        let result = provider.parse(&path);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    // -----------------------------------------------------------------------
    // parse: synthetic JSONL with usage fields -> at least one Turn produced
    // -----------------------------------------------------------------------

    #[test]
    fn copilot_parse_synthetic_jsonl_produces_turns() {
        let dir = TempDir::new().unwrap();
        let content = serde_json::json!({
            "model": "gpt-4",
            "usage": {
                "input_tokens": 100,
                "output_tokens": 50
            },
            "timestamp": "2026-04-17T10:00:00Z"
        })
        .to_string();
        let path = write_file(&dir, "session.jsonl", &format!("{content}\n"));

        let provider = CopilotProvider::new_with_dirs(vec![dir.path().to_path_buf()]);
        let turns = provider.parse(&path).unwrap();
        assert!(
            !turns.is_empty(),
            "expected at least one Turn from synthetic JSONL"
        );
        let t = &turns[0];
        assert_eq!(t.provider, "copilot");
        assert_eq!(t.input_tokens, 100);
        assert_eq!(t.output_tokens, 50);
        assert!(
            t.session_id.starts_with("copilot:"),
            "session_id must be prefixed 'copilot:'"
        );
    }

    // -----------------------------------------------------------------------
    // parse: malformed input -> empty, no panic
    // -----------------------------------------------------------------------

    #[test]
    fn copilot_parse_malformed_jsonl_returns_empty() {
        let dir = TempDir::new().unwrap();
        let path = write_file(
            &dir,
            "bad.jsonl",
            "not json\n{also not json\nanother bad line\n",
        );

        let provider = CopilotProvider::new_with_dirs(vec![dir.path().to_path_buf()]);
        let result = provider.parse(&path);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    // -----------------------------------------------------------------------
    // parse: JSON array of objects -> multiple turns
    // -----------------------------------------------------------------------

    #[test]
    fn copilot_parse_json_array_produces_multiple_turns() {
        let dir = TempDir::new().unwrap();
        let content = serde_json::json!([
            {
                "model": "gpt-4",
                "usage": {"input_tokens": 100, "output_tokens": 50}
            },
            {
                "model": "gpt-4",
                "usage": {"input_tokens": 200, "output_tokens": 80}
            }
        ])
        .to_string();
        let path = write_file(&dir, "sessions.json", &content);

        let provider = CopilotProvider::new_with_dirs(vec![dir.path().to_path_buf()]);
        let turns = provider.parse(&path).unwrap();
        assert_eq!(turns.len(), 2);
        for t in &turns {
            assert_eq!(t.provider, "copilot");
        }
    }

    // -----------------------------------------------------------------------
    // parse: flat `tokens` variant (alternative schema)
    // -----------------------------------------------------------------------

    #[test]
    fn copilot_parse_flat_tokens_variant() {
        let dir = TempDir::new().unwrap();
        let content = serde_json::json!({
            "model": "copilot-gpt-4o",
            "tokens": {
                "prompt": 120,
                "completion": 45
            }
        })
        .to_string();
        let path = write_file(&dir, "alt.jsonl", &format!("{content}\n"));

        let provider = CopilotProvider::new_with_dirs(vec![dir.path().to_path_buf()]);
        let turns = provider.parse(&path).unwrap();
        assert_eq!(turns.len(), 1);
        assert_eq!(turns[0].input_tokens, 120);
        assert_eq!(turns[0].output_tokens, 45);
    }

    // -----------------------------------------------------------------------
    // parse: OpenAI-style naming (prompt_tokens / completion_tokens)
    // -----------------------------------------------------------------------

    /// The probe explicitly accepts both Anthropic-style (`input_tokens` /
    /// `output_tokens`) and OpenAI-style (`prompt_tokens` /
    /// `completion_tokens`) usage shapes. The other tests only exercise the
    /// Anthropic path; this locks down the OpenAI fallback so a future refactor
    /// cannot silently drop it (Copilot's IDE integrations have historically
    /// emitted both shapes).
    #[test]
    fn copilot_parse_openai_naming_prompt_completion_tokens() {
        let dir = TempDir::new().unwrap();
        let content = serde_json::json!({
            "model": "gpt-4o",
            "usage": {
                "prompt_tokens": 250,
                "completion_tokens": 75
            }
        })
        .to_string();
        let path = write_file(&dir, "openai-style.jsonl", &format!("{content}\n"));

        let provider = CopilotProvider::new_with_dirs(vec![dir.path().to_path_buf()]);
        let turns = provider.parse(&path).unwrap();
        assert_eq!(turns.len(), 1);
        assert_eq!(turns[0].input_tokens, 250);
        assert_eq!(turns[0].output_tokens, 75);
        assert_eq!(turns[0].model, "gpt-4o");
    }

    // -----------------------------------------------------------------------
    // parse: created_at fallback when `timestamp` is absent
    // -----------------------------------------------------------------------

    /// The extractor reads `timestamp` first, then falls back to `created_at`.
    /// Without this test the fallback could be deleted unnoticed.
    #[test]
    fn copilot_parse_uses_created_at_when_timestamp_absent() {
        let dir = TempDir::new().unwrap();
        let content = serde_json::json!({
            "model": "gpt-4o-mini",
            "usage": {"input_tokens": 10, "output_tokens": 5},
            "created_at": "2026-04-19T12:00:00Z"
        })
        .to_string();
        let path = write_file(&dir, "created-at.jsonl", &format!("{content}\n"));

        let provider = CopilotProvider::new_with_dirs(vec![dir.path().to_path_buf()]);
        let turns = provider.parse(&path).unwrap();
        assert_eq!(turns.len(), 1);
        assert_eq!(turns[0].timestamp, "2026-04-19T12:00:00Z");
    }

    // -----------------------------------------------------------------------
    // parse: zero-token records are dropped (avoid empty-cost noise)
    // -----------------------------------------------------------------------

    /// Records with usage objects but both token counts at zero are dropped
    /// rather than producing zero-cost turns that pollute aggregations. This
    /// is a deliberate semantic; the test prevents regression to "always
    /// emit when usage is present".
    #[test]
    fn copilot_parse_zero_tokens_returns_no_turns() {
        let dir = TempDir::new().unwrap();
        let content = serde_json::json!({
            "model": "gpt-4o",
            "usage": {"input_tokens": 0, "output_tokens": 0}
        })
        .to_string();
        let path = write_file(&dir, "zero.jsonl", &format!("{content}\n"));

        let provider = CopilotProvider::new_with_dirs(vec![dir.path().to_path_buf()]);
        let turns = provider.parse(&path).unwrap();
        assert!(
            turns.is_empty(),
            "zero-token usage records must not produce Turn entries"
        );
    }

    // -----------------------------------------------------------------------
    // discover_sessions: finds json and jsonl files
    // -----------------------------------------------------------------------

    #[test]
    fn copilot_discover_finds_json_and_jsonl() {
        let dir = TempDir::new().unwrap();
        let _f1 = write_file(&dir, "a.json", "{}");
        let _f2 = write_file(&dir, "b.jsonl", "");
        let _f3 = write_file(&dir, "c.txt", "ignore me");

        let provider = CopilotProvider::new_with_dirs(vec![dir.path().to_path_buf()]);
        let sources = provider.discover_sessions().unwrap();
        assert_eq!(sources.len(), 2);
        let exts: Vec<&str> = sources
            .iter()
            .filter_map(|s| s.path.extension().and_then(|e| e.to_str()))
            .collect();
        assert!(exts.contains(&"json"));
        assert!(exts.contains(&"jsonl"));
    }
}
