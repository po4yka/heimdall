use serde::Serialize;

#[derive(Debug, Clone, Default)]
pub struct Session {
    pub session_id: String,
    pub provider: String,
    pub project_name: String,
    pub project_slug: String,
    pub first_timestamp: String,
    pub last_timestamp: String,
    pub git_branch: String,
    pub model: Option<String>,
    pub entrypoint: String,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_cache_read: i64,
    pub total_cache_creation: i64,
    pub total_reasoning_output: i64,
    pub total_estimated_cost_nanos: i64,
    pub turn_count: i64,
    pub pricing_version: String,
    pub billing_mode: String,
    pub cost_confidence: String,
    pub title: Option<String>,
    /// One-shot classification: `None` if session has no edit activity
    /// (unclassifiable), `Some(true)` if no rework cycle detected,
    /// `Some(false)` if an Edit→Bash→Edit pattern was found.
    pub one_shot: Option<bool>,
}

#[derive(Debug, Clone, Default)]
pub struct Turn {
    pub session_id: String,
    pub provider: String,
    pub timestamp: String,
    pub model: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_creation_tokens: i64,
    pub reasoning_output_tokens: i64,
    pub estimated_cost_nanos: i64,
    pub tool_name: Option<String>,
    pub cwd: String,
    pub message_id: String,
    pub service_tier: Option<String>,
    pub inference_geo: Option<String>,
    pub is_subagent: bool,
    pub agent_id: Option<String>,
    pub source_path: String,
    pub version: Option<String>,
    pub pricing_version: String,
    pub pricing_model: String,
    pub billing_mode: String,
    pub cost_confidence: String,
    /// Task category slug from `scanner::classifier::TaskCategory::as_str()`.
    /// Empty string means unclassified (legacy row or Default-constructed turn).
    pub category: String,
    /// All tool names from content blocks (transient, not persisted to turns table).
    #[allow(dead_code)]
    pub all_tools: Vec<String>,
    /// Pairs of (tool_use_id, tool_name) from content blocks (transient, not persisted to turns table).
    pub tool_use_ids: Vec<(String, String)>,
}

#[derive(Debug, Clone, Default)]
pub struct SessionMeta {
    pub session_id: String,
    pub provider: String,
    pub project_name: String,
    pub project_slug: String,
    pub first_timestamp: String,
    pub last_timestamp: String,
    pub git_branch: String,
    pub model: Option<String>,
    pub entrypoint: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ScanResult {
    pub new: usize,
    pub updated: usize,
    pub skipped: usize,
    pub turns: usize,
    pub sessions: usize,
}

/// One cell in the 7×24 activity heatmap.
/// `dow` = 0..6 (Sunday=0, matching SQLite strftime '%w').
/// `hour` = 0..23.
#[derive(Debug, Clone, Default, Serialize)]
pub struct HeatmapCell {
    pub dow: i64,
    pub hour: i64,
    pub cost_nanos: i64,
    pub call_count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DashboardData {
    pub all_models: Vec<String>,
    pub provider_breakdown: Vec<ProviderSummary>,
    pub confidence_breakdown: Vec<ConfidenceSummary>,
    pub billing_mode_breakdown: Vec<BillingModeSummary>,
    pub daily_by_model: Vec<DailyModelRow>,
    pub sessions_all: Vec<SessionRow>,
    pub subagent_summary: SubagentSummary,
    pub entrypoint_breakdown: Vec<EntrypointSummary>,
    pub service_tiers: Vec<ServiceTierSummary>,
    pub tool_summary: Vec<ToolSummary>,
    pub mcp_summary: Vec<McpServerSummary>,
    pub hourly_distribution: Vec<HourlyRow>,
    pub git_branch_summary: Vec<BranchSummary>,
    pub version_summary: Vec<VersionSummary>,
    pub daily_by_project: Vec<DailyProjectRow>,
    pub openai_reconciliation: Option<OpenAiReconciliation>,
    pub generated_at: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct SubagentSummary {
    pub parent_turns: i64,
    pub parent_input: i64,
    pub parent_output: i64,
    pub subagent_turns: i64,
    pub subagent_input: i64,
    pub subagent_output: i64,
    pub unique_agents: i64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ProviderSummary {
    pub provider: String,
    pub sessions: i64,
    pub turns: i64,
    pub input: i64,
    pub output: i64,
    pub cache_read: i64,
    pub cache_creation: i64,
    pub reasoning_output: i64,
    pub cost: f64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ConfidenceSummary {
    pub confidence: String,
    pub turns: i64,
    pub cost: f64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct BillingModeSummary {
    pub billing_mode: String,
    pub turns: i64,
    pub cost: f64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct OpenAiReconciliation {
    pub available: bool,
    pub lookback_days: i64,
    pub start_date: String,
    pub end_date: String,
    pub estimated_local_cost: f64,
    pub api_usage_cost: f64,
    pub api_input_tokens: i64,
    pub api_output_tokens: i64,
    pub api_cached_input_tokens: i64,
    pub api_requests: i64,
    pub delta_cost: f64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DailyModelRow {
    pub day: String,
    pub provider: String,
    pub model: String,
    pub input: i64,
    pub output: i64,
    pub cache_read: i64,
    pub cache_creation: i64,
    pub reasoning_output: i64,
    pub turns: i64,
    pub cost: f64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct EntrypointSummary {
    pub provider: String,
    pub entrypoint: String,
    pub sessions: i64,
    pub turns: i64,
    pub input: i64,
    pub output: i64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ServiceTierSummary {
    pub provider: String,
    pub service_tier: String,
    pub inference_geo: String,
    pub turns: i64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ToolSummary {
    pub provider: String,
    pub tool_name: String,
    pub category: String,
    pub mcp_server: Option<String>,
    pub invocations: i64,
    pub turns_used: i64,
    pub sessions_used: i64,
    pub errors: i64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct McpServerSummary {
    pub provider: String,
    pub server: String,
    pub tools_used: i64,
    pub invocations: i64,
    pub sessions_used: i64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct HourlyRow {
    pub provider: String,
    pub hour: i64,
    pub turns: i64,
    pub input: i64,
    pub output: i64,
    pub reasoning_output: i64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct BranchSummary {
    pub provider: String,
    pub branch: String,
    pub sessions: i64,
    pub turns: i64,
    pub input: i64,
    pub output: i64,
    pub reasoning_output: i64,
    pub cost: f64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct VersionSummary {
    pub provider: String,
    pub version: String,
    pub turns: i64,
    pub sessions: i64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct DailyProjectRow {
    pub day: String,
    pub provider: String,
    pub project: String,
    pub input: i64,
    pub output: i64,
    pub reasoning_output: i64,
    pub cost: f64,
}

/// A single tool invocation with its share of the parent turn's cost.
///
/// Cost is split evenly across all tool invocations in the turn using integer
/// arithmetic: `cost_per_event = turn.estimated_cost_nanos / n` for all events,
/// with `remainder = turn.estimated_cost_nanos % n` added to the first event so
/// the sum is preserved exactly.
///
/// Note: turns with zero tool invocations do NOT produce `ToolEvent` rows, so
/// `SUM(cost_nanos) FROM tool_events WHERE session_id = X` will under-count relative
/// to `SUM(estimated_cost_nanos) FROM turns WHERE session_id = X` for sessions that
/// contain any tool-free turns.
#[derive(Debug, Clone, Default)]
pub struct ToolEvent {
    pub dedup_key: String,
    pub ts_epoch: i64,
    pub session_id: String,
    pub provider: String,
    pub project: String,
    /// One of: subagent | skill | mcp | bash | file | other
    pub kind: String,
    /// Tool name, file path, bash command, etc.
    pub value: String,
    pub cost_nanos: i64,
    pub source_path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionRow {
    pub session_id: String,
    pub provider: String,
    pub project: String,
    pub last: String,
    pub last_date: String,
    pub duration_min: f64,
    pub model: String,
    pub turns: i64,
    pub input: i64,
    pub output: i64,
    pub cache_read: i64,
    pub cache_creation: i64,
    pub reasoning_output: i64,
    pub cost: f64,
    pub is_billable: bool,
    pub pricing_version: String,
    pub billing_mode: String,
    pub cost_confidence: String,
    pub subagent_count: i64,
    pub subagent_turns: i64,
    pub title: Option<String>,
    pub cache_hit_ratio: f64,
    pub tokens_per_min: f64,
}
