import type { RangeKey } from '../state/types';

// ── Chart colors ───────────────────────────────────────────────────────
export const TOKEN_COLORS: Record<string, string> = {
  input:          'rgba(59,130,246,0.8)',   // blue
  output:         'rgba(167,139,250,0.8)',  // purple
  cache_read:     'rgba(34,197,94,0.5)',    // green
  cache_creation: 'rgba(234,179,8,0.5)',    // yellow
};
export const MODEL_COLORS = ['#6366f1', '#3b82f6', '#22c55e', '#a78bfa', '#eab308', '#f472b6', '#14b8a6', '#60a5fa'];

// ── Time range ─────────────────────────────────────────────────────────
export const RANGE_LABELS: Record<RangeKey, string> = {
  '7d': 'Last 7 Days', '30d': 'Last 30 Days', '90d': 'Last 90 Days', 'all': 'All Time',
};
export const RANGE_TICKS: Record<RangeKey, number> = { '7d': 7, '30d': 15, '90d': 13, 'all': 12 };

// ── ApexCharts theme helper ───────────────────────────────────────────
export function apexThemeMode(): 'light' | 'dark' {
  return document.documentElement.getAttribute('data-theme') === 'light' ? 'light' : 'dark';
}

export function cssVar(name: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
}
