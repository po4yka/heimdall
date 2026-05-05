import { useMemo, useState } from 'preact/hooks';
import type {
  ChangelogEntry,
  RateWindowHistoryRow,
} from '../state/dashboard-types';
import type { ApexOptions } from '../lib/apex';
import { ApexChart } from './charts/ApexChart';
import { resolveCssVar } from '../lib/colors';

interface Props {
  history: RateWindowHistoryRow[];
  changelog: ChangelogEntry[];
}

type ProviderFilter = 'all' | 'claude' | 'codex';

const WINDOW_LABELS: Record<string, string> = {
  five_hour: 'Claude · 5h',
  seven_day: 'Claude · weekly',
  seven_day_opus: 'Claude · weekly Opus',
  seven_day_sonnet: 'Claude · weekly Sonnet',
  codex_primary: 'Codex · primary',
  codex_secondary: 'Codex · secondary',
};

// Monochrome category differentiation per industrial-design skill: never
// colour-encode series. ApexCharts can't accept rgba() expressions with CSS
// vars at runtime, so we differentiate via `stroke.dashArray` patterns
// instead — solid, short, medium, long — cycling through the 6 series.
const DASH_LADDER = [0, 3, 6, 9, 12, 15];

function inferProvider(windowType: string): 'claude' | 'codex' {
  return windowType.startsWith('codex_') ? 'codex' : 'claude';
}

function buildOptions(
  history: RateWindowHistoryRow[],
  changelog: ChangelogEntry[],
  provider: ProviderFilter,
): ApexOptions | null {
  const filtered = history.filter(row => {
    if (row.estimated_cap_tokens == null) return false;
    if (provider === 'all') return true;
    return inferProvider(row.window_type) === provider;
  });
  if (filtered.length === 0) return null;

  // Group rows into series keyed by window_type.
  const seriesMap = new Map<string, Array<{ x: number; y: number }>>();
  for (const row of filtered) {
    if (row.estimated_cap_tokens == null) continue;
    const ts = Date.parse(row.timestamp);
    if (Number.isNaN(ts)) continue;
    let arr = seriesMap.get(row.window_type);
    if (!arr) {
      arr = [];
      seriesMap.set(row.window_type, arr);
    }
    arr.push({ x: ts, y: row.estimated_cap_tokens });
  }
  if (seriesMap.size === 0) return null;

  const seriesKeys = Array.from(seriesMap.keys()).sort();
  const series = seriesKeys.map(key => ({
    name: WINDOW_LABELS[key] ?? key,
    data: (seriesMap.get(key) ?? []).sort((a, b) => a.x - b.x),
  }));
  const dashArray = seriesKeys.map(
    (_, i) => DASH_LADDER[i % DASH_LADDER.length] ?? 0,
  );

  // Resolve CSS variables to concrete colours so ApexCharts can paint them
  // into SVG attributes that don't accept `var(...)` expressions (legend
  // marker fills, annotation markers, axis labels rendered to canvas).
  const textPrimary = resolveCssVar('--text-primary', '#0a0a0a');
  const textSecondary = resolveCssVar('--text-secondary', '#666666');
  const borderColor = resolveCssVar('--border', '#e0e0e0');
  const annotationFill = textPrimary;
  const annotationStroke = resolveCssVar('--bg', '#ffffff');
  const annotationLabelBg = resolveCssVar('--surface-elevated', '#ffffff');

  // Build changelog points (markers on x-axis).
  const annotationPoints = changelog
    .filter(entry => provider === 'all' || entry.provider === provider)
    .map(entry => ({
      x: Date.parse(`${entry.date}T12:00:00Z`),
      y: null,
      marker: {
        size: 4,
        fillColor: annotationFill,
        strokeColor: annotationStroke,
        radius: 0,
      },
      label: {
        text: entry.title,
        style: {
          color: annotationFill,
          background: annotationLabelBg,
          fontFamily: 'var(--font-mono)',
          fontSize: '10px',
        },
      },
    }))
    .filter(p => Number.isFinite(p.x as number));

  const opts: ApexOptions = {
    chart: {
      type: 'line',
      toolbar: { show: false },
      animations: { enabled: false },
      fontFamily: 'var(--font-mono)',
      background: 'transparent',
    },
    // No `theme: { mode: 'dark' }` — the chart inherits the surrounding card
    // via transparent background + CSS-variable colours, so it works in both
    // light and dark dashboard themes.
    series,
    colors: series.map(() => textPrimary),
    stroke: {
      width: 2,
      curve: 'smooth',
      dashArray,
    },
    fill: { type: 'solid', opacity: 0.0 },
    grid: {
      borderColor,
      strokeDashArray: 2,
      xaxis: { lines: { show: false } },
      yaxis: { lines: { show: true } },
    },
    legend: {
      position: 'top',
      labels: { colors: textPrimary, fontFamily: 'var(--font-mono)' },
      itemMargin: { horizontal: 12, vertical: 4 },
      markers: { width: 12, height: 12 },
    },
    xaxis: {
      type: 'datetime',
      labels: {
        style: { colors: textSecondary, fontFamily: 'var(--font-mono)', fontSize: '11px' },
      },
      axisBorder: { show: false },
      axisTicks: { show: false },
    },
    yaxis: {
      labels: {
        style: { colors: textSecondary, fontFamily: 'var(--font-mono)', fontSize: '11px' },
        formatter: (val: number) => {
          if (!Number.isFinite(val)) return '';
          if (val >= 1e9) return `${(val / 1e9).toFixed(2)}B`;
          if (val >= 1e6) return `${(val / 1e6).toFixed(2)}M`;
          if (val >= 1e3) return `${(val / 1e3).toFixed(0)}K`;
          return String(val);
        },
      },
    },
    tooltip: {
      style: { fontFamily: 'var(--font-mono)', fontSize: '11px' },
      y: {
        formatter: (val: number) =>
          Number.isFinite(val) ? `${val.toLocaleString('en-US')} tokens` : '—',
      },
    },
    markers: { size: 0, strokeWidth: 0, hover: { size: 4 } },
    dataLabels: { enabled: false },
  };
  if (annotationPoints.length > 0) {
    opts.annotations = { points: annotationPoints };
  }
  return opts;
}

export function SubscriptionHistoryChart({ history, changelog }: Props) {
  const [provider, setProvider] = useState<ProviderFilter>('all');
  const options = useMemo(
    () => buildOptions(history, changelog, provider),
    [history, changelog, provider],
  );

  return (
    <div class="card subscription-history-card">
      <div class="subscription-history-header">
        <div class="subscription-history-title">Subscription cap history</div>
        <div class="subscription-history-filter">
          {(['all', 'claude', 'codex'] as ProviderFilter[]).map(p => (
            <button
              key={p}
              type="button"
              class={`chip${provider === p ? ' chip-active' : ''}`}
              onClick={() => setProvider(p)}
            >
              {p === 'all' ? 'All' : p === 'claude' ? 'Claude' : 'Codex'}
            </button>
          ))}
        </div>
      </div>
      <div class="subscription-history-body">
        {options ? (
          <div class="chart-wrap tall">
            <ApexChart options={options} id="subscription-history-chart" />
          </div>
        ) : (
          <div class="subscription-quota-empty">
            No historical observations yet — caps will appear once snapshots accumulate.
          </div>
        )}
      </div>
      {changelog.length > 0 && (
        <ul class="subscription-history-changelog">
          {changelog.map(entry => (
            <li key={`${entry.date}-${entry.provider}-${entry.kind}`}>
              <span class="subscription-history-date">{entry.date}</span>
              <span class="subscription-history-provider">{entry.provider}</span>
              <a class="subscription-history-link" href={entry.source_url} target="_blank" rel="noreferrer">
                {entry.title}
              </a>
              <span class="subscription-history-desc">{entry.description}</span>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
