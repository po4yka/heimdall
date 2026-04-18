import { signal } from '@preact/signals';
import type { DashboardData, RangeKey, SessionRow, ProjectAgg, BillingBlocksResponse } from './types';

// ── Core data ────────────────────────────────────────────────────────
export const rawData = signal<DashboardData | null>(null);
export const billingBlocksData = signal<BillingBlocksResponse | null>(null);

// ── Filter state ─────────────────────────────────────────────────────
export type ProviderFilter = 'claude' | 'codex' | 'both';

export const selectedModels = signal<Set<string>>(new Set());
export const selectedRange = signal<RangeKey>('30d');
export const selectedProvider = signal<ProviderFilter>('both');
export const projectSearchQuery = signal('');

// ── Cached derivations (updated by applyFilter) ──────────────────────
export const lastFilteredSessions = signal<SessionRow[]>([]);
export const lastByProject = signal<ProjectAgg[]>([]);

// ── UI chrome state ──────────────────────────────────────────────────
export const metaText = signal<string>('');
export const planBadge = signal<string>('');
export const rescanLabel = signal<string>('\u21bb Rescan');
export const rescanDisabled = signal<boolean>(false);
export const themeMode = signal<'dark' | 'light'>('dark');

// ── Inline status (replaces toasts) ──────────────────────────────────
export type StatusPlacement = 'global' | 'rate-windows' | 'rescan' | 'header-refresh' | 'agent-status' | 'community-signal';
export type StatusKind = 'success' | 'error' | 'loading' | 'info';

export interface StatusEntry {
  kind: StatusKind;
  message: string;
}

export const statusByPlacement = signal<Record<StatusPlacement, StatusEntry | null>>({
  'global': null,
  'rate-windows': null,
  'rescan': null,
  'header-refresh': null,
  'agent-status': null,
  'community-signal': null,
});

// ── Pagination page size (used by SessionsTable via DataTable) ───────
export const SESSIONS_PAGE_SIZE = 25;

// ── Phase 18: data-load state ─────────────────────────────────────────
// 'idle'       — no fetch in progress; data (if any) is current.
// 'refreshing' — a subsequent fetch is in progress; old data remains
//                visible so the UI does not flash blank.
export type LoadState = 'idle' | 'refreshing';
export const loadState = signal<LoadState>('idle');

// ── Phase 16: Version donut metric selector ──────────────────────────
export type VersionMetric = 'cost' | 'calls' | 'tokens';

function readVersionMetric(): VersionMetric {
  const p = new URLSearchParams(window.location.search).get('version_metric');
  return (['cost', 'calls', 'tokens'] as VersionMetric[]).includes(p as VersionMetric)
    ? (p as VersionMetric)
    : 'cost';
}

export const versionDonutMetric = signal<VersionMetric>(readVersionMetric());

// ── Agent status expand/collapse (URL-persistent) ────────────────────
function readAgentStatusExpanded(): boolean {
  const p = new URLSearchParams(window.location.search).get('agent_status_expanded');
  return p === '1' || p === 'true';
}

export const agent_status_expanded = signal<boolean>(readAgentStatusExpanded());
