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

export interface ClaudeUsageFactor {
  factor_key: string;
  display_label: string;
  percent: number;
  description: string;
  advice_text: string;
  display_order: number;
}

export interface ClaudeUsageRunMeta {
  id: number;
  captured_at: string;
  status: string;
  exit_code: number | null;
  invocation_mode: string;
  period: string;
  parser_version: string;
  error_summary?: string | null;
}

export interface ClaudeUsageSnapshot {
  run: ClaudeUsageRunMeta;
  factors: ClaudeUsageFactor[];
}

export interface ClaudeUsageResponse {
  available: boolean;
  last_run?: ClaudeUsageRunMeta | null;
  latest_snapshot?: ClaudeUsageSnapshot | null;
}

export type QuotaSeverity = 'ok' | 'warn' | 'danger';
export type BurnRateTier = 'normal' | 'moderate' | 'high';
export type ContextWindowSeverity = 'ok' | 'warn' | 'danger';

export interface BurnRate {
  tokens_per_min: number;
  cost_per_hour_nanos: number;
  tier?: BurnRateTier;
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

export interface ContextWindowResponse {
  enabled?: false;
  total_input_tokens?: number;
  context_window_size?: number;
  pct?: number;
  severity?: ContextWindowSeverity;
  session_id?: string;
  captured_at?: string;
}

export interface CostReconciliationBreakdownRow {
  day: string;
  hook_nanos: number;
  local_nanos: number;
}

export interface CostReconciliationResponse {
  enabled: boolean;
  period?: 'day' | 'week' | 'month';
  hook_total_nanos?: number;
  local_total_nanos?: number;
  divergence_pct?: number;
  breakdown?: CostReconciliationBreakdownRow[];
}
