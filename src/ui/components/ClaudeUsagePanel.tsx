import type { ClaudeUsageResponse } from '../state/types';
import { fmtRelativeTime } from '../lib/format';
import { SegmentedProgressBar } from './SegmentedProgressBar';

interface ClaudeUsagePanelProps {
  data: ClaudeUsageResponse;
}

function statusColor(status: string | undefined): string {
  if (status === 'failed') return 'var(--accent)';
  if (status === 'unparsed') return 'var(--warning)';
  return 'var(--text-secondary)';
}

export function ClaudeUsagePanel({ data }: ClaudeUsagePanelProps) {
  const snapshot = data.latest_snapshot ?? null;
  const lastRun = data.last_run ?? null;
  const lastSuccess = snapshot?.run.captured_at ?? null;

  return (
    <div class="card card-flat bento-full table-card" aria-label="Claude usage monitor">
      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          gap: '16px',
          alignItems: 'flex-start',
          flexWrap: 'wrap',
          marginBottom: snapshot ? '18px' : '0',
        }}
      >
        <div>
          <h2 style={{ marginBottom: '8px' }}>Claude /usage</h2>
          <div class="stat-sub">
            Last success {fmtRelativeTime(lastSuccess)}
            {lastRun ? ` · Last run ${fmtRelativeTime(lastRun.captured_at)}` : ''}
          </div>
        </div>
        {lastRun && (
          <div
            style={{
              fontFamily: 'var(--font-mono)',
              fontSize: '12px',
              letterSpacing: '0.08em',
              textTransform: 'uppercase',
              color: statusColor(lastRun.status),
            }}
          >
            [{lastRun.status}]
          </div>
        )}
      </div>

      {!snapshot && (
        <div class="stat-sub">
          {lastRun?.error_summary || 'No parsed Claude /usage snapshot has been captured yet.'}
        </div>
      )}

      {snapshot && (
        <div style={{ display: 'grid', gap: '14px' }}>
          {snapshot.factors.map((factor) => (
            <div
              key={factor.factor_key}
              style={{
                borderTop: '1px solid var(--border)',
                paddingTop: '14px',
              }}
            >
              <div
                style={{
                  display: 'flex',
                  justifyContent: 'space-between',
                  alignItems: 'baseline',
                  gap: '16px',
                  marginBottom: '10px',
                  flexWrap: 'wrap',
                }}
              >
                <div style={{ fontWeight: 500 }}>{factor.display_label}</div>
                <div
                  style={{
                    fontFamily: 'var(--font-mono)',
                    fontSize: '14px',
                    whiteSpace: 'nowrap',
                  }}
                >
                  {factor.percent.toFixed(1)}%
                </div>
              </div>
              <SegmentedProgressBar
                value={factor.percent}
                max={100}
                size="compact"
                aria-label={`${factor.display_label} percent`}
              />
              {factor.advice_text && (
                <div class="stat-sub" style={{ marginTop: '10px' }}>
                  {factor.advice_text}
                </div>
              )}
            </div>
          ))}
          {lastRun?.status !== 'success' && lastRun?.error_summary && (
            <div class="stat-sub" style={{ marginTop: '4px' }}>
              Latest run note: {lastRun.error_summary}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
