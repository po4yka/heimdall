use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use anyhow::Result;
use tracing::warn;
use walkdir::WalkDir;

use crate::models::{SessionMeta, Turn};
use crate::pricing;
use crate::scanner::parser::{
    PROVIDER_CODEX, ParseResult, empty_parse_result, file_timestamp_rfc3339, project_name_from_cwd,
    session_key, touch_session_meta, upsert_session_meta,
};
use crate::scanner::provider::{Provider, SessionSource};

#[derive(Debug, Clone, Default)]
struct CodexTurnContext {
    timestamp: String,
    cwd: String,
    model: String,
}

#[derive(Debug, Clone, Default)]
struct TokenUsage {
    input: i64,
    output: i64,
    cache_read: i64,
    reasoning_output: i64,
    plan_type: Option<String>,
}

pub struct CodexProvider {
    pub dirs: Vec<PathBuf>,
}

impl CodexProvider {
    pub fn new(dirs: Vec<PathBuf>) -> Self {
        Self { dirs }
    }
}

impl Provider for CodexProvider {
    fn name(&self) -> &'static str {
        "codex"
    }

    fn discover_sessions(&self) -> Result<Vec<SessionSource>> {
        let mut sources = Vec::new();
        for dir in &self.dirs {
            if !dir.exists() {
                continue;
            }
            for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
                if entry.path().extension().is_some_and(|ext| ext == "jsonl") {
                    sources.push(SessionSource {
                        path: entry.path().to_path_buf(),
                        provider_name: self.name(),
                    });
                }
            }
        }
        Ok(sources)
    }

    fn parse(&self, path: &Path) -> Result<Vec<Turn>> {
        Ok(parse_codex_jsonl_file(path, 0).turns)
    }
}

pub(crate) fn parse_codex_jsonl_file(filepath: &Path, skip_lines: i64) -> ParseResult {
    let mut seen_turns: HashMap<String, Turn> = HashMap::new();
    let mut session_metas: HashMap<String, SessionMeta> = HashMap::new();
    let mut tool_results: HashMap<String, bool> = HashMap::new();
    let mut turn_contexts: HashMap<String, CodexTurnContext> = HashMap::new();
    let mut turn_tools: HashMap<String, Vec<(String, String)>> = HashMap::new();
    let mut line_count: i64 = 0;
    let source_path = filepath.to_string_lossy().to_string();
    let fallback_timestamp = file_timestamp_rfc3339(filepath);
    let fallback_session_id = filepath
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("unknown")
        .to_string();
    let mut raw_session_id = fallback_session_id.clone();
    let mut session_id = session_key(PROVIDER_CODEX, &raw_session_id);
    let mut current_turn_id: Option<String> = None;
    let mut session_cwd = String::new();
    let mut session_entrypoint = String::new();
    let mut session_version: Option<String> = None;
    let mut session_model: Option<String> = None;
    let session_git_branch = String::new();

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

        let timestamp = record
            .get("timestamp")
            .and_then(|v| v.as_str())
            .unwrap_or(&fallback_timestamp)
            .to_string();
        let record_type = record.get("type").and_then(|v| v.as_str()).unwrap_or("");
        let payload = record
            .get("payload")
            .and_then(|value| value.as_object())
            .cloned()
            .unwrap_or_default();

        match record_type {
            "session_meta" => {
                raw_session_id = payload
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&fallback_session_id)
                    .to_string();
                session_id = session_key(PROVIDER_CODEX, &raw_session_id);
                session_cwd = payload
                    .get("cwd")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                session_entrypoint = payload
                    .get("source")
                    .or_else(|| payload.get("originator"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                session_version = payload
                    .get("cli_version")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                let meta_ts = payload
                    .get("timestamp")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&timestamp)
                    .to_string();
                upsert_session_meta(
                    &mut session_metas,
                    &session_id,
                    SessionMeta {
                        session_id: session_id.clone(),
                        provider: PROVIDER_CODEX.into(),
                        project_name: project_name_from_cwd(&session_cwd),
                        project_slug: String::new(),
                        first_timestamp: meta_ts.clone(),
                        last_timestamp: meta_ts,
                        git_branch: session_git_branch.clone(),
                        model: session_model.clone(),
                        entrypoint: session_entrypoint.clone(),
                    },
                );
            }
            "turn_context" => {
                let turn_id = payload
                    .get("turn_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                if turn_id.is_empty() {
                    continue;
                }
                current_turn_id = Some(turn_id.clone());
                let cwd = payload
                    .get("cwd")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&session_cwd)
                    .to_string();
                let model = payload
                    .get("model")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                if !cwd.is_empty() {
                    session_cwd = cwd.clone();
                }
                if !model.is_empty() {
                    session_model = Some(model.clone());
                }
                turn_contexts.insert(
                    turn_id,
                    CodexTurnContext {
                        timestamp: timestamp.clone(),
                        cwd,
                        model,
                    },
                );
                touch_session_meta(
                    &mut session_metas,
                    &session_id,
                    PROVIDER_CODEX,
                    &timestamp,
                    &session_cwd,
                    &session_git_branch,
                    session_model.as_deref(),
                    &session_entrypoint,
                );
            }
            "event_msg" => {
                let payload_type = payload.get("type").and_then(|v| v.as_str()).unwrap_or("");
                match payload_type {
                    "task_started" => {
                        current_turn_id = payload
                            .get("turn_id")
                            .and_then(|v| v.as_str())
                            .map(String::from);
                        if let Some(turn_id) = current_turn_id.clone() {
                            turn_contexts
                                .entry(turn_id)
                                .or_insert_with(|| CodexTurnContext {
                                    timestamp: timestamp.clone(),
                                    cwd: session_cwd.clone(),
                                    model: session_model.clone().unwrap_or_default(),
                                });
                        }
                        touch_session_meta(
                            &mut session_metas,
                            &session_id,
                            PROVIDER_CODEX,
                            &timestamp,
                            &session_cwd,
                            &session_git_branch,
                            session_model.as_deref(),
                            &session_entrypoint,
                        );
                    }
                    "task_complete" => {
                        touch_session_meta(
                            &mut session_metas,
                            &session_id,
                            PROVIDER_CODEX,
                            &timestamp,
                            &session_cwd,
                            &session_git_branch,
                            session_model.as_deref(),
                            &session_entrypoint,
                        );
                    }
                    "token_count" => {
                        let usage = parse_codex_token_usage(&payload);
                        if usage.input + usage.output + usage.cache_read + usage.reasoning_output
                            == 0
                        {
                            continue;
                        }

                        let turn_id = current_turn_id
                            .clone()
                            .unwrap_or_else(|| format!("line-{line_count}"));
                        let context = turn_contexts.get(&turn_id).cloned().unwrap_or_default();
                        let turn_timestamp = if !timestamp.is_empty() {
                            timestamp.clone()
                        } else if !context.timestamp.is_empty() {
                            context.timestamp
                        } else {
                            fallback_timestamp.clone()
                        };
                        let cwd = if !context.cwd.is_empty() {
                            context.cwd
                        } else {
                            session_cwd.clone()
                        };
                        let model = if !context.model.is_empty() {
                            context.model
                        } else {
                            session_model.clone().unwrap_or_else(|| "unknown".into())
                        };
                        if !model.is_empty() && model != "unknown" {
                            session_model = Some(model.clone());
                        }

                        let tool_use_ids = turn_tools.get(&turn_id).cloned().unwrap_or_default();
                        let tool_name = tool_use_ids.first().map(|(_, name)| name.clone());
                        let billing_mode = codex_billing_mode(usage.plan_type.as_deref());
                        let all_tools: Vec<String> =
                            tool_use_ids.iter().map(|(_, name)| name.clone()).collect();
                        let category = crate::scanner::classifier::classify(
                            tool_name.as_deref(),
                            &all_tools,
                            None,
                        )
                        .as_str()
                        .to_string();

                        let turn = Turn {
                            estimated_cost_nanos: 0,
                            session_id: session_id.clone(),
                            provider: PROVIDER_CODEX.into(),
                            timestamp: turn_timestamp.clone(),
                            model,
                            input_tokens: usage.input,
                            output_tokens: usage.output,
                            cache_read_tokens: usage.cache_read,
                            cache_creation_tokens: 0,
                            reasoning_output_tokens: usage.reasoning_output,
                            tool_name,
                            cwd: cwd.clone(),
                            message_id: turn_id.clone(),
                            service_tier: None,
                            inference_geo: None,
                            is_subagent: false,
                            agent_id: None,
                            source_path: source_path.clone(),
                            version: session_version.clone(),
                            pricing_version: String::new(),
                            pricing_model: String::new(),
                            billing_mode,
                            cost_confidence: String::new(),
                            category,
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

                        seen_turns.insert(turn_id, turn);
                        touch_session_meta(
                            &mut session_metas,
                            &session_id,
                            PROVIDER_CODEX,
                            &turn_timestamp,
                            &cwd,
                            &session_git_branch,
                            session_model.as_deref(),
                            &session_entrypoint,
                        );
                    }
                    _ if payload_type.ends_with("_end") => {
                        if let Some(call_id) = payload.get("call_id").and_then(|v| v.as_str()) {
                            let status = payload
                                .get("status")
                                .and_then(|v| v.as_str())
                                .unwrap_or("completed");
                            let exit_code = payload
                                .get("exit_code")
                                .and_then(|v| v.as_i64())
                                .unwrap_or(0);
                            tool_results.insert(
                                call_id.to_string(),
                                status != "completed" || exit_code != 0,
                            );
                        }
                    }
                    _ => {}
                }
            }
            "response_item" => {
                let payload_type = payload.get("type").and_then(|v| v.as_str()).unwrap_or("");
                match payload_type {
                    "function_call" | "custom_tool_call" => {
                        let turn_id = current_turn_id
                            .clone()
                            .unwrap_or_else(|| format!("line-{line_count}"));
                        let call_id = payload
                            .get("call_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let tool_name = payload
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        if !tool_name.is_empty() {
                            let key = if call_id.is_empty() {
                                format!("{turn_id}:{tool_name}:{}", turn_tools.len())
                            } else {
                                call_id.clone()
                            };
                            turn_tools
                                .entry(turn_id)
                                .or_default()
                                .push((key, tool_name));
                        }
                        if let Some(status) = payload.get("status").and_then(|v| v.as_str())
                            && let Some(call_id) = payload.get("call_id").and_then(|v| v.as_str())
                        {
                            tool_results.insert(call_id.to_string(), status != "completed");
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    let mut turns: Vec<Turn> = seen_turns.into_values().collect();
    turns.sort_by(|a, b| {
        a.session_id
            .cmp(&b.session_id)
            .then_with(|| a.timestamp.cmp(&b.timestamp))
            .then_with(|| a.message_id.cmp(&b.message_id))
            .then_with(|| a.model.cmp(&b.model))
    });

    ParseResult {
        session_metas: session_metas.into_values().collect(),
        turns,
        line_count,
        session_titles: HashMap::new(),
        tool_results,
    }
}

fn parse_codex_token_usage(payload: &serde_json::Map<String, serde_json::Value>) -> TokenUsage {
    let info = payload.get("info").and_then(|v| v.as_object());
    let usage = info
        .and_then(|info| info.get("last_token_usage"))
        .and_then(|v| v.as_object())
        .or_else(|| {
            info.and_then(|info| info.get("total_token_usage"))
                .and_then(|v| v.as_object())
        });

    let Some(usage) = usage else {
        return TokenUsage::default();
    };

    TokenUsage {
        input: usage
            .get("input_tokens")
            .and_then(|v| v.as_i64())
            .unwrap_or(0),
        output: usage
            .get("output_tokens")
            .and_then(|v| v.as_i64())
            .unwrap_or(0),
        cache_read: usage
            .get("cached_input_tokens")
            .and_then(|v| v.as_i64())
            .unwrap_or(0),
        reasoning_output: usage
            .get("reasoning_output_tokens")
            .and_then(|v| v.as_i64())
            .unwrap_or(0),
        plan_type: info
            .and_then(|info| info.get("plan_type"))
            .and_then(|v| v.as_str())
            .map(String::from),
    }
}

fn codex_billing_mode(plan_type: Option<&str>) -> String {
    let Some(plan_type) = plan_type.map(str::trim).filter(|value| !value.is_empty()) else {
        return "estimated_local".into();
    };

    match plan_type.to_ascii_lowercase().as_str() {
        "api" | "byok" | "payg" | "paygo" => "estimated_local".into(),
        _ => "subscriber_included".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn write_jsonl(dir: &TempDir, name: &str, lines: &[String]) -> PathBuf {
        let path = dir.path().join(name);
        let mut f = std::fs::File::create(&path).unwrap();
        for line in lines {
            writeln!(f, "{}", line).unwrap();
        }
        path
    }

    #[test]
    fn codex_provider_name() {
        assert_eq!(CodexProvider::new(vec![]).name(), "codex");
    }

    #[test]
    fn codex_parse_delegates_to_existing_logic() {
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
                    "timestamp": "2026-04-09T10:00:04Z",
                    "type": "event_msg",
                    "payload": {
                        "type": "token_count",
                        "info": {
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

        let provider = CodexProvider::new(vec![]);
        let turns = provider.parse(&path).unwrap();
        assert_eq!(turns.len(), 1);
        assert_eq!(turns[0].provider, "codex");
        assert_eq!(turns[0].input_tokens, 120);
    }
}
