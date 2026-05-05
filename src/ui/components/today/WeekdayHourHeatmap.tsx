import { withAlpha } from '../../lib/charts';
import { fmtCost, fmt } from '../../lib/format';
import type { WeekdayHourCell } from '../../state/types';

interface WeekdayHourHeatmapProps {
  cells: WeekdayHourCell[];
}

const DOW_LABELS = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];

function cellOpacity(value: number, max: number): number {
  if (max <= 0 || value <= 0) return 0.05;
  return Math.min(0.05 + 0.85 * (value / max), 0.90);
}

export function WeekdayHourHeatmap({ cells }: WeekdayHourHeatmapProps) {
  const maxCost = Math.max(...cells.map(c => c.cost_nanos), 1);

  const lookup = new Map<string, WeekdayHourCell>();
  for (const c of cells) lookup.set(`${c.dow},${c.hour}`, c);

  const HOUR_LABELS = [0, 6, 12, 18, 23];

  return (
    <div
      class="days-hours-heatmap"
      style={{
        display: 'grid',
        gridTemplateColumns: `40px repeat(24, 1fr)`,
        gap: '1px',
      }}
      role="figure"
      aria-label="7×24 weekday by hour pattern heatmap (90-day window)"
    >
      {/* Header row: empty corner + hour labels */}
      <div />
      {Array.from({ length: 24 }, (_, h) => (
        <div
          key={h}
          style={{
            fontFamily: 'var(--font-mono)',
            fontSize: '8px',
            textAlign: 'center',
            color: 'var(--text-secondary)',
            paddingBottom: '2px',
            visibility: HOUR_LABELS.includes(h) ? 'visible' : 'hidden',
          }}
        >
          {String(h).padStart(2, '0')}
        </div>
      ))}

      {/* Weekday rows (0=Sun … 6=Sat) */}
      {Array.from({ length: 7 }, (_, dow) => [
        <div
          key={`label-${dow}`}
          style={{
            fontFamily: 'var(--font-mono)',
            fontSize: '9px',
            color: 'var(--text-secondary)',
            display: 'flex',
            alignItems: 'center',
          }}
        >
          {DOW_LABELS[dow]}
        </div>,
        ...Array.from({ length: 24 }, (_, hour) => {
          const cell = lookup.get(`${dow},${hour}`);
          const cost = cell?.cost_nanos ?? 0;
          const turns = cell?.turns ?? 0;
          const opacity = cellOpacity(cost, maxCost);
          const bg = withAlpha('--text-primary', opacity);
          const costUsd = cost / 1_000_000_000;
          const title =
            `${DOW_LABELS[dow]} ${String(hour).padStart(2, '0')}:00 — ` +
            `${fmtCost(costUsd)} / ${fmt(turns)} turn${turns !== 1 ? 's' : ''} (90d avg)`;
          return (
            <div
              key={`${dow}-${hour}`}
              class="days-hours-heatmap-cell"
              title={title}
              role="img"
              aria-label={title}
              style={{ background: bg }}
            />
          );
        }),
      ])}
    </div>
  );
}
