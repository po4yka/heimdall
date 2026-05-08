import { fmt } from '../../lib/format';
import { withAlpha, cssVar } from '../../lib/charts';
import type { HourlyRow } from '../../state/types';

export function HourlyChart({ data }: { data: HourlyRow[] }) {
  if (!data.length) return null;

  const maxTurns = Math.max(...data.map(d => d.turns), 1);

  return (
    <div style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      <div class="section-title" style={{ padding: '0', marginBottom: '12px' }}>
        Activity by hour of day
      </div>
      <div style={{ display: 'flex', alignItems: 'flex-end', gap: '2px', flex: 1, minHeight: '60px' }}>
        {Array.from({ length: 24 }, (_, h) => {
          const row = data.find(d => d.hour === h);
          const turns = row?.turns ?? 0;
          const pct = (turns / maxTurns) * 100;
          // Opacity ladder: non-zero bars scale 40% -> 100% with magnitude.
          const background = turns > 0
            ? withAlpha('--text-display', 0.4 + (pct / 100) * 0.6)
            : cssVar('--border');
          return (
            <div
              key={h}
              title={`${h}:00 -- ${fmt(turns)} turns`}
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
        {Array.from({ length: 24 }, (_, h) => (
          <span
            key={h}

            style={{
              flex: 1,
              fontFamily: 'var(--font-mono)',
              fontSize: '9px',
              textAlign: 'center',
              letterSpacing: '0.04em',
              color: cssVar('--text-secondary'),
              visibility: [0, 6, 12, 18].includes(h) ? 'visible' : 'hidden',
            }}
          >
            {String(h).padStart(2, '0')}
          </span>
        ))}
      </div>
    </div>
  );
}
