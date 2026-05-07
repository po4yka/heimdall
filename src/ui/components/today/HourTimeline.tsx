import { withAlpha, cssVar } from '../../lib/charts';
import { fmtCost, fmt } from '../../lib/format';
import type { TodayHourRow } from '../../state/types';

interface HourTimelineProps {
  hours: TodayHourRow[];
}

export function HourTimeline({ hours }: HourTimelineProps) {
  const maxCost = Math.max(...hours.map(h => h.cost_nanos), 1);
  const totalCost = hours.reduce((s, h) => s + h.cost_nanos, 0);

  if (totalCost === 0) {
    return (
      <div class="today-empty-state" style={{ flex: 1 }}>
        <span>No activity for this day</span>
      </div>
    );
  }

  return (
    <div style={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
      <div
        style={{
          display: 'flex',
          alignItems: 'flex-end',
          gap: '2px',
          flex: 1,
          minHeight: '60px',
        }}
      >
        {hours.map(h => {
          const pct = (h.cost_nanos / maxCost) * 100;
          const background =
            h.cost_nanos > 0
              ? withAlpha('--text-display', 0.35 + (pct / 100) * 0.55)
              : cssVar('--border');
          const costUsd = h.cost_nanos / 1_000_000_000;
          const totalTokens = h.input_tokens + h.output_tokens + h.cache_read_tokens + h.cache_creation_tokens;
          const title =
            `${String(h.hour).padStart(2, '0')}:00 — ${fmtCost(costUsd)} / ` +
            `${fmt(h.turns)} turn${h.turns !== 1 ? 's' : ''} / ${fmt(totalTokens)} tokens`;
          return (
            <div
              key={h.hour}
              title={title}
              style={{
                flex: 1,
                height: `${Math.max(pct, 2)}%`,
                background,
                borderRadius: 0,
              }}
            />
          );
        })}
      </div>
      <div style={{ display: 'flex', gap: '2px', marginTop: '6px' }}>
        {hours.map(h => (
          <span
            key={h.hour}
            style={{
              flex: 1,
              fontFamily: 'var(--font-mono)',
              fontSize: '9px',
              textAlign: 'center',
              letterSpacing: '0.04em',
              color: cssVar('--text-secondary'),
              visibility: [0, 6, 12, 18].includes(h.hour) ? 'visible' : 'hidden',
            }}
          >
            {String(h.hour).padStart(2, '0')}
          </span>
        ))}
      </div>
    </div>
  );
}
