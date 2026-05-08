import type { Signal } from '@preact/signals';
import { MetricDonut } from './MetricDonut';
import { fmtCost, fmtCostCompact, fmt } from '../../lib/format';
import type { VersionSummary } from '../../state/types';
import type { VersionMetric } from '../../state/store';

export interface VersionDonutProps {
  rows: VersionSummary[];
  metric: Signal<VersionMetric>;
  onMetricChange: (next: VersionMetric) => void;
}

const METRIC_LABELS: Record<VersionMetric, string> = {
  cost: 'Cost',
  calls: 'Calls',
  tokens: 'Tokens',
};

const METRIC_OPTIONS: VersionMetric[] = ['cost', 'calls', 'tokens'];

function getMetricValue(row: VersionSummary, metric: VersionMetric): number {
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

export function VersionDonut({ rows, metric, onMetricChange }: VersionDonutProps) {
  const metricValue = metric.value;
  const normalized = rows.map(r => ({
    ...r,
    version: r.version === '' || r.version === 'unknown' ? '(unknown)' : r.version,
  }));

  return MetricDonut<VersionSummary & { version: string }, VersionMetric>({
    rows: normalized,
    metric: metricValue,
    metricOptions: METRIC_OPTIONS,
    metricLabel: m => METRIC_LABELS[m],
    metricValue: getMetricValue,
    metricFormat: formatMetricValue,
    rowLabel: row => row.version,
    rowCost: row => row.cost,
    rowCalls: row => row.turns,
    rowTokens: row => row.tokens,
    id: 'chart-version-donut',
    centerKickerPrefix: 'Total',
    onMetricChange,
    showLegend: false,
    formatCost: v => fmtCost(v),
    formatCalls: v => fmt(v),
    formatTokens: v => fmt(v),
  });
}
