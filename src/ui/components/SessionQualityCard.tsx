import { rawData } from '../state/store';
import type { SessionQualitySummary, SessionDepthBucket, SessionCategoryQualityRow } from '../state/dashboard-types';

const BUCKET_LABELS = ['1', '2', '3-5', '6-10', '11-20', '21+'];

function pct(f: number): string {
  return (f * 100).toFixed(0) + '%';
}

function KpiTile({ label, value }: { label: string; value: string | number }) {
  return (
    <div>
      <div class="stat-label" style={{ fontSize: '10px' }}>{label}</div>
      <div style={{ fontFamily: 'var(--font-mono)', fontSize: '18px' }}>{value}</div>
    </div>
  );
}

function DepthHistogram({ buckets }: { buckets: SessionDepthBucket[] }) {
  if (buckets.length === 0) return null;
  const maxCount = Math.max(...buckets.map(b => b.session_count), 1);

  return (
    <div style={{ marginBottom: '16px' }}>
      <div class="stat-label" style={{ marginBottom: '8px', fontSize: '10px' }}>Turn depth distribution</div>
      <div
        style={{
          display: 'flex',
          gap: '6px',
          alignItems: 'flex-end',
          padding: '12px',
          border: '1px solid var(--border)',
          borderRadius: '8px',
          height: '120px',
        }}
      >
        {buckets.map((b) => {
          const heightPct = (b.session_count / maxCount * 100).toFixed(1);
          const opacity = b.session_count > 0 ? 0.9 : 0.15;
          return (
            <div
              key={b.label}
              style={{
                display: 'flex',
                flexDirection: 'column',
                alignItems: 'center',
                gap: '4px',
                flex: '1',
              }}
            >
              <span
                style={{
                  fontFamily: 'var(--font-mono)',
                  fontSize: '9px',
                  color: 'var(--text-secondary)',
                }}
              >
                {b.session_count}
              </span>
              <div style={{ width: '100%', flex: '1', display: 'flex', alignItems: 'flex-end' }}>
                <div
                  style={{
                    width: '100%',
                    height: `${heightPct}%`,
                    background: `rgba(var(--text-primary-rgb,232,232,232),${opacity})`,
                    borderRadius: '2px 2px 0 0',
                    minHeight: '2px',
                  }}
                />
              </div>
              <span
                style={{
                  fontFamily: 'var(--font-mono)',
                  fontSize: '9px',
                  color: 'var(--text-secondary)',
                }}
              >
                {b.label}
              </span>
            </div>
          );
        })}
      </div>
    </div>
  );
}

function CategoryHeatmap({ rows }: { rows: SessionCategoryQualityRow[] }) {
  if (rows.length === 0) return null;
  const topRows = rows.slice(0, 8);

  return (
    <div>
      <div class="stat-label" style={{ marginBottom: '8px', fontSize: '10px' }}>Category × depth</div>
      <div
        style={{
          padding: '12px',
          border: '1px solid var(--border)',
          borderRadius: '8px',
          overflowX: 'auto',
        }}
      >
        <div
          style={{
            display: 'grid',
            gridTemplateColumns: '120px repeat(6, 1fr)',
            gap: '3px',
            alignItems: 'center',
            minWidth: '360px',
          }}
        >
          {/* Header row */}
          <div />
          {BUCKET_LABELS.map((l) => (
            <div
              key={l}
              style={{
                fontFamily: 'var(--font-mono)',
                fontSize: '10px',
                color: 'var(--text-secondary)',
                textAlign: 'center',
                opacity: 0.6,
              }}
            >
              {l}
            </div>
          ))}

          {/* Data rows */}
          {topRows.map((row) => {
            const rowTotal = Math.max(row.session_count, 1);
            return (
              <>
                <div
                  key={`label-${row.category}`}
                  style={{
                    fontFamily: 'var(--font-mono)',
                    fontSize: '10px',
                    color: 'var(--text-secondary)',
                    textAlign: 'right',
                    paddingRight: '8px',
                    lineHeight: '20px',
                    whiteSpace: 'nowrap',
                    overflow: 'hidden',
                    textOverflow: 'ellipsis',
                  }}
                >
                  {row.category}
                </div>
                {row.bucket_counts.map((count, i) => {
                  const intensity = count / rowTotal;
                  const opacity = count > 0 ? Math.max(0.08, intensity * 0.9) : 0.04;
                  return (
                    <div
                      key={`cell-${row.category}-${i}`}
                      style={{
                        background: `rgba(var(--text-primary-rgb,232,232,232),${opacity.toFixed(2)})`,
                        borderRadius: '2px',
                        height: '20px',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                      }}
                    >
                      {count > 0 && (
                        <span
                          style={{
                            fontFamily: 'var(--font-mono)',
                            fontSize: '9px',
                            opacity: 0.8,
                          }}
                        >
                          {count}
                        </span>
                      )}
                    </div>
                  );
                })}
              </>
            );
          })}
        </div>
      </div>
    </div>
  );
}

function SessionQualityCardInner({ summary }: { summary: SessionQualitySummary }) {
  return (
    <div class="card" style={{ padding: '16px' }}>
      <div class="stat-label" style={{ marginBottom: '10px' }}>Session quality distribution</div>

      <div style={{ display: 'flex', gap: '20px', flexWrap: 'wrap', marginBottom: '16px' }}>
        <KpiTile label="Sessions (30d)" value={summary.total_sessions} />
        <KpiTile label="Abandonment rate" value={pct(summary.abandonment_rate)} />
        <KpiTile label="Long-pause sessions" value={summary.long_pause_session_count} />
        <KpiTile label="Avg turns / session" value={summary.avg_turns_per_session.toFixed(1)} />
      </div>

      <DepthHistogram buckets={summary.depth_buckets} />
      <CategoryHeatmap rows={summary.category_rows} />
    </div>
  );
}

export function SessionQualityCard() {
  const data = rawData.value;

  if (!data) {
    return (
      <div class="card" style={{ padding: '16px' }}>
        <div class="stat-label">Session quality distribution</div>
        <div style={{ color: 'var(--text-secondary)', fontFamily: 'var(--font-mono)', fontSize: '12px', marginTop: '8px' }}>
          loading...
        </div>
      </div>
    );
  }

  const summary = data.session_quality;
  if (!summary || summary.total_sessions === 0) {
    return (
      <div class="card" style={{ padding: '16px' }}>
        <div class="stat-label">Session quality distribution</div>
        <div style={{ color: 'var(--text-secondary)', fontFamily: 'var(--font-mono)', fontSize: '12px', marginTop: '8px' }}>
          No data available.
        </div>
      </div>
    );
  }

  return <SessionQualityCardInner summary={summary} />;
}
