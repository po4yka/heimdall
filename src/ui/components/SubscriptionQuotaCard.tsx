import type { ProviderQuotaSnapshot } from '../state/dashboard-types';
import { fmt, fmtLabel, fmtResetTime } from '../lib/format';
import { SegmentedProgressBar } from './SegmentedProgressBar';

interface SubscriptionQuotaCardProps {
  snapshot: ProviderQuotaSnapshot;
}

const PROVIDER_LABELS: Record<string, string> = {
  claude: 'Claude',
  codex: 'Codex',
};

function providerTitle(provider: string): string {
  return PROVIDER_LABELS[provider] ?? fmtLabel(provider);
}

function fmtConfidence(c: number): string {
  if (c >= 0.85) return 'high confidence';
  if (c >= 0.5) return 'medium confidence';
  return 'low confidence';
}

export function SubscriptionQuotaCard({ snapshot }: SubscriptionQuotaCardProps) {
  const title = providerTitle(snapshot.provider);
  const planLabel = snapshot.published.plan_label
    ? fmtLabel(snapshot.published.plan_label)
    : '—';
  const estimated = snapshot.estimated?.windows ?? [];
  const hasEstimates = estimated.length > 0;

  return (
    <div class="card subscription-quota-card">
      <div class="subscription-quota-header">
        <div class="subscription-quota-title">{title}</div>
        <div class="subscription-quota-plan" title={`Source: ${snapshot.source_used}`}>
          {planLabel}
        </div>
      </div>

      <div class="subscription-quota-section">
        <div class="subscription-quota-section-label">Published</div>
        {snapshot.published.windows.length === 0 && (
          <div class="subscription-quota-empty">No active windows reported.</div>
        )}
        {snapshot.published.windows.map(window => {
          const pct = Math.min(100, Math.max(0, window.used_percent));
          return (
            <div class="subscription-quota-row" key={`pub-${window.kind}`}>
              <div class="subscription-quota-row-label">{window.label}</div>
              <div class="subscription-quota-row-value">{pct.toFixed(1)}%</div>
              <div class="subscription-quota-row-bar">
                <SegmentedProgressBar
                  value={pct}
                  max={100}
                  size="standard"
                  aria-label={`${window.label} usage`}
                />
              </div>
              <div class="subscription-quota-row-sub">
                {window.resets_in_minutes != null
                  ? `Resets in ${fmtResetTime(window.resets_in_minutes)}`
                  : window.resets_at ?? ''}
              </div>
            </div>
          );
        })}
        {snapshot.published.budget && (
          <div class="subscription-quota-row" key="pub-budget">
            <div class="subscription-quota-row-label">Monthly $-budget</div>
            <div class="subscription-quota-row-value">
              ${snapshot.published.budget.used_usd.toFixed(2)}
            </div>
            <div class="subscription-quota-row-bar">
              <SegmentedProgressBar
                value={snapshot.published.budget.utilization}
                max={100}
                size="standard"
                aria-label="Budget usage"
              />
            </div>
            <div class="subscription-quota-row-sub">
              of ${snapshot.published.budget.limit_usd.toFixed(2)} ({snapshot.published.budget.currency})
            </div>
          </div>
        )}
      </div>

      <div class="subscription-quota-divider" />

      <div class="subscription-quota-section">
        <div class="subscription-quota-section-label">
          Estimated{' '}
          <small style={{ fontSize: '9px', letterSpacing: '0.08em', color: 'var(--text-disabled)', textTransform: 'uppercase' }}>estimated</small>
        </div>
        {!hasEstimates && (
          <div class="subscription-quota-empty">
            Insufficient data — utilization too low to derive a token cap.
          </div>
        )}
        {estimated.map(w => {
          const dim = w.confidence < 0.3;
          const headlineCap =
            w.smoothed_cap_tokens != null ? w.smoothed_cap_tokens : w.estimated_cap_tokens;
          const shiftGlyph =
            w.cap_shift === 'increase' ? '↑ ' : w.cap_shift === 'decrease' ? '↓ ' : '';
          const shiftClass =
            w.cap_shift === 'decrease'
              ? 'subscription-quota-shift subscription-quota-shift-down'
              : w.cap_shift === 'increase'
                ? 'subscription-quota-shift subscription-quota-shift-up'
                : '';
          const subParts: string[] = [
            `from ${fmt(w.observed_tokens)} observed`,
            fmtConfidence(w.confidence),
          ];
          if (w.sample_count != null && w.sample_count > 0) {
            subParts.push(`n=${w.sample_count}`);
          }
          return (
            <div
              class="subscription-quota-row subscription-quota-row-estimated"
              key={`est-${w.kind}`}
              style={dim ? { opacity: 0.6 } : undefined}
            >
              <div class="subscription-quota-row-label">{w.label}</div>
              <div class="subscription-quota-row-value">
                {shiftGlyph && <span class={shiftClass}>{shiftGlyph}</span>}
                ~{fmt(headlineCap)} tokens
              </div>
              <div class="subscription-quota-row-sub">{subParts.join(' · ')}</div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
