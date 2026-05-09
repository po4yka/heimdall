import { useMemo, useState } from 'preact/hooks';
import type {
  ChangelogEntry,
  RateWindowHistoryRow,
} from '../state/dashboard-types';
import type { ApexOptions } from '../lib/apex';
import { ApexChart } from './charts/ApexChart';
import { resolveCssVar } from '../lib/colors';
import { CHART_CSS_FALLBACKS, withAlpha } from '../lib/charts';

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
// colour-encode series. Three visual dimensions in combination — opacity,
// dash pattern, and stroke weight — make 6 series distinguishable even when
// dense. ApexCharts can't resolve var() in SVG fill attributes, so we bake
// rgba() from the resolved --text-display token at build time via withAlpha.
const OPACITY_LADDER = [1.0, 0.68, 0.46, 0.30, 0.20, 0.14];
const DASH_LADDER    = [0, 4, 8, 4, 8, 12];
const WIDTH_LADDER   = [2.5, 2.0, 2.0, 1.5, 1.5, 1.5];
const MARKER_LADDER  = [4, 3, 3, 2, 2, 2];

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
  const seriesColors = seriesKeys.map(
    (_, i) => withAlpha('--text-display', OPACITY_LADDER[i % OPACITY_LADDER.length] ?? 1.0),
  );
  const dashArray = seriesKeys.map(
    (_, i) => DASH_LADDER[i % DASH_LADDER.length] ?? 0,
  );
  const strokeWidths = seriesKeys.map(
    (_, i) => WIDTH_LADDER[i % WIDTH_LADDER.length] ?? 2,
  );
  const markerSizes = seriesKeys.map(
    (_, i) => MARKER_LADDER[i % MARKER_LADDER.length] ?? 3,
  );

  // Resolve CSS variables to concrete colours so ApexCharts can paint them
  // into SVG attributes that don't accept `var(...)` expressions (legend
  // marker fills, annotation markers, axis labels rendered to canvas).
  const textSecondary = resolveCssVar('--text-secondary', CHART_CSS_FALLBACKS['--text-secondary']!);
  const borderColor = resolveCssVar('--border', CHART_CSS_FALLBACKS['--border']!);

  // Changelog events as vertical dashed lines — no text labels since the
  // list below the chart already shows the full description. This avoids
  // the overlapping-label banner produced by points annotations.
  const annotationsX = changelog
    .filter(entry => provider === 'all' || entry.provider === provider)
    .map(entry => ({
      x: Date.parse(`${entry.date}T12:00:00Z`),
      borderColor: textSecondary,
      strokeDashArray: 3,
    }))
    .filter(a => Number.isFinite(a.x));

  const seriesXValues: number[] = [];
  for (const s of series) {
    for (const p of s.data) seriesXValues.push(p.x);
  }
  // Also include changelog dates so annotations stay on-canvas even when
  // local history is shorter than the full policy timeline.
  const annotationXValues = annotationsX.map(a => a.x);
  const allX = [...seriesXValues, ...annotationXValues];
  const xMin = allX.length ? Math.min(...allX) : undefined;
  const xMax = allX.length ? Math.max(...allX, Date.now()) : undefined;

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
    colors: seriesColors,
    stroke: {
      width: strokeWidths,
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
      labels: { colors: textSecondary, fontFamily: 'var(--font-mono)' },
      itemMargin: { horizontal: 12, vertical: 4 },
      markers: { width: 20, height: 2, radius: 0 },
    },
    xaxis: {
      type: 'datetime',
      ...(xMin !== undefined ? { min: xMin } : {}),
      ...(xMax !== undefined ? { max: xMax } : {}),
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
    // Per-series marker sizes reinforce the opacity ladder: prominent series
    // get larger dots, faint series get smaller ones. This ensures a single
    // observation (common for short history) is still visible per series.
    markers: { size: markerSizes, strokeWidth: 0, hover: { size: 6 } },
    dataLabels: { enabled: false },
  };
  if (annotationsX.length > 0) {
    opts.annotations = { xaxis: annotationsX };
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
