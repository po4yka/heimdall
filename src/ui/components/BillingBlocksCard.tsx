import { SegmentedProgressBar } from './SegmentedProgressBar';
import type { BillingBlocksResponse, BillingBlockView, QuotaSeverity } from '../state/types';
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
        style={{ marginTop: '8px', fontStyle: 'italic', opacity: 0.6 }}
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
      </div>
      <QuotaSection block={activeBlock} />
    </div>
  );
}
