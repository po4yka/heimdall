import { rawData } from '../state/store';
import { fmtLabel } from '../lib/format';
import type { HookTelemetrySummary, HookLatencyBucket, HookOutcomeRow, HookBypassAncestorRow } from '../state/dashboard-types';

function fmtMs(us: number): string {
  return `${Math.round(us / 1000)}ms`;
}

function KpiTile({ label, value }: { label: string; value: string | number }) {
  return (
    <div>
      <div class="stat-label" style={{ fontSize: '10px' }}>{label}</div>
      <div style={{ fontFamily: 'var(--font-mono)', fontSize: '18px' }}>{value}</div>
    </div>
  );
}

function LatencyHistogram({ buckets }: { buckets: HookLatencyBucket[] }) {
  if (buckets.length === 0) return null;
  const maxCount = Math.max(...buckets.map(b => b.count), 1);

  return (
    <div style={{ marginBottom: '16px' }}>
      <div class="stat-label" style={{ marginBottom: '8px', fontSize: '10px' }}>Latency distribution</div>
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
          const heightPct = (b.count / maxCount * 100).toFixed(1);
          const opacity = b.count > 0 ? 0.9 : 0.15;
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
                {b.count}
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
                  whiteSpace: 'nowrap',
                  overflow: 'hidden',
                  maxWidth: '100%',
                  textOverflow: 'ellipsis',
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

function OutcomeTable({ rows }: { rows: HookOutcomeRow[] }) {
  if (rows.length === 0) return null;

  return (
    <div style={{ marginBottom: '16px' }}>
      <div class="stat-label" style={{ marginBottom: '8px', fontSize: '10px' }}>Outcome breakdown</div>
      <div
        style={{
          padding: '12px',
          border: '1px solid var(--border)',
          borderRadius: '8px',
        }}
      >
        <div
          style={{
            display: 'grid',
            gridTemplateColumns: '1fr auto auto auto',
            gap: '3px 16px',
            alignItems: 'center',
          }}
        >
          <div style={{ fontFamily: 'var(--font-mono)', fontSize: '9px', color: 'var(--text-secondary)' }}>Outcome</div>
          <div style={{ fontFamily: 'var(--font-mono)', fontSize: '9px', color: 'var(--text-secondary)', textAlign: 'right' }}>Count</div>
          <div style={{ fontFamily: 'var(--font-mono)', fontSize: '9px', color: 'var(--text-secondary)', textAlign: 'right' }}>p50</div>
          <div style={{ fontFamily: 'var(--font-mono)', fontSize: '9px', color: 'var(--text-secondary)', textAlign: 'right' }}>p95</div>
          {rows.map((r) => (
            <>
              <div key={`outcome-${r.outcome}`} style={{ fontFamily: 'var(--font-mono)', fontSize: '11px', color: 'var(--text-primary)', lineHeight: '20px' }}>{fmtLabel(r.outcome)}</div>
              <div style={{ fontFamily: 'var(--font-mono)', fontSize: '11px', textAlign: 'right', fontFeatureSettings: '"tnum"', color: 'var(--text-primary)' }}>{r.count}</div>
              <div style={{ fontFamily: 'var(--font-mono)', fontSize: '11px', textAlign: 'right', fontFeatureSettings: '"tnum"', color: 'var(--text-secondary)' }}>{fmtMs(r.p50_us)}</div>
              <div style={{ fontFamily: 'var(--font-mono)', fontSize: '11px', textAlign: 'right', fontFeatureSettings: '"tnum"', color: 'var(--text-secondary)' }}>{fmtMs(r.p95_us)}</div>
            </>
          ))}
        </div>
      </div>
    </div>
  );
}

function BypassTable({ rows }: { rows: HookBypassAncestorRow[] }) {
  if (rows.length === 0) return null;

  return (
    <div>
      <div class="stat-label" style={{ marginBottom: '8px', fontSize: '10px' }}>Top bypass ancestors</div>
      <div
        style={{
          padding: '12px',
          border: '1px solid var(--border)',
          borderRadius: '8px',
        }}
      >
        <div
          style={{
            display: 'grid',
            gridTemplateColumns: '1fr auto',
            gap: '3px 16px',
            alignItems: 'center',
          }}
        >
          <div style={{ fontFamily: 'var(--font-mono)', fontSize: '9px', color: 'var(--text-secondary)' }}>Command</div>
          <div style={{ fontFamily: 'var(--font-mono)', fontSize: '9px', color: 'var(--text-secondary)', textAlign: 'right' }}>Bypasses</div>
          {rows.map((a) => (
            <>
              <div
                key={`bypass-${a.command}`}
                style={{
                  fontFamily: 'var(--font-mono)',
                  fontSize: '10px',
                  color: 'var(--text-primary)',
                  overflow: 'hidden',
                  textOverflow: 'ellipsis',
                  whiteSpace: 'nowrap',
                  lineHeight: '20px',
                }}
                title={a.command}
              >
                {a.command.length > 40 ? a.command.slice(0, 40) + '…' : a.command}
              </div>
              <div style={{ fontFamily: 'var(--font-mono)', fontSize: '11px', textAlign: 'right', fontFeatureSettings: '"tnum"', color: 'var(--text-primary)' }}>{a.bypass_count}</div>
            </>
          ))}
        </div>
      </div>
    </div>
  );
}

function HookTelemetryCardInner({ summary }: { summary: HookTelemetrySummary }) {
  return (
    <div class="card" style={{ padding: '16px' }}>
      <div class="stat-label" style={{ marginBottom: '10px' }}>Hook telemetry</div>

      <div style={{ display: 'flex', gap: '20px', flexWrap: 'wrap', marginBottom: '16px' }}>
        <KpiTile label="Invocations (30d)" value={summary.total_invocations} />
        <KpiTile label="p50 latency" value={fmtMs(summary.p50_latency_us)} />
        <KpiTile label="p95 latency" value={fmtMs(summary.p95_latency_us)} />
        <KpiTile label="p99 latency" value={fmtMs(summary.p99_latency_us)} />
        <KpiTile label="Bypasses" value={summary.bypass_count} />
        <KpiTile label="Timeouts" value={summary.stdin_timeout_count} />
        <KpiTile label="Parse errors" value={summary.parse_error_count} />
      </div>

      <LatencyHistogram buckets={summary.latency_buckets} />
      <OutcomeTable rows={summary.outcome_rows} />
      <BypassTable rows={summary.top_bypass_ancestors} />
    </div>
  );
}

export function HookTelemetryCard() {
  const data = rawData.value;

  if (!data) {
    return (
      <div class="card" style={{ padding: '16px' }}>
        <div class="stat-label">Hook telemetry</div>
        <div style={{ color: 'var(--text-secondary)', fontFamily: 'var(--font-mono)', fontSize: '12px', marginTop: '8px' }}>
          loading...
        </div>
      </div>
    );
  }

  const summary = data.hook_telemetry;
  if (!summary || summary.total_invocations === 0) {
    return (
      <div class="card" style={{ padding: '16px' }}>
        <div class="stat-label">Hook telemetry</div>
        <div style={{ color: 'var(--text-secondary)', fontFamily: 'var(--font-mono)', fontSize: '12px', marginTop: '8px' }}>
          No hook invocations recorded yet. Install the hook with <code>heimdall hook install</code>.
        </div>
      </div>
    );
  }

  return <HookTelemetryCardInner summary={summary} />;
}
