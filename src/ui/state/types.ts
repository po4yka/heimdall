// ── Types ──────────────────────────────────────────────────────────────
export interface WindowInfo {
  used_percent: number;
  resets_at: string | null;
  resets_in_minutes: number | null;
}

export interface BudgetInfo {
  used: number;
  limit: number;
  currency: string;
  utilization: number;
}

export interface IdentityInfo {
  plan: string | null;
  rate_limit_tier: string | null;
}

export interface UsageWindowsResponse {
  available: boolean;
  session?: WindowInfo;
  weekly?: WindowInfo;
  weekly_opus?: WindowInfo;
  weekly_sonnet?: WindowInfo;
  budget?: BudgetInfo;
  identity?: IdentityInfo;
  error?: string;
}

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
  /** Phase 21: per-type cost breakdown (USD float) */
  input_cost: number;
  output_cost: number;
  cache_read_cost: number;
  cache_write_cost: number;
}

export interface SessionRow {
  session_id: string;
  provider: string;
  project: string;
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
  input: number;
  output: number;
  reasoning_output: number;
  cost: number;
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
  generated_at: string;
  /** Phase 21: cache-token breakdown and derived hit-rate metric. */
  cache_efficiency: CacheEfficiency;
  error?: string;
}

export interface DailyAgg {
  day: string;
  input: number;
  output: number;
  cache_read: number;
  cache_creation: number;
  reasoning_output: number;
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
  /** Phase 21: per-type cost breakdown (USD float, derived from nanos) */
  input_cost?: number;
  output_cost?: number;
  cache_read_cost?: number;
  cache_write_cost?: number;
}

export interface ProjectAgg {
  provider?: string;
  project: string;
  input: number;
  output: number;
  cache_read: number;
  cache_creation: number;
  reasoning_output: number;
  turns: number;
  sessions: number;
  cost: number;
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
}

export interface StatCard {
  label: string;
  value: string;
  sub: string;
  color?: string;
  isCost?: boolean;
}

export type SortDir = 'asc' | 'desc';
export type RangeKey = '7d' | '30d' | '90d' | 'all';
export type BucketKey = 'day' | 'week';

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

export interface WeeklyAgg {
  week: string;
  input: number;
  output: number;
  cache_read: number;
  cache_creation: number;
  reasoning_output: number;
  cost_nanos: number;
}

// ── Phase 21: Cache Efficiency ──────────────────────────────────────────────

export interface CacheEfficiency {
  cache_read_tokens: number;
  cache_write_tokens: number;
  input_tokens: number;
  output_tokens: number;
  cache_read_cost_nanos: number;
  cache_write_cost_nanos: number;
  input_cost_nanos: number;
  output_cost_nanos: number;
  /** cache_read / (cache_read + input) when denominator > 0; else null */
  cache_hit_rate: number | null;
}

// ── Agent Status ────────────────────────────────────────────────────────────

export type StatusIndicator =
  | 'none'
  | 'minor'
  | 'major'
  | 'critical'
  | 'maintenance'
  | 'unknown';

export interface ComponentStatus {
  id: string;
  name: string;
  status: string;
  uptime_30d: number | null;
  uptime_7d: number | null;
}

export interface IncidentSummary {
  name: string;
  impact: string;
  status: string;
  shortlink: string | null;
  started_at: string;
}

export interface ProviderStatus {
  indicator: StatusIndicator;
  description: string;
  components: ComponentStatus[];
  active_incidents: IncidentSummary[];
  page_url: string;
}

export interface AgentStatusSnapshot {
  claude: ProviderStatus | null;
  openai: ProviderStatus | null;
  fetched_at: string;
}

// ── Community Signal ────────────────────────────────────────────────────────

export type SignalLevel = 'normal' | 'elevated' | 'spike' | 'unknown';

export interface ServiceSignal {
  slug: string;
  name: string;
  level: SignalLevel;
  report_count_last_hour: number | null;
  report_baseline: number | null;
  detail: string;
  source_url: string;
}

export interface CommunitySignal {
  fetched_at: string;
  claude: ServiceSignal[];
  openai: ServiceSignal[];
  enabled: boolean;
}

// ── Phase 2: Billing Blocks ──────────────────────────────────────────────────

export type QuotaSeverity = 'ok' | 'warn' | 'danger';

export interface BurnRate {
  tokens_per_min: number;
  cost_per_hour_nanos: number;
}

export interface BlockProjection {
  projected_cost_nanos: number;
  projected_tokens: number;
}

export interface BlockQuota {
  limit_tokens: number;
  used_tokens: number;
  projected_tokens: number;
  current_pct: number;
  projected_pct: number;
  remaining_tokens: number;
  current_severity: QuotaSeverity;
  projected_severity: QuotaSeverity;
}

export interface BlockTokens {
  input: number;
  output: number;
  cache_read: number;
  cache_creation: number;
  reasoning_output: number;
}

export interface BillingBlockView {
  start: string;
  end: string;
  first_timestamp: string;
  last_timestamp: string;
  tokens: BlockTokens;
  cost_nanos: number;
  models: string[];
  is_active: boolean;
  entry_count: number;
  burn_rate?: BurnRate | null;
  projection?: BlockProjection;
  quota?: BlockQuota;
}

export interface BillingBlocksResponse {
  session_length_hours: number;
  token_limit: number | null;
  historical_max_tokens: number;
  blocks: BillingBlockView[];
}

// ── Phase 5: Context Window ──────────────────────────────────────────────────

export type ContextWindowSeverity = "ok" | "warn" | "danger";

export interface ContextWindowResponse {
  enabled?: false;
  total_input_tokens?: number;
  context_window_size?: number;
  pct?: number;
  severity?: ContextWindowSeverity;
  session_id?: string;
  captured_at?: string;
}

// ── Phase 13: Activity Heatmap ───────────────────────────────────────────────

export interface HeatmapCell {
  dow: number;   // 0..6 (Sunday=0)
  hour: number;  // 0..23
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
