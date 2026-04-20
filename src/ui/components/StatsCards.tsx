import { fmt, fmtCostBig } from '../lib/format';
import { Sparkline } from './charts/Sparkline';
import { CacheEfficiencyCard } from './CacheEfficiencyCard';
import { BillingBlocksCard } from './BillingBlocksCard';
import { ContextWindowCard } from './ContextWindowCard';
import type { Totals, StatCard, DailyAgg, CacheEfficiency, BillingBlocksResponse, ContextWindowResponse } from '../state/types';

interface StatsCardsProps {
  totals: Totals;
  daily?: DailyAgg[] | undefined;
  /** Active-period average: days with non-zero spend in the current filter. */
  activeDays?: number | undefined;
  /** Total cost nanos across the active-day calculation input. */
  activeDayTotalCostNanos?: number | undefined;
  /** Phase 21: cache-efficiency aggregate from /api/data. */
  cacheEfficiency?: CacheEfficiency | undefined;
  /** Phase 2: billing blocks data from /api/billing-blocks. */
  billingBlocks?: BillingBlocksResponse | null;
  /** Phase 5: context window data from /api/context-window. */
  contextWindow?: ContextWindowResponse | null;
}

export function StatsCards({
  totals,
  daily,
  activeDays,
  activeDayTotalCostNanos,
  cacheEfficiency,
  billingBlocks,
  contextWindow,
}: StatsCardsProps) {
  // Active-period average: divide total by active days.
  // Displays "--" when no active days (empty range).
  const avgPerActiveDay: string = (() => {
    if (activeDays === undefined || activeDays === null) return '--';
    if (activeDays === 0) return '--';
    const totalUsd = (activeDayTotalCostNanos ?? 0) / 1_000_000_000;
    return fmtCostBig(totalUsd / activeDays);
  })();

  const activeDayTooltip = activeDays !== undefined && activeDays !== null && activeDays > 0
    ? `Averaged over ${activeDays} day${activeDays === 1 ? '' : 's'} with non-zero spend`
    : 'No spend in selected period';

  const stats: StatCard[] = [
    { label: 'Sessions',       value: totals.sessions.toLocaleString(), sub: '' },
    { label: 'Turns',          value: fmt(totals.turns),                sub: '' },
    { label: 'Input Tokens',   value: fmt(totals.input),                sub: '' },
    { label: 'Output Tokens',  value: fmt(totals.output),               sub: '' },
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
            <div class="stat-value">{s.value}</div>
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
      {/* Phase 2: Billing block quota card — most time-sensitive, rendered first */}
      {billingBlocks && (
        <BillingBlocksCard data={billingBlocks} />
      )}
      {/* Phase 5: Context window card — hides automatically when data unavailable */}
      <ContextWindowCard data={contextWindow ?? null} />
      {/* Phase 21: Cache hit rate card */}
      {cacheEfficiency && (
        <CacheEfficiencyCard data={cacheEfficiency} />
      )}
    </>
  );
}
