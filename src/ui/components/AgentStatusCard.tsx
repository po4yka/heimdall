import type { AgentStatusSnapshot, ProviderStatus, StatusIndicator, CommunitySignal, ServiceSignal, SignalLevel } from '../state/types';
import { agent_status_expanded, syncDashboardUrl } from '../state/store';

const fmtUtc = (ts: string) => ts.slice(0, 19).replace('T', ' ');

interface AgentStatusCardProps {
  snapshot: AgentStatusSnapshot;
  communitySignal?: CommunitySignal | null;
}

function IndicatorDot({ indicator }: { indicator: StatusIndicator }) {
  const isAlert = indicator === 'major' || indicator === 'critical';
  const isMinor = indicator === 'minor';

  return (
    <span
      aria-hidden="true"
      style={{
        display: 'inline-block',
        width: '8px',
        height: '8px',
        borderRadius: '50%',
        flexShrink: 0,
        backgroundColor: isAlert ? 'var(--accent)' : 'var(--text-secondary)',
        opacity: isAlert ? 1 : isMinor ? 0.5 : 0.25,
      }}
    />
  );
}

interface ProviderRowProps {
  name: string;
  status: ProviderStatus | null | undefined;
  expanded: boolean;
  isLast: boolean;
}

function ProviderRow({ name, status, expanded, isLast }: ProviderRowProps) {
  const indicator: StatusIndicator = status?.indicator ?? 'none';
  const isAlert = indicator === 'major' || indicator === 'critical';

  return (
    <div
      style={{
        borderBottom: isLast ? 'none' : '1px solid var(--border)',
        paddingBottom: isLast ? 0 : '10px',
        marginBottom: isLast ? 0 : '10px',
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
        <IndicatorDot indicator={indicator} />
        <span
          style={{
            fontFamily: 'var(--font-mono)',
            fontSize: '13px',
            fontWeight: 500,
            flex: 1,
            color: 'var(--text-primary)',
          }}
        >
          {name}
        </span>

        {!status ? (
          <span
            style={{
              fontFamily: 'var(--font-mono)',
              fontSize: '11px',
              color: 'var(--text-secondary)',
              opacity: 0.6,
            }}
            title="Status API unreachable"
          >
            unavailable
          </span>
        ) : (
          <>
            <span
              style={{
                fontFamily: 'var(--font-mono)',
                fontSize: '11px',
                color: isAlert ? 'var(--accent)' : 'var(--text-secondary)',
              }}
            >
              {status.description}
              {status.active_incidents.length > 0 && (
                <span style={{ opacity: 0.7 }}>
                  {' '}({status.active_incidents.length})
                </span>
              )}
            </span>
            <a
              href={status.page_url}
              target="_blank"
              rel="noopener noreferrer"
              style={{
                color: 'var(--text-secondary)',
                fontSize: '11px',
                opacity: 0.5,
                display: 'inline-flex',
                alignItems: 'center',
                lineHeight: 1,
                flexShrink: 0,
                textDecoration: 'none',
              }}
              aria-label={`${name} status page`}
            >
              ↗
            </a>
          </>
        )}
      </div>

      {expanded && status && (
        <div style={{ paddingLeft: '18px', paddingTop: '8px' }}>
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
                  const showUptime = c.uptime_30d != null || c.uptime_7d != null;
                  return (
                    <>
                      <tr key={i}>
                        <td style={{ padding: '2px 8px 2px 0', fontFamily: 'var(--font-mono)' }}>{c.name}</td>
                        <td style={{ padding: '2px 0', color: 'var(--text-secondary)' }}>{c.status.replace(/_/g, ' ')}</td>
                      </tr>
                      {showUptime && (
                        <tr key={`${i}-uptime`}>
                          <td colSpan={2} style={{ padding: '0 0 4px 0' }}>
                            <span style={{ fontFamily: 'var(--font-mono)', fontSize: '11px', letterSpacing: '0.04em' }}>
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

function signalLevelStyle(level: SignalLevel): { label: string; color: string } {
  switch (level) {
    case 'spike':    return { label: '[Spike]',    color: 'var(--accent)' };
    case 'elevated': return { label: '[Elevated]', color: 'var(--text-primary)' };
    case 'normal':   return { label: '[Normal]',   color: 'var(--text-secondary)' };
    default:         return { label: '[Unknown]',  color: 'var(--text-secondary)' };
  }
}

interface CommunitySignalRowProps {
  label: string;
  signals: ServiceSignal[];
}

function CommunitySignalRow({ label, signals }: CommunitySignalRowProps) {
  const first = signals[0];
  if (!first) return null;

  const levelOrder: SignalLevel[] = ['spike', 'elevated', 'normal', 'unknown'];
  const worstLevel: SignalLevel = levelOrder.find(l => signals.some(s => s.level === l)) ?? 'unknown';
  const { label: levelLabel, color } = signalLevelStyle(worstLevel);

  return (
    <div style={{ display: 'flex', alignItems: 'center', padding: '4px 0', gap: '8px' }}>
      <span style={{ fontFamily: 'var(--font-mono)', fontSize: '13px', flex: 1 }}>{label}</span>
      <span style={{ fontFamily: 'var(--font-mono)', fontSize: '12px', color }}>{levelLabel}</span>
      <a
        href={first.source_url}
        target="_blank"
        rel="noopener noreferrer"
        style={{ color: 'var(--text-secondary)', fontSize: '11px', opacity: 0.5 }}
        aria-label={`${label} community signal source`}
      >
        ↗
      </a>
    </div>
  );
}

export function AgentStatusCard({ snapshot, communitySignal }: AgentStatusCardProps) {
  const expanded = agent_status_expanded.value;
  const hasData = snapshot.claude != null || snapshot.openai != null;

  const hasCommunity = !!(
    communitySignal?.enabled &&
    (communitySignal.claude.length > 0 || communitySignal.openai.length > 0)
  );

  return (
    <div
      class="card"
      style={{
        minWidth: '300px',
        display: 'flex',
        flexDirection: 'column',
      }}
    >
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          marginBottom: '12px',
        }}
      >
        <div class="stat-label" style={{ margin: 0 }}>Agent status</div>
        {hasData && (
          <button
            type="button"
            onClick={() => {
              agent_status_expanded.value = !expanded;
              syncDashboardUrl();
            }}
            style={{
              display: 'inline-flex',
              alignItems: 'center',
              gap: '4px',
              background: 'none',
              border: 'none',
              cursor: 'pointer',
              color: 'var(--text-secondary)',
              fontSize: '11px',
              fontFamily: 'var(--font-mono)',
              padding: '4px 0 4px 8px',
              opacity: 0.7,
            }}
            aria-expanded={expanded}
          >
            <span aria-hidden="true" style={{ fontSize: '9px' }}>{expanded ? '▲' : '▼'}</span>
            <span>{expanded ? 'Collapse' : 'Expand'}</span>
          </button>
        )}
      </div>

      <ProviderRow name="Claude"         status={snapshot.claude} expanded={expanded} isLast={false} />
      <ProviderRow name="OpenAI / Codex" status={snapshot.openai} expanded={expanded} isLast />

      {hasCommunity && communitySignal && (
        <div style={{ marginTop: '12px', borderTop: '1px solid var(--border)', paddingTop: '8px' }}>
          <div
            style={{
              fontSize: 'var(--font-size-tertiary)',
              fontFamily: 'var(--font-sans)',
              color: 'var(--text-secondary)',
              marginBottom: '6px',
              letterSpacing: 0,
            }}
          >
            Community signal
          </div>
          <CommunitySignalRow label="Claude" signals={communitySignal.claude} />
          <CommunitySignalRow label="OpenAI" signals={communitySignal.openai} />
          {communitySignal.fetched_at && (
            <div
              style={{
                fontSize: '10px',
                color: 'var(--text-secondary)',
                marginTop: '4px',
                fontFamily: 'var(--font-mono)',
                opacity: 0.6,
              }}
            >
              Crowd data {fmtUtc(communitySignal.fetched_at)} UTC
            </div>
          )}
        </div>
      )}

      {/* Timestamp pinned to bottom */}
      {snapshot.fetched_at && (
        <div
          style={{
            marginTop: 'auto',
            paddingTop: '12px',
            fontSize: '10px',
            color: 'var(--text-secondary)',
            fontFamily: 'var(--font-mono)',
            opacity: 0.6,
          }}
        >
          Last checked {fmtUtc(snapshot.fetched_at)} UTC
        </div>
      )}
    </div>
  );
}
