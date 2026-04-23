import { SegmentedProgressBar } from './SegmentedProgressBar';
import { fmt, fmtResetTime } from '../lib/format';
import type { DepletionForecast, DepletionForecastSignal, QuotaSeverity } from '../state/types';
import type { SegmentedBarStatus } from './SegmentedProgressBar';

function severityToStatus(severity: QuotaSeverity): SegmentedBarStatus {
  if (severity === 'danger') return 'accent';
  if (severity === 'warn') return 'warning';
  return 'success';
}

function primaryValueLabel(signal: DepletionForecastSignal): string {
  const percent = signal.projected_percent ?? signal.used_percent;
  return `${Math.round(percent)}% ${signal.projected_percent != null ? 'projected' : 'used'}`;
}

function remainingLabel(signal: DepletionForecastSignal): string | null {
  if (signal.remaining_tokens != null) {
    return `${fmt(signal.remaining_tokens)} tokens left`;
  }
  if (signal.remaining_percent != null) {
    return `${Math.round(signal.remaining_percent)}% remaining`;
  }
  return null;
}

function timingLabel(signal: DepletionForecastSignal): string | null {
  if (signal.resets_in_minutes != null) {
    return `Resets in ${fmtResetTime(signal.resets_in_minutes)}`;
  }
  if (!signal.end_time) return null;
  const date = new Date(signal.end_time);
  if (Number.isNaN(date.getTime())) return signal.end_time;
  return `Ends ${date.toISOString().slice(11, 16)} UTC`;
}

function supportValue(signal: DepletionForecastSignal): string {
  const percent = signal.projected_percent ?? signal.used_percent;
  return `${Math.round(percent)}%`;
}

export function DepletionForecastCard({
  forecast,
  title = 'Depletion Forecast',
}: {
  forecast: DepletionForecast;
  title?: string;
}) {
  const primary = forecast.primary_signal;
  const primaryPercent = Math.max(0, primary.projected_percent ?? primary.used_percent);

  return (
    <div class="card stat-card">
      <div class="stat-content" style={{ display: 'grid', gap: '12px' }}>
        <div>
          <div class="stat-label">{title}</div>
          <div class="stat-value" style={{ fontSize: '22px' }}>{primary.title}</div>
          <div class="stat-sub">{forecast.summary_label}</div>
        </div>

        <div style={{ display: 'grid', gap: '6px' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', gap: '12px', alignItems: 'baseline' }}>
            <span class="stat-sub">{primaryValueLabel(primary)}</span>
            {primary.pace_label && <span class="stat-sub">{primary.pace_label}</span>}
          </div>
          <SegmentedProgressBar
            value={Math.min(primaryPercent, 100)}
            max={100}
            status={severityToStatus(forecast.severity)}
            aria-label="Depletion forecast pressure"
          />
          <div style={{ display: 'grid', gap: '3px' }}>
            {timingLabel(primary) && <div class="stat-sub">{timingLabel(primary)}</div>}
            {remainingLabel(primary) && <div class="stat-sub">{remainingLabel(primary)}</div>}
          </div>
        </div>

        {forecast.secondary_signals.length > 0 && (
          <div style={{ display: 'grid', gap: '8px' }}>
            <div class="stat-sub" style={{ fontSize: '10px', letterSpacing: '0.08em' }}>
              SUPPORTING SIGNALS
            </div>
            {forecast.secondary_signals.map(signal => (
              <div key={`${signal.kind}-${signal.title}`} style={{ display: 'grid', gap: '2px' }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', gap: '12px', alignItems: 'baseline' }}>
                  <span class="stat-sub">{signal.title}</span>
                  <span class="stat-sub" style={{ fontFamily: 'var(--font-mono)', fontSize: '11px' }}>
                    {supportValue(signal)}
                  </span>
                </div>
                <div class="stat-sub">
                  {[timingLabel(signal), remainingLabel(signal)].filter(Boolean).join(' · ')}
                </div>
              </div>
            ))}
          </div>
        )}

        {forecast.note && <div class="stat-sub" style={{ fontStyle: 'italic' }}>{forecast.note}</div>}
      </div>
    </div>
  );
}
