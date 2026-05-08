import { rawData } from '../state/store';
import type { CostForecastSummary, CostTrend, DailyCostPoint } from '../state/dashboard-types';
import { fmtCostBig } from '../lib/format';

const NANOS_PER_USD = 1_000_000_000;

function nanosToUsd(nanos: number): number {
  return nanos / NANOS_PER_USD;
}

function trendLabel(t: CostTrend): string {
  if (t === 'rising') return '[RISING]';
  if (t === 'falling') return '[FALLING]';
  if (t === 'flat') return '[FLAT]';
  return '';
}

function trendColor(t: CostTrend): string {
  if (t === 'rising') return 'var(--accent,#D71921)';
  if (t === 'falling') return 'var(--success,#4caf50)';
  return 'var(--text-secondary)';
}

function KpiTile({ label, value, color }: { label: string; value: string; color?: string }) {
  return (
    <div>
      <div class="stat-label" style={{ fontSize: '10px' }}>{label}</div>
      <div style={{ fontFamily: 'var(--font-mono)', fontSize: '18px', color: color ?? 'var(--text-primary)' }}>
        {value}
      </div>
    </div>
  );
}

function DayBar({ point, maxNanos }: { point: DailyCostPoint; maxNanos: number }) {
  const frac = maxNanos > 0 ? Math.min(1, point.cost_nanos / maxNanos) : 0;
  const label = point.day.slice(5); // MM-DD
  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        flex: '1',
        minWidth: '0',
        gap: '2px',
      }}
    >
      <div
        style={{
          width: '100%',
          height: '40px',
          display: 'flex',
          alignItems: 'flex-end',
        }}
      >
        <div
          style={{
            width: '100%',
            height: `${Math.max(2, frac * 40)}px`,
            background: 'var(--text-primary)',
            opacity: point.cost_nanos > 0 ? 0.55 : 0.10,
            borderRadius: '1px 1px 0 0',
          }}
        />
      </div>
      {label.endsWith('-01') || label === rawData.value?.daily_by_model?.[0]?.day?.slice(5) ? (
        <div style={{ fontSize: '8px', color: 'var(--text-secondary)', fontFamily: 'var(--font-mono)', whiteSpace: 'nowrap' }}>
          {label}
        </div>
      ) : (
        <div style={{ height: '10px' }} />
      )}
    </div>
  );
}

function CostForecastCardInner({ summary }: { summary: CostForecastSummary }) {
  const burn7 = fmtCostBig(nanosToUsd(summary.rolling_7d_avg_nanos));
  const burn30 = fmtCostBig(nanosToUsd(summary.rolling_30d_avg_nanos));
  const projected = fmtCostBig(nanosToUsd(summary.projected_month_nanos));
  const maxNanos = Math.max(...summary.days.map((d) => d.cost_nanos), 1);
  const showTrend = summary.trend !== 'insufficient';

  return (
    <div class="card" style={{ padding: '16px' }}>
      <div class="stat-label" style={{ marginBottom: '10px' }}>Cost forecast</div>

      <div style={{ display: 'flex', gap: '20px', flexWrap: 'wrap', marginBottom: '14px', alignItems: 'flex-end' }}>
        <KpiTile label="7-day burn / day" value={burn7} />
        <KpiTile label="30-day burn / day" value={burn30} />
        <div>
          <div class="stat-label" style={{ fontSize: '10px' }}>Projected month</div>
          <div style={{ display: 'flex', alignItems: 'baseline', gap: '6px' }}>
            <div style={{ fontFamily: 'var(--font-mono)', fontSize: '18px' }}>{projected}</div>
            {showTrend && (
              <div
                style={{
                  fontFamily: 'var(--font-mono)',
                  fontSize: '10px',
                  color: trendColor(summary.trend),
                }}
              >
                {trendLabel(summary.trend)}
              </div>
            )}
          </div>
        </div>
        {summary.regression && (
          <div>
            <div class="stat-label" style={{ fontSize: '10px' }}>R²</div>
            <div style={{ fontFamily: 'var(--font-mono)', fontSize: '18px' }}>
              {summary.regression.r_squared.toFixed(2)}
            </div>
          </div>
        )}
      </div>

      {summary.trend === 'insufficient' && (
        <div
          style={{
            fontFamily: 'var(--font-mono)',
            fontSize: '10px',
            color: 'var(--text-secondary)',
            marginBottom: '10px',
          }}
        >
          Need ≥7 days of activity for regression — showing rolling average.
        </div>
      )}

      <div style={{ display: 'flex', gap: '2px', alignItems: 'flex-end', height: '52px' }}>
        {summary.days.map((d) => (
          <DayBar key={d.day} point={d} maxNanos={maxNanos} />
        ))}
      </div>
    </div>
  );
}

export function CostForecastCard() {
  const data = rawData.value;

  if (!data) {
    return (
      <div class="card" style={{ padding: '16px' }}>
        <div class="stat-label">Cost forecast</div>
        <div style={{ color: 'var(--text-secondary)', fontFamily: 'var(--font-mono)', fontSize: '12px', marginTop: '8px' }}>
          loading...
        </div>
      </div>
    );
  }

  const summary = data.cost_forecast;
  if (!summary) {
    return (
      <div class="card" style={{ padding: '16px' }}>
        <div class="stat-label">Cost forecast</div>
        <div style={{ color: 'var(--text-secondary)', fontFamily: 'var(--font-mono)', fontSize: '12px', marginTop: '8px' }}>
          No data available.
        </div>
      </div>
    );
  }

  return <CostForecastCardInner summary={summary} />;
}
