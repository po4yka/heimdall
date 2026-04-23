import { SegmentedProgressBar } from './SegmentedProgressBar';
import type { BillingBlocksResponse, BillingBlockView, BurnRateTier, QuotaSeverity } from '../state/types';
import type { SegmentedBarStatus } from './SegmentedProgressBar';

// ── Helpers ──────────────────────────────────────────────────────────

function severityToStatus(s: QuotaSeverity): SegmentedBarStatus {
  if (s === 'ok') return 'success';
  if (s === 'warn') return 'warning';
  return 'accent';
}

function severityLabel(s: QuotaSeverity): string {
  return s === 'ok' ? '[OK]' : s === 'warn' ? '[WARN]' : '[CRIT]';
}

function tierLabel(t: BurnRateTier): string {
  if (t === 'normal') return '[NORMAL]';
  if (t === 'moderate') return '[WARN]';
  return '[CRIT]';
}

function tierColor(t: BurnRateTier): string {
  if (t === 'normal') return 'var(--success)';
  if (t === 'moderate') return 'var(--warning)';
  return 'var(--accent)';
}

function formatDuration(from: string, to: string): string {
  const diffMs = new Date(to).getTime() - new Date(from).getTime();
  if (isNaN(diffMs) || diffMs < 0) return '--';
  const totalMin = Math.floor(diffMs / 60_000);
  const h = Math.floor(totalMin / 60);
  const m = totalMin % 60;
  if (h === 0) return `${m}m`;
  return `${h}h ${m}m`;
}

function fmtTokens(n: number): string {
  if (n >= 1_000_000) return (n / 1_000_000).toFixed(1) + 'M';
  if (n >= 1_000) return (n / 1_000).toFixed(1) + 'K';
  return n.toString();
}

function fmtUtcTime(iso: string): string {
  try {
    const d = new Date(iso);
    return d.toISOString().slice(11, 16) + ' UTC';
  } catch {
    return '--';
  }
}

function QuotaSuggestionsSection({ data }: { data: BillingBlocksResponse }) {
  const suggestions = data.quota_suggestions;
  if (!suggestions || suggestions.levels.length === 0) {
    return null;
  }

  return (
    <div style={{ marginTop: '12px', display: 'grid', gap: '8px' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', gap: '12px', alignItems: 'baseline' }}>
        <span class="stat-sub" style={{ fontSize: '10px', letterSpacing: '0.08em' }}>
          SUGGESTED QUOTAS
        </span>
        <span class="stat-sub" style={{ fontFamily: 'var(--font-mono)', fontSize: '11px' }}>
          {suggestions.sample_label}
        </span>
      </div>

      {suggestions.sample_count !== suggestions.population_count && (
        <div class="stat-sub" style={{ fontStyle: 'italic' }}>
          Derived from {suggestions.population_count} completed blocks, biased toward near-limit history.
        </div>
      )}

      {data.token_limit != null && (
        <div
          style={{
            display: 'flex',
            justifyContent: 'space-between',
            gap: '12px',
            alignItems: 'baseline',
          }}
        >
          <span class="stat-sub">Configured</span>
          <span class="stat-sub" style={{ fontFamily: 'var(--font-mono)', fontSize: '11px' }}>
            {fmtTokens(data.token_limit)}
          </span>
        </div>
      )}

      <div style={{ display: 'grid', gap: '6px' }}>
        {suggestions.levels.map(level => (
          <div
            key={level.key}
            style={{
              display: 'flex',
              justifyContent: 'space-between',
              gap: '12px',
              alignItems: 'baseline',
            }}
          >
            <span class="stat-sub">
              {level.label}
              {level.key === suggestions.recommended_key && (
                <span style={{ marginLeft: '6px', color: 'var(--success)' }}>[RECOMMENDED]</span>
              )}
            </span>
            <span class="stat-sub" style={{ fontFamily: 'var(--font-mono)', fontSize: '11px' }}>
              {fmtTokens(level.limit_tokens)}
            </span>
          </div>
        ))}
      </div>

      {suggestions.note && (
        <div class="stat-sub" style={{ fontStyle: 'italic' }}>
          {suggestions.note}
        </div>
      )}
    </div>
  );
}

// ── Sub-components ───────────────────────────────────────────────────

interface QuotaSectionProps {
  block: BillingBlockView;
}

function QuotaSection({ block }: QuotaSectionProps) {
  const { quota } = block;

  if (!quota) {
    return (
      <div
        class="stat-sub"
        style={{ marginTop: '8px', fontStyle: 'italic' }}
      >
        Token quota not configured — set [blocks.token_limit] in config.
      </div>
    );
  }

  const currentPct = Math.min(100, quota.current_pct).toFixed(0);
  const projectedPct = Math.min(999, quota.projected_pct).toFixed(0);

  return (
    <div style={{ marginTop: '10px' }}>
      {/* USED bar */}
      <div style={{ marginBottom: '4px' }}>
        <div
          style={{
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'baseline',
            marginBottom: '3px',
          }}
        >
          <span class="stat-sub" style={{ fontSize: '10px', letterSpacing: '0.08em' }}>
            USED
          </span>
          <span
            class="stat-sub"
            style={{ fontFamily: 'var(--font-mono)', fontSize: '11px' }}
          >
            {fmtTokens(quota.used_tokens)} / {fmtTokens(quota.limit_tokens)}{' '}
            {currentPct}%{' '}
            <span
              style={{
                color:
                  quota.current_severity === 'danger'
                    ? 'var(--accent)'
                    : quota.current_severity === 'warn'
                    ? 'var(--warning)'
                    : undefined,
              }}
            >
              {severityLabel(quota.current_severity)}
            </span>
          </span>
        </div>
        <SegmentedProgressBar
          value={quota.used_tokens}
          max={quota.limit_tokens}
          status={severityToStatus(quota.current_severity)}
          aria-label="Token quota used"
        />
      </div>

      {/* 1px gap then PROJECTED bar */}
      <div style={{ marginTop: '1px' }}>
        <div
          style={{
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'baseline',
            marginBottom: '3px',
          }}
        >
          <span class="stat-sub" style={{ fontSize: '10px', letterSpacing: '0.08em' }}>
            PROJECTED
          </span>
          <span
            class="stat-sub"
            style={{ fontFamily: 'var(--font-mono)', fontSize: '11px' }}
          >
            {fmtTokens(quota.projected_tokens)} / {fmtTokens(quota.limit_tokens)}{' '}
            {projectedPct}%{' '}
            <span
              style={{
                color:
                  quota.projected_severity === 'danger'
                    ? 'var(--accent)'
                    : quota.projected_severity === 'warn'
                    ? 'var(--warning)'
                    : undefined,
              }}
            >
              {severityLabel(quota.projected_severity)}
            </span>
          </span>
        </div>
        <SegmentedProgressBar
          value={quota.projected_tokens}
          max={quota.limit_tokens}
          status={severityToStatus(quota.projected_severity)}
          aria-label="Projected token quota"
        />
      </div>
    </div>
  );
}

// ── Main component ───────────────────────────────────────────────────

interface BillingBlocksCardProps {
  data: BillingBlocksResponse;
}

export function BillingBlocksCard({ data }: BillingBlocksCardProps) {
  const activeBlock = data.blocks.find(b => b.is_active) ?? null;

  if (!activeBlock) {
    // No active block — show historical summary
    return (
      <div class="card stat-card">
        <div class="stat-content">
          <div class="stat-label" style={{ letterSpacing: '0.08em', fontSize: '11px' }}>
            BILLING BLOCK
          </div>
          <div class="stat-value" style={{ opacity: 0.4 }}>
            NO ACTIVE BLOCK
          </div>
          <div class="stat-sub">
            7d historical max:{' '}
            <span style={{ fontFamily: 'var(--font-mono)' }}>
              {fmtTokens(data.historical_max_tokens)}
            </span>{' '}
            tokens
          </div>
          <QuotaSuggestionsSection data={data} />
        </div>
      </div>
    );
  }

  const totalTokens =
    activeBlock.tokens.input +
    activeBlock.tokens.output +
    activeBlock.tokens.cache_read +
    activeBlock.tokens.cache_creation +
    activeBlock.tokens.reasoning_output;

  const elapsed = formatDuration(activeBlock.first_timestamp, activeBlock.last_timestamp);
  const blockEnd = fmtUtcTime(activeBlock.end);

  return (
    <div class="card stat-card">
      <div class="stat-content">
        <div class="stat-label" style={{ letterSpacing: '0.08em', fontSize: '11px' }}>
          BILLING BLOCK
        </div>
        <div
          class="stat-value"
          style={{ fontFamily: 'var(--font-mono)', letterSpacing: '-0.02em' }}
        >
          {fmtTokens(totalTokens)}
        </div>
        <div class="stat-sub">
          {elapsed} elapsed &middot; ends {blockEnd} &middot; {activeBlock.entry_count} entries
        </div>
        {activeBlock.burn_rate && (
          <div class="stat-sub" style={{ fontFamily: 'var(--font-mono)', fontSize: '12px', marginTop: '4px' }}>
            ${(activeBlock.burn_rate.cost_per_hour_nanos / 1e9).toFixed(4)}/hr
            {activeBlock.burn_rate.tier && (
              <span
                style={{
                  marginLeft: '6px',
                  color: tierColor(activeBlock.burn_rate.tier),
                  fontSize: '11px',
                  letterSpacing: '0.04em',
                }}
              >
                {tierLabel(activeBlock.burn_rate.tier)}
              </span>
            )}
          </div>
        )}
        {activeBlock.projection && (
          <div class="stat-sub" style={{ fontFamily: 'var(--font-mono)', fontSize: '12px', marginTop: '4px' }}>
            Projects {fmtTokens(activeBlock.projection.projected_tokens)} tokens · $
            {(activeBlock.projection.projected_cost_nanos / 1e9).toFixed(4)}
          </div>
        )}
      </div>
      <QuotaSection block={activeBlock} />
      <QuotaSuggestionsSection data={data} />
    </div>
  );
}
