import { signal } from '@preact/signals';
import type { ToolErrorRow } from '../state/types';

export type LoadState = 'idle' | 'loading' | 'error';

export const toolName = signal<string>('');
export const providerFilter = signal<string>('');
export const rangeFilter = signal<string>('30d');
export const pageOffset = signal<number>(0);
export const rows = signal<ToolErrorRow[]>([]);
export const total = signal<number>(0);
export const loadState = signal<LoadState>('idle');
export const errorMessage = signal<string | null>(null);

export const PAGE_SIZE = 100;

export function readUrlParams(): void {
  const p = new URLSearchParams(window.location.search);
  toolName.value = p.get('tool') ?? '';
  providerFilter.value = p.get('provider') ?? '';
  rangeFilter.value = p.get('range') ?? '30d';
  const off = Number.parseInt(p.get('offset') ?? '0', 10);
  pageOffset.value = Number.isFinite(off) && off >= 0 ? off : 0;
}

export function syncUrl(): void {
  const p = new URLSearchParams();
  if (toolName.value) p.set('tool', toolName.value);
  if (providerFilter.value) p.set('provider', providerFilter.value);
  if (rangeFilter.value !== '30d') p.set('range', rangeFilter.value);
  if (pageOffset.value > 0) p.set('offset', String(pageOffset.value));
  const next = `${window.location.pathname}?${p.toString()}`;
  window.history.replaceState(null, '', next);
}
