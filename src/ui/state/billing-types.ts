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

export interface ClaudeAdminSummary {
  organization_name: string;
  lookback_days: number;
  start_date: string;
  end_date: string;
  data_latency_note: string;
  today_active_users: number;
  today_sessions: number;
  lookback_lines_accepted: number;
  lookback_estimated_cost_usd: number;
  lookback_input_tokens: number;
  lookback_output_tokens: number;
  lookback_cache_read_tokens: number;
  lookback_cache_creation_tokens: number;
  error?: string | null;
}

export interface UsageWindowsResponse {
  available: boolean;
  source: 'oauth' | 'admin' | 'unavailable' | string;
  session?: WindowInfo;
  weekly?: WindowInfo;
  weekly_opus?: WindowInfo;
  weekly_sonnet?: WindowInfo;
  budget?: BudgetInfo;
  identity?: IdentityInfo;
  admin_fallback?: ClaudeAdminSummary;
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

export type QuotaSuggestionKey = 'p90' | 'p95' | 'max';

export interface QuotaSuggestionLevel {
  key: QuotaSuggestionKey | string;
  label: string;
  limit_tokens: number;
}

export interface QuotaSuggestions {
  sample_count: number;
  population_count: number;
  recommended_key: QuotaSuggestionKey | string;
  sample_strategy: string;
  sample_label: string;
  levels: QuotaSuggestionLevel[];
  note?: string | null;
}

export type DepletionSignalKind = 'billing_block' | 'primary_window' | 'secondary_window' | string;

export interface DepletionForecastSignal {
  kind: DepletionSignalKind;
  title: string;
  used_percent: number;
  projected_percent?: number | null;
  remaining_tokens?: number | null;
  remaining_percent?: number | null;
  resets_in_minutes?: number | null;
  pace_label?: string | null;
  end_time?: string | null;
}

export interface DepletionForecast {
  primary_signal: DepletionForecastSignal;
  secondary_signals: DepletionForecastSignal[];
  summary_label: string;
  severity: QuotaSeverity;
  note?: string | null;
}

export interface PredictiveBurnRate {
  tokens_per_min: number;
  cost_per_hour_nanos: number;
  coverage_minutes: number;
  tier: BurnRateTier | string;
}

export interface IntegerPercentiles {
  average: number;
  p50: number;
  p75: number;
  p90: number;
  p95: number;
}

export interface FloatPercentiles {
  average: number;
  p50: number;
  p75: number;
  p90: number;
  p95: number;
}

export interface HistoricalEnvelope {
  sample_count: number;
  tokens: IntegerPercentiles;
  cost_usd: FloatPercentiles;
  turns: IntegerPercentiles;
}

export interface LimitHitAnalysis {
  sample_count: number;
  hit_count: number;
  hit_rate: number;
  threshold_tokens: number;
  threshold_percent: number;
  active_current_hit?: boolean | null;
  active_projected_hit?: boolean | null;
  risk_level: string;
  summary_label: string;
}

export interface PredictiveInsights {
  rolling_hour_burn?: PredictiveBurnRate | null;
  historical_envelope?: HistoricalEnvelope | null;
  limit_hit_analysis?: LimitHitAnalysis | null;
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
  quota_suggestions?: QuotaSuggestions | null;
  depletion_forecast?: DepletionForecast | null;
  predictive_insights?: PredictiveInsights | null;
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
