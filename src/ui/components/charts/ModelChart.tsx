import { useState } from 'preact/hooks';
import { ApexChart } from './ApexChart';
import { industrialChartOptions, cssVar, withAlpha } from '../../lib/charts';
import { esc, fmt, fmtCost, fmtCostBig, truncateMid } from '../../lib/format';
import type { ModelAgg } from '../../state/types';

type ModelMetric = 'cost' | 'tokens' | 'calls';

interface MetricRow {
  label: string;
  value: number;
  share: number;
  cost: number;
  calls: number;
  tokens: number;
  color: string;
  isOther: boolean;
}

const METRIC_LABELS: Record<ModelMetric, string> = {
  cost: 'Cost',
  tokens: 'Tokens',
  calls: 'Calls',
};

const SLICE_OPACITY_LADDER = [1.0, 0.64, 0.46, 0.34, 0.24, 0.16];

function totalTokens(row: ModelAgg): number {
  return row.input + row.output + row.cache_read + row.cache_creation + row.reasoning_output;
}

function metricValue(row: ModelAgg, metric: ModelMetric): number {
  switch (metric) {
    case 'cost':
      return row.cost;
    case 'tokens':
      return totalTokens(row);
    case 'calls':
      return row.turns;
  }
}

function formatMetricValue(value: number, metric: ModelMetric, large = false): string {
  switch (metric) {
    case 'cost':
      return large ? (value < 1 ? fmtCost(value) : fmtCostBig(value)) : fmtCost(value);
    case 'tokens':
    case 'calls':
      return fmt(value);
  }
}

function formatShare(share: number): string {
  if (share >= 99.5) return '100%';
  if (share >= 10) return `${share.toFixed(0)}%`;
  return `${share.toFixed(1)}%`;
}

export function ModelChart({
  byModel,
  onSelectModel,
}: {
  byModel: ModelAgg[];
  onSelectModel?: (model: string) => void;
}) {
  if (!byModel.length) return null;

  const [selectedMetric, setSelectedMetric] = useState<ModelMetric>('cost');

  const totals: Record<ModelMetric, number> = {
    cost: byModel.reduce((sum, row) => sum + row.cost, 0),
    tokens: byModel.reduce((sum, row) => sum + totalTokens(row), 0),
    calls: byModel.reduce((sum, row) => sum + row.turns, 0),
  };
  const enabledMetrics = (Object.keys(METRIC_LABELS) as ModelMetric[]).filter(metric => totals[metric] > 0);
  const metric = enabledMetrics.includes(selectedMetric) ? selectedMetric : (enabledMetrics[0] ?? 'cost');

  const sorted = [...byModel]
    .map(row => ({
      row,
      value: metricValue(row, metric),
      tokens: totalTokens(row),
    }))
    .filter(entry => entry.value > 0)
    .sort((a, b) => b.value - a.value);

  if (!sorted.length) return null;

  const TOP_N = 5;
  const top = sorted.slice(0, TOP_N);
  const rest = sorted.slice(TOP_N);
  const total = sorted.reduce((sum, entry) => sum + entry.value, 0);
  const rows: MetricRow[] = top.map((entry, index) => ({
    label: entry.row.model,
    value: entry.value,
    share: total > 0 ? (entry.value / total) * 100 : 0,
    cost: entry.row.cost,
    calls: entry.row.turns,
    tokens: entry.tokens,
    color: withAlpha('--text-display', SLICE_OPACITY_LADDER[Math.min(index, SLICE_OPACITY_LADDER.length - 1)] ?? 0.16),
    isOther: false,
  }));

  const otherValue = rest.reduce((sum, entry) => sum + entry.value, 0);
  const hasOther = otherValue > 0;
  if (hasOther) {
    rows.push({
      label: `Other (${rest.length})`,
      value: otherValue,
      share: total > 0 ? (otherValue / total) * 100 : 0,
      cost: rest.reduce((sum, entry) => sum + entry.row.cost, 0),
      calls: rest.reduce((sum, entry) => sum + entry.row.turns, 0),
      tokens: rest.reduce((sum, entry) => sum + entry.tokens, 0),
      color: withAlpha('--text-display', SLICE_OPACITY_LADDER[Math.min(rows.length, SLICE_OPACITY_LADDER.length - 1)] ?? 0.16),
      isOther: true,
    });
  }

  const base = industrialChartOptions('donut');
  const options = {
    ...base,
    chart: {
      ...base.chart,
      type: 'donut',
      ...(onSelectModel
        ? {
            events: {
              dataPointSelection: (
                _event: unknown,
                _ctx: unknown,
                config: { dataPointIndex: number }
              ) => {
                const row = rows[config.dataPointIndex];
                if (row && !row.isOther) onSelectModel(row.label);
              },
            },
          }
        : {}),
    },
    series: rows.map(row => row.value),
    labels: rows.map(row => row.label),
    colors: rows.map(row => row.color),
    stroke: { width: 2, colors: [cssVar('--surface')] },
    legend: { ...base.legend, show: false },
    states: {
      hover: { filter: { type: 'none', value: 0 } },
      active: { filter: { type: 'none', value: 0 } },
    },
    tooltip: {
      ...base.tooltip,
      custom: ({ seriesIndex }: { seriesIndex: number }) => {
        const row = rows[seriesIndex];
        if (!row) return '';
        return (
          `<div style="padding:8px 12px;font-family:var(--font-mono,'Space Mono',monospace);font-size:11px;line-height:1.6">` +
          `<strong>${esc(row.label)}</strong><br/>` +
          `${esc(METRIC_LABELS[metric])}: ${esc(formatMetricValue(row.value, metric))} ` +
          `(${esc(formatShare(row.share))} share)<br/>` +
          `Cost: ${esc(formatMetricValue(row.cost, 'cost'))} &nbsp;&bull;&nbsp; ` +
          `Calls: ${esc(formatMetricValue(row.calls, 'calls'))} &nbsp;&bull;&nbsp; ` +
          `Tokens: ${esc(formatMetricValue(row.tokens, 'tokens'))}` +
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
      <div class="range-group" aria-label="Model metric">
        {(Object.keys(METRIC_LABELS) as ModelMetric[]).map(nextMetric => (
          <button
            key={nextMetric}
            type="button"
            class={`range-btn${metric === nextMetric ? ' active' : ''}`}
            disabled={totals[nextMetric] <= 0}
            aria-pressed={metric === nextMetric}
            onClick={() => setSelectedMetric(nextMetric)}
          >
            {METRIC_LABELS[nextMetric]}
          </button>
        ))}
      </div>

      <div class="model-chart-ring">
        <ApexChart options={options} id="chart-model-apex" />
        <div class="model-chart-center" aria-hidden="true">
          <div class="model-chart-center-inner">
            <div class="model-chart-center-kicker">{METRIC_LABELS[metric]}</div>
            <div class="model-chart-center-total">{formatMetricValue(total, metric, true)}</div>
            {hasOther ? <div class="model-chart-center-meta">Top 5 + Other</div> : null}
          </div>
        </div>
      </div>

      <div class="model-share-list">
        {rows.map(row => (
          <button
            key={row.label}
            type="button"
            class={`model-share-row${onSelectModel && !row.isOther ? ' interactive' : ''}`}
            onClick={onSelectModel && !row.isOther ? () => onSelectModel(row.label) : undefined}
            disabled={!onSelectModel || row.isOther}
            aria-label={row.isOther ? `${row.label} ${METRIC_LABELS[metric]} summary` : `Filter to ${row.label}`}
          >
            <div class="model-share-row-head">
              <div class="model-share-label">
                <span class="model-share-swatch" style={{ background: row.color }} aria-hidden="true" />
                <span title={row.label}>{truncateMid(row.label, row.isOther ? 18 : 24, 8)}</span>
              </div>
              <div class="model-share-value">{formatMetricValue(row.value, metric)}</div>
            </div>
            <div class="model-share-row-meta">
              <div class="model-share-bar" aria-label={`${row.label} ${METRIC_LABELS[metric]} share`}>
                <div class="model-share-bar-fill" style={{ width: `${Math.min(100, row.share)}%`, background: row.color }} />
              </div>
              <div class="model-share-percent">{formatShare(row.share)}</div>
            </div>
          </button>
        ))}
      </div>
    </div>
  );
}
