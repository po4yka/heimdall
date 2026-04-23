import type {
  BurnRateTier,
  ContextWindowSeverity,
  DepletionForecast,
  PredictiveInsights,
  QuotaSeverity,
  QuotaSuggestions,
} from './billing-types';

export type LiveMonitorFocus = 'all' | 'claude' | 'codex';
export type LiveMonitorVisualState = 'healthy' | 'degraded' | 'stale' | 'incident' | 'error';

export interface LiveMonitorFreshness {
  newest_provider_refresh?: string | null;
  oldest_provider_refresh?: string | null;
  stale_providers: string[];
  has_stale_providers: boolean;
  refresh_state: string;
}

export interface LiveMonitorBurnRate {
  tokens_per_min: number;
  cost_per_hour_nanos: number;
  tier?: BurnRateTier | string | null;
}

export interface LiveMonitorProjection {
  projected_cost_nanos: number;
  projected_tokens: number;
}

export interface LiveMonitorQuota {
  limit_tokens: number;
  used_tokens: number;
  projected_tokens: number;
  current_pct: number;
  projected_pct: number;
  remaining_tokens: number;
  current_severity: QuotaSeverity;
  projected_severity: QuotaSeverity;
}

export interface LiveMonitorBlock {
  start: string;
  end: string;
  first_timestamp: string;
  last_timestamp: string;
  cost_nanos: number;
  entry_count: number;
  burn_rate?: LiveMonitorBurnRate | null;
  projection?: LiveMonitorProjection | null;
  quota?: LiveMonitorQuota | null;
  tokens: {
    input: number;
    output: number;
    cache_read: number;
    cache_creation: number;
    reasoning_output: number;
  };
}

export interface LiveMonitorContextWindow {
  total_input_tokens: number;
  context_window_size: number;
  pct: number;
  severity: ContextWindowSeverity;
  session_id?: string | null;
  captured_at?: string | null;
}

export interface LiveMonitorRecentSession {
  session_id: string;
  display_name: string;
  started_at: string;
  duration_minutes: number;
  turns: number;
  cost_usd: number;
  model?: string | null;
}

export interface LiveMonitorProvider {
  provider: 'claude' | 'codex' | string;
  title: string;
  visual_state: LiveMonitorVisualState;
  source_label: string;
  warnings: string[];
  identity_label?: string | null;
  primary?: {
    used_percent: number;
    resets_at?: string | null;
    resets_in_minutes?: number | null;
    window_minutes?: number | null;
    reset_label?: string | null;
  } | null;
  secondary?: {
    used_percent: number;
    resets_at?: string | null;
    resets_in_minutes?: number | null;
    window_minutes?: number | null;
    reset_label?: string | null;
  } | null;
  today_cost_usd: number;
  projected_weekly_spend_usd?: number | null;
  last_refresh: string;
  last_refresh_label: string;
  active_block?: LiveMonitorBlock | null;
  context_window?: LiveMonitorContextWindow | null;
  recent_session?: LiveMonitorRecentSession | null;
  quota_suggestions?: QuotaSuggestions | null;
  depletion_forecast?: DepletionForecast | null;
  predictive_insights?: PredictiveInsights | null;
}

export interface LiveMonitorResponse {
  contract_version: number;
  generated_at: string;
  default_focus: LiveMonitorFocus;
  global_issue?: string | null;
  freshness: LiveMonitorFreshness;
  providers: LiveMonitorProvider[];
}
