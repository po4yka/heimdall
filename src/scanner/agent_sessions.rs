use std::collections::{BTreeMap, HashMap};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::{Context, Result};
use regex::Regex;
use tracing::{debug, warn};
use walkdir::WalkDir;

use crate::models::{AgentSessionRecord, AgentSource, RoleConfidence};
use crate::pricing;

// ---------------------------------------------------------------------------
// Discovery configuration
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    pub projects_dirs: Vec<PathBuf>,
    pub include_task_outputs: bool,
}

impl DiscoveryConfig {
    pub fn from_projects_dirs(dirs: Vec<PathBuf>) -> Self {
        Self {
            projects_dirs: dirs,
            include_task_outputs: cfg!(unix) && !cfg!(test),
        }
    }
}

// ---------------------------------------------------------------------------
// Lazy regex statics
// ---------------------------------------------------------------------------

fn desc_prefix_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)^([a-z][a-z0-9_-]{1,30})\s*:").expect("desc_prefix_re is valid")
    })
}

fn you_are_the_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)You are the\s+([A-Za-z][A-Za-z0-9_-]{1,30})")
            .expect("you_are_the_re is valid")
    })
}

// ---------------------------------------------------------------------------
// Meta JSON sidecar
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Deserialize, Default)]
pub struct MetaJson {
    #[serde(default, rename = "agentType")]
    pub agent_type: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
struct UsageRow {
    input_tokens: i64,
    output_tokens: i64,
    cache_creation_tokens: i64,
    cache_read_tokens: i64,
}

// ---------------------------------------------------------------------------
// Role detection
// ---------------------------------------------------------------------------

pub fn detect_role(meta: Option<&MetaJson>, first_user_text: &str) -> (String, RoleConfidence) {
    if let Some(m) = meta {
        if let Some(t) = m.agent_type.as_deref().filter(|s| !s.is_empty()) {
            return (t.to_string(), RoleConfidence::Meta);
        }
        if let Some(d) = m.description.as_deref()
            && let Some(cap) = desc_prefix_re().captures(d).and_then(|c| c.get(1))
        {
            return (cap.as_str().to_string(), RoleConfidence::Meta);
        }
    }
    if let Some(cap) = you_are_the_re()
        .captures(first_user_text)
        .and_then(|c| c.get(1))
    {
        return (cap.as_str().to_string(), RoleConfidence::Prompt);
    }
    ("unknown".to_string(), RoleConfidence::Unknown)
}

// ---------------------------------------------------------------------------
// Single-file parser
// ---------------------------------------------------------------------------

pub fn parse_agent_file(
    path: &Path,
    source: AgentSource,
    project: &str,
    session_id: Option<&str>,
    meta: Option<&MetaJson>,
) -> Result<Option<AgentSessionRecord>> {
    let file = std::fs::File::open(path)
        .with_context(|| format!("open agent file: {}", path.display()))?;
    let reader = BufReader::new(file);

    // Per-requestId usage dedup (last-write-wins)
    let mut usage_map: HashMap<String, UsageRow> = HashMap::new();
    // Model frequency counter
    let mut model_counts: HashMap<String, usize> = HashMap::new();
    let mut model_last_seen: HashMap<String, usize> = HashMap::new();
    // Tool invocation counts
    let mut tool_counts: BTreeMap<String, i64> = BTreeMap::new();

    let mut ts_start: Option<String> = None;
    let mut ts_end: Option<String> = None;
    let mut first_prompt_id: Option<String> = None;
    let mut last_stop_reason: Option<String> = None;
    let mut first_user_text = String::new();
    let mut synthetic_idx: usize = 0;
    let mut line_idx: usize = 0;

    for line_result in reader.lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(e) => {
                warn!("failed to read line in {}: {}", path.display(), e);
                continue;
            }
        };
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let record: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(e) => {
                warn!(
                    "json parse error in {} line {}: {}",
                    path.display(),
                    line_idx,
                    e
                );
                line_idx += 1;
                continue;
            }
        };

        // --- Timestamps ---
        if let Some(ts) = record.get("timestamp").and_then(|v| v.as_str())
            && !ts.is_empty()
        {
            if ts_start.as_deref().is_none_or(|s| ts < s) {
                ts_start = Some(ts.to_string());
            }
            if ts_end.as_deref().is_none_or(|s| ts > s) {
                ts_end = Some(ts.to_string());
            }
        }

        // --- First promptId ---
        if first_prompt_id.is_none()
            && let Some(pid) = record.get("promptId").and_then(|v| v.as_str())
            && !pid.is_empty()
        {
            first_prompt_id = Some(pid.to_string());
        }

        // --- First user text (for role detection) ---
        if first_user_text.is_empty() {
            let role_str = record.get("type").and_then(|v| v.as_str()).unwrap_or("");
            let msg_role = record
                .get("message")
                .and_then(|m| m.get("role"))
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if role_str == "user" || msg_role == "user" || role_str == "system" {
                let content = record
                    .get("message")
                    .and_then(|m| m.get("content"))
                    .unwrap_or(&serde_json::Value::Null);

                let text = match content {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Array(arr) => arr
                        .iter()
                        .filter_map(|b| {
                            if b.get("type").and_then(|t| t.as_str()) == Some("text") {
                                b.get("text").and_then(|t| t.as_str())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n"),
                    _ => String::new(),
                };

                if !text.is_empty() {
                    first_user_text = text.chars().take(5000).collect();
                }
            }
        }

        // --- Assistant message with usage ---
        if let Some(message) = record.get("message") {
            // Model tracking
            if let Some(model) = message.get("model").and_then(|v| v.as_str())
                && !model.is_empty()
            {
                *model_counts.entry(model.to_string()).or_insert(0) += 1;
                model_last_seen.insert(model.to_string(), line_idx);
            }

            // stop_reason
            if let Some(sr) = message.get("stop_reason").and_then(|v| v.as_str())
                && !sr.is_empty()
            {
                last_stop_reason = Some(sr.to_string());
            }

            // Usage
            if let Some(usage) = message.get("usage") {
                let input = usage
                    .get("input_tokens")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                let output = usage
                    .get("output_tokens")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                let cache_creation = usage
                    .get("cache_creation_input_tokens")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                let cache_read = usage
                    .get("cache_read_input_tokens")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);

                let request_id = record
                    .get("requestId")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| {
                        let key = format!("__no_rid__{}", synthetic_idx);
                        synthetic_idx += 1;
                        key
                    });

                usage_map.insert(
                    request_id,
                    UsageRow {
                        input_tokens: input,
                        output_tokens: output,
                        cache_creation_tokens: cache_creation,
                        cache_read_tokens: cache_read,
                    },
                );
            }

            // Tool uses in content blocks
            if let Some(content) = message.get("content").and_then(|v| v.as_array()) {
                for block in content {
                    if block.get("type").and_then(|t| t.as_str()) == Some("tool_use")
                        && let Some(tool_name) = block.get("name").and_then(|n| n.as_str())
                    {
                        *tool_counts.entry(tool_name.to_string()).or_insert(0) += 1;
                    }
                }
            }
        }

        line_idx += 1;
    }

    // Nothing parsed
    if usage_map.is_empty() && ts_start.is_none() {
        debug!(path = %path.display(), "agent file empty or no events");
        return Ok(None);
    }

    // Sum tokens
    let mut total_input: i64 = 0;
    let mut total_output: i64 = 0;
    let mut total_cache_creation: i64 = 0;
    let mut total_cache_read: i64 = 0;
    let mut api_calls: i64 = 0;

    for (key, row) in &usage_map {
        total_input += row.input_tokens;
        total_output += row.output_tokens;
        total_cache_creation += row.cache_creation_tokens;
        total_cache_read += row.cache_read_tokens;
        if !key.starts_with("__no_rid__") {
            api_calls += 1;
        }
    }

    let total_tokens = total_input + total_cache_creation + total_cache_read + total_output;

    // Sanity gate
    if total_tokens <= 0 || total_tokens >= 500_000_000 {
        warn!(
            path = %path.display(),
            total_tokens,
            "agent file failed sanity gate"
        );
        return Ok(None);
    }

    // Most-frequent model (tie-break: latest seen)
    let best_model = model_counts
        .iter()
        .max_by_key(|(model, count)| {
            let last = model_last_seen.get(*model).copied().unwrap_or(0);
            (*count, last)
        })
        .map(|(m, _)| m.clone())
        .unwrap_or_default();

    // Cost
    let cost_nanos = pricing::calc_cost_nanos(
        &best_model,
        total_input,
        total_output,
        total_cache_read,
        total_cache_creation,
    );

    // Duration
    let duration_s = match (&ts_start, &ts_end) {
        (Some(start), Some(end)) => {
            let s = chrono::DateTime::parse_from_rfc3339(start)
                .map(|dt| dt.timestamp())
                .unwrap_or(0);
            let e = chrono::DateTime::parse_from_rfc3339(end)
                .map(|dt| dt.timestamp())
                .unwrap_or(0);
            (e - s).max(0)
        }
        _ => 0,
    };

    // ts_start_epoch
    let ts_start_str = ts_start.clone().unwrap_or_default();
    let ts_start_epoch = chrono::DateTime::parse_from_rfc3339(&ts_start_str)
        .map(|dt| dt.timestamp_millis())
        .unwrap_or(0);

    // Tool uses total
    let tool_uses: i64 = tool_counts.values().sum();

    // Tools JSON (BTreeMap is already sorted)
    let tools_json = serde_json::to_string(&tool_counts).unwrap_or_else(|_| "{}".to_string());

    // Role detection
    let (role, role_confidence) = detect_role(meta, &first_user_text);

    // Description
    let description = meta
        .and_then(|m| m.description.as_deref())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            let trimmed = first_user_text.trim();
            trimmed
                .chars()
                .take(120)
                .collect::<String>()
                .trim()
                .to_string()
        });

    let agent_id = path
        .file_stem()
        .and_then(|s| s.to_str())
        .and_then(|s| s.strip_prefix("agent-"))
        .unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
        })
        .to_string();

    Ok(Some(AgentSessionRecord {
        agent_id,
        source,
        project: project.to_string(),
        session_id: session_id.map(|s| s.to_string()),
        ts_start: ts_start_str,
        ts_start_epoch,
        duration_s,
        role,
        role_confidence,
        description,
        model: best_model,
        input_tokens: total_input,
        cache_create_tokens: total_cache_creation,
        cache_read_tokens: total_cache_read,
        output_tokens: total_output,
        total_tokens,
        cost_nanos,
        api_calls,
        tool_uses,
        tools_json,
        prompt_id: first_prompt_id,
        stop_reason: last_stop_reason,
        source_path: path.to_string_lossy().to_string(),
    }))
}

// ---------------------------------------------------------------------------
// Task output discovery (unix only)
// ---------------------------------------------------------------------------

/// Cheap content sniff: reads up to 64 bytes from `path` and returns true iff
/// the first non-whitespace byte is `{` — i.e. this looks like an agent JSONL
/// record. Returns false for shell outputs (`>`, plain text) and for unreadable
/// files. Used to filter out non-agent `.output` files written by
/// `Bash(run_in_background=true)` that happen to share the `tasks/<id>.output`
/// path convention with the legacy task-output JSONL contract.
fn looks_like_jsonl(path: &std::path::Path) -> bool {
    use std::io::Read;
    let Ok(mut f) = std::fs::File::open(path) else {
        return false;
    };
    let mut buf = [0u8; 64];
    let Ok(n) = f.read(&mut buf) else {
        return false;
    };
    buf[..n].iter().find(|b| !b.is_ascii_whitespace()).copied() == Some(b'{')
}

#[cfg(unix)]
fn task_output_roots() -> Vec<PathBuf> {
    // SAFETY: POSIX `getuid()` has no preconditions, does not dereference
    // pointers, and only reads the current process' real UID.
    let uid = unsafe { libc::getuid() };
    let candidates = [
        format!("/private/tmp/claude-{}", uid),
        format!("/tmp/claude-{}", uid),
    ];

    let mut seen_canonical: std::collections::HashSet<PathBuf> = std::collections::HashSet::new();
    let mut result = Vec::new();

    for candidate in &candidates {
        let p = PathBuf::from(candidate);
        if !p.exists() {
            continue;
        }
        let canonical = p.canonicalize().unwrap_or_else(|_| p.clone());
        if seen_canonical.insert(canonical) {
            result.push(p);
        }
    }
    result
}

// ---------------------------------------------------------------------------
// File-level discovery metadata (for incremental ingest)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct DiscoveredAgentFile {
    pub path: PathBuf,
    pub source: AgentSource,
    pub project: String,
    pub session_id: Option<String>,
    pub meta_path: Option<PathBuf>,
}

/// Discover all agent JSONL / task-output files without parsing their contents.
/// Callers can then do mtime checks against `processed_files` and only call
/// `parse_one` for files that have changed.
pub fn discover_files(cfg: &DiscoveryConfig) -> Vec<DiscoveredAgentFile> {
    let mut files = Vec::new();

    // --- Subagent JSONL files ---
    for root in &cfg.projects_dirs {
        if !root.exists() {
            debug!(root = %root.display(), "projects root does not exist, skipping");
            continue;
        }

        for entry in WalkDir::new(root)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| match e {
                Ok(e) => Some(e),
                Err(err) => {
                    warn!("walkdir error: {}", err);
                    None
                }
            })
        {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            if !file_name.starts_with("agent-") || !file_name.ends_with(".jsonl") {
                continue;
            }

            let parent = match path.parent() {
                Some(p) => p,
                None => continue,
            };
            if parent.file_name().and_then(|n| n.to_str()) != Some("subagents") {
                continue;
            }

            // project_key = grandgrandparent, session_id = grandparent
            let session_dir = match parent.parent() {
                Some(p) => p,
                None => continue,
            };
            let project_dir = match session_dir.parent() {
                Some(p) => p,
                None => continue,
            };

            let session_id = session_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            let project = project_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            // Sibling meta.json path (may not exist yet)
            let agent_stem = file_name.strip_suffix(".jsonl").unwrap_or(file_name);
            let meta_candidate = parent.join(format!("{}.meta.json", agent_stem));
            let meta_path = if meta_candidate.exists() {
                Some(meta_candidate)
            } else {
                None
            };

            files.push(DiscoveredAgentFile {
                path: path.to_path_buf(),
                source: AgentSource::Subagent,
                project,
                session_id: Some(session_id),
                meta_path,
            });
        }
    }

    // --- Task output files (unix only) ---
    #[cfg(unix)]
    if cfg.include_task_outputs {
        for task_root in task_output_roots() {
            if !task_root.exists() {
                debug!(root = %task_root.display(), "task root does not exist, skipping");
                continue;
            }

            for entry in WalkDir::new(&task_root)
                .follow_links(false)
                .into_iter()
                .filter_map(|e| match e {
                    Ok(e) => Some(e),
                    Err(err) => {
                        warn!("walkdir error in task root: {}", err);
                        None
                    }
                })
            {
                if !entry.file_type().is_file() {
                    continue;
                }
                let path = entry.path();
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                if !file_name.ends_with(".output") {
                    continue;
                }

                let parent = match path.parent() {
                    Some(p) => p,
                    None => continue,
                };
                if parent.file_name().and_then(|n| n.to_str()) != Some("tasks") {
                    continue;
                }

                // Modern Claude Code writes background-bash stdout to the same
                // `tasks/<id>.output` path that legacy task-agents used for JSONL.
                // Skip files whose first non-whitespace byte isn't `{` so we
                // don't burn the JSONL parser (and the user's logs) on shell
                // output.
                if !looks_like_jsonl(path) {
                    continue;
                }

                let session_dir = match parent.parent() {
                    Some(p) => p,
                    None => continue,
                };
                let project_dir = match session_dir.parent() {
                    Some(p) => p,
                    None => continue,
                };

                let session_id = session_dir
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                let project = project_dir
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();

                files.push(DiscoveredAgentFile {
                    path: path.to_path_buf(),
                    source: AgentSource::Task,
                    project,
                    session_id: Some(session_id),
                    meta_path: None,
                });
            }
        }
    }

    files
}

/// Parse a single discovered agent file, loading its sibling meta.json if present.
pub fn parse_one(file: &DiscoveredAgentFile) -> Result<Option<AgentSessionRecord>> {
    let meta: Option<MetaJson> = file.meta_path.as_ref().and_then(|p| {
        std::fs::read_to_string(p)
            .ok()
            .and_then(|s| match serde_json::from_str::<MetaJson>(&s) {
                Ok(m) => Some(m),
                Err(e) => {
                    warn!(path = %p.display(), "failed to parse meta.json: {}", e);
                    None
                }
            })
    });
    parse_agent_file(
        &file.path,
        file.source.clone(),
        &file.project,
        file.session_id.as_deref(),
        meta.as_ref(),
    )
}

// ---------------------------------------------------------------------------
// Main discovery entry point
// ---------------------------------------------------------------------------

pub fn discover_and_parse(cfg: &DiscoveryConfig) -> Vec<AgentSessionRecord> {
    discover_files(cfg)
        .iter()
        .filter_map(|f| match parse_one(f) {
            Ok(Some(rec)) => Some(rec),
            Ok(None) => None,
            Err(e) => {
                warn!("agent_sessions: parse failed for {:?}: {:#}", f.path, e);
                None
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn write_jsonl(dir: &Path, name: &str, lines: &[&str]) -> PathBuf {
        let path = dir.join(name);
        let mut f = std::fs::File::create(&path).unwrap();
        for line in lines {
            writeln!(f, "{}", line).unwrap();
        }
        path
    }

    // Test helper: builds a single agent-session JSONL event line; argument count
    // mirrors the JSONL schema fields rather than business-logic boundaries.
    #[allow(clippy::too_many_arguments)]
    fn make_event(
        ts: &str,
        request_id: &str,
        model: &str,
        input: i64,
        output: i64,
        cache_create: i64,
        cache_read: i64,
        stop_reason: &str,
    ) -> String {
        serde_json::json!({
            "timestamp": ts,
            "requestId": request_id,
            "type": "assistant",
            "message": {
                "role": "assistant",
                "model": model,
                "stop_reason": stop_reason,
                "usage": {
                    "input_tokens": input,
                    "output_tokens": output,
                    "cache_creation_input_tokens": cache_create,
                    "cache_read_input_tokens": cache_read
                },
                "content": []
            }
        })
        .to_string()
    }

    fn make_event_with_tools(
        ts: &str,
        request_id: &str,
        model: &str,
        input: i64,
        output: i64,
        tools: &[&str],
    ) -> String {
        let content: Vec<serde_json::Value> = tools
            .iter()
            .map(|t| serde_json::json!({ "type": "tool_use", "name": t }))
            .collect();
        serde_json::json!({
            "timestamp": ts,
            "requestId": request_id,
            "type": "assistant",
            "message": {
                "role": "assistant",
                "model": model,
                "stop_reason": "tool_use",
                "usage": {
                    "input_tokens": input,
                    "output_tokens": output,
                    "cache_creation_input_tokens": 0i64,
                    "cache_read_input_tokens": 0i64
                },
                "content": content
            }
        })
        .to_string()
    }

    // -----------------------------------------------------------------------
    // detect_role tests
    // -----------------------------------------------------------------------

    #[test]
    fn detect_role_uses_meta_agent_type_first() {
        let meta = MetaJson {
            agent_type: Some("explorer".to_string()),
            description: Some("reviewer: some description".to_string()),
        };
        let (role, conf) = detect_role(Some(&meta), "You are the Researcher");
        assert_eq!(role, "explorer");
        assert_eq!(conf, RoleConfidence::Meta);
    }

    #[test]
    fn detect_role_falls_to_meta_description_prefix() {
        let meta = MetaJson {
            agent_type: None,
            description: Some("reviewer: code review specialist".to_string()),
        };
        let (role, conf) = detect_role(Some(&meta), "");
        assert_eq!(role, "reviewer");
        assert_eq!(conf, RoleConfidence::Meta);
    }

    #[test]
    fn detect_role_falls_to_prompt_regex() {
        let (role, conf) = detect_role(None, "You are the Researcher tasked with finding bugs.");
        assert_eq!(role, "Researcher");
        assert_eq!(conf, RoleConfidence::Prompt);
    }

    #[test]
    fn detect_role_returns_unknown() {
        let (role, conf) = detect_role(None, "Do some work.");
        assert_eq!(role, "unknown");
        assert_eq!(conf, RoleConfidence::Unknown);
    }

    #[test]
    fn detect_role_case_insensitive() {
        let (role, conf) = detect_role(None, "you are the QA-Engineer responsible for testing.");
        assert_eq!(role, "QA-Engineer");
        assert_eq!(conf, RoleConfidence::Prompt);
    }

    // -----------------------------------------------------------------------
    // parse_agent_file tests
    // -----------------------------------------------------------------------

    #[test]
    fn parse_agent_file_dedup_by_request_id() {
        let dir = TempDir::new().unwrap();
        // Two events with same requestId — second wins
        let e1 = make_event(
            "2024-01-01T00:00:00Z",
            "req-1",
            "claude-3-5-sonnet",
            100,
            50,
            0,
            0,
            "end_turn",
        );
        let e2 = make_event(
            "2024-01-01T00:01:00Z",
            "req-1",
            "claude-3-5-sonnet",
            200,
            100,
            0,
            0,
            "end_turn",
        );
        let path = write_jsonl(dir.path(), "agent-abc.jsonl", &[&e1, &e2]);
        let rec = parse_agent_file(&path, AgentSource::Subagent, "proj", Some("sess"), None)
            .unwrap()
            .unwrap();
        // last-write-wins: input=200, output=100
        assert_eq!(rec.input_tokens, 200);
        assert_eq!(rec.output_tokens, 100);
        assert_eq!(rec.api_calls, 1); // only one unique real requestId
    }

    #[test]
    fn parse_agent_file_sanity_gate_rejects_zero() {
        let dir = TempDir::new().unwrap();
        let e = make_event(
            "2024-01-01T00:00:00Z",
            "req-1",
            "claude-3-5-sonnet",
            0,
            0,
            0,
            0,
            "end_turn",
        );
        let path = write_jsonl(dir.path(), "agent-abc.jsonl", &[&e]);
        let result =
            parse_agent_file(&path, AgentSource::Subagent, "proj", Some("sess"), None).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn parse_agent_file_sanity_gate_rejects_huge() {
        let dir = TempDir::new().unwrap();
        let e = make_event(
            "2024-01-01T00:00:00Z",
            "req-1",
            "claude-3-5-sonnet",
            600_000_000,
            0,
            0,
            0,
            "end_turn",
        );
        let path = write_jsonl(dir.path(), "agent-abc.jsonl", &[&e]);
        let result =
            parse_agent_file(&path, AgentSource::Subagent, "proj", Some("sess"), None).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn parse_agent_file_collects_tools() {
        let dir = TempDir::new().unwrap();
        let e1 = make_event_with_tools(
            "2024-01-01T00:00:00Z",
            "req-1",
            "claude-3-5-sonnet",
            1000,
            500,
            &["Bash", "Read"],
        );
        let e2 = make_event_with_tools(
            "2024-01-01T00:01:00Z",
            "req-2",
            "claude-3-5-sonnet",
            1000,
            500,
            &["Bash"],
        );
        let path = write_jsonl(dir.path(), "agent-abc.jsonl", &[&e1, &e2]);
        let rec = parse_agent_file(&path, AgentSource::Subagent, "proj", Some("sess"), None)
            .unwrap()
            .unwrap();
        let tools: serde_json::Value = serde_json::from_str(&rec.tools_json).unwrap();
        assert_eq!(tools["Bash"], 2);
        assert_eq!(tools["Read"], 1);
        assert_eq!(rec.tool_uses, 3);
    }

    #[test]
    fn parse_agent_file_picks_most_frequent_model() {
        let dir = TempDir::new().unwrap();
        let e1 = make_event(
            "2024-01-01T00:00:00Z",
            "req-1",
            "model-A",
            1000,
            500,
            0,
            0,
            "end_turn",
        );
        let e2 = make_event(
            "2024-01-01T00:01:00Z",
            "req-2",
            "model-A",
            1000,
            500,
            0,
            0,
            "end_turn",
        );
        let e3 = make_event(
            "2024-01-01T00:02:00Z",
            "req-3",
            "model-B",
            1000,
            500,
            0,
            0,
            "end_turn",
        );
        let path = write_jsonl(dir.path(), "agent-abc.jsonl", &[&e1, &e2, &e3]);
        let rec = parse_agent_file(&path, AgentSource::Subagent, "proj", Some("sess"), None)
            .unwrap()
            .unwrap();
        assert_eq!(rec.model, "model-A");
    }

    #[test]
    fn parse_agent_file_captures_first_prompt_id_and_last_stop_reason() {
        let dir = TempDir::new().unwrap();
        let e1 = serde_json::json!({
            "timestamp": "2024-01-01T00:00:00Z",
            "requestId": "req-1",
            "promptId": "prompt-first",
            "type": "assistant",
            "message": {
                "model": "claude-3-5-sonnet",
                "stop_reason": "tool_use",
                "usage": { "input_tokens": 1000i64, "output_tokens": 500i64,
                    "cache_creation_input_tokens": 0i64, "cache_read_input_tokens": 0i64 },
                "content": []
            }
        })
        .to_string();
        let e2 = serde_json::json!({
            "timestamp": "2024-01-01T00:01:00Z",
            "requestId": "req-2",
            "promptId": "prompt-second",
            "type": "assistant",
            "message": {
                "model": "claude-3-5-sonnet",
                "stop_reason": "end_turn",
                "usage": { "input_tokens": 500i64, "output_tokens": 200i64,
                    "cache_creation_input_tokens": 0i64, "cache_read_input_tokens": 0i64 },
                "content": []
            }
        })
        .to_string();
        let path = write_jsonl(dir.path(), "agent-abc.jsonl", &[&e1, &e2]);
        let rec = parse_agent_file(&path, AgentSource::Subagent, "proj", Some("sess"), None)
            .unwrap()
            .unwrap();
        assert_eq!(rec.prompt_id.as_deref(), Some("prompt-first"));
        assert_eq!(rec.stop_reason.as_deref(), Some("end_turn"));
    }

    #[test]
    fn parse_agent_file_computes_duration_from_first_to_last_timestamp() {
        let dir = TempDir::new().unwrap();
        let e1 = make_event(
            "2024-01-01T00:00:00Z",
            "req-1",
            "claude-3-5-sonnet",
            1000,
            500,
            0,
            0,
            "end_turn",
        );
        let e2 = make_event(
            "2024-01-01T00:02:30Z",
            "req-2",
            "claude-3-5-sonnet",
            1000,
            500,
            0,
            0,
            "end_turn",
        );
        let path = write_jsonl(dir.path(), "agent-abc.jsonl", &[&e1, &e2]);
        let rec = parse_agent_file(&path, AgentSource::Subagent, "proj", Some("sess"), None)
            .unwrap()
            .unwrap();
        assert_eq!(rec.duration_s, 150); // 2m30s = 150s
    }

    // -----------------------------------------------------------------------
    // discover_and_parse tests
    // -----------------------------------------------------------------------

    #[test]
    fn discover_and_parse_finds_subagent_files() {
        let dir = TempDir::new().unwrap();
        // Structure: <root>/myproj/sess1/subagents/agent-abc.jsonl
        let subagents_dir = dir.path().join("myproj").join("sess1").join("subagents");
        std::fs::create_dir_all(&subagents_dir).unwrap();

        let agent_jsonl = subagents_dir.join("agent-abc.jsonl");
        let e = make_event(
            "2024-06-01T10:00:00Z",
            "req-1",
            "claude-3-5-sonnet",
            5000,
            2000,
            0,
            0,
            "end_turn",
        );
        std::fs::write(&agent_jsonl, format!("{}\n", e)).unwrap();

        // Sibling meta.json with agentType="qa"
        let meta_path = subagents_dir.join("agent-abc.meta.json");
        std::fs::write(
            &meta_path,
            r#"{"agentType": "qa", "description": "QA specialist"}"#,
        )
        .unwrap();

        let cfg = DiscoveryConfig {
            projects_dirs: vec![dir.path().to_path_buf()],
            include_task_outputs: false,
        };
        let records = discover_and_parse(&cfg);
        assert_eq!(records.len(), 1);
        let rec = &records[0];
        assert_eq!(rec.role, "qa");
        assert_eq!(rec.role_confidence, RoleConfidence::Meta);
        assert_eq!(rec.project, "myproj");
        assert_eq!(rec.session_id.as_deref(), Some("sess1"));
        assert_eq!(rec.agent_id, "abc");
    }

    #[test]
    fn discover_and_parse_skips_unreadable_dirs() {
        let cfg = DiscoveryConfig {
            projects_dirs: vec![PathBuf::from("/nonexistent/path/that/does/not/exist")],
            include_task_outputs: false,
        };
        let records = discover_and_parse(&cfg);
        assert!(records.is_empty());
    }

    #[test]
    fn discover_and_parse_handles_empty_file_gracefully() {
        let dir = TempDir::new().unwrap();
        let subagents_dir = dir.path().join("proj").join("sess").join("subagents");
        std::fs::create_dir_all(&subagents_dir).unwrap();
        // Write empty file
        std::fs::write(subagents_dir.join("agent-xyz.jsonl"), "").unwrap();

        let cfg = DiscoveryConfig {
            projects_dirs: vec![dir.path().to_path_buf()],
            include_task_outputs: false,
        };
        let records = discover_and_parse(&cfg);
        assert!(records.is_empty());
    }

    #[test]
    fn looks_like_jsonl_accepts_brace_start() {
        let dir = TempDir::new().unwrap();
        let p = dir.path().join("a.output");
        std::fs::write(&p, r#"{"timestamp":"2026-05-01T10:00:00Z"}"#).unwrap();
        assert!(looks_like_jsonl(&p));
    }

    #[test]
    fn looks_like_jsonl_accepts_brace_after_whitespace() {
        let dir = TempDir::new().unwrap();
        let p = dir.path().join("a.output");
        std::fs::write(&p, "   \n\t {\"k\":1}").unwrap();
        assert!(looks_like_jsonl(&p));
    }

    #[test]
    fn looks_like_jsonl_rejects_shell_output() {
        let dir = TempDir::new().unwrap();
        let p = dir.path().join("a.output");
        std::fs::write(&p, "> Task :feature:digest:bundle\nw: ATTENTION!").unwrap();
        assert!(!looks_like_jsonl(&p));
    }

    #[test]
    fn looks_like_jsonl_rejects_empty_file() {
        let dir = TempDir::new().unwrap();
        let p = dir.path().join("a.output");
        std::fs::write(&p, "").unwrap();
        assert!(!looks_like_jsonl(&p));
    }

    #[test]
    fn looks_like_jsonl_rejects_missing_file() {
        assert!(!looks_like_jsonl(std::path::Path::new(
            "/nonexistent/missing.output"
        )));
    }
}
