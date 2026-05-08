import { useState } from 'preact/hooks';
import { rawData } from '../state/store';
import type { AgentTreeSummary, SessionAgentTree, AgentTreeNode } from '../state/dashboard-types';

function fmtCost(nanos: number): string {
  const cents = nanos / 1e7;
  if (cents >= 100) return '$' + (cents / 100).toFixed(2);
  return (cents).toFixed(2) + '¢';
}

function fmtTokens(n: number): string {
  if (n >= 1_000_000) return (n / 1_000_000).toFixed(1) + 'M';
  if (n >= 1_000) return (n / 1_000).toFixed(0) + 'K';
  return String(n);
}

function NodeRow({ node, indent = 0 }: { node: AgentTreeNode; indent?: number }) {
  const label = node.agent_id
    ? node.role
      ? `${node.role} (${node.agent_id.slice(0, 8)})`
      : node.agent_id.slice(0, 12)
    : 'root';

  return (
    <div
      style={{
        display: 'grid',
        gridTemplateColumns: '1fr auto auto',
        gap: '8px',
        alignItems: 'center',
        paddingLeft: `${indent * 16}px`,
        paddingTop: '3px',
        paddingBottom: '3px',
        borderTop: indent === 0
          ? '1px solid rgba(var(--text-primary-rgb,232,232,232),0.08)'
          : 'none',
        fontSize: '11px',
      }}
    >
      <span
        style={{
          fontFamily: 'var(--font-mono)',
          fontSize: '10px',
          color: indent === 0 ? 'var(--text-primary)' : 'var(--text-secondary)',
        }}
      >
        {indent > 0 && <span style={{ opacity: 0.4, marginRight: '4px' }}>└</span>}
        {label}
        {node.role && indent === 0 && (
          <span style={{ marginLeft: '6px', opacity: 0.5, fontSize: '9px' }}>root</span>
        )}
      </span>
      <span style={{ fontFamily: 'var(--font-mono)', fontSize: '10px', color: 'var(--text-secondary)' }}>
        {fmtTokens(node.input_tokens + node.output_tokens)} tok
      </span>
      <span style={{ fontFamily: 'var(--font-mono)', fontSize: '10px', minWidth: '40px', textAlign: 'right' }}>
        {fmtCost(node.estimated_cost_nanos)}
      </span>
    </div>
  );
}

function SessionTree({ tree }: { tree: SessionAgentTree }) {
  const [open, setOpen] = useState(false);
  const project = tree.project ? tree.project.slice(-28) : null;

  return (
    <div style={{ marginBottom: '2px' }}>
      <button
        type="button"
        onClick={() => setOpen(!open)}
        style={{
          background: 'none',
          border: 'none',
          cursor: 'pointer',
          width: '100%',
          textAlign: 'left',
          padding: '6px 0',
          display: 'flex',
          alignItems: 'center',
          gap: '8px',
          borderTop: '1px solid rgba(var(--text-primary-rgb,232,232,232),0.08)',
          color: 'var(--text-primary)',
        }}
      >
        <span style={{ fontFamily: 'var(--font-mono)', fontSize: '10px', opacity: 0.6 }}>
          {tree.session_id.slice(0, 8)}
        </span>
        {project && (
          <span style={{ fontSize: '10px', color: 'var(--text-secondary)' }}>{project}</span>
        )}
        <span
          style={{
            fontFamily: 'var(--font-mono)',
            fontSize: '10px',
            color: 'var(--text-secondary)',
          }}
        >
          {tree.subagent_count} subagent{tree.subagent_count !== 1 ? 's' : ''}
        </span>
        <span style={{ marginLeft: 'auto', fontFamily: 'var(--font-mono)', fontSize: '10px' }}>
          {fmtCost(tree.total_cost_nanos)}
        </span>
        <span style={{ fontFamily: 'var(--font-mono)', fontSize: '10px', color: 'var(--text-secondary)', flexShrink: 0 }}>
          {open ? '▲' : '▼'}
        </span>
      </button>

      {open && (
        <div style={{ paddingBottom: '6px' }}>
          <NodeRow node={tree.root} indent={0} />
          {tree.root.children.map((child, i) => (
            <NodeRow key={child.agent_id ?? i} node={child} indent={1} />
          ))}
        </div>
      )}
    </div>
  );
}

function AgentTreeCardInner({ summary }: { summary: AgentTreeSummary }) {
  const topRole = summary.top_subagent_roles[0];
  const totalSubagentCost = summary.sessions.reduce(
    (acc, s) => acc + s.root.children.reduce((a, c) => a + c.estimated_cost_nanos, 0),
    0,
  );

  return (
    <div class="card" style={{ padding: '16px' }}>
      <div class="stat-label" style={{ marginBottom: '10px' }}>Subagent cost attribution</div>

      <div style={{ display: 'flex', gap: '20px', flexWrap: 'wrap', marginBottom: '14px' }}>
        <div>
          <div class="stat-label" style={{ fontSize: '10px' }}>Sessions</div>
          <div style={{ fontFamily: 'var(--font-mono)', fontSize: '18px' }}>{summary.sessions.length}</div>
        </div>
        <div>
          <div class="stat-label" style={{ fontSize: '10px' }}>Subagent cost</div>
          <div style={{ fontFamily: 'var(--font-mono)', fontSize: '18px' }}>{fmtCost(totalSubagentCost)}</div>
        </div>
        {topRole && (
          <div>
            <div class="stat-label" style={{ fontSize: '10px' }}>Top role</div>
            <div style={{ fontFamily: 'var(--font-mono)', fontSize: '12px', marginTop: '4px' }}>
              {topRole[0]} · {fmtCost(topRole[1])}
            </div>
          </div>
        )}
      </div>

      {summary.sessions.length === 0 && (
        <div style={{ fontSize: '11px', color: 'var(--text-secondary)', fontFamily: 'var(--font-mono)' }}>
          No multi-agent sessions found.
        </div>
      )}

      {summary.sessions.slice(0, 30).map((tree) => (
        <SessionTree key={tree.session_id} tree={tree} />
      ))}
    </div>
  );
}

export function AgentTreeCard() {
  const data = rawData.value;

  if (!data) {
    return (
      <div class="card" style={{ padding: '16px' }}>
        <div class="stat-label">Subagent cost attribution</div>
        <div style={{ color: 'var(--text-secondary)', fontFamily: 'var(--font-mono)', fontSize: '12px', marginTop: '8px' }}>
          loading...
        </div>
      </div>
    );
  }

  const summary = data.agent_tree;
  if (!summary) {
    return (
      <div class="card" style={{ padding: '16px' }}>
        <div class="stat-label">Subagent cost attribution</div>
        <div style={{ color: 'var(--text-secondary)', fontFamily: 'var(--font-mono)', fontSize: '12px', marginTop: '8px' }}>
          No data available.
        </div>
      </div>
    );
  }

  return <AgentTreeCardInner summary={summary} />;
}
