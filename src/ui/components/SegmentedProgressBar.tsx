export type SegmentedBarSize = 'hero' | 'standard' | 'compact';
export type SegmentedBarStatus = 'auto' | 'neutral' | 'success' | 'warning' | 'accent';

interface SegmentedProgressBarProps {
  value: number;
  max: number;
  /** Retained for API compatibility; the bar is now smooth and ignores segmentation. */
  segments?: number;
  size?: SegmentedBarSize;
  status?: SegmentedBarStatus;
  'aria-label'?: string;
}

function resolveStatus(pct: number, status: SegmentedBarStatus): string {
  if (status === 'neutral') return 'var(--accent-interactive)';
  if (status === 'success') return 'var(--success)';
  if (status === 'warning') return 'var(--warning)';
  if (status === 'accent') return 'var(--accent)';
  // auto — threshold-encoded
  if (pct >= 90) return 'var(--accent)';
  if (pct >= 70) return 'var(--warning)';
  return 'var(--success)';
}

/**
 * Smooth single-fill pill progress bar. Color encodes threshold status.
 *
 * Migrated from the prior segmented-LED geometry to a continuous rounded
 * fill. Threshold semantics (<70% success / 70-90% warning / >=90% accent)
 * and the overflow-reads-red rule are preserved; only the visual form
 * changed. The `segments` prop is retained for call-site compatibility
 * during the design migration but no longer influences rendering.
 */
export function SegmentedProgressBar({
  value,
  max,
  segments: _segments,
  size = 'standard',
  status = 'auto',
  'aria-label': ariaLabel,
}: SegmentedProgressBarProps) {
  const safeMax = max > 0 ? max : 1;
  const ratio = value / safeMax;
  const pct = Math.min(100, Math.max(0, ratio * 100));
  const overflow = ratio > 1;
  const fillColor = overflow ? 'var(--accent)' : resolveStatus(pct, status);

  return (
    <div
      class={`segmented-bar segmented-bar--${size}`}
      role="progressbar"
      aria-label={ariaLabel}
      aria-valuenow={Math.round(pct)}
      aria-valuemin={0}
      aria-valuemax={100}
    >
      <div
        class="segmented-bar__fill"
        style={{ width: `${pct}%`, background: fillColor, minWidth: pct > 0 ? '8px' : '0' }}
      />
    </div>
  );
}
