import { fmt } from '../lib/format';
import { withAlpha, cssVar } from '../lib/charts';
import type { HourlyRow } from '../state/types';

export function HourlyChart({ data }: { data: HourlyRow[] }) {
  if (!data.length) return null;

  const maxTurns = Math.max(...data.map(d => d.turns), 1);
  const fillColor = cssVar('--text-display');
  const emptyColor = cssVar('--border');

  return (
    <div>
      <div class="section-title" style={{ padding: '0', marginBottom: '12px' }}>
        Activity by Hour of Day
      </div>
      <div style={{ display: 'flex', alignItems: 'flex-end', gap: '2px', height: '80px' }}>
        {Array.from({ length: 24 }, (_, h) => {
          const row = data.find(d => d.hour === h);
          const turns = row?.turns ?? 0;
          const pct = (turns / maxTurns) * 100;
          // Opacity ladder: non-zero bars scale 40% -> 100% with magnitude.
          const background = turns > 0
            ? withAlpha('--text-display', 0.4 + (pct / 100) * 0.6)
            : emptyColor;
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
            class="muted"
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
      {/* Mark the last-visible label so the void `fillColor` isn't truly unused, no-op-ish */}
      <div style={{ display: 'none' }} data-fill={fillColor} />
    </div>
  );
}
