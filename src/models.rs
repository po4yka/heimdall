use serde::{Deserialize, Serialize};

pub const LIVE_PROVIDERS_CONTRACT_VERSION: u32 = 1;
pub const MOBILE_SNAPSHOT_CONTRACT_VERSION: u32 = 1;

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

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct TokenBreakdown {
    pub input: i64,
    pub output: i64,
    pub cache_read: i64,
    pub cache_creation: i64,
    pub reasoning_output: i64,
}

impl TokenBreakdown {
    pub fn total(&self) -> i64 {
        self.input + self.output + self.cache_read + self.cache_creation + self.reasoning_output
    }

    /// Cache hit rate = cache_read / (cache_read + cache_creation + input).
    /// Denominator includes cache_creation so the ratio reflects the fraction
    /// of input-side tokens that were served from cache, not just the reuse
    /// ratio of non-newly-cached content. The narrow formula
    /// `cr / (cr + input)` rounds to ~100% for heavy Claude Code users
    /// because `input` is only the uncached remainder (typically < 1% of the
    /// total input stream); the broad formula below reads meaningfully
    /// between 0% and 100%.
    /// Returns None when the denominator is zero (no input-side tokens yet).
    pub fn cache_hit_rate(&self) -> Option<f64> {
        let denom = self.cache_read + self.cache_creation + self.input;
        if denom <= 0 {
            return None;
        }
        Some(self.cache_read as f64 / denom as f64)
    }

    pub fn accumulate(&mut self, other: &Self) {
        self.input += other.input;
        self.output += other.output;
        self.cache_read += other.cache_read;
        self.cache_creation += other.cache_creation;
        self.reasoning_output += other.reasoning_output;
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ProviderCostHistoryPoint {
    pub day: String,
    pub total_tokens: i64,
    pub cost_usd: f64,
    #[serde(default)]
    pub breakdown: TokenBreakdown,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ProviderCostSummary {
    pub today_tokens: i64,
    pub today_cost_usd: f64,
    pub last_30_days_tokens: i64,
    pub last_30_days_cost_usd: f64,
    pub daily: Vec<ProviderCostHistoryPoint>,
    #[serde(default)]
    pub today_breakdown: TokenBreakdown,
    #[serde(default)]
    pub last_30_days_breakdown: TokenBreakdown,
    /// Cache-read fraction over the 30-day window. `None` when denominator is
    /// zero (no usage yet) so the UI can distinguish "0% hit rate" from
    /// "no data".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_hit_rate_30d: Option<f64>,
    /// Cache-read fraction for today only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_hit_rate_today: Option<f64>,
    /// Estimated dollar savings from cache reads over the 30-day window,
    /// computed as Σ cache_read_tokens × (input_price − cache_read_price) per
    /// model. None when no cache reads happened.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_savings_30d_usd: Option<f64>,
    /// Top models by 30-day cost for this provider.
    #[serde(default)]
    pub by_model: Vec<ProviderModelRow>,
    /// Top projects by 30-day cost for this provider.
    #[serde(default)]
    pub by_project: Vec<ProviderProjectRow>,
    /// Top tools by 30-day invocation count for this provider.
    #[serde(default)]
    pub by_tool: Vec<ProviderToolRow>,
    /// MCP server invocation totals for this provider (all rows, usually small).
    #[serde(default)]
    pub by_mcp: Vec<ProviderMcpRow>,
    /// Hourly activity buckets (0..=23) for the 30-day window.
    #[serde(default)]
    pub hourly_activity: Vec<ProviderHourlyBucket>,
    /// Sparse heatmap cells (day_of_week × hour) for the 30-day window.
    #[serde(default)]
    pub activity_heatmap: Vec<ProviderHeatmapCell>,
    /// Most-recent sessions for this provider, newest first.
    #[serde(default)]
    pub recent_sessions: Vec<ProviderSession>,
    /// Subagent breakdown for the 30-day window; `None` when no subagent turns.
    #[serde(default)]
    pub subagent_breakdown: Option<ProviderSubagentBreakdown>,
    /// Top versions by cost for the 30-day window.
    #[serde(default)]
    pub version_breakdown: Vec<ProviderVersionRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct ProviderHourlyBucket {
    pub hour: u8,
    pub turns: u64,
    pub cost_usd: f64,
    pub tokens: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct ProviderHeatmapCell {
    pub day_of_week: u8,
    pub hour: u8,
    pub turns: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct ProviderSession {
    pub session_id: String,
    pub display_name: String,
    pub started_at: String,
    pub duration_minutes: u64,
    pub turns: u64,
    pub cost_usd: f64,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct ProviderSubagentBreakdown {
    pub total_turns: u64,
    pub total_cost_usd: f64,
    pub session_count: u64,
    pub agent_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct ProviderVersionRow {
    pub version: String,
    pub turns: u64,
    pub sessions: u64,
    pub cost_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct ProviderModelRow {
    pub model: String,
    pub cost_usd: f64,
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
    pub cache_creation: u64,
    pub reasoning_output: u64,
    pub turns: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct ProviderProjectRow {
    pub project: String,
    pub display_name: String,
    pub cost_usd: f64,
    pub turns: u64,
    pub sessions: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct ProviderToolRow {
    pub tool_name: String,
    pub category: Option<String>,
    pub mcp_server: Option<String>,
    pub invocations: u64,
    pub errors: u64,
    pub turns_used: u64,
    pub sessions_used: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct ProviderMcpRow {
    pub server: String,
    pub invocations: u64,
    pub tools_used: u64,
    pub sessions_used: u64,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quota_suggestions: Option<LiveQuotaSuggestions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depletion_forecast: Option<DepletionForecast>,
    pub last_refresh: String,
    pub stale: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct LocalNotificationCondition {
    pub id: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    pub service_label: String,
    pub is_active: bool,
    pub activation_title: String,
    pub activation_body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery_body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub day_key: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct LocalNotificationState {
    pub generated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_threshold_usd: Option<f64>,
    pub conditions: Vec<LocalNotificationCondition>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct LiveProvidersResponse {
    pub contract_version: u32,
    pub providers: Vec<LiveProviderSnapshot>,
    pub fetched_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_provider: Option<String>,
    pub response_scope: String,
    pub cache_hit: bool,
    pub refreshed_providers: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_notification_state: Option<LocalNotificationState>,
}

pub const LIVE_MONITOR_CONTRACT_VERSION: u32 = 1;

#[derive(Debug, Clone, Default, Serialize)]
pub struct LiveMonitorFreshness {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub newest_provider_refresh: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oldest_provider_refresh: Option<String>,
    pub stale_providers: Vec<String>,
    pub has_stale_providers: bool,
    pub refresh_state: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct LiveMonitorBurnRate {
    pub tokens_per_min: f64,
    pub cost_per_hour_nanos: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tier: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct LiveMonitorProjection {
    pub projected_cost_nanos: i64,
    pub projected_tokens: i64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct LiveMonitorQuota {
    pub limit_tokens: i64,
    pub used_tokens: i64,
    pub projected_tokens: i64,
    pub current_pct: f64,
    pub projected_pct: f64,
    pub remaining_tokens: i64,
    pub current_severity: String,
    pub projected_severity: String,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq)]
pub struct DepletionForecastSignal {
    pub kind: String,
    pub title: String,
    pub used_percent: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projected_percent: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining_tokens: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining_percent: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resets_in_minutes: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pace_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq)]
pub struct DepletionForecast {
    pub primary_signal: DepletionForecastSignal,
    pub secondary_signals: Vec<DepletionForecastSignal>,
    pub summary_label: String,
    pub severity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct LiveQuotaSuggestionLevel {
    pub key: String,
    pub label: String,
    pub limit_tokens: i64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct LiveQuotaSuggestions {
    pub sample_count: usize,
    pub recommended_key: String,
    pub levels: Vec<LiveQuotaSuggestionLevel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct LiveMonitorBillingBlock {
    pub start: String,
    pub end: String,
    pub first_timestamp: String,
    pub last_timestamp: String,
    pub tokens: TokenBreakdown,
    pub cost_nanos: i64,
    pub entry_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub burn_rate: Option<LiveMonitorBurnRate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projection: Option<LiveMonitorProjection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quota: Option<LiveMonitorQuota>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct LiveMonitorContextWindow {
    pub total_input_tokens: i64,
    pub context_window_size: i64,
    pub pct: f64,
    pub severity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub captured_at: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct LiveMonitorProvider {
    pub provider: String,
    pub title: String,
    pub visual_state: String,
    pub source_label: String,
    pub warnings: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary: Option<LiveRateWindow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secondary: Option<LiveRateWindow>,
    pub today_cost_usd: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projected_weekly_spend_usd: Option<f64>,
    pub last_refresh: String,
    pub last_refresh_label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_block: Option<LiveMonitorBillingBlock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_window: Option<LiveMonitorContextWindow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recent_session: Option<ProviderSession>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quota_suggestions: Option<LiveQuotaSuggestions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depletion_forecast: Option<DepletionForecast>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct LiveMonitorResponse {
    pub contract_version: u32,
    pub generated_at: String,
    pub default_focus: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub global_issue: Option<String>,
    pub freshness: LiveMonitorFreshness,
    pub providers: Vec<LiveMonitorProvider>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct LiveProviderHistoryResponse {
    pub provider: String,
    pub summary: ProviderCostSummary,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct MobileProviderHistorySeries {
    pub provider: String,
    pub daily: Vec<ProviderCostHistoryPoint>,
    pub total_tokens: i64,
    pub total_cost_usd: f64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct MobileSnapshotTotals {
    pub today_tokens: i64,
    pub today_cost_usd: f64,
    pub last_90_days_tokens: i64,
    pub last_90_days_cost_usd: f64,
    #[serde(default)]
    pub today_breakdown: TokenBreakdown,
    #[serde(default)]
    pub last_90_days_breakdown: TokenBreakdown,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct MobileSnapshotFreshness {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub newest_provider_refresh: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oldest_provider_refresh: Option<String>,
    pub stale_providers: Vec<String>,
    pub has_stale_providers: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct MobileSnapshotEnvelope {
    pub contract_version: u32,
    pub generated_at: String,
    pub source_device: String,
    pub providers: Vec<LiveProviderSnapshot>,
    pub history_90d: Vec<MobileProviderHistorySeries>,
    pub totals: MobileSnapshotTotals,
    pub freshness: MobileSnapshotFreshness,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_hit_rate_uses_broad_denominator() {
        // Broad formula: cache_read / (cache_read + cache_creation + input)
        let bd = TokenBreakdown {
            input: 10,
            output: 99,
            cache_read: 80,
            cache_creation: 10,
            reasoning_output: 0,
        };
        // 80 / (80 + 10 + 10) = 0.8
        let rate = bd.cache_hit_rate().expect("denom > 0");
        assert!((rate - 0.8).abs() < 1e-9, "expected 0.80, got {rate}");
    }

    #[test]
    fn cache_hit_rate_fully_cached_edge_case() {
        // The pure-cache-read edge case — no fresh input and no cache
        // creation — still yields 1.0 (legitimately "fully cached"). The UI
        // can render this as "Fully cached" rather than "100.0%".
        let bd = TokenBreakdown {
            input: 0,
            output: 0,
            cache_read: 1000,
            cache_creation: 0,
            reasoning_output: 0,
        };
        let rate = bd.cache_hit_rate().expect("denom > 0");
        assert!((rate - 1.0).abs() < 1e-9);
    }

    #[test]
    fn cache_hit_rate_none_when_empty() {
        let bd = TokenBreakdown::default();
        assert!(bd.cache_hit_rate().is_none());
    }

    #[test]
    fn cache_hit_rate_drops_when_cache_creation_large() {
        // Demonstrates the fix: with the OLD narrow formula this would be
        // near 1.0 (1000 / (1000 + 50) = 95%), but with cache_creation
        // counted (big first-time prompt build) the ratio is lower.
        let bd = TokenBreakdown {
            input: 50,
            output: 0,
            cache_read: 1000,
            cache_creation: 500,
            reasoning_output: 0,
        };
        let rate = bd.cache_hit_rate().expect("denom > 0");
        // 1000 / 1550 ≈ 0.645
        assert!(rate > 0.60 && rate < 0.70, "expected ~0.645, got {rate}");
    }
}
