import type { WindowInfo } from '../state/types';
import { fmtResetTime, progressColor } from '../lib/format';

interface RateWindowCardProps {
  label: string;
  window: WindowInfo;
}

export function RateWindowCard({ label, window }: RateWindowCardProps) {
  const pct = Math.min(100, window.used_percent);
  const color = progressColor(pct);
  const resetText = window.resets_in_minutes != null
    ? `Resets in ${fmtResetTime(window.resets_in_minutes)}`
    : '';

  return (
    <div class="card stat-card">
      <div class="stat-content">
        <div class="stat-label">{label}</div>
        <div class="stat-value" style={{ fontSize: '28px', color }}>{pct.toFixed(1)}%</div>
        <div class="progress-track" style={{ marginTop: '12px' }}>
          <div class="progress-fill" style={{ background: color, width: `${pct}%` }} />
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
  const pct = Math.min(100, utilization);
  const color = progressColor(pct);
  return (
    <div class="card stat-card">
      <div class="stat-content">
        <div class="stat-label">Monthly Budget</div>
        <div class="stat-value" style={{ fontSize: '24px', color }}>
          ${used.toFixed(2)} / ${limit.toFixed(2)}
        </div>
        <div class="progress-track" style={{ marginTop: '12px' }}>
          <div class="progress-fill" style={{ background: color, width: `${pct}%` }} />
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
        <div class="stat-value" style={{ fontSize: '16px', color: 'var(--text-secondary)' }}>
          Unavailable
        </div>
        <div class="stat-sub">{error}</div>
      </div>
    </div>
  );
}
