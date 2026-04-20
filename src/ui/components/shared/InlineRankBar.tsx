export function InlineRankBar({
  value,
  max,
  label,
}: {
  value: number;
  max: number;
  label: string;
}) {
  const pct = max > 0 ? (value / max) * 100 : 0;
  const tooltip = `${value} (${pct.toFixed(1)}% of max ${max})`;

  return (
    <span
      style={{ position: 'relative', display: 'inline-block', width: '100%' }}
      title={tooltip}
    >
      <span
        data-testid="rank-bar"
        style={{
          position: 'absolute',
          top: 0,
          left: 0,
          bottom: 0,
          width: `${pct}%`,
          backgroundColor: 'var(--color-text-primary)',
          opacity: 0.12,
          pointerEvents: 'none',
        }}
      />
      <span class="num" style={{ position: 'relative', zIndex: 1 }}>
        {label}
      </span>
    </span>
  );
}
