import { fmt } from '../lib/format';
import type { HourlyRow } from '../state/types';

export function HourlyChart({ data }: { data: HourlyRow[] }) {
  if (!data.length) return null;

  const maxTurns = Math.max(...data.map(d => d.turns), 1);

  return (
    <div>
      <div class="section-title" style={{ padding: '0', marginBottom: '12px' }}>Activity by Hour of Day</div>
      <div style={{ display: 'flex', alignItems: 'flex-end', gap: '2px', height: '80px' }}>
        {Array.from({ length: 24 }, (_, h) => {
          const row = data.find(d => d.hour === h);
          const turns = row?.turns ?? 0;
          const pct = (turns / maxTurns) * 100;
          return (
            <div
              key={h}
              title={`${h}:00 -- ${fmt(turns)} turns`}
              style={{
                flex: 1,
                height: `${Math.max(pct, 2)}%`,
                background: turns > 0 ? 'var(--accent)' : 'var(--border)',
                borderRadius: '2px 2px 0 0',
                opacity: turns > 0 ? 0.6 + (pct / 100) * 0.4 : 0.3,
              }}
            />
          );
        })}
      </div>
      <div style={{ display: 'flex', gap: '2px', marginTop: '4px' }}>
        {[0, 6, 12, 18, 23].map(h => (
          <span key={h} class="muted" style={{ flex: 1, fontSize: '9px', textAlign: h === 0 ? 'left' : h === 23 ? 'right' : 'center' }}>
            {h}:00
          </span>
        ))}
      </div>
    </div>
  );
}
