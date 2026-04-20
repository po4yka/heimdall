import { ApexChart } from './ApexChart';
import { industrialChartOptions, cssVar, withAlpha } from '../../lib/charts';
import { esc, fmt, fmtCost, fmtCostCompact } from '../../lib/format';
import type { VersionSummary } from '../../state/types';
import type { VersionMetric } from '../../state/store';

export interface VersionDonutProps {
  rows: VersionSummary[];
  metric: VersionMetric;
  onMetricChange: (next: VersionMetric) => void;
}

interface DonutRow {
  label: string;
  value: number;
  share: number;
  cost: number;
  calls: number;
  tokens: number;
  color: string;
  isOther: boolean;
}

const METRIC_LABELS: Record<VersionMetric, string> = {
  cost: 'Cost',
  calls: 'Calls',
  tokens: 'Tokens',
};

const SLICE_OPACITY_LADDER = [1.0, 0.64, 0.46, 0.34, 0.24, 0.16];
const TOP_N = 5;

function metricValue(row: VersionSummary, metric: VersionMetric): number {
  switch (metric) {
    case 'cost': return row.cost;
    case 'calls': return row.turns;
    case 'tokens': return row.tokens;
  }
}

function formatMetricValue(value: number, metric: VersionMetric, large = false): string {
  switch (metric) {
    case 'cost': return large ? fmtCostCompact(value) : fmtCost(value);
    case 'calls':
    case 'tokens': return fmt(value);
  }
}

function formatShare(share: number): string {
  if (share >= 99.5) return '100%';
  if (share >= 10) return `${share.toFixed(0)}%`;
  if (share >= 0.1) return `${share.toFixed(1)}%`;
  if (share > 0) return '<0.1%';
  return '0%';
}

export function VersionDonut({ rows, metric, onMetricChange }: VersionDonutProps) {
  if (!rows.length) return null;

  const normalized = rows.map(r => ({
    ...r,
    version: r.version === '' || r.version === 'unknown' ? '(unknown)' : r.version,
  }));

  const sorted = normalized
    .map(r => ({ row: r, value: metricValue(r, metric) }))
    .filter(entry => entry.value > 0)
    .sort((a, b) => b.value - a.value);

  if (!sorted.length) return null;

  const top = sorted.slice(0, TOP_N);
  const rest = sorted.slice(TOP_N);
  const total = sorted.reduce((s, e) => s + e.value, 0);

  const donutRows: DonutRow[] = top.map((entry, index) => ({
    label: entry.row.version,
    value: entry.value,
    share: total > 0 ? (entry.value / total) * 100 : 0,
    cost: entry.row.cost,
    calls: entry.row.turns,
    tokens: entry.row.tokens,
    color: withAlpha('--text-display', SLICE_OPACITY_LADDER[Math.min(index, SLICE_OPACITY_LADDER.length - 1)] ?? 0.16),
    isOther: false,
  }));

  const otherValue = rest.reduce((s, e) => s + e.value, 0);
  const hasOther = otherValue > 0;
  if (hasOther) {
    donutRows.push({
      label: `Other (${rest.length})`,
      value: otherValue,
      share: total > 0 ? (otherValue / total) * 100 : 0,
      cost: rest.reduce((s, e) => s + e.row.cost, 0),
      calls: rest.reduce((s, e) => s + e.row.turns, 0),
      tokens: rest.reduce((s, e) => s + e.row.tokens, 0),
      color: withAlpha('--text-display', SLICE_OPACITY_LADDER[Math.min(donutRows.length, SLICE_OPACITY_LADDER.length - 1)] ?? 0.16),
      isOther: true,
    });
  }

  const base = industrialChartOptions('donut');
  const options = {
    ...base,
    chart: { ...base.chart, type: 'donut' },
    series: donutRows.map(r => r.value),
    labels: donutRows.map(r => r.label),
    colors: donutRows.map(r => r.color),
    stroke: { width: 2, colors: [cssVar('--surface')] },
    legend: { ...base.legend, show: false },
    states: {
      hover: { filter: { type: 'none', value: 0 } },
      active: { filter: { type: 'none', value: 0 } },
    },
    tooltip: {
      ...base.tooltip,
      custom: ({ seriesIndex }: { seriesIndex: number }) => {
        const r = donutRows[seriesIndex];
        if (!r) return '';
        return (
          `<div style="padding:8px 12px;font-family:var(--font-mono,'Space Mono',monospace);font-size:11px;line-height:1.6">` +
          `<strong>${esc(r.label)}</strong><br/>` +
          `${esc(METRIC_LABELS[metric])}: ${esc(formatMetricValue(r.value, metric))} ` +
          `(${esc(formatShare(r.share))} share)<br/>` +
          `Cost: ${esc(fmtCost(r.cost))} &nbsp;&bull;&nbsp; ` +
          `Calls: ${esc(fmt(r.calls))} &nbsp;&bull;&nbsp; ` +
          `Tokens: ${esc(fmt(r.tokens))}` +
          `</div>`
        );
      },
    },
    plotOptions: {
      pie: {
        expandOnClick: false,
        donut: {
          size: '72%',
          labels: { show: false },
        },
      },
    },
  };

  return (
    <div class="model-chart-panel">
      <div class="range-group" aria-label="Version metric">
        {(Object.keys(METRIC_LABELS) as VersionMetric[]).map(m => (
          <button
            key={m}
            type="button"
            class={`range-btn${metric === m ? ' active' : ''}`}
            aria-pressed={metric === m}
            onClick={() => onMetricChange(m)}
          >
            {METRIC_LABELS[m]}
          </button>
        ))}
      </div>
      <div class="model-chart-ring">
        <ApexChart options={options} id="chart-version-donut" />
        <div class="model-chart-center" aria-hidden="true">
          <div class="model-chart-center-inner">
            <div class="model-chart-center-kicker">Total {METRIC_LABELS[metric]}</div>
            <div class="model-chart-center-total">{formatMetricValue(total, metric, true)}</div>
            {hasOther ? <div class="model-chart-center-meta">Top {TOP_N} + Other</div> : null}
          </div>
        </div>
      </div>
    </div>
  );
}
