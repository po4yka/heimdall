import { fmtCostBig, fmt } from '../../lib/format';
import type { TodayTotals } from '../../state/types';

interface TodayKpisProps {
  totals: TodayTotals;
  day: string;
}

export function TodayKpis({ totals, day }: TodayKpisProps) {
  const costUsd = totals.cost_nanos / 1_000_000_000;
  const peakLabel =
    totals.peak_hour !== null
      ? `${String(totals.peak_hour).padStart(2, '0')}:00`
      : '--';

  const cards = [
    { label: 'Cost', value: fmtCostBig(costUsd), sub: day },
    { label: 'Tokens', value: fmt(totals.total_tokens), sub: 'input + output + cache' },
    { label: 'Turns', value: fmt(totals.turns), sub: 'API calls' },
    { label: 'Peak hour', value: peakLabel, sub: 'highest cost hour' },
  ];

  return (
    <div class="today-kpi-grid">
      {cards.map(card => (
        <div key={card.label} class="stat-card">
          <div class="stat-label">{card.label}</div>
          <div class="stat-value">{card.value}</div>
          {card.sub && <div class="stat-sub">{card.sub}</div>}
        </div>
      ))}
    </div>
  );
}
