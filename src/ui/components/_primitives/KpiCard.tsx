import type { ComponentChildren } from 'preact';
import { SegmentedProgressBar, type SegmentedBarSize, type SegmentedBarStatus } from '../SegmentedProgressBar';

export type KpiValueTone = 'default' | 'cost' | 'success' | 'warning' | 'accent' | 'muted';
export type KpiSize = 'compact' | 'standard' | 'hero';

interface KpiCardBarProps {
  value: number;
  max?: number;
  size?: SegmentedBarSize;
  status?: SegmentedBarStatus;
  ariaLabel?: string;
}

interface KpiCardProps {
  label: string;
  value: ComponentChildren;
  /** Optional secondary text rendered below the bar / value. */
  sub?: ComponentChildren;
  /** Optional progress bar. */
  bar?: KpiCardBarProps;
  /** Visual tone of the value. */
  valueTone?: KpiValueTone;
  /** Density variant. `compact` shrinks the value to --font-size-value (20px). */
  size?: KpiSize;
  /** Optional right-rail content: actions, trend pill, sparkline. */
  actions?: ComponentChildren;
  /** Extra class on the card root. */
  className?: string;
}

const TONE_CLASS: Record<KpiValueTone, string> = {
  default: '',
  cost: 'cost-value',
  success: 'kpi-value--success',
  warning: 'kpi-value--warning',
  accent: 'kpi-value--accent',
  muted: 'kpi-value--muted',
};

export function KpiCard({
  label,
  value,
  sub,
  bar,
  valueTone = 'default',
  size = 'standard',
  actions,
  className,
}: KpiCardProps) {
  const cardClass = ['card', 'stat-card', 'kpi-card', `kpi-card--${size}`, className]
    .filter(Boolean)
    .join(' ');
  const valueClass = ['stat-value', TONE_CLASS[valueTone]].filter(Boolean).join(' ');

  return (
    <div class={cardClass}>
      <div class="stat-content">
        <div class="stat-label">{label}</div>
        <div class={valueClass}>{value}</div>
        {bar && (
          <div class="kpi-card__bar">
            <SegmentedProgressBar
              value={bar.value}
              max={bar.max ?? 100}
              size={bar.size ?? 'standard'}
              status={bar.status ?? 'auto'}
              aria-label={bar.ariaLabel ?? label}
            />
          </div>
        )}
        {sub && <div class="stat-sub">{sub}</div>}
      </div>
      {actions && <div class="kpi-card__actions">{actions}</div>}
    </div>
  );
}
