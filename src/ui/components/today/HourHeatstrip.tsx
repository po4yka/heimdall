import { withAlpha } from '../../lib/charts';
import { fmtCost, fmt } from '../../lib/format';
import type { TodayHourRow } from '../../state/types';

interface HourHeatstripProps {
  hours: TodayHourRow[];
}

function cellOpacity(value: number, max: number): number {
  if (max <= 0 || value <= 0) return 0.05;
  return Math.min(0.05 + 0.85 * (value / max), 0.90);
}

export function HourHeatstrip({ hours }: HourHeatstripProps) {
  const maxCost = Math.max(...hours.map(h => h.cost_nanos), 1);

  return (
    <div class="hour-heatstrip" role="figure" aria-label="Hour-by-hour cost heatstrip">
      {hours.map(h => {
        const opacity = cellOpacity(h.cost_nanos, maxCost);
        const bg = withAlpha('--text-primary', opacity);
        const costUsd = h.cost_nanos / 1_000_000_000;
        const title =
          `${String(h.hour).padStart(2, '0')}:00 — ${fmtCost(costUsd)} / ` +
          `${fmt(h.turns)} turn${h.turns !== 1 ? 's' : ''}`;
        return (
          <div
            key={h.hour}
            class="hour-heatstrip-cell"
            title={title}
            role="img"
            aria-label={title}
            style={{ background: bg }}
          />
        );
      })}
    </div>
  );
}
