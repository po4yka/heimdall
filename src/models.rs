use serde::{Deserialize, Serialize};

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
    /// Total credits consumed across all turns in this session (Amp only).
    /// `None` for non-Amp sessions.
    pub total_credits: Option<f64>,
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
    /// Pairs of (tool_use_id, extracted_arg_text) extracted from tool `input` blocks.
    /// For file tools (Edit/Write/MultiEdit/NotebookEdit/Read): the `file_path` argument.
    /// For Bash: first 120 chars of the `command` argument (truncated with trailing `…`).
    /// For all other tools: empty string (use tool name from tool_use_ids instead).
    /// Transient — not persisted to the DB turns table.
    #[allow(dead_code)]
    pub tool_inputs: Vec<(String, String)>,
    /// Abstract credits consumed by this turn (Amp provider only).
    /// `None` for all non-Amp providers.  Persisted to `turns.credits`.
    pub credits: Option<f64>,
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
    pub official_sync: OfficialSyncSummary,
    pub generated_at: String,
    /// Phase 21: cache-token breakdown and derived hit-rate metric.
    pub cache_efficiency: CacheEfficiency,
    /// Phase 3: weekly aggregation by model — always populated.
    /// The frontend buckets and filters client-side.
    pub weekly_by_model: Vec<WeeklyModelRow>,
}

/// One entry in the `weekly_by_model` array of `/api/data`.
///
/// Aggregated across all providers for `(week, model)`.
/// `week` is a `"YYYY-WW"` ISO calendar-week label.
#[derive(Debug, Clone, Default, Serialize)]
pub struct WeeklyModelRow {
    pub week: String,
    pub model: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_creation_tokens: i64,
    pub reasoning_output_tokens: i64,
    pub cost_nanos: i64,
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

#[derive(Debug, Clone, Default, Serialize)]
pub struct OfficialSyncSourceStatus {
    pub source_slug: String,
    pub source_kind: String,
    pub provider: String,
    pub status: String,
    pub fetched_at: String,
    pub record_count: i64,
    pub error_text: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct OfficialSyncRecordCount {
    pub record_type: String,
    pub count: i64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct OfficialSyncSummary {
    pub available: bool,
    pub last_sync_at: Option<String>,
    pub latest_success_at: Option<String>,
    pub total_runs: i64,
    pub total_records: i64,
    pub sources_success: i64,
    pub sources_error: i64,
    pub sources_skipped: i64,
    pub sources: Vec<OfficialSyncSourceStatus>,
    pub record_counts: Vec<OfficialSyncRecordCount>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ClaudeUsageFactor {
    pub factor_key: String,
    pub display_label: String,
    pub percent: f64,
    pub description: String,
    pub advice_text: String,
    pub display_order: i64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ClaudeUsageRunMeta {
    pub id: i64,
    pub captured_at: String,
    pub status: String,
    pub exit_code: Option<i32>,
    pub invocation_mode: String,
    pub period: String,
    pub parser_version: String,
    pub error_summary: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ClaudeUsageSnapshot {
    pub run: ClaudeUsageRunMeta,
    pub factors: Vec<ClaudeUsageFactor>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ClaudeUsageResponse {
    pub available: bool,
    pub last_run: Option<ClaudeUsageRunMeta>,
    pub latest_snapshot: Option<ClaudeUsageSnapshot>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct LiveRateWindow {
    pub used_percent: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resets_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resets_in_minutes: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_minutes: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reset_label: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct LiveProviderIdentity {
    pub provider: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_organization: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub login_method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct LiveProviderStatus {
    pub indicator: String,
    pub description: String,
    pub page_url: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ProviderCostHistoryPoint {
    pub day: String,
    pub total_tokens: i64,
    pub cost_usd: f64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ProviderCostSummary {
    pub today_tokens: i64,
    pub today_cost_usd: f64,
    pub last_30_days_tokens: i64,
    pub last_30_days_cost_usd: f64,
    pub daily: Vec<ProviderCostHistoryPoint>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct LiveProviderSourceAttempt {
    pub source: String,
    pub outcome: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct LiveProviderRecoveryAction {
    pub label: String,
    pub action_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct LiveProviderAuth {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub login_method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_backend: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_mode: Option<String>,
    pub is_authenticated: bool,
    pub is_refreshable: bool,
    pub is_source_compatible: bool,
    pub requires_relogin: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub managed_restriction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostic_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_validated_at: Option<String>,
    pub recovery_actions: Vec<LiveProviderRecoveryAction>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct LiveProviderSnapshot {
    pub provider: String,
    pub available: bool,
    pub source_used: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_attempted_source: Option<String>,
    pub resolved_via_fallback: bool,
    pub refresh_duration_ms: u64,
    pub source_attempts: Vec<LiveProviderSourceAttempt>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity: Option<LiveProviderIdentity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary: Option<LiveRateWindow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secondary: Option<LiveRateWindow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tertiary: Option<LiveRateWindow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credits: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<LiveProviderStatus>,
    pub auth: LiveProviderAuth,
    pub cost_summary: ProviderCostSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub claude_usage: Option<ClaudeUsageSnapshot>,
    pub last_refresh: String,
    pub stale: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct LiveProvidersResponse {
    pub providers: Vec<LiveProviderSnapshot>,
    pub fetched_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_provider: Option<String>,
    pub response_scope: String,
    pub cache_hit: bool,
    pub refreshed_providers: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct LiveProviderHistoryResponse {
    pub provider: String,
    pub summary: ProviderCostSummary,
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
    /// Phase 21: per-type cost breakdown (USD float, derived from integer nanos).
    pub input_cost: f64,
    pub output_cost: f64,
    pub cache_read_cost: f64,
    pub cache_write_cost: f64,
    /// Aggregated credits for this day/model bucket (Amp only).  `None` when no Amp rows.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credits: Option<f64>,
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
    /// Aggregated cost in USD (display-layer float, not stored as nanos).
    pub cost: f64,
    /// Total tokens (input + output) for this version bucket.
    pub tokens: i64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct DailyProjectRow {
    pub day: String,
    pub provider: String,
    pub project: String,
    /// Human-readable alias for `project`.  Equals `project` when no alias is configured.
    pub display_name: String,
    pub input: i64,
    pub output: i64,
    pub reasoning_output: i64,
    pub cost: f64,
    /// Aggregated credits for this day/project (Amp only).  `None` when no Amp rows.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credits: Option<f64>,
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

/// Aggregated cache-efficiency metrics for the `/api/data` response.
///
/// `cache_hit_rate` formula: `cache_read / (cache_read + input_tokens)`.
/// Denominator rationale (ROADMAP Phase 21): cache_read is the tokens we avoided
/// re-billing; input is the tokens we still paid for; their sum is the
/// "addressable" token stream — the universe of tokens that could have been
/// served from cache. A rate of 0% means no cache reuse; 50% means half the
/// addressable tokens came from cache; 100% is the theoretical maximum (cache
/// served everything). `None` when the denominator is zero (no cache activity
/// and no input tokens recorded) to distinguish "not enough data" from a
/// meaningful 0% hit-rate.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheEfficiency {
    pub cache_read_tokens: i64,
    pub cache_write_tokens: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_cost_nanos: i64,
    pub cache_write_cost_nanos: i64,
    pub input_cost_nanos: i64,
    pub output_cost_nanos: i64,
    /// `cache_read / (cache_read + input)` when denominator > 0; else `None`.
    pub cache_hit_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionRow {
    pub session_id: String,
    pub provider: String,
    pub project: String,
    /// Human-readable alias for `project`.  Equals `project` when no alias is configured.
    pub display_name: String,
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
    /// Total credits for this session (Amp only).  `None` when no Amp data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credits: Option<f64>,
}
