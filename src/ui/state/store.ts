import { signal } from '@preact/signals';
import type { DashboardData, RangeKey, SortDir, SessionRow, ProjectAgg } from './types';

// Core data
export const rawData = signal<DashboardData | null>(null);

// Filter state
export const selectedModels = signal<Set<string>>(new Set());
export const selectedRange = signal<RangeKey>('30d');
export const projectSearchQuery = signal('');

// Sort state
export const sessionSortCol = signal('last');
export const sessionSortDir = signal<SortDir>('desc');
export const modelSortCol = signal('cost');
export const modelSortDir = signal<SortDir>('desc');
export const projectSortCol = signal('cost');
export const projectSortDir = signal<SortDir>('desc');

// Pagination
export const sessionsCurrentPage = signal(0);
export const SESSIONS_PAGE_SIZE = 25;

// Cached results (updated by applyFilter)
export const lastFilteredSessions = signal<SessionRow[]>([]);
export const lastByProject = signal<ProjectAgg[]>([]);
