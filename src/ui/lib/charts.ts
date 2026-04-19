import type { ApexOptions } from './apex';
import type { RangeKey } from '../state/types';

// ── Time range ─────────────────────────────────────────────────────────
export const RANGE_LABELS: Record<RangeKey, string> = {
  '7d': 'Last 7 Days', '30d': 'Last 30 Days', '90d': 'Last 90 Days', 'all': 'All Time',
};
export const RANGE_TICKS: Record<RangeKey, number> = { '7d': 7, '30d': 15, '90d': 13, 'all': 12 };

// ── Runtime CSS variable readers ───────────────────────────────────────
export function apexThemeMode(): 'light' | 'dark' {
  return document.documentElement.getAttribute('data-theme') === 'light' ? 'light' : 'dark';
}

export function cssVar(name: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
}

function hexToRgba(hex: string, alpha: number): string {
  let h = hex.trim();
  if (h.startsWith('#')) h = h.slice(1);
  if (h.length === 3) h = h.split('').map(c => c + c).join('');
  if (h.length !== 6) return hex; // unparseable — return as-is
  const r = parseInt(h.slice(0, 2), 16);
  const g = parseInt(h.slice(2, 4), 16);
  const b = parseInt(h.slice(4, 6), 16);
  return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}

export function withAlpha(varName: string, alpha: number): string {
  return hexToRgba(cssVar(varName), alpha);
}

// ── Monochrome series palettes ─────────────────────────────────────────
// Token palette: opacity ladder off --text-display.
// input -> 100%, output -> 60%, cache_read -> 30%, cache_creation -> 15%.
export function tokenSeriesColors(): string[] {
  return [
    withAlpha('--text-display', 1.0),
    withAlpha('--text-display', 0.6),
    withAlpha('--text-display', 0.3),
    withAlpha('--text-display', 0.15),
  ];
}

// Categorical palette for donuts / model distribution.
// Base four: --text-display, --success, --warning, --interactive.
// Overflow cycles the base with decreasing opacity.
export function modelSeriesColors(n: number): string[] {
  const baseVars = ['--text-display', '--success', '--warning', '--interactive'] as const;
  const out: string[] = [];
  for (let i = 0; i < n; i++) {
    const slot = i % baseVars.length;
    const cycle = Math.floor(i / baseVars.length);
    const alpha = Math.max(0.25, 1 - cycle * 0.25);
    const v = baseVars[slot]!;
    out.push(cycle === 0 ? cssVar(v) : withAlpha(v, alpha));
  }
  return out;
}

// ── Industrial base ApexCharts options ────────────────────────────────
export function industrialChartOptions(type: 'bar' | 'donut' | 'line'): ApexOptions {
  const axisLabelStyle = {
    colors: cssVar('--text-secondary'),
    fontFamily: 'var(--font-mono), "Space Mono", monospace',
    fontSize: '11px',
    letterSpacing: '0.04em',
  };

  const base: ApexOptions = {
    chart: {
      type,
      height: '100%',
      background: 'transparent',
      toolbar: { show: false },
      fontFamily: 'var(--font-mono), "Space Mono", monospace',
      animations: { enabled: false },
    },
    theme: { mode: apexThemeMode() },
    legend: {
      show: true,
      position: type === 'donut' ? 'bottom' : 'top',
      fontFamily: 'var(--font-mono), "Space Mono", monospace',
      fontSize: '11px',
      labels: { colors: cssVar('--text-secondary') },
      markers: { width: 8, height: 8, radius: 0 },
      itemMargin: { horizontal: 12, vertical: 4 },
    },
    grid: {
      borderColor: cssVar('--border'),
      strokeDashArray: 0,
      xaxis: { lines: { show: false } },
      yaxis: { lines: { show: type !== 'donut' } },
    },
    xaxis: {
      labels: { style: axisLabelStyle },
      axisBorder: { color: cssVar('--border-visible') },
      axisTicks: { color: cssVar('--border-visible') },
    },
    yaxis: {
      labels: { style: axisLabelStyle },
    },
    stroke: { width: type === 'line' ? 1.5 : 0, curve: 'straight' },
    tooltip: {
      theme: apexThemeMode(),
      style: { fontFamily: 'var(--font-mono), "Space Mono", monospace', fontSize: '11px' },
    },
    dataLabels: { enabled: false },
  };

  if (type === 'line') {
    if (base.legend) base.legend.show = false;
    base.fill = { type: 'solid', opacity: 0 };
  }

  return base;
}
