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
