import { fmt, fmtCost, fmtCostBig } from '../lib/format';
import { RANGE_LABELS } from '../lib/charts';
import { selectedRange } from '../state/store';
import { Sparkline } from './Sparkline';
import type { Totals, StatCard, DailyAgg } from '../state/types';

interface StatsCardsProps {
  totals: Totals;
  daily?: DailyAgg[];
  /** Active-period average: days with non-zero spend. From /api/heatmap. */
  activeDays?: number;
  /** Total cost nanos for the heatmap period (matches activeDays). */
  heatmapTotalNanos?: number;
  /** Total calendar days in the heatmap period (for tooltip). */
  calendarDays?: number;
}

export function StatsCards({ totals, daily, activeDays, heatmapTotalNanos }: StatsCardsProps) {
  const rangeLabel = RANGE_LABELS[selectedRange.value].toLowerCase();

  // Active-period average: divide total by active days.
  // Displays "--" when no active days (empty range).
  const avgPerActiveDay: string = (() => {
    if (activeDays === undefined || activeDays === null) return '--';
    if (activeDays === 0) return '--';
    const totalUsd = (heatmapTotalNanos ?? 0) / 1_000_000_000;
    return fmtCost(totalUsd / activeDays);
  })();

  const activeDayTooltip = activeDays !== undefined && activeDays !== null && activeDays > 0
    ? `Averaged over ${activeDays} day${activeDays === 1 ? '' : 's'} with non-zero spend`
    : 'No spend in selected period';

  const stats: StatCard[] = [
    { label: 'Sessions',       value: totals.sessions.toLocaleString(), sub: rangeLabel },
    { label: 'Turns',          value: fmt(totals.turns),                sub: rangeLabel },
    { label: 'Input Tokens',   value: fmt(totals.input),                sub: rangeLabel },
    { label: 'Output Tokens',  value: fmt(totals.output),               sub: rangeLabel },
    { label: 'Cached Input',   value: fmt(totals.cache_read),           sub: 'prompt cache' },
    { label: 'Cache Creation', value: fmt(totals.cache_creation),       sub: 'cache writes' },
    { label: 'Reasoning',      value: fmt(totals.reasoning_output),     sub: 'subset of output' },
    { label: 'Est. Cost',      value: fmtCostBig(totals.cost),          sub: 'API pricing', isCost: true },
  ];

  return (
    <>
      {stats.map(s => (
        <div class="card stat-card" key={s.label}>
          <div class="stat-content">
            <div class="stat-label">{s.label}</div>
            <div class={`stat-value${s.isCost ? ' cost-value doto-hero' : ''}`}>{s.value}</div>
            {s.sub ? <div class="stat-sub">{s.sub}</div> : null}
          </div>
          {s.isCost && daily && daily.length >= 2 ? (
            <div class="stat-sparkline">
              <Sparkline daily={daily} />
            </div>
          ) : null}
        </div>
      ))}
      {/* Active-period average cost card (Phase 13) */}
      <div class="card stat-card" title={activeDayTooltip}>
        <div class="stat-content">
          <div class="stat-label">Avg / Active Day</div>
          <div class="stat-value">{avgPerActiveDay}</div>
          <div class="stat-sub">
            {activeDays !== undefined && activeDays !== null && activeDays > 0
              ? `${activeDays} active ${activeDays === 1 ? 'day' : 'days'}`
              : 'no spend'}
          </div>
        </div>
      </div>
    </>
  );
}
