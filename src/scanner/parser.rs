use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::Path;

use tracing::warn;

use crate::models::{Session, SessionMeta, Turn};
use crate::pricing;

pub const PROVIDER_CLAUDE: &str = "claude";
pub const PROVIDER_CODEX: &str = "codex";
pub const PROVIDER_XCODE: &str = "xcode";

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
    pub line_count: i64,
    #[allow(dead_code)]
    pub session_titles: HashMap<String, String>,
    pub tool_results: HashMap<String, bool>,
}

/// Parse a provider-specific log file.
pub fn parse_jsonl_file(provider: &str, filepath: &Path, skip_lines: i64) -> ParseResult {
    match provider {
        PROVIDER_CODEX => {
            crate::scanner::providers::codex::parse_codex_jsonl_file(filepath, skip_lines)
        }
        PROVIDER_XCODE => retag_claude_result(parse_claude_jsonl_file(filepath, skip_lines)),
        _ => parse_claude_jsonl_file(filepath, skip_lines),
    }
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
    let mut line_count: i64 = 0;
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
        line_count += 1;
        if line_count <= skip_lines {
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
            let usage = msg
                .get("usage")
                .cloned()
                .unwrap_or(serde_json::Value::Object(Default::default()));
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

            let input_tokens = usage
                .get("input_tokens")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let output_tokens = usage
                .get("output_tokens")
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

            if input_tokens + output_tokens + cache_read + cache_creation == 0 {
                continue;
            }

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

            let service_tier = usage
                .get("service_tier")
                .and_then(|v| v.as_str())
                .map(String::from);
            let inference_geo = usage
                .get("inference_geo")
                .and_then(|v| v.as_str())
                .map(String::from);

            if !model.is_empty()
                && let Some(meta) = session_meta.get_mut(&session_id)
            {
                meta.model = Some(model.clone());
            }

            let turn = Turn {
                estimated_cost_nanos: 0,
                session_id: session_id.clone(),
                provider: PROVIDER_CLAUDE.into(),
                timestamp: timestamp.clone(),
                model,
                input_tokens,
                output_tokens,
                cache_read_tokens: cache_read,
                cache_creation_tokens: cache_creation,
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
                all_tools,
                tool_use_ids,
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

    ParseResult {
        session_metas: session_meta.into_values().collect(),
        turns,
        line_count,
        session_titles,
        tool_results,
    }
}

pub(crate) fn empty_parse_result() -> ParseResult {
    ParseResult {
        session_metas: vec![],
        turns: vec![],
        line_count: 0,
        session_titles: HashMap::new(),
        tool_results: HashMap::new(),
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
pub fn aggregate_sessions(metas: &[SessionMeta], turns: &[Turn]) -> Vec<Session> {
    struct Stats {
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
    }

    let mut stats_map: HashMap<&str, Stats> = HashMap::new();
    for t in turns {
        let entry = stats_map.entry(&t.session_id).or_insert(Stats {
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
    }

    metas
        .iter()
        .map(|meta| {
            let empty = Stats {
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
            };
            let s = stats_map.get(meta.session_id.as_str()).unwrap_or(&empty);
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
        assert_eq!(result.line_count, 2);
    }

    #[test]
    fn test_skips_zero_tokens() {
        let dir = TempDir::new().unwrap();
        let path = write_jsonl(
            &dir,
            "test.jsonl",
            &[make_assistant_record("s1", "claude-sonnet-4-6", 0, 0, "")],
        );
        let result = parse_jsonl_file(PROVIDER_CLAUDE, &path, 0);
        assert_eq!(result.turns.len(), 0);
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
}
