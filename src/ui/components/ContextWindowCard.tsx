import { SegmentedProgressBar } from './SegmentedProgressBar';
import type { SegmentedBarStatus } from './SegmentedProgressBar';
import { fmt } from '../lib/format';
import type { ContextWindowResponse, ContextWindowSeverity } from '../state/types';

interface Props {
  data: ContextWindowResponse | null;
}

function severityToStatus(s: ContextWindowSeverity): SegmentedBarStatus {
  if (s === 'ok') return 'success';
  if (s === 'warn') return 'warning';
  return 'accent';
}

function severityLabel(s: ContextWindowSeverity): string {
  return s === 'ok' ? '[OK]' : s === 'warn' ? '[WARN]' : '[CRIT]';
}

export function ContextWindowCard({ data }: Props) {
  if (!data || data.enabled === false) return null;
  if (data.total_input_tokens == null || data.context_window_size == null) return null;
  if (data.context_window_size <= 0) return null;

  const used = data.total_input_tokens;
  const size = data.context_window_size;
  const pct = Math.max(0, Math.min(999, (data.pct ?? used / size) * 100));
  const severity = data.severity ?? 'ok';

  return (
    <div class="card stat-card">
      <div class="stat-content">
        <div class="stat-label" style={{ letterSpacing: '0.08em', fontSize: '11px' }}>
          CONTEXT WINDOW
        </div>
        <div
          class="stat-value"
          style={{ fontFamily: 'var(--font-mono)', letterSpacing: '-0.02em' }}
        >
          {fmt(used)}
        </div>
        <div class="stat-sub">
          of {fmt(size)} &middot; {pct.toFixed(1)}%{' '}
          <span
            style={{
              color:
                severity === 'danger'
                  ? 'var(--accent)'
                  : severity === 'warn'
                  ? 'var(--warning)'
                  : undefined,
            }}
          >
            {severityLabel(severity)}
          </span>
        </div>
      </div>
      <SegmentedProgressBar
        value={used}
        max={size}
        status={severityToStatus(severity)}
        aria-label="Context window usage"
      />
    </div>
  );
}
