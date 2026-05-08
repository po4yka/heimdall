import { rawData } from '../state/store';
import type { ContextPressureSummary, ContextPressureRow, ContextPressureBucket } from '../state/dashboard-types';

function bucketLabel(b: ContextPressureBucket): string {
  if (b === 'healthy') return 'HEALTHY';
  if (b === 'warm') return 'WARM';
  if (b === 'tight') return 'TIGHT';
  return 'COMPACTED';
}

function bucketColor(b: ContextPressureBucket): string {
  if (b === 'healthy') return 'var(--success, #4caf50)';
  if (b === 'warm') return 'var(--warning, #ff9800)';
  if (b === 'tight') return 'var(--accent, #D71921)';
  return 'var(--accent, #D71921)';
}

function pct(f: number): string {
  return (f * 100).toFixed(1) + '%';
}

function truncate(s: string, n: number): string {
  return s.length > n ? s.slice(0, n) + '…' : s;
}

function KpiTile({ label, value, color }: { label: string; value: string | number; color?: string }) {
  return (
    <div>
      <div class="stat-label" style={{ fontSize: '10px' }}>{label}</div>
      <div style={{ fontFamily: 'var(--font-mono)', fontSize: '18px', color: color ?? 'var(--text-primary)' }}>
        {value}
      </div>
    </div>
  );
}

function SessionRow({ row }: { row: ContextPressureRow }) {
  const color = bucketColor(row.bucket);
  const label = bucketLabel(row.bucket);
  const project = row.project ? truncate(row.project, 24) : null;

  return (
    <div
      style={{
        display: 'grid',
        gridTemplateColumns: '1fr auto auto auto',
        gap: '8px',
        alignItems: 'center',
        padding: '5px 0',
        borderTop: '1px solid rgba(var(--text-primary-rgb,232,232,232),0.07)',
        fontSize: '11px',
      }}
    >
      <div>
        <span style={{ fontFamily: 'var(--font-mono)', fontSize: '10px', color: 'var(--text-secondary)' }}>
          {row.session_id.slice(0, 8)}
        </span>
        {project && (
          <span style={{ marginLeft: '6px', color: 'var(--text-secondary)', fontSize: '10px' }}>
            {project}
          </span>
        )}
        {row.compaction_count > 0 && (
          <span style={{ marginLeft: '6px', fontFamily: 'var(--font-mono)', fontSize: '9px', color: 'var(--accent,#D71921)' }}>
            [{row.compaction_count}× compacted]
          </span>
        )}
      </div>
      <span style={{ fontFamily: 'var(--font-mono)', fontSize: '10px', color: 'var(--text-secondary)' }}>
        {row.model.replace('claude-', '').replace('gpt-', '')}
      </span>
      <div
        style={{
          height: '4px',
          width: '60px',
          borderRadius: '2px',
          background: 'rgba(var(--text-primary-rgb,232,232,232),0.10)',
          overflow: 'hidden',
        }}
      >
        <div
          style={{
            height: '100%',
            width: `${Math.min(100, row.peak_fraction * 100).toFixed(1)}%`,
            background: color,
            borderRadius: '2px',
          }}
        />
      </div>
      <span style={{ fontFamily: 'var(--font-mono)', fontSize: '10px', color, minWidth: '52px', textAlign: 'right' }}>
        [{label} {pct(row.peak_fraction)}]
      </span>
    </div>
  );
}

function ContextPressureCardInner({ summary }: { summary: ContextPressureSummary }) {
  const tightColor = summary.tight_count > 0 ? 'var(--accent,#D71921)' : undefined;
  const compactedColor = summary.overcompacted_count > 0 ? 'var(--accent,#D71921)' : undefined;
  const top = summary.rows.slice(0, 20);

  return (
    <div class="card" style={{ padding: '16px' }}>
      <div class="stat-label" style={{ marginBottom: '10px' }}>Context pressure</div>

      <div style={{ display: 'flex', gap: '20px', flexWrap: 'wrap', marginBottom: '14px' }}>
        <KpiTile label="Healthy" value={summary.healthy_count} />
        <KpiTile label="Warm" value={summary.warm_count} color="var(--warning,#ff9800)" />
        <KpiTile label="Tight" value={summary.tight_count} {...(tightColor ? { color: tightColor } : {})} />
        <KpiTile label="Compacted" value={summary.overcompacted_count} {...(compactedColor ? { color: compactedColor } : {})} />
        <KpiTile label="Avg peak" value={pct(summary.avg_peak_fraction)} />
      </div>

      {top.length === 0 && (
        <div style={{ fontSize: '11px', color: 'var(--text-secondary)', fontFamily: 'var(--font-mono)' }}>
          No session data yet.
        </div>
      )}

      {top.map((row) => (
        <SessionRow key={row.session_id} row={row} />
      ))}
    </div>
  );
}

export function ContextPressureCard() {
  const data = rawData.value;

  if (!data) {
    return (
      <div class="card" style={{ padding: '16px' }}>
        <div class="stat-label">Context pressure</div>
        <div style={{ color: 'var(--text-secondary)', fontFamily: 'var(--font-mono)', fontSize: '12px', marginTop: '8px' }}>
          loading...
        </div>
      </div>
    );
  }

  const summary = data.context_pressure;
  if (!summary) {
    return (
      <div class="card" style={{ padding: '16px' }}>
        <div class="stat-label">Context pressure</div>
        <div style={{ color: 'var(--text-secondary)', fontFamily: 'var(--font-mono)', fontSize: '12px', marginTop: '8px' }}>
          No data available.
        </div>
      </div>
    );
  }

  return <ContextPressureCardInner summary={summary} />;
}
