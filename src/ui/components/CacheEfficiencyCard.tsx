import type { CacheEfficiency } from '../state/types';

interface CacheEfficiencyCardProps {
  data: CacheEfficiency;
  /** Input rate in $/MTok for the dominant model (used for savings estimate).
   *  When undefined, the tooltip omits the savings line. */
  inputRatePerMtok?: number;
  /** Cache-read rate in $/MTok for the dominant model. */
  cacheReadRatePerMtok?: number;
}

/**
 * Cache Hit Rate card.
 *
 * Shows the percentage as a large Geist Mono display number and a horizontal
 * monochrome progress bar below it.
 *
 * Formula: cache_read / (cache_read + cache_write + input_tokens)
 *
 * Anthropic reports `input_tokens` as the uncached remainder only (typically
 * just the latest user message), so the narrow ratio `cr / (cr + in)` rounds
 * to ~100% for heavy Claude Code users. Including cache writes in the
 * denominator reflects the fraction of input-side tokens actually served
 * from cache and reads meaningfully between 0% and 100%.
 *
 * Displays "--" when cache_hit_rate is null (no addressable token stream)
 * and "Fully cached" when the rate is ≥ 99.9% (legit but misleading if shown
 * as 100.0%).
 */
export function CacheEfficiencyCard({
  data,
  inputRatePerMtok,
  cacheReadRatePerMtok,
}: CacheEfficiencyCardProps) {
  const rate = data.cache_hit_rate;
  const hasRate = rate !== null && rate !== undefined;

  const displayPct = hasRate
    ? rate! >= 0.999
      ? 'Fully cached'
      : (rate! * 100).toFixed(1) + '%'
    : '--';
  const barFill = hasRate ? Math.max(0, Math.min(1, rate!)) : 0;

  // Tooltip content
  const tooltipParts: string[] = [];
  if (hasRate) {
    const readM = (data.cache_read_tokens / 1_000_000).toFixed(2);
    const totalM = (
      (data.cache_read_tokens + data.cache_write_tokens + data.input_tokens) /
      1_000_000
    ).toFixed(2);
    tooltipParts.push(
      `${readM}M cache reads / ${totalM}M total input-side tokens (cache reads + cache writes + fresh input)`
    );

    if (
      inputRatePerMtok !== undefined &&
      cacheReadRatePerMtok !== undefined &&
      data.cache_read_tokens > 0
    ) {
      const savedUsd =
        (data.cache_read_tokens / 1_000_000) * (inputRatePerMtok - cacheReadRatePerMtok);
      tooltipParts.push(
        `saved approx $${savedUsd.toFixed(2)} vs. no-cache`
      );
    }
  } else {
    tooltipParts.push('No cache activity recorded');
  }
  const tooltip = tooltipParts.join(' \u00b7 ');

  return (
    <div class="card stat-card" title={tooltip}>
      <div class="stat-content">
        <div class="stat-label">Cache hit rate</div>
        <div
          class="stat-value"
          style={{ fontFamily: 'var(--font-mono)', letterSpacing: '-0.02em' }}
        >
          {displayPct}
        </div>
        <div class="stat-sub">prompt cache reuse</div>
      </div>
      {/* Monochrome horizontal progress bar */}
      <div
        role="img"
        style={{
          marginTop: '10px',
          height: '4px',
          borderRadius: '2px',
          background: 'rgba(var(--text-primary-rgb, 232,232,232), 0.12)',
          overflow: 'hidden',
        }}
        aria-label={`Cache hit rate: ${displayPct}`}
      >
        <div
          style={{
            height: '100%',
            width: `${(barFill * 100).toFixed(2)}%`,
            background: 'rgba(var(--text-primary-rgb, 232,232,232), 0.70)',
            borderRadius: '2px',
            transition: 'width 300ms cubic-bezier(0.25,0.1,0.25,1)',
          }}
        />
      </div>
    </div>
  );
}
