import type { ClaudeAdminSummary, WindowInfo } from '../state/types';
import { fmtResetTime } from '../lib/format';
import { SegmentedProgressBar } from './SegmentedProgressBar';

interface RateWindowCardProps {
  label: string;
  window: WindowInfo;
}

export function RateWindowCard({ label, window }: RateWindowCardProps) {
  const pct = Math.min(100, window.used_percent);
  const resetText = window.resets_in_minutes != null
    ? `Resets in ${fmtResetTime(window.resets_in_minutes)}`
    : '';

  return (
    <div class="card stat-card">
      <div class="stat-content">
        <div class="stat-label">{label}</div>
        <div class="stat-value" style={{ fontSize: '28px' }}>{pct.toFixed(1)}%</div>
        <div style={{ marginTop: '12px' }}>
          <SegmentedProgressBar
            value={window.used_percent}
            max={100}
            size="standard"
            aria-label={`${label} usage`}
          />
        </div>
        {resetText && <div class="stat-sub">{resetText}</div>}
      </div>
    </div>
  );
}

interface BudgetCardProps {
  used: number;
  limit: number;
  currency: string;
  utilization: number;
}

export function BudgetCard({ used, limit, currency, utilization }: BudgetCardProps) {
  return (
    <div class="card stat-card">
      <div class="stat-content">
        <div class="stat-label">Monthly Budget</div>
        <div class="stat-value" style={{ fontSize: '24px' }}>
          ${used.toFixed(2)} / ${limit.toFixed(2)}
        </div>
        <div style={{ marginTop: '12px' }}>
          <SegmentedProgressBar
            value={utilization}
            max={100}
            size="standard"
            aria-label="Monthly budget usage"
          />
        </div>
        <div class="stat-sub">{currency}</div>
      </div>
    </div>
  );
}

interface UnavailableCardProps {
  error: string;
}

export function RateWindowUnavailable({ error }: UnavailableCardProps) {
  return (
    <div class="card stat-card">
      <div class="stat-content">
        <div class="stat-label">Rate Windows</div>
        <div class="stat-value" style={{ fontSize: '18px', color: 'var(--text-secondary)' }}>
          Unavailable
        </div>
        <div class="stat-sub">{error}</div>
      </div>
    </div>
  );
}

interface ClaudeAdminCardProps {
  label: string;
  value: string;
  subtitle: string;
}

export function ClaudeAdminCard({ label, value, subtitle }: ClaudeAdminCardProps) {
  return (
    <div class="card stat-card">
      <div class="stat-content">
        <div class="stat-label">{label}</div>
        <div class="stat-value" style={{ fontSize: '24px' }}>{value}</div>
        <div class="stat-sub">{subtitle}</div>
      </div>
    </div>
  );
}

interface ClaudeAdminFallbackGridProps {
  summary: ClaudeAdminSummary;
}

export function ClaudeAdminFallbackGrid({ summary }: ClaudeAdminFallbackGridProps) {
  const subtitle = `${summary.organization_name || 'Org-wide'} · ${summary.data_latency_note}`;
  return (
    <>
      <ClaudeAdminCard
        label="Active Users Today"
        value={summary.today_active_users.toLocaleString()}
        subtitle={subtitle}
      />
      <ClaudeAdminCard
        label="Sessions Today"
        value={summary.today_sessions.toLocaleString()}
        subtitle={subtitle}
      />
      <ClaudeAdminCard
        label={`Accepted Lines (${summary.lookback_days}d)`}
        value={summary.lookback_lines_accepted.toLocaleString()}
        subtitle={subtitle}
      />
      <ClaudeAdminCard
        label={`Estimated Spend (${summary.lookback_days}d)`}
        value={`$${summary.lookback_estimated_cost_usd.toFixed(2)}`}
        subtitle={subtitle}
      />
    </>
  );
}
