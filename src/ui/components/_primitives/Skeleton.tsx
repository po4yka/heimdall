import type { ComponentChildren } from 'preact';

interface SkeletonProps {
  /** CSS width (e.g. '60%', '120px'). Defaults to 100%. */
  width?: string;
  /** CSS height (e.g. '12px', 'var(--font-size-body)'). Defaults to a body line. */
  height?: string;
  /** Border radius — accepts a token or px. Defaults to --radius-1. */
  radius?: string;
  className?: string;
  ariaLabel?: string;
}

/**
 * Single shimmer block. Every other skeleton variant in this file
 * composes this one. The class drives a single CSS keyframe; respects
 * prefers-reduced-motion (animation drops to a flat tint).
 */
export function Skeleton({
  width = '100%',
  height = 'var(--font-size-body)',
  radius = 'var(--radius-1)',
  className,
  ariaLabel,
}: SkeletonProps) {
  const cls = ['skeleton', className].filter(Boolean).join(' ');
  return (
    <span
      class={cls}
      role={ariaLabel ? 'status' : undefined}
      aria-label={ariaLabel}
      aria-busy={ariaLabel ? 'true' : undefined}
      style={{ width, height, borderRadius: radius, display: 'block' }}
    />
  );
}

export type KpiSkeletonSize = 'compact' | 'standard' | 'hero';

interface KpiSkeletonProps {
  /** Mirrors KpiCard's size prop so the placeholder line-grid matches. */
  size?: KpiSkeletonSize;
  /** Show a fake progress bar block under the value. */
  withBar?: boolean;
  /** Show the small sub-text line. */
  withSub?: boolean;
}

/**
 * Stat-card-shaped placeholder — a label, a tall value bar, an optional
 * progress-bar block, and an optional sub-text line. Heights mirror the
 * real KpiCard so the swap from skeleton to data does not cause CLS.
 */
export function KpiSkeleton({
  size = 'standard',
  withBar = false,
  withSub = true,
}: KpiSkeletonProps) {
  const valueHeight =
    size === 'hero'
      ? 'var(--font-size-display)'
      : size === 'compact'
        ? 'var(--font-size-value)'
        : 'var(--font-size-display-sm)';

  return (
    <div class={`card stat-card kpi-card kpi-card--${size}`} aria-busy="true">
      <div class="stat-content">
        <Skeleton width="40%" height="var(--font-size-tertiary)" />
        <Skeleton width="60%" height={valueHeight} />
        {withBar && <Skeleton width="100%" height="6px" />}
        {withSub && <Skeleton width="50%" height="var(--font-size-tertiary)" />}
      </div>
    </div>
  );
}

interface ChartSkeletonProps {
  /** Use --chart-h-lg (300px) instead of --chart-h-md (240px). */
  tall?: boolean;
  /** Bar count. Default 12 — enough for a daily-by-week strip. */
  bars?: number;
}

/**
 * Vertical-bar placeholder, tall enough to occupy a chart wrap so
 * the layout doesn't shift when the real ApexChart paints.
 */
export function ChartSkeleton({ tall = false, bars = 12 }: ChartSkeletonProps) {
  // Deterministic heights so the skeleton doesn't strobe between
  // re-renders. Repeats every 5 bars; range 30%–95%.
  const heights = ['55%', '80%', '40%', '95%', '65%', '50%', '75%', '35%', '90%', '60%', '45%', '85%'];
  const height = tall ? 'var(--chart-h-lg)' : 'var(--chart-h-md)';
  return (
    <div
      class="chart-wrap"
      aria-busy="true"
      style={{
        height,
        display: 'flex',
        alignItems: 'flex-end',
        gap: 'var(--space-1)',
        padding: 'var(--space-3)',
      }}
    >
      {Array.from({ length: bars }).map((_, i) => (
        <span
          key={i}
          class="skeleton"
          style={{
            flex: 1,
            height: heights[i % heights.length],
            borderRadius: 'var(--radius-1) var(--radius-1) 0 0',
            display: 'block',
          }}
        />
      ))}
    </div>
  );
}

interface TableSkeletonProps {
  rows?: number;
  columns?: number;
}

/**
 * Table-shaped placeholder. Renders real <table>/<thead>/<tbody> markup
 * so the skeleton inherits the dashboard's td density tokens.
 */
export function TableSkeleton({ rows = 6, columns = 4 }: TableSkeletonProps) {
  return (
    <table aria-busy="true" style={{ width: '100%' }}>
      <thead>
        <tr>
          {Array.from({ length: columns }).map((_, i) => (
            <th key={i}>
              <Skeleton width="60%" height="var(--font-size-tertiary)" />
            </th>
          ))}
        </tr>
      </thead>
      <tbody>
        {Array.from({ length: rows }).map((_, ri) => (
          <tr key={ri}>
            {Array.from({ length: columns }).map((_, ci) => (
              <td key={ci}>
                <Skeleton width={ci === 0 ? '70%' : '50%'} />
              </td>
            ))}
          </tr>
        ))}
      </tbody>
    </table>
  );
}

interface HeatmapSkeletonProps {
  cols?: number;
  rows?: number;
}

/**
 * Heatmap-shaped placeholder. 12px squares, 1px gap — same geometry as
 * the real days-hours / weekday-hour heatmaps.
 */
export function HeatmapSkeleton({ cols = 24, rows = 7 }: HeatmapSkeletonProps) {
  return (
    <div
      aria-busy="true"
      style={{
        display: 'grid',
        gridTemplateColumns: `repeat(${cols}, 1fr)`,
        gap: '1px',
      }}
    >
      {Array.from({ length: cols * rows }).map((_, i) => (
        <span
          key={i}
          class="skeleton"
          style={{ height: '12px', borderRadius: '1px', display: 'block' }}
        />
      ))}
    </div>
  );
}

/**
 * Inline three-dot pulse for in-button or in-status-line use. Cheaper
 * than a full skeleton; uses the same shimmer keyframe via the dot's
 * background.
 */
export function SpinnerInline({
  ariaLabel = 'Loading',
}: { ariaLabel?: string } = {}) {
  return (
    <span class="spinner-inline" role="status" aria-label={ariaLabel} aria-busy="true">
      <span class="spinner-inline__dot" />
      <span class="spinner-inline__dot" />
      <span class="spinner-inline__dot" />
    </span>
  );
}

interface SkeletonGroupProps {
  children: ComponentChildren;
}

/**
 * Convenience wrapper: stacks skeleton lines with consistent spacing
 * for ad-hoc form-row / list-row placeholders.
 */
export function SkeletonGroup({ children }: SkeletonGroupProps) {
  return (
    <div
      aria-busy="true"
      style={{ display: 'flex', flexDirection: 'column', gap: 'var(--space-2)' }}
    >
      {children}
    </div>
  );
}
