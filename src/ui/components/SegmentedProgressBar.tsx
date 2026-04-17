export type SegmentedBarSize = 'hero' | 'standard' | 'compact';
export type SegmentedBarStatus = 'auto' | 'neutral' | 'success' | 'warning' | 'accent';

interface SegmentedProgressBarProps {
  value: number;
  max: number;
  segments?: number;
  size?: SegmentedBarSize;
  status?: SegmentedBarStatus;
  'aria-label'?: string;
}

function resolveStatus(pct: number, status: SegmentedBarStatus): string {
  if (status === 'neutral') return 'var(--text-display)';
  if (status === 'success') return 'var(--success)';
  if (status === 'warning') return 'var(--warning)';
  if (status === 'accent') return 'var(--accent)';
  // auto
  if (pct >= 90) return 'var(--accent)';
  if (pct >= 70) return 'var(--warning)';
  return 'var(--success)';
}

export function SegmentedProgressBar({
  value,
  max,
  segments = 20,
  size = 'standard',
  status = 'auto',
  'aria-label': ariaLabel,
}: SegmentedProgressBarProps) {
  const safeMax = max > 0 ? max : 1;
  const ratio = value / safeMax;
  const pct = Math.min(100, Math.max(0, ratio * 100));
  const overflow = ratio > 1;
  const filled = Math.round((pct / 100) * segments);
  const fillColor = overflow ? 'var(--accent)' : resolveStatus(pct, status);
  const emptyColor = 'var(--border)';

  return (
    <div
      class={`segmented-bar segmented-bar--${size}`}
      role="progressbar"
      aria-label={ariaLabel}
      aria-valuenow={Math.round(pct)}
      aria-valuemin={0}
      aria-valuemax={100}
    >
      {Array.from({ length: segments }).map((_, i) => (
        <div
          key={i}
          class="segmented-bar__segment"
          style={{ background: i < filled ? fillColor : emptyColor }}
        />
      ))}
    </div>
  );
}
