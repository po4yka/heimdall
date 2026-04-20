import type { OpenAiReconciliation } from '../state/types';

interface ReconciliationBlockProps {
  reconciliation: OpenAiReconciliation;
}

export function ReconciliationBlock({ reconciliation }: ReconciliationBlockProps) {
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
            OpenAI Reconciliation
          </span>
          <span style={{ color: 'var(--text-disabled)' }}>·</span>
          <span>{reconciliation.error ?? 'Unavailable'}</span>
        </div>
      </div>
    );
  }

  return (
    <div class="card card-flat bento-full">
      <h2>OpenAI Org Usage Reconciliation</h2>
      <div class="muted" style={{ marginBottom: '12px' }}>
        Official OpenAI organization usage buckets for Codex-compatible models over the last {reconciliation.lookback_days} days.
      </div>
      {reconciliation.available ? (
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
              <div class="stat-label">Local Estimated Cost</div>
              <div class="stat-value cost-value" style={{ fontSize: '20px' }}>
                ${reconciliation.estimated_local_cost.toFixed(4)}
              </div>
              <div class="stat-sub">Codex local logs</div>
            </div>
          </div>
          <div class="stat-card">
            <div class="stat-content">
              <div class="stat-label">Org Usage Cost</div>
              <div class="stat-value cost-value" style={{ fontSize: '20px' }}>
                ${reconciliation.api_usage_cost.toFixed(4)}
              </div>
              <div class="stat-sub">OpenAI organization usage API</div>
            </div>
          </div>
          <div class="stat-card">
            <div class="stat-content">
              <div class="stat-label">Delta</div>
              <div class="stat-value" style={{ fontSize: '20px', color: deltaMatch ? 'var(--text-primary)' : 'var(--accent)' }}>
                {reconciliation.delta_cost >= 0 ? '+' : ''}${reconciliation.delta_cost.toFixed(4)}
              </div>
              <div class="stat-sub">Org usage cost minus local estimate</div>
            </div>
          </div>
          <div class="stat-card">
            <div class="stat-content">
              <div class="stat-label">API Tokens</div>
              <div class="stat-value" style={{ fontSize: '16px' }}>
                {reconciliation.api_input_tokens.toLocaleString()} / {reconciliation.api_output_tokens.toLocaleString()}
              </div>
              <div class="stat-sub">Input / output tokens</div>
            </div>
          </div>
          <div class="stat-card">
            <div class="stat-content">
              <div class="stat-label">Cached Input + Requests</div>
              <div class="stat-value" style={{ fontSize: '16px' }}>
                {reconciliation.api_cached_input_tokens.toLocaleString()} / {reconciliation.api_requests.toLocaleString()}
              </div>
              <div class="stat-sub">Cached input tokens / requests</div>
            </div>
          </div>
        </div>
      ) : (
        <div class="muted">{reconciliation.error ?? 'Unavailable'}</div>
      )}
    </div>
  );
}
