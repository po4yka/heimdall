import { ApexChart } from './ApexChart';
import { industrialChartOptions, modelSeriesColors, cssVar } from '../lib/charts';
import { esc, fmt, fmtCost } from '../lib/format';
import type { VersionSummary } from '../state/types';
import type { VersionMetric } from '../state/store';

export interface VersionDonutProps {
  rows: VersionSummary[];
  metric: VersionMetric;
  onMetricChange: (next: VersionMetric) => void;
}

const METRIC_LABELS: Record<VersionMetric, string> = {
  cost: 'Cost',
  calls: 'Calls',
  tokens: 'Tokens',
};

function metricValue(row: VersionSummary, metric: VersionMetric): number {
  switch (metric) {
    case 'cost': return row.cost;
    case 'calls': return row.turns;
    case 'tokens': return row.tokens;
  }
}

function formatMetricTotal(total: number, metric: VersionMetric): string {
  if (metric === 'cost') return fmtCost(total);
  return fmt(total);
}

function formatMetricSlice(val: number, metric: VersionMetric): string {
  if (metric === 'cost') return fmtCost(val);
  return fmt(val);
}

export function VersionDonut({ rows, metric, onMetricChange }: VersionDonutProps) {
  if (!rows.length) return null;

  // Normalize empty version strings to "(unknown)" for display.
  const normalized = rows.map(r => ({
    ...r,
    version: r.version === '' || r.version === 'unknown' ? '(unknown)' : r.version,
  }));

  const series = normalized.map(r => metricValue(r, metric));
  const labels = normalized.map(r => r.version);
  const total = series.reduce((s, v) => s + v, 0);

  const base = industrialChartOptions('donut');
  const options = {
    ...base,
    chart: { ...base.chart, type: 'donut' },
    series,
    labels,
    colors: modelSeriesColors(normalized.length),
    stroke: { width: 2, colors: [cssVar('--surface')] },
    plotOptions: {
      pie: {
        donut: {
          size: '64%',
          labels: {
            show: true,
            total: {
              show: true,
              label: 'TOTAL',
              fontFamily: 'var(--font-mono), "Space Mono", monospace',
              fontSize: '11px',
              color: cssVar('--text-secondary'),
              formatter: () => formatMetricTotal(total, metric),
            },
            value: {
              fontFamily: 'var(--font-mono), "Space Mono", monospace',
              fontSize: '18px',
              color: cssVar('--text-display'),
              formatter: (val: string) => formatMetricSlice(Number(val), metric),
            },
            name: {
              fontFamily: 'var(--font-mono), "Space Mono", monospace',
              fontSize: '11px',
              color: cssVar('--text-secondary'),
            },
          },
        },
      },
    },
    tooltip: {
      ...base.tooltip,
      custom: ({ seriesIndex }: { seriesIndex: number }) => {
        const r = normalized[seriesIndex];
        if (!r) return '';
        const label = esc(r.version);
        const cost = fmtCost(r.cost);
        const calls = fmt(r.turns);
        const tokens = fmt(r.tokens);
        return (
          `<div style="padding:8px 12px;font-family:var(--font-mono,'Space Mono',monospace);font-size:11px;line-height:1.6">` +
          `<strong>${label}</strong><br/>` +
          `${cost} &nbsp;&bull;&nbsp; ${calls} calls &nbsp;&bull;&nbsp; ${tokens} tokens` +
          `</div>`
        );
      },
    },
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '8px', height: '100%' }}>
      <div style={{ display: 'flex', gap: '4px', alignItems: 'center' }}>
        {(Object.keys(METRIC_LABELS) as VersionMetric[]).map(m => (
          <button
            key={m}
            type="button"
            class={`range-btn${metric === m ? ' active' : ''}`}
            onClick={() => onMetricChange(m)}
          >
            {METRIC_LABELS[m]}
          </button>
        ))}
      </div>
      <div style={{ flex: 1, minHeight: 0 }}>
        <ApexChart options={options} id="chart-version-donut" />
      </div>
    </div>
  );
}
