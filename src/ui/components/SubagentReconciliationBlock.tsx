import type { SubagentReconciliation } from '../state/types';

interface SubagentReconciliationBlockProps {
  reconciliation: SubagentReconciliation;
}

export function SubagentReconciliationBlock({ reconciliation }: SubagentReconciliationBlockProps) {
  const deltaMatch = Math.abs(reconciliation.delta_cost) < 0.01;

  if (!reconciliation.available) {
    return (
      <div class="card card-flat bento-full" style={{ padding: '12px 20px' }}>
        <div style={{
          display: 'flex',
          alignItems: 'center',
          flexWrap: 'wrap',
          gap: '12px',
          fontFamily: 'var(--font-mono)',
          fontSize: '12px',
          letterSpacing: '0.04em',
          color: 'var(--text-secondary)',
        }}>
          <span style={{
            fontSize: '10px',
            letterSpacing: '0.08em',
            textTransform: 'uppercase',
            color: 'var(--text-disabled)',
          }}>
            Subagent Reconciliation
          </span>
          <span style={{ color: 'var(--text-disabled)' }}>·</span>
          <span>{reconciliation.error ?? 'Unavailable'}</span>
        </div>
      </div>
    );
  }

  const statusBracket = deltaMatch
    ? { label: '[OK]', color: 'var(--success, var(--text-primary))' }
    : { label: `[DRIFT: ${reconciliation.delta_cost >= 0 ? '+' : ''}$${reconciliation.delta_cost.toFixed(4)}]`, color: 'var(--accent)' };

  return (
    <div class="card card-flat bento-full">
      <div style={{ display: 'flex', alignItems: 'baseline', gap: '12px', flexWrap: 'wrap' }}>
        <h2 style={{ margin: 0 }}>Subagent Cost Reconciliation</h2>
        <span
          style={{
            fontFamily: 'var(--font-mono)',
            fontSize: '11px',
            letterSpacing: '0.04em',
            color: statusBracket.color,
          }}
          aria-label={deltaMatch ? 'reconciliation matches within tolerance' : 'reconciliation drift detected'}
        >
          {statusBracket.label}
        </span>
      </div>
      <div class="muted" style={{ marginBottom: '12px', marginTop: '4px' }}>
        Compares the child agent JSONL view (<code>agent_sessions</code>) against the parent
        sidechain view (<code>turns WHERE is_subagent = 1</code>) over the last{' '}
        {reconciliation.lookback_days} days. Drift signals parser divergence.
      </div>
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit,minmax(200px,1fr))', gap: '16px' }}>
        <div class="stat-card">
          <div class="stat-content">
            <div class="stat-label">Period</div>
            <div class="stat-value" style={{ fontSize: '16px' }}>
              {reconciliation.start_date} - {reconciliation.end_date}
            </div>
            <div class="stat-sub">Rolling comparison window</div>
          </div>
        </div>
        <div class="stat-card">
          <div class="stat-content">
            <div class="stat-label">Agent-Sessions Cost</div>
            <div class="stat-value cost-value" style={{ fontSize: '20px' }}>
              ${reconciliation.agent_sessions_cost.toFixed(4)}
            </div>
            <div class="stat-sub">Child JSONL view</div>
          </div>
        </div>
        <div class="stat-card">
          <div class="stat-content">
            <div class="stat-label">Sidechain Turns Cost</div>
            <div class="stat-value cost-value" style={{ fontSize: '20px' }}>
              ${reconciliation.turns_subagent_cost.toFixed(4)}
            </div>
            <div class="stat-sub">Parent JSONL view</div>
          </div>
        </div>
        <div class="stat-card">
          <div class="stat-content">
            <div class="stat-label">Delta</div>
            <div class="stat-value" style={{ fontSize: '20px', color: deltaMatch ? 'var(--text-primary)' : 'var(--accent)' }}>
              {reconciliation.delta_cost >= 0 ? '+' : ''}${reconciliation.delta_cost.toFixed(4)}
            </div>
            <div class="stat-sub">Agent-sessions minus sidechain</div>
          </div>
        </div>
        <div class="stat-card">
          <div class="stat-content">
            <div class="stat-label">Spawns / Sidechain Turns</div>
            <div class="stat-value" style={{ fontSize: '16px' }}>
              {reconciliation.agent_session_rows.toLocaleString()} / {reconciliation.subagent_turn_rows.toLocaleString()}
            </div>
            <div class="stat-sub">Row counts per view</div>
          </div>
        </div>
        <div class="stat-card">
          <div class="stat-content">
            <div class="stat-label">Distinct Agents</div>
            <div class="stat-value" style={{ fontSize: '16px' }}>
              {reconciliation.distinct_agents_in_agent_sessions.toLocaleString()} / {reconciliation.distinct_agents_in_turns.toLocaleString()}
            </div>
            <div class="stat-sub">Child / parent</div>
          </div>
        </div>
      </div>
    </div>
  );
}
