import { signal } from '@preact/signals';
import type { DashboardData, RangeKey, SessionRow, ProjectAgg } from './types';

// ── Core data ────────────────────────────────────────────────────────
export const rawData = signal<DashboardData | null>(null);

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
export type StatusPlacement = 'global' | 'rate-windows' | 'rescan';
export type StatusKind = 'success' | 'error' | 'loading' | 'info';

export interface StatusEntry {
  kind: StatusKind;
  message: string;
}

export const statusByPlacement = signal<Record<StatusPlacement, StatusEntry | null>>({
  'global': null,
  'rate-windows': null,
  'rescan': null,
});

// ── Pagination page size (used by SessionsTable via DataTable) ───────
export const SESSIONS_PAGE_SIZE = 25;
