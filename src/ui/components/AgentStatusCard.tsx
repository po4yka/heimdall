import type { AgentStatusSnapshot, ProviderStatus, StatusIndicator } from '../state/types';
import { agent_status_expanded } from '../state/store';

interface AgentStatusCardProps {
  snapshot: AgentStatusSnapshot;
}

/** Monochrome dot at three opacity levels; red only for Major/Critical. */
function IndicatorDot({ indicator }: { indicator: StatusIndicator }) {
  const isAlert = indicator === 'major' || indicator === 'critical';
  const isMinor = indicator === 'minor';

  const color = isAlert
    ? 'var(--accent)'
    : 'var(--text-secondary)';

  const opacity = isAlert ? 1.0 : isMinor ? 0.6 : 0.3;

  return (
    <span
      aria-label={`Status: ${indicator}`}
      style={{
        display: 'inline-block',
        width: '10px',
        height: '10px',
        borderRadius: '50%',
        backgroundColor: color,
        opacity,
        marginRight: '8px',
        flexShrink: 0,
      }}
    />
  );
}

interface ProviderRowProps {
  name: string;
  status: ProviderStatus | null | undefined;
  expanded: boolean;
}

function ProviderRow({ name, status, expanded }: ProviderRowProps) {
  if (!status) {
    return (
      <div style={{ display: 'flex', alignItems: 'center', padding: '8px 0', gap: '8px' }}>
        <IndicatorDot indicator="none" />
        <span style={{ fontFamily: 'var(--font-mono)', fontSize: '13px', flex: 1 }}>{name}</span>
        <span style={{ color: 'var(--text-secondary)', fontSize: '12px' }}>unavailable</span>
      </div>
    );
  }

  const incidentCount = status.active_incidents.length;

  return (
    <div>
      <div style={{ display: 'flex', alignItems: 'center', padding: '8px 0', gap: '8px' }}>
        <IndicatorDot indicator={status.indicator} />
        <span style={{ fontFamily: 'var(--font-mono)', fontSize: '13px', flex: 1 }}>{name}</span>
        <span style={{ color: 'var(--text-secondary)', fontSize: '12px' }}>{status.description}</span>
        {incidentCount > 0 && (
          <span
            style={{
              fontFamily: 'var(--font-mono)',
              fontSize: '11px',
              color: status.indicator === 'major' || status.indicator === 'critical'
                ? 'var(--accent)'
                : 'var(--text-secondary)',
              marginLeft: '8px',
            }}
          >
            ({incidentCount} active)
          </span>
        )}
        <a
          href={status.page_url}
          target="_blank"
          rel="noopener noreferrer"
          style={{ color: 'var(--text-secondary)', fontSize: '11px', marginLeft: '4px' }}
          aria-label={`${name} status page`}
        >
          ↗
        </a>
      </div>

      {expanded && (
        <div style={{ paddingLeft: '18px', paddingBottom: '8px' }}>
          {status.components.length > 0 && (
            <table style={{ width: '100%', fontSize: '12px', borderCollapse: 'collapse', marginBottom: '8px' }}>
              <thead>
                <tr style={{ color: 'var(--text-secondary)' }}>
                  <th style={{ textAlign: 'left', padding: '2px 8px 2px 0', fontWeight: 500 }}>Component</th>
                  <th style={{ textAlign: 'left', padding: '2px 0', fontWeight: 500 }}>Status</th>
                </tr>
              </thead>
              <tbody>
                {status.components.map((c, i) => {
                  const fmt = (v: number | null | undefined) =>
                    v != null ? `${(v * 100).toFixed(2)}%` : '--';
                  const has30 = c.uptime_30d != null;
                  const has7 = c.uptime_7d != null;
                  const showUptime = has30 || has7;
                  return (
                    <>
                      <tr key={i}>
                        <td style={{ padding: '2px 8px 2px 0', fontFamily: 'var(--font-mono)' }}>{c.name}</td>
                        <td style={{ padding: '2px 0', color: 'var(--text-secondary)' }}>{c.status.replace(/_/g, ' ')}</td>
                      </tr>
                      {showUptime && (
                        <tr key={`${i}-uptime`}>
                          <td colSpan={2} style={{ padding: '0 0 4px 0' }}>
                            <span style={{
                              fontFamily: 'var(--font-mono)',
                              fontSize: '11px',
                              letterSpacing: '0.04em',
                            }}>
                              <span style={{ color: 'var(--text-secondary)' }}>30D </span>
                              <span style={{ color: 'var(--text-primary)' }}>{fmt(c.uptime_30d)}</span>
                              <span style={{ color: 'var(--text-secondary)' }}> · 7D </span>
                              <span style={{ color: 'var(--text-primary)' }}>{fmt(c.uptime_7d)}</span>
                            </span>
                          </td>
                        </tr>
                      )}
                    </>
                  );
                })}
              </tbody>
            </table>
          )}

          {status.active_incidents.map((inc, i) => (
            <div
              key={i}
              style={{
                fontSize: '12px',
                color: 'var(--text-secondary)',
                marginBottom: '4px',
                paddingLeft: '4px',
                borderLeft: '2px solid var(--border)',
              }}
            >
              <span style={{ fontFamily: 'var(--font-mono)' }}>
                {inc.shortlink ? (
                  <a
                    href={inc.shortlink}
                    target="_blank"
                    rel="noopener noreferrer"
                    style={{ color: 'inherit', textDecoration: 'underline' }}
                  >
                    {inc.name}
                  </a>
                ) : (
                  inc.name
                )}
              </span>
              {' '}
              <span style={{ opacity: 0.7 }}>
                [{inc.impact}] {inc.status} — {inc.started_at.slice(0, 16).replace('T', ' ')}
              </span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

export function AgentStatusCard({ snapshot }: AgentStatusCardProps) {
  const expanded = agent_status_expanded.value;

  const hasData = snapshot.claude != null || snapshot.openai != null;

  return (
    <div class="card stat-card" style={{ minWidth: '300px' }}>
      <div class="stat-content">
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            marginBottom: '8px',
          }}
        >
          <div class="stat-label">Agent Status</div>
          {hasData && (
            <button
              onClick={() => { agent_status_expanded.value = !expanded; }}
              style={{
                background: 'none',
                border: 'none',
                cursor: 'pointer',
                color: 'var(--text-secondary)',
                fontSize: '11px',
                fontFamily: 'var(--font-mono)',
                padding: '2px 4px',
              }}
              aria-expanded={expanded}
              aria-label="Toggle agent status details"
            >
              {expanded ? '▲ collapse' : '▼ expand'}
            </button>
          )}
        </div>

        <ProviderRow name="Claude" status={snapshot.claude} expanded={expanded} />
        <ProviderRow name="OpenAI / Codex" status={snapshot.openai} expanded={expanded} />

        {snapshot.fetched_at && (
          <div style={{ fontSize: '10px', color: 'var(--text-secondary)', marginTop: '8px', fontFamily: 'var(--font-mono)' }}>
            Refreshed {snapshot.fetched_at.slice(0, 19).replace('T', ' ')} UTC
          </div>
        )}
      </div>
    </div>
  );
}
