import { signal } from '@preact/signals';
import type { PaginationState, VisibilityState } from '@tanstack/table-core';
import type { DashboardData, RangeKey, BucketKey, SessionRow, ProjectAgg, BillingBlocksResponse, ContextWindowResponse, CostReconciliationResponse } from './types';

// ── Core data ────────────────────────────────────────────────────────
export const rawData = signal<DashboardData | null>(null);
export const billingBlocksData = signal<BillingBlocksResponse | null>(null);
export const contextWindowData = signal<ContextWindowResponse | null>(null);
export const costReconciliationData = signal<CostReconciliationResponse | null>(null);

// ── Backup / snapshots ────────────────────────────────────────────────
export interface SnapshotMeta {
  snapshot_id: string;
  created_at: string;
  total_files: number;
  total_bytes: number;
}

export const backupSnapshots = signal<SnapshotMeta[]>([]);
export const backupLoadState = signal<'idle' | 'loading' | 'error'>('idle');

// ── Web captures (companion extension) ───────────────────────────────
export interface WebConversationSummary {
  vendor: string;
  conversation_id: string;
  captured_at: string;
  history_count: number;
}

export interface CompanionHeartbeat {
  last_seen_at: string;
  extension_version: string | null;
  user_agent: string | null;
  vendors_seen: string[];
}

export const webConversations = signal<WebConversationSummary[]>([]);
export const companionHeartbeat = signal<CompanionHeartbeat | null>(null);

// ── Imports (chat-export ingests) ─────────────────────────────────────
export interface ImportMeta {
  import_id: string;
  vendor: string;
  created_at: string;
  conversation_count: number;
  parser_version: number;
  schema_fingerprint: string | null;
}
export const archiveImports = signal<ImportMeta[]>([]);

// ── Filter state ─────────────────────────────────────────────────────
export type ProviderFilter = 'claude' | 'codex' | 'both';
export type DashboardTab = 'overview' | 'activity' | 'breakdowns' | 'tables' | 'backup';
const SESSIONS_PAGE_PARAM = 'sessions_page';
const SESSIONS_HIDDEN_COLUMNS_PARAM = 'sessions_hidden';
const FILTERS_EXPANDED_PARAM = 'filters_expanded';
const DASHBOARD_TAB_PARAM = 'tab';
const COLLAPSED_SECTIONS_PARAM = 'collapsed_sections';

// Allowlist of column IDs declared in SessionsTable.tsx. Used to validate
// untrusted columnIds parsed from the URL before they are used as object keys,
// preventing prototype pollution via crafted ?sessions_hidden=__proto__ etc.
const SESSIONS_TABLE_COLUMN_IDS: ReadonlySet<string> = new Set([
  'session',
  'project',
  'provider',
  'last',
  'duration_min',
  'model',
  'turns',
  'input',
  'output',
  'cost',
  'credits',
  'cost_meta',
  'cache_hit_ratio',
  'tokens_per_min',
]);

export const selectedModels = signal<Set<string>>(new Set());
export const selectedRange = signal<RangeKey>('30d');
export const selectedProvider = signal<ProviderFilter>('both');
export const projectSearchQuery = signal('');

export function readBucket(): BucketKey {
  const p = new URLSearchParams(window.location.search).get('bucket');
  return (['day', 'week'] as BucketKey[]).includes(p as BucketKey) ? (p as BucketKey) : 'day';
}

export const selectedBucket = signal<BucketKey>(readBucket());

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
export type StatusPlacement = 'global' | 'rate-windows' | 'rescan' | 'header-refresh' | 'agent-status' | 'community-signal' | 'snapshot';
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
  'snapshot': null,
});

// ── Pagination page size (used by SessionsTable via DataTable) ───────
export const SESSIONS_PAGE_SIZE = 25;

function readSearchParam(name: string): string | null {
  return new URLSearchParams(window.location.search).get(name);
}

function readPositiveIntParam(name: string): number | null {
  const raw = readSearchParam(name);
  if (!raw) return null;
  const parsed = Number.parseInt(raw, 10);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : null;
}

function readRangeFromUrl(): RangeKey {
  const p = readSearchParam('range');
  return (['7d', '30d', '90d', 'all'] as RangeKey[]).includes(p as RangeKey) ? (p as RangeKey) : '30d';
}

function readDashboardTab(): DashboardTab {
  const p = readSearchParam(DASHBOARD_TAB_PARAM);
  return (['overview', 'activity', 'breakdowns', 'tables', 'backup'] as DashboardTab[]).includes(p as DashboardTab)
    ? (p as DashboardTab)
    : 'overview';
}

function readProviderFromUrl(): ProviderFilter {
  const p = readSearchParam('provider');
  return (['claude', 'codex', 'both'] as ProviderFilter[]).includes(p as ProviderFilter)
    ? (p as ProviderFilter)
    : 'both';
}

function readModelsFromUrl(allModels: string[]): Set<string> {
  const param = readSearchParam('models');
  if (!param) return new Set(allModels);
  const fromUrl = new Set(param.split(',').map(s => s.trim()).filter(Boolean));
  return new Set(allModels.filter(model => fromUrl.has(model)));
}

function readSessionsTablePagination(): PaginationState {
  return {
    pageIndex: Math.max((readPositiveIntParam(SESSIONS_PAGE_PARAM) ?? 1) - 1, 0),
    pageSize: SESSIONS_PAGE_SIZE,
  };
}

function readSessionsTableColumnVisibility(): VisibilityState {
  const hiddenColumns = readSearchParam(SESSIONS_HIDDEN_COLUMNS_PARAM);
  if (!hiddenColumns) return {};

  const visibility: VisibilityState = Object.create(null) as VisibilityState;
  for (const columnId of hiddenColumns.split(',').map(value => value.trim()).filter(Boolean)) {
    if (SESSIONS_TABLE_COLUMN_IDS.has(columnId)) {
      visibility[columnId] = false;
    }
  }
  return visibility;
}

function isDefaultModelSelection(allModels: string[]): boolean {
  if (selectedModels.value.size !== allModels.length) return false;
  return allModels.every(model => selectedModels.value.has(model));
}

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

// ── Activity heatmap metric selector ─────────────────────────────────
export type HeatmapMetric = 'cost' | 'calls';

function readHeatmapMetric(): HeatmapMetric {
  const p = readSearchParam('hm_metric');
  return p === 'calls' ? 'calls' : 'cost';
}

export const heatmapMetric = signal<HeatmapMetric>(readHeatmapMetric());

// ── Agent status expand/collapse (URL-persistent) ────────────────────
function readAgentStatusExpanded(): boolean {
  const p = readSearchParam('agent_status_expanded');
  return p === '1' || p === 'true';
}

function readOfficialSyncExpanded(): boolean {
  const p = readSearchParam('official_sync_expanded');
  return p === '1' || p === 'true';
}

function readCollapsedSections(): Set<string> {
  const p = readSearchParam(COLLAPSED_SECTIONS_PARAM);
  if (!p) return new Set();
  return new Set(p.split(',').map(value => value.trim()).filter(Boolean));
}

function readFiltersExpanded(): boolean {
  const p = readSearchParam(FILTERS_EXPANDED_PARAM);
  return p === '1' || p === 'true';
}

export const activeDashboardTab = signal<DashboardTab>(readDashboardTab());
export const agent_status_expanded = signal<boolean>(readAgentStatusExpanded());
export const official_sync_expanded = signal<boolean>(readOfficialSyncExpanded());
export const mobile_filters_expanded = signal<boolean>(readFiltersExpanded());
export const collapsedSectionKeys = signal<Set<string>>(readCollapsedSections());
export const sessionsTablePagination = signal<PaginationState>(readSessionsTablePagination());
export const sessionsTableColumnVisibility = signal<VisibilityState>(readSessionsTableColumnVisibility());

export function isSectionCollapsed(sectionKey: string): boolean {
  return collapsedSectionKeys.value.has(sectionKey);
}

export function setSectionCollapsed(sectionKey: string, collapsed: boolean): void {
  const next = new Set(collapsedSectionKeys.value);
  if (collapsed) next.add(sectionKey);
  else next.delete(sectionKey);
  collapsedSectionKeys.value = next;
}

export function restoreDashboardStateFromUrl(allModels: string[]): void {
  activeDashboardTab.value = readDashboardTab();
  selectedRange.value = readRangeFromUrl();
  selectedProvider.value = readProviderFromUrl();
  selectedModels.value = readModelsFromUrl(allModels);
  projectSearchQuery.value = readSearchParam('project') ?? '';
  selectedBucket.value = readBucket();
  versionDonutMetric.value = readVersionMetric();
  heatmapMetric.value = readHeatmapMetric();
  agent_status_expanded.value = readAgentStatusExpanded();
  official_sync_expanded.value = readOfficialSyncExpanded();
  mobile_filters_expanded.value = readFiltersExpanded();
  collapsedSectionKeys.value = readCollapsedSections();
  sessionsTablePagination.value = readSessionsTablePagination();
  sessionsTableColumnVisibility.value = readSessionsTableColumnVisibility();
}

export function syncDashboardUrl(): void {
  const allModels = rawData.value?.all_models ?? [];
  const params = new URLSearchParams();

  if (activeDashboardTab.value !== 'overview') params.set(DASHBOARD_TAB_PARAM, activeDashboardTab.value);
  if (selectedRange.value !== '30d') params.set('range', selectedRange.value);
  if (selectedProvider.value !== 'both') params.set('provider', selectedProvider.value);
  if (!isDefaultModelSelection(allModels)) {
    params.set('models', Array.from(selectedModels.value).join(','));
  }
  if (projectSearchQuery.value) params.set('project', projectSearchQuery.value);
  if (versionDonutMetric.value !== 'cost') params.set('version_metric', versionDonutMetric.value);
  if (heatmapMetric.value !== 'cost') params.set('hm_metric', heatmapMetric.value);
  if (selectedBucket.value !== 'day') params.set('bucket', selectedBucket.value);
  if (agent_status_expanded.value) params.set('agent_status_expanded', '1');
  if (official_sync_expanded.value) params.set('official_sync_expanded', '1');
  if (mobile_filters_expanded.value) params.set(FILTERS_EXPANDED_PARAM, '1');
  const collapsedSections = Array.from(collapsedSectionKeys.value).sort();
  if (collapsedSections.length) {
    params.set(COLLAPSED_SECTIONS_PARAM, collapsedSections.join(','));
  }

  const pageNumber = sessionsTablePagination.value.pageIndex + 1;
  if (pageNumber > 1) params.set(SESSIONS_PAGE_PARAM, String(pageNumber));

  const hiddenColumns = Object.entries(sessionsTableColumnVisibility.value)
    .filter(([, isVisible]) => isVisible === false)
    .map(([columnId]) => columnId)
    .sort();
  if (hiddenColumns.length) {
    params.set(SESSIONS_HIDDEN_COLUMNS_PARAM, hiddenColumns.join(','));
  }

  const search = params.toString();
  const nextUrl = search ? `${window.location.pathname}?${search}` : window.location.pathname;
  history.replaceState(null, '', nextUrl);
}
