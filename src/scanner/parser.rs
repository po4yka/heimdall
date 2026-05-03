use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::Path;

use tracing::{debug, warn};

use crate::models::{Session, SessionMeta, Turn};
use crate::pricing;
use crate::scanner::cowork::{is_cowork_session_path, resolve_cowork_label};

pub const PROVIDER_CLAUDE: &str = "claude";
pub const PROVIDER_CODEX: &str = "codex";
pub const PROVIDER_XCODE: &str = "xcode";
/// Pi is JSONL-backed; its parser is called via `parse_jsonl_file`.
pub const PROVIDER_PI: &str = "pi";
/// OpenCode is SQLite-backed; it parses via its Provider trait `parse()` directly.
/// `parse_jsonl_file` is not used for opencode — this constant exists for
/// consistent naming across the codebase.
pub const PROVIDER_OPENCODE: &str = "opencode";
/// Copilot is mixed-format (JSON/JSONL best-effort probe); it parses via its
/// Provider trait `parse()` directly. `parse_jsonl_file` is not used for
/// copilot — this constant exists for consistent naming.
pub const PROVIDER_COPILOT: &str = "copilot";
/// Amp is JSON-backed (one file per thread); its parser is called via `parse_jsonl_file`.
pub use crate::scanner::providers::amp::PROVIDER_AMP;

/// Classify a tool name into (category, mcp_server, mcp_tool).
/// MCP tools follow the pattern: `mcp__<server>__<tool>`.
#[cfg_attr(not(test), allow(dead_code))]
pub fn classify_tool(name: &str) -> (&str, Option<&str>, Option<&str>) {
    if let Some(rest) = name.strip_prefix("mcp__") {
        if let Some(idx) = rest.find("__") {
            let server = &rest[..idx];
            let tool = &rest[idx + 2..];
            return ("mcp", Some(server), Some(tool));
        }
        return ("mcp", Some(rest), None);
    }
    ("builtin", None, None)
}

/// Extract human-readable text from a tool_result content block.
/// The `content` field may be a plain string or an array of typed blocks.
fn extract_tool_result_text(block: &serde_json::Value) -> String {
    match block.get("content") {
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(serde_json::Value::Array(items)) => items
            .iter()
            .filter_map(|item| {
                if item.get("type").and_then(|t| t.as_str()) == Some("text") {
                    item.get("text").and_then(|t| t.as_str())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}

/// Derive a friendly project name from cwd (last 2 path components).
pub fn project_name_from_cwd(cwd: &str) -> String {
    if cwd.is_empty() {
        return "unknown".into();
    }
    let normalized = cwd.replace('\\', "/");
    let trimmed = normalized.trim_end_matches('/');
    let parts: Vec<&str> = trimmed.split('/').collect();
    match parts.len() {
        0 => "unknown".into(),
        1 => parts[0].to_string(),
        _ => format!("{}/{}", parts[parts.len() - 2], parts[parts.len() - 1]),
    }
}

pub fn session_key(provider: &str, session_id: &str) -> String {
    format!("{provider}:{session_id}")
}

pub fn raw_session_id(session_key: &str) -> &str {
    session_key
        .split_once(':')
        .map(|(_, raw)| raw)
        .unwrap_or(session_key)
}

pub struct ParseResult {
    pub session_metas: Vec<SessionMeta>,
    pub turns: Vec<Turn>,
    pub progress_marker: i64,
    pub session_titles: HashMap<String, String>,
    pub tool_results: HashMap<String, bool>,
    /// Error message text per tool_use_id (populated only when is_error=true).
    pub tool_error_texts: HashMap<String, String>,
    /// Compact JSON of the full tool input object per tool_use_id, capped at 4 KB.
    pub tool_input_jsons: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, Default)]
struct ClaudeTokenUsage {
    input_tokens: i64,
    output_tokens: i64,
    cache_read_tokens: i64,
    cache_creation_tokens: i64,
    has_usage_fields: bool,
}

fn extract_claude_token_usage(
    message: &serde_json::Value,
    session_id: &str,
    message_id: &str,
) -> ClaudeTokenUsage {
    let usage = message
        .get("usage")
        .and_then(|value| value.as_object())
        .or_else(|| {
            message
                .get("usage_metadata")
                .and_then(|value| value.as_object())
        });

    let Some(usage) = usage else {
        debug!(
            "claude parser: assistant turn {} in session {} has no usage object; preserving zero-token turn",
            message_id, session_id
        );
        return ClaudeTokenUsage::default();
    };

    let input_tokens = usage
        .get("input_tokens")
        .or_else(|| usage.get("inputTokens"))
        .and_then(|value| value.as_i64())
        .unwrap_or(0);
    let output_tokens = usage
        .get("output_tokens")
        .or_else(|| usage.get("outputTokens"))
        .and_then(|value| value.as_i64())
        .unwrap_or(0);
    let cache_read_tokens = usage
        .get("cache_read_input_tokens")
        .or_else(|| usage.get("cached_input_tokens"))
        .or_else(|| usage.get("cacheReadInputTokens"))
        .or_else(|| usage.get("cachedInputTokens"))
        .and_then(|value| value.as_i64())
        .unwrap_or(0);
    let cache_creation_tokens = usage
        .get("cache_creation_input_tokens")
        .or_else(|| usage.get("cacheCreationInputTokens"))
        .and_then(|value| value.as_i64())
        .unwrap_or(0);

    let has_usage_fields = usage.contains_key("input_tokens")
        || usage.contains_key("inputTokens")
        || usage.contains_key("output_tokens")
        || usage.contains_key("outputTokens")
        || usage.contains_key("cache_read_input_tokens")
        || usage.contains_key("cached_input_tokens")
        || usage.contains_key("cacheReadInputTokens")
        || usage.contains_key("cachedInputTokens")
        || usage.contains_key("cache_creation_input_tokens")
        || usage.contains_key("cacheCreationInputTokens");

    if !has_usage_fields {
        debug!(
            "claude parser: assistant turn {} in session {} has an unrecognized usage shape; preserving zero-token turn",
            message_id, session_id
        );
    }

    ClaudeTokenUsage {
        input_tokens,
        output_tokens,
        cache_read_tokens,
        cache_creation_tokens,
        has_usage_fields,
    }
}

/// Parse a provider-specific log file.
///
/// # Dispatcher routing
///
/// - `codex` — dedicated Codex JSONL parser (own record schema).
/// - `xcode` — Claude JSONL parser with provider retag to "xcode".
/// - `pi` — Pi JSONL parser (`responseId`-keyed dedup).
/// - `opencode` — **SQLite-backed**; this arm is never reached in normal
///   operation. OpenCode sessions are parsed via `OpenCodeProvider::parse()`
///   directly. If somehow called here, returns empty and logs a warning.
/// - `copilot` — **Mixed-format / best-effort probe**; this arm is never
///   reached in normal operation. Copilot sessions are parsed via
///   `CopilotProvider::parse()` directly. Returns empty with a warning if
///   called here.
/// - `_` — Falls through to the Claude JSONL parser (default format).
pub fn parse_jsonl_file(provider: &str, filepath: &Path, skip_lines: i64) -> ParseResult {
    match provider {
        PROVIDER_CODEX => {
            crate::scanner::providers::codex::parse_codex_jsonl_file(filepath, skip_lines)
        }
        PROVIDER_XCODE => retag_claude_result(parse_claude_jsonl_file(filepath, skip_lines)),
        PROVIDER_PI => parse_pi_result(
            crate::scanner::providers::pi::parse_pi_jsonl_file(filepath),
            filepath,
        ),
        PROVIDER_AMP => parse_amp_result(
            crate::scanner::providers::amp::parse_amp_thread_file(filepath),
            filepath,
        ),
        PROVIDER_OPENCODE | PROVIDER_COPILOT => {
            // These providers are SQLite-backed or mixed-format and must be
            // parsed via their Provider trait `parse()`. The JSONL dispatcher
            // is not the right path for them. Return empty gracefully.
            warn!(
                "parse_jsonl_file called for SQLite/mixed provider '{}' on {} — use Provider::parse() instead",
                provider,
                filepath.display()
            );
            empty_parse_result()
        }
        _ => parse_claude_jsonl_file(filepath, skip_lines),
    }
}

pub(crate) fn parse_provider_turns_result(
    provider: &str,
    turns: Vec<Turn>,
    filepath: &Path,
    progress_marker: Option<i64>,
) -> ParseResult {
    ParseResult {
        session_metas: session_metas_from_turns(provider, &turns),
        turns,
        progress_marker: progress_marker.unwrap_or(progress_marker_fallback(filepath)),
        session_titles: HashMap::new(),
        tool_results: HashMap::new(),
        tool_error_texts: HashMap::new(),
        tool_input_jsons: HashMap::new(),
    }
}

fn progress_marker_fallback(filepath: &Path) -> i64 {
    std::fs::metadata(filepath)
        .ok()
        .map(|m| m.len() as i64)
        .unwrap_or(0)
}

fn session_metas_from_turns(provider: &str, turns: &[Turn]) -> Vec<SessionMeta> {
    let mut metas: HashMap<String, SessionMeta> = HashMap::new();
    for t in turns {
        metas
            .entry(t.session_id.clone())
            .and_modify(|m| {
                if !t.timestamp.is_empty() {
                    if m.first_timestamp.is_empty() || t.timestamp < m.first_timestamp {
                        m.first_timestamp.clone_from(&t.timestamp);
                    }
                    if m.last_timestamp.is_empty() || t.timestamp > m.last_timestamp {
                        m.last_timestamp.clone_from(&t.timestamp);
                    }
                }
                if m.model.is_none() && !t.model.is_empty() {
                    m.model = Some(t.model.clone());
                }
            })
            .or_insert_with(|| SessionMeta {
                session_id: t.session_id.clone(),
                provider: provider.into(),
                project_name: project_name_from_cwd(&t.cwd),
                project_slug: String::new(),
                first_timestamp: t.timestamp.clone(),
                last_timestamp: t.timestamp.clone(),
                git_branch: String::new(),
                model: if t.model.is_empty() {
                    None
                } else {
                    Some(t.model.clone())
                },
                entrypoint: String::new(),
            });
    }
    metas.into_values().collect()
}

/// Wrap a `Vec<Turn>` from the Pi provider into a `ParseResult`.
fn parse_pi_result(turns: Vec<crate::models::Turn>, filepath: &Path) -> ParseResult {
    let progress_marker = std::fs::File::open(filepath)
        .map(|f| {
            use std::io::BufRead;
            std::io::BufReader::new(f).lines().count() as i64
        })
        .unwrap_or(0);
    parse_provider_turns_result(PROVIDER_PI, turns, filepath, Some(progress_marker))
}

/// Wrap a `Vec<Turn>` from the Amp provider into a `ParseResult`.
///
/// Each Amp thread file (`.json`) maps to one session.  Session metas are
/// built from the turns in the same way as Pi.
fn parse_amp_result(turns: Vec<crate::models::Turn>, _filepath: &Path) -> ParseResult {
    // Use turns.len() as the incremental-scan guard: if events are added to the
    // thread file, the count grows and triggers reprocessing on the next scan.
    let progress_marker = turns.len() as i64;
    parse_provider_turns_result(PROVIDER_AMP, turns, Path::new(""), Some(progress_marker))
}

/// Rewrite a Claude-parsed result so it carries the Xcode provider tag on
/// both turns and session metadata. Xcode's CodingAssistant writes the same
/// JSONL format but must be attributed separately in the DB.
fn retag_claude_result(mut result: ParseResult) -> ParseResult {
    let claude_prefix = format!("{}:", PROVIDER_CLAUDE);
    let xcode_prefix = format!("{}:", PROVIDER_XCODE);
    for t in &mut result.turns {
        if let Some(raw) = t.session_id.strip_prefix(&claude_prefix) {
            t.session_id = format!("{}{}", xcode_prefix, raw);
        }
        t.provider = PROVIDER_XCODE.into();
    }
    for m in &mut result.session_metas {
        if let Some(raw) = m.session_id.strip_prefix(&claude_prefix) {
            m.session_id = format!("{}{}", xcode_prefix, raw);
        }
        m.provider = PROVIDER_XCODE.into();
    }
    result
}

pub(crate) fn parse_claude_jsonl_file(filepath: &Path, skip_lines: i64) -> ParseResult {
    let mut seen_messages: HashMap<String, Turn> = HashMap::new();
    let mut turns_no_id: Vec<Turn> = Vec::new();
    let mut session_meta: HashMap<String, SessionMeta> = HashMap::new();
    let mut session_titles: HashMap<String, String> = HashMap::new();
    let mut tool_results: HashMap<String, bool> = HashMap::new();
    let mut tool_error_texts: HashMap<String, String> = HashMap::new();
    let mut tool_input_jsons: HashMap<String, String> = HashMap::new();
    let mut progress_marker: i64 = 0;
    let source_path = filepath.to_string_lossy().to_string();

    let file = match std::fs::File::open(filepath) {
        Ok(f) => f,
        Err(e) => {
            warn!("Error opening {}: {}", filepath.display(), e);
            return empty_parse_result();
        }
    };

    let reader = BufReader::new(file);
    for line_result in reader.lines() {
        progress_marker += 1;
        if progress_marker <= skip_lines {
            continue;
        }

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

        let rtype = record.get("type").and_then(|v| v.as_str()).unwrap_or("");

        if rtype == "custom-title" {
            if let (Some(raw_sid), Some(title)) = (
                record.get("sessionId").and_then(|v| v.as_str()),
                record.get("customTitle").and_then(|v| v.as_str()),
            ) {
                session_titles.insert(session_key(PROVIDER_CLAUDE, raw_sid), title.to_string());
            }
            continue;
        }

        if rtype != "assistant" && rtype != "user" {
            continue;
        }

        let raw_session_id = match record.get("sessionId").and_then(|v| v.as_str()) {
            Some(s) if !s.is_empty() => s.to_string(),
            _ => continue,
        };
        let session_id = session_key(PROVIDER_CLAUDE, &raw_session_id);

        let version = record
            .get("version")
            .and_then(|v| v.as_str())
            .map(String::from);

        let timestamp = record
            .get("timestamp")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let cwd = record
            .get("cwd")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let git_branch = record
            .get("gitBranch")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let entrypoint = record
            .get("entrypoint")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let slug = record
            .get("slug")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let is_subagent = record
            .get("isSidechain")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let agent_id = record
            .get("agentId")
            .and_then(|v| v.as_str())
            .map(String::from);

        session_meta
            .entry(session_id.clone())
            .and_modify(|meta| {
                if !timestamp.is_empty() {
                    if meta.first_timestamp.is_empty() || timestamp < meta.first_timestamp {
                        meta.first_timestamp = timestamp.clone();
                    }
                    if meta.last_timestamp.is_empty() || timestamp > meta.last_timestamp {
                        meta.last_timestamp = timestamp.clone();
                    }
                }
                if !git_branch.is_empty() && meta.git_branch.is_empty() {
                    meta.git_branch.clone_from(&git_branch);
                }
            })
            .or_insert_with(|| SessionMeta {
                session_id: session_id.clone(),
                provider: PROVIDER_CLAUDE.into(),
                project_name: project_name_from_cwd(&cwd),
                project_slug: slug.clone(),
                first_timestamp: timestamp.clone(),
                last_timestamp: timestamp.clone(),
                git_branch: git_branch.clone(),
                model: None,
                entrypoint: entrypoint.clone(),
            });

        if rtype == "assistant" {
            let msg = match record.get("message") {
                Some(m) => m,
                None => continue,
            };
            let model = msg
                .get("model")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let message_id = msg
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let usage = extract_claude_token_usage(msg, &session_id, &message_id);

            let content_arr = msg.get("content").and_then(|c| c.as_array());

            let tool_name = content_arr
                .and_then(|arr| {
                    arr.iter()
                        .find(|item| item.get("type").and_then(|t| t.as_str()) == Some("tool_use"))
                })
                .and_then(|item| item.get("name").and_then(|n| n.as_str()))
                .map(String::from);

            let all_tools: Vec<String> = content_arr
                .map(|arr| {
                    arr.iter()
                        .filter(|item| {
                            item.get("type").and_then(|t| t.as_str()) == Some("tool_use")
                        })
                        .filter_map(|item| {
                            item.get("name").and_then(|n| n.as_str()).map(String::from)
                        })
                        .collect()
                })
                .unwrap_or_default();

            let tool_use_ids: Vec<(String, String)> = content_arr
                .map(|arr| {
                    arr.iter()
                        .filter(|item| {
                            item.get("type").and_then(|t| t.as_str()) == Some("tool_use")
                        })
                        .filter_map(|item| {
                            let id = item.get("id").and_then(|v| v.as_str())?.to_string();
                            let name = item.get("name").and_then(|v| v.as_str())?.to_string();
                            Some((id, name))
                        })
                        .collect()
                })
                .unwrap_or_default();

            // Extract the single most-relevant argument per tool invocation.
            // File tools: `file_path` argument.
            // Bash: first 120 chars of `command` (truncated with `…`).
            // Everything else: empty string (caller falls back to tool name).
            let tool_inputs: Vec<(String, String)> = content_arr
                .map(|arr| {
                    arr.iter()
                        .filter(|item| {
                            item.get("type").and_then(|t| t.as_str()) == Some("tool_use")
                        })
                        .filter_map(|item| {
                            let id = item.get("id").and_then(|v| v.as_str())?.to_string();
                            let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("");
                            let input = item.get("input");
                            let arg = match name {
                                "Edit" | "Write" | "MultiEdit" | "NotebookEdit" | "Read" => input
                                    .and_then(|inp| inp.get("file_path"))
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                                "Bash" => {
                                    let cmd = input
                                        .and_then(|inp| inp.get("command"))
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("");
                                    // Truncate to 120 bytes at a valid char boundary; append
                                    // ellipsis if truncated. Raw byte slicing would panic on
                                    // multi-byte UTF-8 characters that straddle the boundary.
                                    const MAX_CMD: usize = 120;
                                    if cmd.len() > MAX_CMD {
                                        let mut end = MAX_CMD;
                                        while end > 0 && !cmd.is_char_boundary(end) {
                                            end -= 1;
                                        }
                                        let truncated = &cmd[..end];
                                        format!("{truncated}\u{2026}")
                                    } else {
                                        cmd.to_string()
                                    }
                                }
                                _ => String::new(),
                            };
                            Some((id, arg))
                        })
                        .collect()
                })
                .unwrap_or_default();

            // Capture full compact input JSON per tool_use_id for the error detail view.
            // content_arr is Option<&Vec<_>> (Copy), so re-use after tool_inputs above.
            if let Some(arr) = content_arr {
                const MAX_INPUT: usize = 4096;
                for item in arr {
                    if item.get("type").and_then(|t| t.as_str()) != Some("tool_use") {
                        continue;
                    }
                    let Some(id) = item.get("id").and_then(|v| v.as_str()) else {
                        continue;
                    };
                    if let Some(input) = item.get("input") {
                        let s = serde_json::to_string(input).unwrap_or_default();
                        let truncated = if s.len() > MAX_INPUT {
                            let mut end = MAX_INPUT;
                            while end > 0 && !s.is_char_boundary(end) {
                                end -= 1;
                            }
                            s[..end].to_string()
                        } else {
                            s
                        };
                        tool_input_jsons.insert(id.to_string(), truncated);
                    }
                }
            }

            let service_tier = msg
                .get("usage")
                .or_else(|| msg.get("usage_metadata"))
                .and_then(|usage| usage.get("service_tier"))
                .and_then(|v| v.as_str())
                .map(String::from);
            let inference_geo = msg
                .get("usage")
                .or_else(|| msg.get("usage_metadata"))
                .and_then(|usage| usage.get("inference_geo"))
                .and_then(|v| v.as_str())
                .map(String::from);

            if !model.is_empty()
                && let Some(meta) = session_meta.get_mut(&session_id)
            {
                meta.model = Some(model.clone());
            }

            let category =
                crate::scanner::classifier::classify(tool_name.as_deref(), &all_tools, None)
                    .as_str()
                    .to_string();
            let turn = Turn {
                estimated_cost_nanos: 0,
                session_id: session_id.clone(),
                provider: PROVIDER_CLAUDE.into(),
                timestamp: timestamp.clone(),
                model,
                input_tokens: usage.input_tokens,
                output_tokens: usage.output_tokens,
                cache_read_tokens: usage.cache_read_tokens,
                cache_creation_tokens: usage.cache_creation_tokens,
                reasoning_output_tokens: 0,
                tool_name,
                cwd,
                message_id: message_id.clone(),
                service_tier,
                inference_geo,
                is_subagent,
                agent_id: agent_id.clone(),
                source_path: source_path.clone(),
                version: version.clone(),
                pricing_version: String::new(),
                pricing_model: String::new(),
                billing_mode: "estimated_local".into(),
                cost_confidence: String::new(),
                category,
                all_tools,
                tool_use_ids,
                tool_inputs,
                credits: None,
            };
            let estimate = pricing::estimate_cost(
                &turn.model,
                turn.input_tokens,
                turn.output_tokens,
                turn.cache_read_tokens,
                turn.cache_creation_tokens,
            );
            let mut turn = turn;
            turn.estimated_cost_nanos = estimate.estimated_cost_nanos;
            turn.pricing_version = estimate.pricing_version;
            turn.pricing_model = estimate.pricing_model;
            turn.cost_confidence = estimate.cost_confidence;

            if !message_id.is_empty() {
                seen_messages.insert(message_id, turn);
            } else {
                if !usage.has_usage_fields {
                    debug!(
                        "claude parser: assistant turn without message id in session {} preserved with zero-token usage fallback",
                        session_id
                    );
                }
                turns_no_id.push(turn);
            }
        }

        if rtype == "user"
            && let Some(content) = record
                .get("message")
                .and_then(|m| m.get("content"))
                .and_then(|c| c.as_array())
        {
            for block in content {
                if block.get("type").and_then(|t| t.as_str()) == Some("tool_result")
                    && let Some(tool_use_id) = block.get("tool_use_id").and_then(|v| v.as_str())
                {
                    let is_error = block
                        .get("is_error")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    tool_results.insert(tool_use_id.to_string(), is_error);
                    if is_error {
                        let text = extract_tool_result_text(block);
                        if !text.is_empty() {
                            const MAX_ERROR: usize = 4096;
                            let truncated = if text.len() > MAX_ERROR {
                                let mut end = MAX_ERROR;
                                while end > 0 && !text.is_char_boundary(end) {
                                    end -= 1;
                                }
                                text[..end].to_string()
                            } else {
                                text
                            };
                            tool_error_texts.insert(tool_use_id.to_string(), truncated);
                        }
                    }
                }
            }
        }
    }

    let mut turns = turns_no_id;
    turns.extend(seen_messages.into_values());
    turns.sort_by(|a, b| {
        a.session_id
            .cmp(&b.session_id)
            .then_with(|| a.timestamp.cmp(&b.timestamp))
            .then_with(|| a.message_id.cmp(&b.message_id))
            .then_with(|| a.model.cmp(&b.model))
    });

    // Cowork label resolution: if this session JSONL lives under
    // `local-agent-mode-sessions/<slug>/`, extract the first user message from
    // `audit.jsonl` in the slug directory and override `project_name`.
    let cowork_label: Option<String> =
        is_cowork_session_path(filepath).and_then(|slug_dir| resolve_cowork_label(&slug_dir));

    let mut metas: Vec<SessionMeta> = session_meta.into_values().collect();
    if let Some(ref label) = cowork_label {
        for meta in &mut metas {
            meta.project_name.clone_from(label);
        }
    }

    ParseResult {
        session_metas: metas,
        turns,
        progress_marker,
        session_titles,
        tool_results,
        tool_error_texts,
        tool_input_jsons,
    }
}

pub(crate) fn empty_parse_result() -> ParseResult {
    ParseResult {
        session_metas: vec![],
        turns: vec![],
        progress_marker: 0,
        session_titles: HashMap::new(),
        tool_results: HashMap::new(),
        tool_error_texts: HashMap::new(),
        tool_input_jsons: HashMap::new(),
    }
}

pub(crate) fn file_timestamp_rfc3339(filepath: &Path) -> String {
    std::fs::metadata(filepath)
        .ok()
        .and_then(|meta| meta.modified().ok())
        .map(chrono::DateTime::<chrono::Utc>::from)
        .map(|ts| ts.to_rfc3339())
        .unwrap_or_default()
}

pub(crate) fn upsert_session_meta(
    metas: &mut HashMap<String, SessionMeta>,
    session_id: &str,
    meta: SessionMeta,
) {
    metas
        .entry(session_id.to_string())
        .and_modify(|existing| {
            merge_session_meta(
                existing,
                &meta.first_timestamp,
                &meta.project_name,
                &meta.git_branch,
                meta.model.as_deref(),
                &meta.entrypoint,
            );
            if existing.project_slug.is_empty() {
                existing.project_slug = meta.project_slug.clone();
            }
        })
        .or_insert(meta);
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn touch_session_meta(
    metas: &mut HashMap<String, SessionMeta>,
    session_id: &str,
    provider: &str,
    timestamp: &str,
    cwd: &str,
    git_branch: &str,
    model: Option<&str>,
    entrypoint: &str,
) {
    let project_name = project_name_from_cwd(cwd);
    metas
        .entry(session_id.to_string())
        .and_modify(|meta| {
            merge_session_meta(
                meta,
                timestamp,
                &project_name,
                git_branch,
                model,
                entrypoint,
            )
        })
        .or_insert_with(|| SessionMeta {
            session_id: session_id.to_string(),
            provider: provider.into(),
            project_name,
            project_slug: String::new(),
            first_timestamp: timestamp.to_string(),
            last_timestamp: timestamp.to_string(),
            git_branch: git_branch.to_string(),
            model: model.map(String::from),
            entrypoint: entrypoint.to_string(),
        });
}

pub(crate) fn merge_session_meta(
    meta: &mut SessionMeta,
    timestamp: &str,
    project_name: &str,
    git_branch: &str,
    model: Option<&str>,
    entrypoint: &str,
) {
    if !timestamp.is_empty() {
        if meta.first_timestamp.is_empty() || timestamp < meta.first_timestamp.as_str() {
            meta.first_timestamp = timestamp.to_string();
        }
        if meta.last_timestamp.is_empty() || timestamp > meta.last_timestamp.as_str() {
            meta.last_timestamp = timestamp.to_string();
        }
    }
    if meta.project_name == "unknown" && project_name != "unknown" {
        meta.project_name = project_name.to_string();
    }
    if meta.git_branch.is_empty() && !git_branch.is_empty() {
        meta.git_branch = git_branch.to_string();
    }
    if meta.entrypoint.is_empty() && !entrypoint.is_empty() {
        meta.entrypoint = entrypoint.to_string();
    }
    if let Some(model) = model
        && !model.is_empty()
    {
        meta.model = Some(model.to_string());
    }
}

/// Aggregate turn data into session-level stats.
/// Per-session aggregated token/cost/model counters built by `group_turns_by_session`.
struct SessionStats {
    total_input: i64,
    total_output: i64,
    total_cache_read: i64,
    total_cache_creation: i64,
    total_reasoning_output: i64,
    total_estimated_cost_nanos: i64,
    turn_count: i64,
    model: Option<String>,
    pricing_version: String,
    billing_mode: String,
    cost_confidence: String,
    /// Accumulated credits (Amp only); `None` when no turn in the session has credits.
    total_credits: Option<f64>,
    /// Turns belonging to this session in chronological order,
    /// used for one-shot classification after all turns are collected.
    session_turns: Vec<Turn>,
}

/// Phase A: group raw turns into per-session aggregates keyed by session_id.
fn group_turns_by_session(turns: &[Turn]) -> HashMap<&str, SessionStats> {
    let mut stats_map: HashMap<&str, SessionStats> = HashMap::new();
    for t in turns {
        let entry = stats_map.entry(&t.session_id).or_insert(SessionStats {
            total_input: 0,
            total_output: 0,
            total_cache_read: 0,
            total_cache_creation: 0,
            total_reasoning_output: 0,
            total_estimated_cost_nanos: 0,
            turn_count: 0,
            model: None,
            pricing_version: String::new(),
            billing_mode: String::new(),
            cost_confidence: String::new(),
            total_credits: None,
            session_turns: Vec::new(),
        });
        entry.total_input += t.input_tokens;
        entry.total_output += t.output_tokens;
        entry.total_cache_read += t.cache_read_tokens;
        entry.total_cache_creation += t.cache_creation_tokens;
        entry.total_reasoning_output += t.reasoning_output_tokens;
        entry.total_estimated_cost_nanos += t.estimated_cost_nanos;
        entry.turn_count += 1;
        if !t.model.is_empty() {
            entry.model = Some(t.model.clone());
        }
        entry.pricing_version = merge_pricing_version(&entry.pricing_version, &t.pricing_version);
        entry.billing_mode = merge_billing_mode(&entry.billing_mode, &t.billing_mode);
        entry.cost_confidence =
            merge_cost_confidence(&entry.cost_confidence, &t.cost_confidence).to_string();
        if let Some(c) = t.credits {
            *entry.total_credits.get_or_insert(0.0) += c;
        }
        entry.session_turns.push(t.clone());
    }
    stats_map
}

pub fn aggregate_sessions(metas: &[SessionMeta], turns: &[Turn]) -> Vec<Session> {
    let stats_map = group_turns_by_session(turns);

    let empty_stats = SessionStats {
        total_input: 0,
        total_output: 0,
        total_cache_read: 0,
        total_cache_creation: 0,
        total_reasoning_output: 0,
        total_estimated_cost_nanos: 0,
        turn_count: 0,
        model: None,
        pricing_version: String::new(),
        billing_mode: "estimated_local".into(),
        cost_confidence: pricing::COST_CONFIDENCE_LOW.into(),
        total_credits: None,
        session_turns: Vec::new(),
    };

    metas
        .iter()
        .map(|meta| {
            let s = stats_map
                .get(meta.session_id.as_str())
                .unwrap_or(&empty_stats);
            let one_shot = crate::scanner::oneshot::classify_one_shot(s.session_turns.as_slice());
            Session {
                session_id: meta.session_id.clone(),
                provider: meta.provider.clone(),
                project_name: meta.project_name.clone(),
                project_slug: meta.project_slug.clone(),
                first_timestamp: meta.first_timestamp.clone(),
                last_timestamp: meta.last_timestamp.clone(),
                git_branch: meta.git_branch.clone(),
                model: s.model.clone().or_else(|| meta.model.clone()),
                entrypoint: meta.entrypoint.clone(),
                total_input_tokens: s.total_input,
                total_output_tokens: s.total_output,
                total_cache_read: s.total_cache_read,
                total_cache_creation: s.total_cache_creation,
                total_reasoning_output: s.total_reasoning_output,
                total_estimated_cost_nanos: s.total_estimated_cost_nanos,
                turn_count: s.turn_count,
                pricing_version: s.pricing_version.clone(),
                billing_mode: s.billing_mode.clone(),
                cost_confidence: s.cost_confidence.clone(),
                title: None,
                one_shot,
                total_credits: s.total_credits,
            }
        })
        .collect()
}

fn merge_pricing_version(current: &str, next: &str) -> String {
    if current.is_empty() {
        return next.to_string();
    }
    if next.is_empty() || current == next {
        return current.to_string();
    }
    "mixed".into()
}

fn merge_billing_mode(current: &str, next: &str) -> String {
    if current.is_empty() {
        return next.to_string();
    }
    if next.is_empty() || current == next {
        return current.to_string();
    }
    "mixed".into()
}

fn merge_cost_confidence<'a>(current: &'a str, next: &'a str) -> &'a str {
    if current.is_empty() {
        return next;
    }
    if next.is_empty() {
        return current;
    }
    let rank = |value: &str| match value {
        pricing::COST_CONFIDENCE_LOW => 0,
        pricing::COST_CONFIDENCE_MEDIUM => 1,
        pricing::COST_CONFIDENCE_HIGH => 2,
        _ => 0,
    };

    if rank(next) < rank(current) {
        next
    } else {
        current
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn make_assistant_record(
        session_id: &str,
        model: &str,
        input: i64,
        output: i64,
        message_id: &str,
    ) -> String {
        let mut msg = serde_json::json!({
            "model": model,
            "usage": {
                "input_tokens": input,
                "output_tokens": output,
                "cache_read_input_tokens": 0,
                "cache_creation_input_tokens": 0,
            },
            "content": [],
        });
        if !message_id.is_empty() {
            msg["id"] = serde_json::json!(message_id);
        }
        serde_json::json!({
            "type": "assistant",
            "sessionId": session_id,
            "timestamp": "2026-04-08T10:00:00Z",
            "cwd": "/home/user/project",
            "message": msg,
        })
        .to_string()
    }

    fn make_user_record(session_id: &str) -> String {
        serde_json::json!({
            "type": "user",
            "sessionId": session_id,
            "timestamp": "2026-04-08T09:59:00Z",
            "cwd": "/home/user/project",
        })
        .to_string()
    }

    fn make_assistant_record_with_usage(
        session_id: &str,
        model: &str,
        usage: serde_json::Value,
    ) -> String {
        serde_json::json!({
            "type": "assistant",
            "sessionId": session_id,
            "timestamp": "2026-04-08T10:00:00Z",
            "cwd": "/home/user/project",
            "message": {
                "model": model,
                "usage": usage,
                "content": [],
            },
        })
        .to_string()
    }

    fn write_jsonl(dir: &TempDir, name: &str, lines: &[String]) -> std::path::PathBuf {
        let path = dir.path().join(name);
        let mut f = std::fs::File::create(&path).unwrap();
        for line in lines {
            writeln!(f, "{}", line).unwrap();
        }
        path
    }

    #[test]
    fn test_project_name_from_cwd() {
        assert_eq!(project_name_from_cwd("/home/user/project"), "user/project");
        assert_eq!(project_name_from_cwd("C:\\Users\\me\\proj"), "me/proj");
        assert_eq!(project_name_from_cwd("/a/b/c/d"), "c/d");
        assert_eq!(project_name_from_cwd(""), "unknown");
        assert_eq!(project_name_from_cwd("/home/user/project/"), "user/project");
    }

    #[test]
    fn test_basic_claude_parsing() {
        let dir = TempDir::new().unwrap();
        let path = write_jsonl(
            &dir,
            "test.jsonl",
            &[
                make_user_record("s1"),
                make_assistant_record("s1", "claude-sonnet-4-6", 100, 50, ""),
            ],
        );
        let result = parse_jsonl_file(PROVIDER_CLAUDE, &path, 0);
        assert_eq!(result.session_metas.len(), 1);
        assert_eq!(result.turns.len(), 1);
        assert_eq!(result.turns[0].input_tokens, 100);
        assert_eq!(result.turns[0].provider, PROVIDER_CLAUDE);
        assert_eq!(result.turns[0].session_id, "claude:s1");
        assert_eq!(result.progress_marker, 2);
    }

    #[test]
    fn test_preserves_zero_token_assistant_turns() {
        let dir = TempDir::new().unwrap();
        let path = write_jsonl(
            &dir,
            "test.jsonl",
            &[make_assistant_record("s1", "claude-sonnet-4-6", 0, 0, "")],
        );
        let result = parse_jsonl_file(PROVIDER_CLAUDE, &path, 0);
        assert_eq!(result.turns.len(), 1);
        assert_eq!(result.turns[0].input_tokens, 0);
        assert_eq!(result.turns[0].output_tokens, 0);
    }

    #[test]
    fn test_preserves_partial_usage_shapes() {
        let dir = TempDir::new().unwrap();
        let path = write_jsonl(
            &dir,
            "test.jsonl",
            &[make_assistant_record_with_usage(
                "s1",
                "claude-sonnet-4-6",
                serde_json::json!({
                    "output_tokens": 42
                }),
            )],
        );
        let result = parse_jsonl_file(PROVIDER_CLAUDE, &path, 0);
        assert_eq!(result.turns.len(), 1);
        assert_eq!(result.turns[0].input_tokens, 0);
        assert_eq!(result.turns[0].output_tokens, 42);
    }

    #[test]
    fn test_streaming_dedup() {
        let dir = TempDir::new().unwrap();
        let path = write_jsonl(
            &dir,
            "test.jsonl",
            &[
                make_assistant_record("s1", "claude-sonnet-4-6", 50, 10, "msg-1"),
                make_assistant_record("s1", "claude-sonnet-4-6", 100, 50, "msg-1"),
                make_assistant_record("s1", "claude-sonnet-4-6", 150, 80, "msg-1"),
            ],
        );
        let result = parse_jsonl_file(PROVIDER_CLAUDE, &path, 0);
        assert_eq!(result.turns.len(), 1);
        assert_eq!(result.turns[0].input_tokens, 150);
    }

    #[test]
    fn test_parse_codex_turn_uses_last_token_usage() {
        let dir = TempDir::new().unwrap();
        let path = write_jsonl(
            &dir,
            "rollout-test.jsonl",
            &[
                serde_json::json!({
                    "timestamp": "2026-04-09T10:00:00Z",
                    "type": "session_meta",
                    "payload": {
                        "id": "sess-1",
                        "timestamp": "2026-04-09T10:00:00Z",
                        "cwd": "/Users/test/work/proj",
                        "cli_version": "0.119.0",
                        "source": "desktop"
                    }
                })
                .to_string(),
                serde_json::json!({
                    "timestamp": "2026-04-09T10:00:01Z",
                    "type": "turn_context",
                    "payload": {
                        "turn_id": "turn-1",
                        "cwd": "/Users/test/work/proj",
                        "model": "gpt-5.4"
                    }
                })
                .to_string(),
                serde_json::json!({
                    "timestamp": "2026-04-09T10:00:02Z",
                    "type": "response_item",
                    "payload": {
                        "type": "function_call",
                        "name": "exec_command",
                        "call_id": "call-1"
                    }
                })
                .to_string(),
                serde_json::json!({
                    "timestamp": "2026-04-09T10:00:03Z",
                    "type": "event_msg",
                    "payload": {
                        "type": "exec_command_end",
                        "call_id": "call-1",
                        "status": "failed",
                        "exit_code": 1
                    }
                })
                .to_string(),
                serde_json::json!({
                    "timestamp": "2026-04-09T10:00:04Z",
                    "type": "event_msg",
                    "payload": {
                        "type": "token_count",
                        "info": {
                            "total_token_usage": {
                                "input_tokens": 999,
                                "cached_input_tokens": 999,
                                "output_tokens": 999,
                                "reasoning_output_tokens": 999
                            },
                            "last_token_usage": {
                                "input_tokens": 120,
                                "cached_input_tokens": 30,
                                "output_tokens": 40,
                                "reasoning_output_tokens": 12
                            }
                        }
                    }
                })
                .to_string(),
            ],
        );

        let result = parse_jsonl_file(PROVIDER_CODEX, &path, 0);
        assert_eq!(result.turns.len(), 1);
        let turn = &result.turns[0];
        assert_eq!(turn.provider, PROVIDER_CODEX);
        assert_eq!(turn.session_id, "codex:sess-1");
        assert_eq!(turn.input_tokens, 120);
        assert_eq!(turn.cache_read_tokens, 30);
        assert_eq!(turn.output_tokens, 40);
        assert_eq!(turn.reasoning_output_tokens, 12);
        assert_eq!(turn.model, "gpt-5.4");
        assert_eq!(turn.version.as_deref(), Some("0.119.0"));
        assert_eq!(
            turn.tool_use_ids,
            vec![("call-1".into(), "exec_command".into())]
        );
        assert_eq!(result.tool_results.get("call-1"), Some(&true));
        assert_eq!(result.session_metas[0].project_name, "work/proj");
    }

    #[test]
    fn test_xcode_dispatcher_tags_session_and_turns() {
        // Xcode CodingAssistant writes the same JSONL format as Claude Code.
        // The dispatcher must tag the output with provider="xcode" and rewrite
        // session_ids so the dashboard provider filter is consistent at both
        // the session and turn level.
        let dir = TempDir::new().unwrap();
        let path = write_jsonl(
            &dir,
            "test.jsonl",
            &[
                make_user_record("s1"),
                make_assistant_record("s1", "claude-sonnet-4-6", 100, 50, ""),
            ],
        );
        let result = parse_jsonl_file(PROVIDER_XCODE, &path, 0);
        assert_eq!(result.turns.len(), 1);
        assert_eq!(result.turns[0].provider, PROVIDER_XCODE);
        assert_eq!(result.turns[0].session_id, "xcode:s1");
        assert_eq!(result.session_metas.len(), 1);
        assert_eq!(result.session_metas[0].provider, PROVIDER_XCODE);
        assert_eq!(result.session_metas[0].session_id, "xcode:s1");
    }

    // -----------------------------------------------------------------------
    // Deliverable 1: tool-argument capture in tool_inputs
    // -----------------------------------------------------------------------

    fn make_assistant_with_tool_use(
        session_id: &str,
        tool_use_id: &str,
        tool_name: &str,
        tool_input: serde_json::Value,
    ) -> String {
        serde_json::json!({
            "type": "assistant",
            "sessionId": session_id,
            "timestamp": "2026-04-08T10:00:00Z",
            "cwd": "/home/user/project",
            "message": {
                "model": "claude-sonnet-4-6",
                "usage": {
                    "input_tokens": 100,
                    "output_tokens": 50,
                    "cache_read_input_tokens": 0,
                    "cache_creation_input_tokens": 0,
                },
                "content": [{
                    "type": "tool_use",
                    "id": tool_use_id,
                    "name": tool_name,
                    "input": tool_input,
                }],
            }
        })
        .to_string()
    }

    #[test]
    fn test_tool_inputs_edit_captures_file_path() {
        let dir = TempDir::new().unwrap();
        let path = write_jsonl(
            &dir,
            "test.jsonl",
            &[make_assistant_with_tool_use(
                "s1",
                "call-1",
                "Edit",
                serde_json::json!({ "file_path": "/src/main.rs", "old_string": "a", "new_string": "b" }),
            )],
        );
        let result = parse_claude_jsonl_file(&path, 0);
        assert_eq!(result.turns.len(), 1);
        let turn = &result.turns[0];
        assert_eq!(turn.tool_inputs.len(), 1);
        assert_eq!(turn.tool_inputs[0].0, "call-1");
        assert_eq!(turn.tool_inputs[0].1, "/src/main.rs");
    }

    #[test]
    fn test_tool_inputs_read_captures_file_path() {
        let dir = TempDir::new().unwrap();
        let path = write_jsonl(
            &dir,
            "test.jsonl",
            &[make_assistant_with_tool_use(
                "s1",
                "call-r",
                "Read",
                serde_json::json!({ "file_path": "/abs/path.rs" }),
            )],
        );
        let result = parse_claude_jsonl_file(&path, 0);
        assert_eq!(result.turns[0].tool_inputs[0].1, "/abs/path.rs");
    }

    #[test]
    fn test_tool_inputs_bash_truncates_long_command() {
        let long_cmd = "x".repeat(200);
        let dir = TempDir::new().unwrap();
        let path = write_jsonl(
            &dir,
            "test.jsonl",
            &[make_assistant_with_tool_use(
                "s1",
                "call-b",
                "Bash",
                serde_json::json!({ "command": long_cmd }),
            )],
        );
        let result = parse_claude_jsonl_file(&path, 0);
        let arg = &result.turns[0].tool_inputs[0].1;
        // Should be truncated to 120 chars + ellipsis.
        assert!(
            arg.ends_with('\u{2026}'),
            "expected trailing ellipsis: {arg}"
        );
        // char count: 120 + 1 ellipsis = 121
        assert_eq!(arg.chars().count(), 121);
    }

    #[test]
    fn test_tool_inputs_bash_short_command_no_truncation() {
        let cmd = "cargo test";
        let dir = TempDir::new().unwrap();
        let path = write_jsonl(
            &dir,
            "test.jsonl",
            &[make_assistant_with_tool_use(
                "s1",
                "call-b",
                "Bash",
                serde_json::json!({ "command": cmd }),
            )],
        );
        let result = parse_claude_jsonl_file(&path, 0);
        assert_eq!(result.turns[0].tool_inputs[0].1, "cargo test");
    }

    #[test]
    fn test_tool_inputs_two_edits_same_file_across_turns() {
        // Two separate assistant turns each editing the same file_path.
        // The tool_events table would have 2 rows with the same value.
        let dir = TempDir::new().unwrap();
        let path = write_jsonl(
            &dir,
            "test.jsonl",
            &[
                make_assistant_with_tool_use(
                    "s1",
                    "call-e1",
                    "Edit",
                    serde_json::json!({ "file_path": "/src/lib.rs", "old_string": "a", "new_string": "b" }),
                ),
                make_assistant_with_tool_use(
                    "s1",
                    "call-e2",
                    "Edit",
                    serde_json::json!({ "file_path": "/src/lib.rs", "old_string": "c", "new_string": "d" }),
                ),
            ],
        );
        let result = parse_claude_jsonl_file(&path, 0);
        // Two turns (different content blocks = different turns here since no message_id dedup).
        // Both should have tool_inputs pointing to /src/lib.rs.
        let all_file_args: Vec<&str> = result
            .turns
            .iter()
            .flat_map(|t| t.tool_inputs.iter())
            .map(|(_, arg)| arg.as_str())
            .collect();
        assert_eq!(all_file_args.len(), 2);
        assert!(all_file_args.iter().all(|&p| p == "/src/lib.rs"));
    }

    #[test]
    fn test_tool_inputs_non_file_tool_produces_empty_arg() {
        let dir = TempDir::new().unwrap();
        let path = write_jsonl(
            &dir,
            "test.jsonl",
            &[make_assistant_with_tool_use(
                "s1",
                "call-w",
                "WebSearch",
                serde_json::json!({ "query": "rust async" }),
            )],
        );
        let result = parse_claude_jsonl_file(&path, 0);
        // tool_inputs entry for WebSearch should be empty string.
        assert_eq!(result.turns[0].tool_inputs[0].1, "");
    }

    #[test]
    fn provider_parse_source_wraps_copilot_provider_results() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("copilot.json");
        std::fs::write(
            &path,
            serde_json::json!([{
                "id": "cop-1",
                "model": "gpt-5.4",
                "timestamp": "2026-04-18T10:00:00Z",
                "usage": {
                    "input_tokens": 120,
                    "output_tokens": 45
                }
            }])
            .to_string(),
        )
        .unwrap();

        let provider = crate::scanner::providers::copilot::CopilotProvider::new_with_dirs(vec![]);
        let result = crate::scanner::provider::Provider::parse_source(&provider, &path, 0);
        assert_eq!(result.turns.len(), 1);
        assert_eq!(result.session_metas.len(), 1);
        assert_eq!(result.turns[0].provider, "copilot");
        assert_eq!(result.turns[0].source_path, path.to_string_lossy());
    }

    #[test]
    fn provider_parse_source_wraps_cursor_provider_results() {
        let dir = TempDir::new().unwrap();
        let hash_dir = dir.path().join("cafebabe1234");
        std::fs::create_dir_all(&hash_dir).unwrap();
        let db_path = hash_dir.join("state.vscdb");
        let conn = rusqlite::Connection::open(&db_path).unwrap();
        conn.execute_batch("CREATE TABLE ItemTable (key TEXT PRIMARY KEY, value TEXT NOT NULL);")
            .unwrap();
        let chat_json = serde_json::json!({
            "tabs": [{
                "tabId": "tab-1",
                "bubbles": [{
                    "type": "ai",
                    "modelType": "claude-3-5-sonnet",
                    "requestId": "req-1",
                    "timingInfo": { "clientStartTime": 1712570400000_i64 },
                    "tokenCount": {
                        "promptTokens": 100,
                        "generationTokens": 50
                    }
                }]
            }]
        });
        conn.execute(
            "INSERT INTO ItemTable (key, value) VALUES (?1, ?2)",
            rusqlite::params![
                "workbench.panel.aichat.view.aichat.chatdata",
                chat_json.to_string()
            ],
        )
        .unwrap();
        drop(conn);

        let provider = crate::scanner::providers::cursor::CursorProvider::new();
        let result = crate::scanner::provider::Provider::parse_source(&provider, &db_path, 0);
        assert_eq!(result.turns.len(), 1);
        assert_eq!(result.session_metas.len(), 1);
        assert_eq!(result.turns[0].provider, "cursor");
        assert_eq!(result.turns[0].source_path, db_path.to_string_lossy());
    }

    #[test]
    fn provider_parse_source_wraps_opencode_provider_results() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("chat.sqlite");
        let conn = rusqlite::Connection::open(&db_path).unwrap();
        conn.execute_batch(
            "CREATE TABLE messages (
                session_id TEXT,
                message_id TEXT,
                input_tokens INTEGER,
                output_tokens INTEGER,
                model TEXT,
                timestamp TEXT
            );",
        )
        .unwrap();
        conn.execute(
            "INSERT INTO messages
                (session_id, message_id, input_tokens, output_tokens, model, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                "sess-1",
                "msg-1",
                64_i64,
                32_i64,
                "claude-sonnet-4-6",
                "2026-04-18T10:00:00Z",
            ],
        )
        .unwrap();
        drop(conn);

        let provider = crate::scanner::providers::opencode::OpenCodeProvider::new_with_dirs(vec![]);
        let result = crate::scanner::provider::Provider::parse_source(&provider, &db_path, 0);
        assert_eq!(result.turns.len(), 1);
        assert_eq!(result.session_metas.len(), 1);
        assert_eq!(result.turns[0].provider, "opencode");
    }

    // -----------------------------------------------------------------------
    // aggregate_sessions: credits accumulation (FIX 3)
    // -----------------------------------------------------------------------

    fn make_amp_turn(session_id: &str, credits: Option<f64>) -> Turn {
        Turn {
            session_id: session_id.to_string(),
            provider: crate::scanner::providers::amp::PROVIDER_AMP.to_string(),
            timestamp: "2026-04-18T10:00:00Z".to_string(),
            model: "claude-sonnet-4-6".to_string(),
            input_tokens: 100,
            output_tokens: 50,
            cache_read_tokens: 0,
            cache_creation_tokens: 0,
            reasoning_output_tokens: 0,
            estimated_cost_nanos: 0,
            tool_name: None,
            cwd: String::new(),
            message_id: format!("amp:{}:{}", session_id, credits.unwrap_or(0.0)),
            service_tier: None,
            inference_geo: None,
            is_subagent: false,
            agent_id: None,
            source_path: String::new(),
            version: None,
            pricing_version: String::new(),
            pricing_model: String::new(),
            billing_mode: "credits".to_string(),
            cost_confidence: "low".to_string(),
            category: String::new(),
            all_tools: Vec::new(),
            tool_use_ids: Vec::new(),
            tool_inputs: Vec::new(),
            credits,
        }
    }

    fn make_amp_session_meta(session_id: &str) -> SessionMeta {
        SessionMeta {
            session_id: session_id.to_string(),
            provider: crate::scanner::providers::amp::PROVIDER_AMP.to_string(),
            project_name: "test/proj".to_string(),
            project_slug: String::new(),
            first_timestamp: "2026-04-18T10:00:00Z".to_string(),
            last_timestamp: "2026-04-18T10:00:00Z".to_string(),
            git_branch: String::new(),
            model: None,
            entrypoint: String::new(),
        }
    }

    #[test]
    fn aggregate_sessions_sums_amp_credits() {
        let meta = make_amp_session_meta("amp:T-sum");
        let turns = vec![
            make_amp_turn("amp:T-sum", Some(5.0)),
            make_amp_turn("amp:T-sum", Some(3.5)),
        ];
        let sessions = aggregate_sessions(&[meta], &turns);
        assert_eq!(sessions.len(), 1);
        let credits = sessions[0].total_credits;
        assert!(
            credits.is_some(),
            "total_credits should be Some for Amp sessions"
        );
        let diff = (credits.unwrap() - 8.5).abs();
        assert!(diff < 1e-9, "expected 8.5 credits, got {:?}", credits);
    }

    #[test]
    fn aggregate_sessions_no_credits_for_non_amp_turns() {
        let meta = SessionMeta {
            session_id: "claude:s-nocredits".to_string(),
            provider: PROVIDER_CLAUDE.to_string(),
            project_name: "user/proj".to_string(),
            project_slug: String::new(),
            first_timestamp: "2026-04-18T10:00:00Z".to_string(),
            last_timestamp: "2026-04-18T10:00:00Z".to_string(),
            git_branch: String::new(),
            model: None,
            entrypoint: String::new(),
        };
        // Non-Amp turns have credits: None
        let turn = Turn {
            session_id: "claude:s-nocredits".to_string(),
            provider: PROVIDER_CLAUDE.to_string(),
            timestamp: "2026-04-18T10:00:00Z".to_string(),
            model: "claude-sonnet-4-6".to_string(),
            input_tokens: 200,
            output_tokens: 80,
            cache_read_tokens: 0,
            cache_creation_tokens: 0,
            reasoning_output_tokens: 0,
            estimated_cost_nanos: 1_000_000,
            tool_name: None,
            cwd: String::new(),
            message_id: "msg-nc-1".to_string(),
            service_tier: None,
            inference_geo: None,
            is_subagent: false,
            agent_id: None,
            source_path: String::new(),
            version: None,
            pricing_version: String::new(),
            pricing_model: String::new(),
            billing_mode: "estimated_local".to_string(),
            cost_confidence: "high".to_string(),
            category: String::new(),
            all_tools: Vec::new(),
            tool_use_ids: Vec::new(),
            tool_inputs: Vec::new(),
            credits: None,
        };
        let sessions = aggregate_sessions(&[meta], &[turn]);
        assert_eq!(sessions.len(), 1);
        assert_eq!(
            sessions[0].total_credits, None,
            "non-Amp sessions must have total_credits = None"
        );
    }
}
