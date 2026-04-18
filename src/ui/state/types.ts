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
  openai_reconciliation: OpenAiReconciliation | null;
  generated_at: string;
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
