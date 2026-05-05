import { withAlpha } from '../../lib/charts';
import { fmtCost, fmt } from '../../lib/format';
import type { DayHourCell } from '../../state/types';

interface DaysHoursHeatmapProps {
  cells: DayHourCell[];
  daysCount: 7 | 30;
  title: string;
  onDayClick?: (day: string) => void;
}

function cellOpacity(value: number, max: number): number {
  if (max <= 0 || value <= 0) return 0.05;
  return Math.min(0.05 + 0.85 * (value / max), 0.90);
}

export function DaysHoursHeatmap({ cells, daysCount, title, onDayClick }: DaysHoursHeatmapProps) {
  const maxCost = Math.max(...cells.map(c => c.cost_nanos), 1);

  // Build sorted unique days (ascending) and a lookup map.
  const daysSet = new Set<string>();
  for (const c of cells) daysSet.add(c.day);
  // Sort descending (newest first = top row).
  const days = Array.from(daysSet).sort((a, b) => b.localeCompare(a));

  const lookup = new Map<string, DayHourCell>();
  for (const c of cells) lookup.set(`${c.day},${c.hour}`, c);

  const HOUR_LABELS = [0, 6, 12, 18, 23];

  return (
    <div class="days-hours-heatmap-wrap">
      <div class="days-hours-heatmap-title">{title}</div>
      <div
        class="days-hours-heatmap"
        style={{
          display: 'grid',
          gridTemplateColumns: `60px repeat(24, 1fr)`,
          gap: '1px',
        }}
        role="figure"
        aria-label={`${daysCount} days by 24 hours heatmap`}
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

        {/* Day rows */}
        {days.map(day => [
          // Day label
          <div
            key={`label-${day}`}
            style={{
              fontFamily: 'var(--font-mono)',
              fontSize: '9px',
              color: 'var(--text-secondary)',
              display: 'flex',
              alignItems: 'center',
              paddingRight: '4px',
              whiteSpace: 'nowrap',
            }}
          >
            {day.slice(5)} {/* MM-DD */}
          </div>,
          // Hour cells
          ...Array.from({ length: 24 }, (_, hour) => {
            const cell = lookup.get(`${day},${hour}`);
            const cost = cell?.cost_nanos ?? 0;
            const turns = cell?.turns ?? 0;
            const opacity = cellOpacity(cost, maxCost);
            const bg = withAlpha('--text-primary', opacity);
            const costUsd = cost / 1_000_000_000;
            const title_ =
              `${day} ${String(hour).padStart(2, '0')}:00 — ` +
              `${fmtCost(costUsd)} / ${fmt(turns)} turn${turns !== 1 ? 's' : ''}`;
            const clickable = onDayClick && cost > 0;
            return (
              <div
                key={`${day}-${hour}`}
                class={`days-hours-heatmap-cell${clickable ? ' days-hours-heatmap-cell--clickable' : ''}`}
                title={title_}
                role="img"
                aria-label={title_}
                style={{ background: bg }}
                onClick={clickable ? () => onDayClick!(day) : undefined}
              />
            );
          }),
        ])}
      </div>
    </div>
  );
}
