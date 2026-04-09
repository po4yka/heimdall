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
  entrypoint: string;
  sessions: number;
  turns: number;
  input: number;
  output: number;
}

export interface ServiceTierSummary {
  service_tier: string;
  inference_geo: string;
  turns: number;
}

export interface DailyModelRow {
  day: string;
  model: string;
  input: number;
  output: number;
  cache_read: number;
  cache_creation: number;
  turns: number;
  cost: number;
}

export interface SessionRow {
  session_id: string;
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
  cost: number;
  is_billable: boolean;
  subagent_count: number;
  subagent_turns: number;
}

export interface DashboardData {
  all_models: string[];
  daily_by_model: DailyModelRow[];
  sessions_all: SessionRow[];
  subagent_summary: SubagentSummary;
  entrypoint_breakdown: EntrypointSummary[];
  service_tiers: ServiceTierSummary[];
  generated_at: string;
  error?: string;
}

export interface DailyAgg {
  day: string;
  input: number;
  output: number;
  cache_read: number;
  cache_creation: number;
}

export interface ModelAgg {
  model: string;
  input: number;
  output: number;
  cache_read: number;
  cache_creation: number;
  turns: number;
  sessions: number;
  cost: number;
  is_billable: boolean;
}

export interface ProjectAgg {
  project: string;
  input: number;
  output: number;
  cache_read: number;
  cache_creation: number;
  turns: number;
  sessions: number;
  cost: number;
}

export interface Totals {
  sessions: number;
  turns: number;
  input: number;
  output: number;
  cache_read: number;
  cache_creation: number;
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
