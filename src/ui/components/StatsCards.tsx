import { fmt, fmtCostBig } from '../lib/format';
import { RANGE_LABELS } from '../lib/charts';
import { selectedRange } from '../state/store';
import type { Totals, StatCard } from '../state/types';

export function StatsCards({ totals }: { totals: Totals }) {
  const rangeLabel = RANGE_LABELS[selectedRange.value].toLowerCase();
  const stats: StatCard[] = [
    { label: 'Sessions',       value: totals.sessions.toLocaleString(), sub: rangeLabel },
    { label: 'Turns',          value: fmt(totals.turns),                sub: rangeLabel },
    { label: 'Input Tokens',   value: fmt(totals.input),                sub: rangeLabel },
    { label: 'Output Tokens',  value: fmt(totals.output),               sub: rangeLabel },
    { label: 'Cache Read',     value: fmt(totals.cache_read),           sub: 'prompt cache' },
    { label: 'Cache Creation', value: fmt(totals.cache_creation),       sub: 'cache writes' },
    { label: 'Est. Cost',      value: fmtCostBig(totals.cost),          sub: 'API pricing', isCost: true },
  ];

  return (
    <>
      {stats.map(s => (
        <div class="card stat-card" key={s.label}>
          <div class="stat-content">
            <div class="stat-label">{s.label}</div>
            <div class={`stat-value ${s.isCost ? 'cost-value' : ''}`}>{s.value}</div>
            {s.sub ? <div class="stat-sub">{s.sub}</div> : null}
          </div>
        </div>
      ))}
    </>
  );
}
