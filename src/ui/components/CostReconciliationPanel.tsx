import { fmtCost } from '../lib/format';
import type { CostReconciliationResponse } from '../state/types';

interface Props {
  data: CostReconciliationResponse | null;
}

export function CostReconciliationPanel({ data }: Props) {
  if (!data || !data.enabled) return null;
  if (data.hook_total_nanos == null || data.local_total_nanos == null) return null;

  const hookUsd = data.hook_total_nanos / 1e9;
  const localUsd = data.local_total_nanos / 1e9;
  const divergence = data.divergence_pct ?? 0;
  const divergencePctSigned = divergence * 100;
  const divergencePctAbs = Math.abs(divergencePctSigned);
  const isWarn = divergencePctAbs > 10;

  return (
    <div class="card card-flat" style={{ gridColumn: '1 / -1' }}>
      <div style={{ marginBottom: '16px' }}>
        <span style={{
          fontFamily: 'var(--font-mono)',
          fontSize: '11px',
          fontWeight: 400,
          textTransform: 'uppercase' as const,
          letterSpacing: '0.08em',
          color: 'var(--text-secondary)',
        }}>
          COST RECONCILIATION
        </span>
        {data.period && (
          <span style={{
            fontFamily: 'var(--font-mono)',
            fontSize: '11px',
            color: 'var(--text-disabled)',
            marginLeft: '8px',
          }}>
            ({data.period})
          </span>
        )}
      </div>

      <div style={{ display: 'flex', gap: '32px', flexWrap: 'wrap' as const, marginBottom: '20px' }}>
        <div>
          <div style={{
            fontFamily: 'var(--font-mono)',
            fontSize: '11px',
            textTransform: 'uppercase' as const,
            letterSpacing: '0.08em',
            color: 'var(--text-secondary)',
            marginBottom: '8px',
          }}>HOOK-REPORTED</div>
          <div class="stat-value" style={{ fontSize: '24px' }}>{fmtCost(hookUsd)}</div>
        </div>
        <div>
          <div style={{
            fontFamily: 'var(--font-mono)',
            fontSize: '11px',
            textTransform: 'uppercase' as const,
            letterSpacing: '0.08em',
            color: 'var(--text-secondary)',
            marginBottom: '8px',
          }}>LOCAL ESTIMATE</div>
          <div class="stat-value" style={{ fontSize: '24px' }}>{fmtCost(localUsd)}</div>
        </div>
        <div>
          <div style={{
            fontFamily: 'var(--font-mono)',
            fontSize: '11px',
            textTransform: 'uppercase' as const,
            letterSpacing: '0.08em',
            color: 'var(--text-secondary)',
            marginBottom: '8px',
          }}>DIVERGENCE</div>
          <div
            class="stat-value"
            style={{ fontSize: '24px', color: isWarn ? 'var(--accent)' : undefined }}
          >
            {divergencePctSigned >= 0 ? '+' : ''}{divergencePctSigned.toFixed(1)}%
            {isWarn && (
              <span style={{
                display: 'inline-block',
                marginLeft: '8px',
                fontFamily: 'var(--font-mono)',
                fontSize: '11px',
                fontWeight: 400,
                letterSpacing: '0.06em',
                padding: '2px 8px',
                border: '1px solid var(--accent)',
                borderRadius: '4px',
                color: 'var(--accent)',
                verticalAlign: 'middle',
              }}>
                [DRIFT]
              </span>
            )}
          </div>
        </div>
      </div>

      {data.breakdown && data.breakdown.length > 0 && (
        <div style={{ overflowX: 'auto' as const }}>
          <table>
            <thead>
              <tr>
                <th>DAY</th>
                <th style={{ textAlign: 'right' as const }}>HOOK</th>
                <th style={{ textAlign: 'right' as const }}>LOCAL</th>
                <th style={{ textAlign: 'right' as const }}>&Delta;</th>
              </tr>
            </thead>
            <tbody>
              {data.breakdown.slice().reverse().slice(0, 30).map(r => {
                const h = r.hook_nanos / 1e9;
                const l = r.local_nanos / 1e9;
                const delta = h - l;
                const rowWarn = l > 1e-9 && Math.abs(delta) / l > 0.10;
                return (
                  <tr key={r.day}>
                    <td class="num">{r.day}</td>
                    <td class="num" style={{ textAlign: 'right' as const }}>{fmtCost(h)}</td>
                    <td class="num" style={{ textAlign: 'right' as const }}>{fmtCost(l)}</td>
                    <td
                      class="num"
                      style={{
                        textAlign: 'right' as const,
                        color: rowWarn ? 'var(--accent)' : 'var(--text-secondary)',
                      }}
                    >
                      {delta >= 0 ? '+' : ''}{fmtCost(delta)}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
