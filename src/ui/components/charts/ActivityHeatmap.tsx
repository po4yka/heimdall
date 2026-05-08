// Heatmap aggregates by hour/day across all projects; per-project breakdown is
// intentionally not surfaced here. If we add it, decide first:
//   1. Filter integration: does the existing FilterBar `project` filter scope
//      the heatmap, or does the heatmap need its own selector?
//   2. Tooltip shape: top-N projects per cell with shares, or one project at a
//      time driven by a hover-locked legend swatch?
//   3. Cell encoding: keep the monochrome opacity ladder (one project = full
//      opacity, others dimmed) or switch to small-multiples (7×24 grid per
//      project, capped at top-N)?
// Until those questions are answered the all-projects view is the contract.
import type { Signal } from '@preact/signals';
import { withAlpha } from '../../lib/charts';
import { fmt, fmtCost, fmtCostBig, fmtCostCompact, fmtTzOffset } from '../../lib/format';
import type { HeatmapData } from '../../state/types';
import type { HeatmapMetric } from '../../state/store';

const DOW_LABELS = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
const METRIC_LABELS: Record<HeatmapMetric, string> = {
  cost: 'Cost',
  calls: 'Calls',
};
const LEGEND_STEPS = [0.05, 0.2, 0.4, 0.6, 0.9];

export interface ActivityHeatmapProps {
  data: HeatmapData;
  metric: Signal<HeatmapMetric>;
  onMetricChange: (next: HeatmapMetric) => void;
}

function cellOpacity(value: number, max: number): number {
  if (max <= 0 || value <= 0) return 0.05;
  const ratio = value / max;
  return Math.min(0.05 + 0.85 * ratio, 0.90);
}

function formatPeak(value: number, metric: HeatmapMetric): string {
  if (metric === 'cost') {
    if (value >= 1000) return fmtCostCompact(value);
    return fmtCostBig(value);
  }
  return fmt(value);
}

export function ActivityHeatmap({ data, metric: metricSignal, onMetricChange }: ActivityHeatmapProps) {
  const metric = metricSignal.value;
  const {
    cells,
    max_cost_nanos,
    max_call_count,
    active_days,
    total_cost_nanos,
    period,
    tz_offset_min,
  } = data;

  const lookup = new Map<string, { cost_nanos: number; call_count: number }>();
  for (const c of cells) lookup.set(`${c.dow},${c.hour}`, c);

  const avgPerDayUsd =
    active_days > 0 ? total_cost_nanos / 1_000_000_000 / active_days : 0;
  const avgPerDay = active_days > 0 ? fmtCostBig(avgPerDayUsd) : '\u2014';

  const metricMaxRaw = metric === 'cost' ? max_cost_nanos : max_call_count;
  const metricMaxDisplay =
    metric === 'cost' ? metricMaxRaw / 1_000_000_000 : metricMaxRaw;

  let peakKey: string | null = null;
  let peakVal = 0;
  for (const c of cells) {
    const v = metric === 'cost' ? c.cost_nanos : c.call_count;
    if (v > peakVal) {
      peakVal = v;
      peakKey = `${c.dow},${c.hour}`;
    }
  }

  return (
    <div class="heatmap-panel">
      <div class="heatmap-header">
        <span class="heatmap-title">
          Activity / 7x24 / {period}
        </span>
        <span class="heatmap-subtitle">
          {active_days} active {active_days === 1 ? 'day' : 'days'}
          {' \u00b7 '}
          {avgPerDay} per active day
          {' \u00b7 '}
          {fmtTzOffset(tz_offset_min)}
        </span>
        <div class="range-group heatmap-metric" aria-label="Heatmap metric">
          {(Object.keys(METRIC_LABELS) as HeatmapMetric[]).map(m => (
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
      </div>

      <div
        class="heatmap-grid"
        role="figure"
        aria-label="Activity heatmap: 7 days by 24 hours"
      >
        <div />
        <div class="heatmap-hour-labels" aria-hidden="true">
          <span>00</span>
          <span>06</span>
          <span>12</span>
          <span>18</span>
        </div>

        {Array.from({ length: 7 }, (_, dow) => {
          const isWeekend = dow === 0 || dow === 6;
          return [
            <div key={`label-${dow}`} class="heatmap-dow-label">
              {DOW_LABELS[dow]}
            </div>,
            ...Array.from({ length: 24 }, (_, hour) => {
              const key = `${dow},${hour}`;
              const cell = lookup.get(key);
              const costNanos = cell?.cost_nanos ?? 0;
              const callCount = cell?.call_count ?? 0;
              const raw = metric === 'cost' ? costNanos : callCount;
              const opacity = cellOpacity(raw, metricMaxRaw);
              const bg = withAlpha('--text-display', opacity);
              const costUsd = costNanos / 1_000_000_000;
              const title =
                `${DOW_LABELS[dow]} ${String(hour).padStart(2, '0')}:00 — ` +
                `${fmtCost(costUsd)} / ${callCount} call${callCount !== 1 ? 's' : ''}`;
              const isPeak = key === peakKey && peakVal > 0;
              const classes = [
                'heatmap-cell',
                isWeekend ? 'heatmap-cell--weekend' : '',
                isPeak ? 'heatmap-cell--peak' : '',
              ]
                .filter(Boolean)
                .join(' ');
              return (
                <div
                  key={key}
                  role="img"
                  aria-label={title}
                  title={title}
                  class={classes}
                  style={{ background: bg }}
                />
              );
            }),
          ];
        })}
      </div>

      <div class="heatmap-legend" aria-hidden="true">
        <span>Less</span>
        <div class="heatmap-legend-track">
          {LEGEND_STEPS.map((op, i) => (
            <div
              key={i}
              class="heatmap-legend-step"
              style={{ background: withAlpha('--text-display', op) }}
            />
          ))}
        </div>
        <span>
          More
          {peakVal > 0
            ? ` (peak ${formatPeak(metricMaxDisplay, metric)})`
            : ''}
        </span>
      </div>
    </div>
  );
}
