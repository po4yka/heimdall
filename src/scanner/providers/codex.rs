use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use std::collections::BTreeMap;

use anyhow::Result;
use tracing::{debug, warn};
use walkdir::WalkDir;

use crate::models::{CodexPlanDailyRow, CodexPlanSnapshot, SessionMeta, Turn};
use crate::pricing;
use crate::scanner::parser::{
    CodexLimitHit, PROVIDER_CODEX, ParseResult, empty_parse_result, file_timestamp_rfc3339,
    project_name_from_cwd, session_key, touch_session_meta, upsert_session_meta,
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
    source: Option<&'static str>,
    has_usage_fields: bool,
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
                    });
                }
            }
        }
        Ok(sources)
    }

    fn parse(&self, path: &Path) -> Result<Vec<Turn>> {
        Ok(parse_codex_jsonl_file(path, 0).turns)
    }

    fn archive_paths(&self) -> Vec<PathBuf> {
        self.dirs.clone()
    }
}

pub(crate) fn parse_codex_jsonl_file(filepath: &Path, skip_lines: i64) -> ParseResult {
    let mut seen_turns: HashMap<String, Turn> = HashMap::new();
    let mut session_metas: HashMap<String, SessionMeta> = HashMap::new();
    let mut tool_results: HashMap<String, bool> = HashMap::new();
    let mut turn_contexts: HashMap<String, CodexTurnContext> = HashMap::new();
    let mut turn_tools: HashMap<String, Vec<(String, String)>> = HashMap::new();
    let mut progress_marker: i64 = 0;
    // Feature 1: Codex plan tracking accumulators.
    let mut codex_plan_snapshots: Vec<crate::models::CodexPlanSnapshot> = Vec::new();
    let mut codex_limit_hits: Vec<crate::scanner::parser::CodexLimitHit> = Vec::new();
    // Track the last seen plan_type so limit events can carry it forward.
    let mut last_plan_type: Option<String> = None;
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
                        if !usage.has_usage_fields {
                            debug!(
                                "codex parser: preserving token_count event without recognized usage fields from {}",
                                usage.source.unwrap_or("unknown")
                            );
                        }
                        // Feature 1: extract rate_limits snapshot if present.
                        if let Some(snap) = parse_codex_rate_limits(&payload, &timestamp) {
                            if let Some(ref pt) = snap.plan_type
                                && !pt.is_empty()
                            {
                                last_plan_type = Some(pt.clone());
                            }
                            codex_plan_snapshots.push(snap);
                        }

                        let turn_id = current_turn_id
                            .clone()
                            .unwrap_or_else(|| format!("line-{progress_marker}"));
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
                            tool_inputs: Vec::new(),
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
                    "usage_limit_exceeded" => {
                        // Feature 1: record a limit-hit event, carrying forward
                        // the plan_type from the most recent rate_limits snapshot.
                        codex_limit_hits.push(crate::scanner::parser::CodexLimitHit {
                            ts: timestamp.clone(),
                            plan_type: last_plan_type.clone(),
                        });
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
                            .unwrap_or_else(|| format!("line-{progress_marker}"));
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
        progress_marker,
        session_titles: HashMap::new(),
        tool_results,
        tool_error_texts: HashMap::new(),
        tool_input_jsons: HashMap::new(),
        codex_plan_snapshots,
        codex_limit_hits,
    }
}

fn parse_codex_token_usage(payload: &serde_json::Map<String, serde_json::Value>) -> TokenUsage {
    let info = payload.get("info").and_then(|v| v.as_object());
    let usage_and_source = info
        .and_then(|info| {
            info.get("last_token_usage")
                .and_then(|v| v.as_object())
                .map(|usage| (usage, "last_token_usage"))
        })
        .or_else(|| {
            info.and_then(|info| {
                info.get("total_token_usage")
                    .and_then(|v| v.as_object())
                    .map(|usage| (usage, "total_token_usage"))
            })
        });

    let Some((usage, source)) = usage_and_source else {
        debug!("codex parser: token_count event had no recognized usage container");
        return TokenUsage::default();
    };

    let has_usage_fields = usage.contains_key("input_tokens")
        || usage.contains_key("output_tokens")
        || usage.contains_key("cached_input_tokens")
        || usage.contains_key("reasoning_output_tokens");

    if !has_usage_fields {
        debug!(
            "codex parser: token_count event used {} but had no recognized token fields",
            source
        );
    }

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
        source: Some(source),
        has_usage_fields,
    }
}

/// Extract a `CodexPlanSnapshot` from a `token_count` payload if it contains
/// a `rate_limits` object. Returns `None` when the field is absent or null.
fn parse_codex_rate_limits(
    payload: &serde_json::Map<String, serde_json::Value>,
    timestamp: &str,
) -> Option<crate::models::CodexPlanSnapshot> {
    let rl = payload.get("rate_limits")?.as_object()?;

    let primary = rl.get("primary").and_then(|v| v.as_object()).map(|w| {
        let resets_at = w.get("resets_at").and_then(|v| v.as_i64()).map(|epoch| {
            chrono::DateTime::from_timestamp(epoch, 0)
                .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
                .unwrap_or_default()
        });
        crate::models::CodexPlanWindow {
            used_percent: w
                .get("used_percent")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            window_minutes: w
                .get("window_minutes")
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
            resets_at,
        }
    });

    let secondary = rl.get("secondary").and_then(|v| v.as_object()).map(|w| {
        let resets_at = w.get("resets_at").and_then(|v| v.as_i64()).map(|epoch| {
            chrono::DateTime::from_timestamp(epoch, 0)
                .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
                .unwrap_or_default()
        });
        crate::models::CodexPlanWindow {
            used_percent: w
                .get("used_percent")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            window_minutes: w
                .get("window_minutes")
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
            resets_at,
        }
    });

    let credits =
        rl.get("credits")
            .and_then(|v| v.as_object())
            .map(|c| crate::models::CodexCredits {
                has_credits: c
                    .get("has_credits")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                unlimited: c
                    .get("unlimited")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                balance: c.get("balance").and_then(|v| v.as_f64()),
            });

    let plan_type = rl
        .get("plan_type")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(String::from);

    Some(crate::models::CodexPlanSnapshot {
        plan_type,
        limit_id: None,
        primary,
        secondary,
        credits,
        rate_limit_reached_type: None,
        captured_at: Some(timestamp.to_string()),
    })
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

/// Aggregate per-scan Codex plan snapshots and limit-hit events into per-day
/// `CodexPlanDailyRow`s ready for upsert into `codex_plan_daily`.
///
/// `tz_offset_min` is the client timezone offset in minutes (0 = UTC).
pub fn aggregate_codex_plan_daily(
    snapshots: &[CodexPlanSnapshot],
    limit_events: &[CodexLimitHit],
    tz_offset_min: i32,
) -> Vec<CodexPlanDailyRow> {
    // Helper: convert an ISO 8601 UTC timestamp to a local YYYY-MM-DD string.
    let local_day = |ts: &str| -> Option<String> {
        let dt = chrono::DateTime::parse_from_rfc3339(ts).ok()?;
        let shifted = dt + chrono::Duration::minutes(tz_offset_min as i64);
        Some(shifted.format("%Y-%m-%d").to_string())
    };

    // Group snapshots by local day.
    // BTreeMap keeps days sorted for deterministic output.
    let mut by_day: BTreeMap<String, Vec<&CodexPlanSnapshot>> = BTreeMap::new();
    for snap in snapshots {
        if let Some(ts) = &snap.captured_at
            && let Some(day) = local_day(ts)
        {
            by_day.entry(day).or_default().push(snap);
        }
    }

    // Group limit hits by local day.
    let mut hits_by_day: BTreeMap<String, Vec<&CodexLimitHit>> = BTreeMap::new();
    for hit in limit_events {
        if let Some(day) = local_day(&hit.ts) {
            hits_by_day.entry(day).or_default().push(hit);
        }
    }

    // Collect all days that appear in either snapshots or hits.
    let mut all_days: std::collections::BTreeSet<String> =
        BTreeMap::keys(&by_day).cloned().collect();
    for day in hits_by_day.keys() {
        all_days.insert(day.clone());
    }

    let mut result = Vec::with_capacity(all_days.len());
    for day in all_days {
        let snaps = by_day.get(&day).map(|v| v.as_slice()).unwrap_or(&[]);
        let hits = hits_by_day.get(&day).map(|v| v.as_slice()).unwrap_or(&[]);

        // Peak primary_pct across all snapshots in the day.
        let primary_pct = snaps
            .iter()
            .filter_map(|s| s.primary.as_ref().map(|w| w.used_percent))
            .fold(f64::NEG_INFINITY, f64::max);
        let primary_pct = if primary_pct.is_finite() {
            primary_pct
        } else {
            0.0
        };

        // Peak secondary_pct (nullable).
        let secondary_values: Vec<f64> = snaps
            .iter()
            .filter_map(|s| s.secondary.as_ref().map(|w| w.used_percent))
            .collect();
        let secondary_pct = if secondary_values.is_empty() {
            None
        } else {
            secondary_values
                .iter()
                .copied()
                .reduce(f64::max)
                .filter(|v| v.is_finite())
        };

        // Per-plan peak primary_pct.
        let mut by_plan: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
        for snap in snaps {
            let plan_key = snap
                .plan_type
                .clone()
                .unwrap_or_else(|| "unknown".to_string());
            if let Some(w) = &snap.primary {
                let entry = by_plan.entry(plan_key).or_insert(f64::NEG_INFINITY);
                if w.used_percent > *entry {
                    *entry = w.used_percent;
                }
            }
        }

        // Limit hit aggregation: collect unique plan types (insertion-order preserved).
        let mut limit_hit_plans: Vec<String> = Vec::new();
        for hit in hits {
            let plan = hit
                .plan_type
                .clone()
                .unwrap_or_else(|| "unknown".to_string());
            if !limit_hit_plans.contains(&plan) {
                limit_hit_plans.push(plan);
            }
        }
        let limit_hit_count = hits.len() as i64;

        // Last snapshot of the day (latest captured_at).
        let snapshot = snaps
            .iter()
            .max_by(|a, b| {
                a.captured_at
                    .as_deref()
                    .unwrap_or("")
                    .cmp(b.captured_at.as_deref().unwrap_or(""))
            })
            .map(|s| (*s).clone())
            .unwrap_or_default();

        result.push(CodexPlanDailyRow {
            day,
            primary_pct,
            secondary_pct,
            by_plan,
            limit_hit_plans,
            limit_hit_count,
            snapshot,
        });
    }
    result
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
    fn codex_archive_paths_returns_dirs() {
        let provider = CodexProvider::new(vec![PathBuf::from("/tmp/codex-sessions")]);
        let paths = provider.archive_paths();
        assert!(
            paths.iter().any(|p| p.ends_with("codex-sessions")),
            "expected codex-sessions in archive_paths, got: {:?}",
            paths
        );
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

    #[test]
    fn codex_parse_preserves_token_count_with_partial_usage() {
        let dir = TempDir::new().unwrap();
        let path = write_jsonl(
            &dir,
            "partial-usage.jsonl",
            &[
                serde_json::json!({
                    "timestamp": "2026-04-09T10:00:00Z",
                    "type": "session_meta",
                    "payload": {
                        "id": "sess-1",
                        "cwd": "/Users/test/work/proj",
                        "cli_version": "0.119.0"
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
                            "total_token_usage": {
                                "output_tokens": 41
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
        assert_eq!(turns[0].input_tokens, 0);
        assert_eq!(turns[0].output_tokens, 41);
    }

    // -----------------------------------------------------------------
    // Feature 1: Codex plan utilisation tracking tests
    // -----------------------------------------------------------------

    fn make_token_count_line(
        ts: &str,
        primary_pct: f64,
        secondary_pct: Option<f64>,
        plan_type: Option<&str>,
        input: i64,
        output: i64,
    ) -> String {
        let mut rl = serde_json::json!({
            "primary": {
                "used_percent": primary_pct,
                "window_minutes": 300,
                "resets_at": 1764079740_i64
            },
            "credits": {
                "has_credits": false,
                "unlimited": false,
                "balance": null
            }
        });
        if let Some(sp) = secondary_pct {
            rl["secondary"] = serde_json::json!({
                "used_percent": sp,
                "window_minutes": 10080,
                "resets_at": 1764079740_i64
            });
        }
        if let Some(pt) = plan_type {
            rl["plan_type"] = serde_json::Value::String(pt.to_string());
        }
        serde_json::json!({
            "timestamp": ts,
            "type": "event_msg",
            "payload": {
                "type": "token_count",
                "info": {
                    "last_token_usage": {
                        "input_tokens": input,
                        "output_tokens": output,
                        "cached_input_tokens": 0
                    }
                },
                "rate_limits": rl
            }
        })
        .to_string()
    }

    fn session_meta_line(ts: &str) -> String {
        serde_json::json!({
            "timestamp": ts,
            "type": "session_meta",
            "payload": {
                "id": "sess-plan-test",
                "cwd": "/tmp/proj",
                "cli_version": "0.119.0"
            }
        })
        .to_string()
    }

    fn turn_context_line(ts: &str, turn_id: &str) -> String {
        serde_json::json!({
            "timestamp": ts,
            "type": "turn_context",
            "payload": {
                "turn_id": turn_id,
                "cwd": "/tmp/proj",
                "model": "gpt-5.4"
            }
        })
        .to_string()
    }

    #[test]
    fn parses_rate_limits_event() {
        let dir = TempDir::new().unwrap();
        let path = write_jsonl(
            &dir,
            "rate-limits-test.jsonl",
            &[
                session_meta_line("2026-05-05T10:00:00Z"),
                turn_context_line("2026-05-05T10:00:01Z", "t1"),
                make_token_count_line(
                    "2026-05-05T10:00:04Z",
                    42.0,
                    Some(14.0),
                    Some("plus"),
                    100,
                    50,
                ),
            ],
        );
        let result = parse_codex_jsonl_file(&path, 0);
        assert_eq!(result.codex_plan_snapshots.len(), 1);
        let snap = &result.codex_plan_snapshots[0];
        assert_eq!(snap.plan_type.as_deref(), Some("plus"));
        let primary = snap.primary.as_ref().expect("primary should be present");
        assert!((primary.used_percent - 42.0).abs() < 0.001);
        assert_eq!(primary.window_minutes, 300);
        let secondary = snap
            .secondary
            .as_ref()
            .expect("secondary should be present");
        assert!((secondary.used_percent - 14.0).abs() < 0.001);
        assert!(
            snap.captured_at
                .as_deref()
                .unwrap()
                .starts_with("2026-05-05")
        );
    }

    #[test]
    fn parses_usage_limit_exceeded_event() {
        let dir = TempDir::new().unwrap();
        let path = write_jsonl(
            &dir,
            "limit-exceeded-test.jsonl",
            &[
                session_meta_line("2026-05-05T10:00:00Z"),
                turn_context_line("2026-05-05T10:00:01Z", "t1"),
                // Rate-limits snapshot first — establishes plan_type = "pro"
                make_token_count_line("2026-05-05T10:00:04Z", 95.0, None, Some("pro"), 100, 50),
                // usage_limit_exceeded event
                serde_json::json!({
                    "timestamp": "2026-05-05T10:00:10Z",
                    "type": "event_msg",
                    "payload": { "type": "usage_limit_exceeded" }
                })
                .to_string(),
            ],
        );
        let result = parse_codex_jsonl_file(&path, 0);
        assert_eq!(result.codex_limit_hits.len(), 1);
        let hit = &result.codex_limit_hits[0];
        assert!(hit.ts.starts_with("2026-05-05T10:00:10"));
        // plan_type carried forward from the preceding rate_limits snapshot
        assert_eq!(hit.plan_type.as_deref(), Some("pro"));
    }

    #[test]
    fn aggregates_per_day_takes_max_primary() {
        let snaps = vec![
            crate::models::CodexPlanSnapshot {
                plan_type: Some("plus".into()),
                primary: Some(crate::models::CodexPlanWindow {
                    used_percent: 30.0,
                    window_minutes: 300,
                    resets_at: None,
                }),
                captured_at: Some("2026-05-05T10:00:00Z".into()),
                ..Default::default()
            },
            crate::models::CodexPlanSnapshot {
                plan_type: Some("plus".into()),
                primary: Some(crate::models::CodexPlanWindow {
                    used_percent: 85.0,
                    window_minutes: 300,
                    resets_at: None,
                }),
                captured_at: Some("2026-05-05T11:00:00Z".into()),
                ..Default::default()
            },
            crate::models::CodexPlanSnapshot {
                plan_type: Some("plus".into()),
                primary: Some(crate::models::CodexPlanWindow {
                    used_percent: 40.0,
                    window_minutes: 300,
                    resets_at: None,
                }),
                captured_at: Some("2026-05-05T12:00:00Z".into()),
                ..Default::default()
            },
        ];
        let rows = aggregate_codex_plan_daily(&snaps, &[], 0);
        assert_eq!(rows.len(), 1);
        assert!((rows[0].primary_pct - 85.0).abs() < 0.001);
    }

    #[test]
    fn aggregates_per_day_secondary_nullable() {
        let snaps = vec![crate::models::CodexPlanSnapshot {
            plan_type: Some("plus".into()),
            primary: Some(crate::models::CodexPlanWindow {
                used_percent: 50.0,
                window_minutes: 300,
                resets_at: None,
            }),
            secondary: None, // no secondary
            captured_at: Some("2026-05-05T10:00:00Z".into()),
            ..Default::default()
        }];
        let rows = aggregate_codex_plan_daily(&snaps, &[], 0);
        assert_eq!(rows.len(), 1);
        assert!(rows[0].secondary_pct.is_none());
    }

    #[test]
    fn aggregates_per_day_groups_by_plan() {
        let snaps = vec![
            crate::models::CodexPlanSnapshot {
                plan_type: Some("free".into()),
                primary: Some(crate::models::CodexPlanWindow {
                    used_percent: 20.0,
                    window_minutes: 300,
                    resets_at: None,
                }),
                captured_at: Some("2026-05-05T08:00:00Z".into()),
                ..Default::default()
            },
            crate::models::CodexPlanSnapshot {
                plan_type: Some("plus".into()),
                primary: Some(crate::models::CodexPlanWindow {
                    used_percent: 55.0,
                    window_minutes: 300,
                    resets_at: None,
                }),
                captured_at: Some("2026-05-05T14:00:00Z".into()),
                ..Default::default()
            },
        ];
        let rows = aggregate_codex_plan_daily(&snaps, &[], 0);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].by_plan.get("free").copied().unwrap_or(0.0), 20.0);
        assert_eq!(rows[0].by_plan.get("plus").copied().unwrap_or(0.0), 55.0);
    }

    #[test]
    fn aggregates_per_day_counts_limit_hits() {
        let snaps = vec![crate::models::CodexPlanSnapshot {
            plan_type: Some("free".into()),
            primary: Some(crate::models::CodexPlanWindow {
                used_percent: 99.0,
                window_minutes: 300,
                resets_at: None,
            }),
            captured_at: Some("2026-05-05T10:00:00Z".into()),
            ..Default::default()
        }];
        let hits = vec![
            crate::scanner::parser::CodexLimitHit {
                ts: "2026-05-05T10:01:00Z".into(),
                plan_type: Some("free".into()),
            },
            crate::scanner::parser::CodexLimitHit {
                ts: "2026-05-05T10:02:00Z".into(),
                plan_type: Some("plus".into()),
            },
        ];
        let rows = aggregate_codex_plan_daily(&snaps, &hits, 0);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].limit_hit_count, 2);
        assert!(rows[0].limit_hit_plans.contains(&"free".to_string()));
        assert!(rows[0].limit_hit_plans.contains(&"plus".to_string()));
    }
}
