import { fmtCostBig, fmt } from '../../lib/format';
import type { AgentTelemetry } from '../../state/types';

interface AgentKpisProps {
  telemetry: AgentTelemetry;
  totalCostUsd: number;
}

export function AgentKpis({ telemetry, totalCostUsd }: AgentKpisProps) {
  const { totals } = telemetry;

  if (totals.sessions === 0) {
    return (
      <div class="table-card" style={{ padding: '20px' }}>
        <div class="stat-label" style={{ marginBottom: 0 }}>Agent delegation</div>
        <div class="empty-state">No agent activity yet</div>
      </div>
    );
  }

  const delegationPct =
    totalCostUsd > 0 ? ((totals.cost_usd / totalCostUsd) * 100).toFixed(1) : '0.0';
  const tokensPerSession =
    totals.sessions > 0 ? Math.round(totals.total_tokens / totals.sessions) : 0;
  const costPerSession =
    totals.sessions > 0 ? totals.cost_usd / totals.sessions : 0;

  const cards = [
    {
      label: 'Agent delegation',
      value: `${delegationPct}%`,
      sub: `${fmtCostBig(totals.cost_usd)} agent cost`,
    },
    {
      label: 'Agent sessions',
      value: totals.sessions.toLocaleString(),
      sub: `${fmt(totals.total_tokens)} total tokens`,
    },
    {
      label: 'Tokens / session',
      value: totals.sessions > 0 ? fmt(tokensPerSession) : '—',
      sub: totals.sessions > 0 ? `${fmtCostBig(costPerSession)} avg cost` : 'no sessions',
    },
  ];

  return (
    <>
      {cards.map(c => (
        <div class="card stat-card" key={c.label}>
          <div class="stat-content">
            <div class="stat-label">{c.label}</div>
            <div class="stat-value">{c.value}</div>
            <div class="stat-sub">{c.sub}</div>
          </div>
        </div>
      ))}
    </>
  );
}
