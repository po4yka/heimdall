import type { ClaudeAdminSummary, WindowInfo } from '../state/types';
import { fmtResetTime } from '../lib/format';
import { KpiCard } from './_primitives/KpiCard';

interface RateWindowCardProps {
  label: string;
  window: WindowInfo;
}

export function RateWindowCard({ label, window }: RateWindowCardProps) {
  const pct = Math.min(100, window.used_percent);
  const resetText =
    window.resets_in_minutes != null
      ? `Resets in ${fmtResetTime(window.resets_in_minutes)}`
      : '';

  return (
    <KpiCard
      label={label}
      value={`${pct.toFixed(1)}%`}
      bar={{ value: window.used_percent, max: 100, ariaLabel: `${label} usage` }}
      sub={resetText || undefined}
    />
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
    <KpiCard
      size="compact"
      label="Monthly budget"
      value={`$${used.toFixed(2)} / $${limit.toFixed(2)}`}
      bar={{ value: utilization, max: 100, ariaLabel: 'Monthly budget usage' }}
      sub={currency}
    />
  );
}

interface UnavailableCardProps {
  error: string;
}

export function RateWindowUnavailable({ error }: UnavailableCardProps) {
  return (
    <KpiCard
      size="compact"
      valueTone="muted"
      label="Rate windows"
      value="Unavailable"
      sub={error}
    />
  );
}

interface ClaudeAdminCardProps {
  label: string;
  value: string;
  subtitle: string;
}

export function ClaudeAdminCard({ label, value, subtitle }: ClaudeAdminCardProps) {
  return <KpiCard size="compact" label={label} value={value} sub={subtitle} />;
}

interface ClaudeAdminFallbackGridProps {
  summary: ClaudeAdminSummary;
}

export function ClaudeAdminFallbackGrid({ summary }: ClaudeAdminFallbackGridProps) {
  const subtitle = `${summary.organization_name || 'Org-wide'} · ${summary.data_latency_note}`;
  return (
    <>
      <ClaudeAdminCard
        label="Active users today"
        value={summary.today_active_users.toLocaleString()}
        subtitle={subtitle}
      />
      <ClaudeAdminCard
        label="Sessions today"
        value={summary.today_sessions.toLocaleString()}
        subtitle={subtitle}
      />
      <ClaudeAdminCard
        label={`Accepted lines (${summary.lookback_days}d)`}
        value={summary.lookback_lines_accepted.toLocaleString()}
        subtitle={subtitle}
      />
      <ClaudeAdminCard
        label={`Estimated spend (${summary.lookback_days}d)`}
        value={`$${summary.lookback_estimated_cost_usd.toFixed(2)}`}
        subtitle={subtitle}
      />
    </>
  );
}
