// TODO Phase-11: spec ambiguity — heatmap currently has no per-project breakdown.
// The heatmap aggregates by hour/day across all projects; tooltip enhancement
// is deferred until the per-project heatmap requirement is clarified.
import { withAlpha } from '../lib/charts';
import { fmtCost } from '../lib/format';
import type { HeatmapData } from '../state/types';

const DOW_LABELS = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];

function cellOpacity(costNanos: number, maxCostNanos: number): number {
  if (maxCostNanos <= 0 || costNanos <= 0) return 0.05;
  const ratio = costNanos / maxCostNanos;
  // Scale: 0.05 base + up to 0.85 additional. Clamp to 0.90.
  return Math.min(0.05 + 0.85 * ratio, 0.90);
}

export function ActivityHeatmap({ data }: { data: HeatmapData }) {
  const { cells, max_cost_nanos, active_days, total_cost_nanos, period } = data;

  // Build lookup: (dow, hour) -> cell
  const lookup = new Map<string, { cost_nanos: number; call_count: number }>();
  for (const c of cells) {
    lookup.set(`${c.dow},${c.hour}`, c);
  }

  const avgPerDay =
    active_days > 0
      ? fmtCost(total_cost_nanos / 1_000_000_000 / active_days)
      : '--';

  return (
    <div>
      {/* Caption row */}
      <div
        style={{
          display: 'flex',
          alignItems: 'baseline',
          gap: '12px',
          marginBottom: '8px',
          flexWrap: 'wrap',
        }}
      >
        <span
          class="section-title"
          style={{
            padding: 0,
            fontFamily: 'var(--font-mono)',
            letterSpacing: '0.08em',
            textTransform: 'uppercase',
          }}
        >
          ACTIVITY / 7x24 / {period.toUpperCase()}
        </span>
        <span
          style={{
            fontFamily: 'var(--font-mono)',
            fontSize: '11px',
            color: 'var(--text-secondary)',
            letterSpacing: '0.04em',
          }}
        >
          {active_days} active {active_days === 1 ? 'day' : 'days'} &middot; {avgPerDay} per active day
        </span>
      </div>

      {/* Grid: 8 columns (label + 24 hours) × 8 rows (header + 7 days) */}
      <div
        style={{
          display: 'grid',
          gridTemplateColumns: '28px repeat(24, 1fr)',
          gap: '2px',
        }}
      >
        {/* Header row: empty corner + hour labels */}
        <div />
        {Array.from({ length: 24 }, (_, h) => (
          <div
            key={h}
            style={{
              fontFamily: 'var(--font-mono)',
              fontSize: '9px',
              color: 'var(--text-secondary)',
              textAlign: 'center',
              letterSpacing: '0.04em',
              // Show only 0, 6, 12, 18 to avoid crowding
              visibility: [0, 6, 12, 18].includes(h) ? 'visible' : 'hidden',
            }}
          >
            {String(h).padStart(2, '0')}
          </div>
        ))}

        {/* Data rows: 7 days */}
        {Array.from({ length: 7 }, (_, dow) => (
          <>
            {/* Day label */}
            <div
              key={`label-${dow}`}
              style={{
                fontFamily: 'var(--font-mono)',
                fontSize: '9px',
                color: 'var(--text-secondary)',
                display: 'flex',
                alignItems: 'center',
                letterSpacing: '0.04em',
              }}
            >
              {DOW_LABELS[dow]}
            </div>

            {/* 24 hour cells for this day */}
            {Array.from({ length: 24 }, (_, hour) => {
              const cell = lookup.get(`${dow},${hour}`);
              const costNanos = cell?.cost_nanos ?? 0;
              const callCount = cell?.call_count ?? 0;
              const opacity = cellOpacity(costNanos, max_cost_nanos);
              const bg = withAlpha('--text-display', opacity);
              const costUsd = costNanos / 1_000_000_000;
              const title = `${DOW_LABELS[dow]} ${String(hour).padStart(2, '0')}:00 — ${fmtCost(costUsd)} / ${callCount} call${callCount !== 1 ? 's' : ''}`;

              return (
                <div
                  key={`${dow}-${hour}`}
                  title={title}
                  style={{
                    background: bg,
                    borderRadius: '2px',
                    aspectRatio: '1',
                    minHeight: '10px',
                  }}
                />
              );
            })}
          </>
        ))}
      </div>
    </div>
  );
}
