export interface SubagentSummary {
  parent_turns: number;
  parent_input: number;
  parent_output: number;
  subagent_turns: number;
  subagent_input: number;
  subagent_output: number;
  unique_agents: number;
}

export interface EntrypointSummary {
  provider: string;
  entrypoint: string;
  sessions: number;
  turns: number;
  input: number;
  output: number;
}

export interface ServiceTierSummary {
  provider: string;
  service_tier: string;
  inference_geo: string;
  turns: number;
}

export interface DailyModelRow {
  day: string;
  provider: string;
  model: string;
  input: number;
  output: number;
  cache_read: number;
  cache_creation: number;
  reasoning_output: number;
  turns: number;
  cost: number;
  input_cost: number;
  output_cost: number;
  cache_read_cost: number;
  cache_write_cost: number;
  credits?: number | null;
}

export interface SessionRow {
  session_id: string;
  provider: string;
  project: string;
  display_name: string;
  last: string;
  last_date: string;
  duration_min: number;
  model: string;
  turns: number;
  input: number;
  output: number;
  cache_read: number;
  cache_creation: number;
  reasoning_output: number;
  cost: number;
  is_billable: boolean;
  pricing_version: string;
  billing_mode: string;
  cost_confidence: string;
  subagent_count: number;
  subagent_turns: number;
  title: string | null;
  cache_hit_ratio: number;
  tokens_per_min: number;
  credits?: number | null;
}

export interface ToolSummary {
  provider: string;
  tool_name: string;
  category: string;
  mcp_server: string | null;
  invocations: number;
  turns_used: number;
  sessions_used: number;
  errors: number;
}

export interface ToolErrorRow {
  timestamp: string;
  session_id: string;
  project: string;
  model: string;
  provider: string;
  tool_name: string;
  mcp_server: string | null;
  tool_input: string | null;
  error_text: string | null;
  source_path: string;
}

export interface ToolErrorsResponse {
  rows: ToolErrorRow[];
  total: number;
}

export interface McpServerSummary {
  provider: string;
  server: string;
  tools_used: number;
  invocations: number;
  sessions_used: number;
}

export interface HourlyRow {
  provider: string;
  hour: number;
  turns: number;
  input: number;
  output: number;
  reasoning_output: number;
}

export interface BranchSummary {
  provider: string;
  branch: string;
  sessions: number;
  turns: number;
  input: number;
  output: number;
  reasoning_output: number;
  cost: number;
}

export interface VersionSummary {
  provider: string;
  version: string;
  turns: number;
  sessions: number;
  cost: number;
  tokens: number;
}

export interface DailyProjectRow {
  day: string;
  provider: string;
  project: string;
  display_name: string;
  input: number;
  output: number;
  reasoning_output: number;
  cost: number;
  credits?: number | null;
}

export interface ProviderSummary {
  provider: string;
  sessions: number;
  turns: number;
  input: number;
  output: number;
  cache_read: number;
  cache_creation: number;
  reasoning_output: number;
  cost: number;
}

export interface ConfidenceSummary {
  confidence: string;
  turns: number;
  cost: number;
}

export interface BillingModeSummary {
  billing_mode: string;
  turns: number;
  cost: number;
}

export interface OfficialSyncSourceStatus {
  source_slug: string;
  source_kind: string;
  provider: string;
  status: string;
  fetched_at: string;
  record_count: number;
  error_text: string;
}

export interface OfficialSyncRecordCount {
  record_type: string;
  count: number;
}

export interface OfficialSyncSummary {
  available: boolean;
  last_sync_at: string | null;
  latest_success_at: string | null;
  total_runs: number;
  total_records: number;
  sources_success: number;
  sources_error: number;
  sources_skipped: number;
  sources: OfficialSyncSourceStatus[];
  record_counts: OfficialSyncRecordCount[];
}

export interface WeeklyModelRow {
  week: string;
  model: string;
  input_tokens: number;
  output_tokens: number;
  cache_read_tokens: number;
  cache_creation_tokens: number;
  reasoning_output_tokens: number;
  cost_nanos: number;
}

export interface DailyAgg {
  day: string;
  input: number;
  output: number;
  cache_read: number;
  cache_creation: number;
  reasoning_output: number;
  cost: number;
}

export interface ModelAgg {
  provider?: string;
  model: string;
  input: number;
  output: number;
  cache_read: number;
  cache_creation: number;
  reasoning_output: number;
  turns: number;
  sessions: number;
  cost: number;
  is_billable: boolean;
  input_cost?: number;
  output_cost?: number;
  cache_read_cost?: number;
  cache_write_cost?: number;
  credits?: number | null;
}

export interface ProjectAgg {
  provider?: string;
  project: string;
  display_name: string;
  input: number;
  output: number;
  cache_read: number;
  cache_creation: number;
  reasoning_output: number;
  turns: number;
  sessions: number;
  cost: number;
  credits?: number | null;
}

export interface Totals {
  provider?: string;
  sessions: number;
  turns: number;
  input: number;
  output: number;
  cache_read: number;
  cache_creation: number;
  reasoning_output: number;
  cost: number;
  /** Sum of Amp credits across the active filter; absent when no Amp data. */
  credits?: number | null;
}

export interface StatCard {
  label: string;
  value: string;
  sub: string;
  color?: string;
  isCost?: boolean;
}

export interface WeeklyAgg {
  week: string;
  input: number;
  output: number;
  cache_read: number;
  cache_creation: number;
  reasoning_output: number;
  cost_nanos: number;
}

export interface CacheEfficiency {
  cache_read_tokens: number;
  cache_write_tokens: number;
  input_tokens: number;
  output_tokens: number;
  cache_read_cost_nanos: number;
  cache_write_cost_nanos: number;
  input_cost_nanos: number;
  output_cost_nanos: number;
  cache_hit_rate: number | null;
}

export interface OpenAiReconciliation {
  available: boolean;
  lookback_days: number;
  start_date: string;
  end_date: string;
  estimated_local_cost: number;
  api_usage_cost: number;
  api_input_tokens: number;
  api_output_tokens: number;
  api_cached_input_tokens: number;
  api_requests: number;
  delta_cost: number;
  error: string | null;
}

export interface HeatmapCell {
  dow: number;
  hour: number;
  cost_nanos: number;
  call_count: number;
}

export interface HeatmapData {
  cells: HeatmapCell[];
  max_cost_nanos: number;
  max_call_count: number;
  active_days: number;
  total_cost_nanos: number;
  period: string;
  tz_offset_min: number;
}

// Subscription-quota widget (Phase 22). Mirrors src/models.rs.
export interface PublishedWindow {
  kind: string;
  label: string;
  used_percent: number;
  resets_at?: string | null;
  resets_in_minutes?: number | null;
  window_seconds?: number | null;
}

export interface PublishedBudget {
  used_usd: number;
  limit_usd: number;
  utilization: number;
  currency: string;
}

export interface ProviderQuotaPublished {
  plan_label?: string | null;
  windows: PublishedWindow[];
  budget?: PublishedBudget | null;
}

export interface EstimatedWindow {
  kind: string;
  label: string;
  estimated_cap_tokens: number;
  observed_tokens: number;
  confidence: number;
  /**
   * Confidence-weighted EMA of recent observations + the current one.
   * Absent on stale clients / before any history exists.
   */
  smoothed_cap_tokens?: number | null;
  /** Number of (history + current) samples that fed the smoother. */
  sample_count?: number | null;
  /** Lowercase tag the backend emits when a ≥ ±20% shift is detected. */
  cap_shift?: 'increase' | 'decrease' | null;
}

export interface ProviderQuotaEstimated {
  windows: EstimatedWindow[];
}

export interface ProviderQuotaSnapshot {
  provider: string;
  plan?: string | null;
  source_used: string;
  published: ProviderQuotaPublished;
  estimated?: ProviderQuotaEstimated | null;
}

export interface RateWindowHistoryRow {
  timestamp: string;
  provider: string;
  window_type: string;
  used_percent: number | null;
  estimated_cap_tokens: number | null;
  confidence: number | null;
  plan: string | null;
}

export interface ChangelogEntry {
  date: string;
  provider: string;
  kind: string;
  title: string;
  description: string;
  source_url: string;
}

export interface SubscriptionQuotaSection {
  providers: ProviderQuotaSnapshot[];
  history: RateWindowHistoryRow[];
  changelog: ChangelogEntry[];
  generated_at: string;
}

export interface DashboardData {
  all_models: string[];
  provider_breakdown: ProviderSummary[];
  confidence_breakdown: ConfidenceSummary[];
  billing_mode_breakdown: BillingModeSummary[];
  daily_by_model: DailyModelRow[];
  sessions_all: SessionRow[];
  subagent_summary: SubagentSummary;
  entrypoint_breakdown: EntrypointSummary[];
  service_tiers: ServiceTierSummary[];
  tool_summary: ToolSummary[];
  mcp_summary: McpServerSummary[];
  hourly_distribution: HourlyRow[];
  git_branch_summary: BranchSummary[];
  version_summary: VersionSummary[];
  daily_by_project: DailyProjectRow[];
  weekly_by_model: WeeklyModelRow[];
  openai_reconciliation: OpenAiReconciliation | null;
  official_sync: OfficialSyncSummary;
  generated_at: string;
  cache_efficiency: CacheEfficiency;
  subscription_quota?: SubscriptionQuotaSection | null;
  error?: string;
}
