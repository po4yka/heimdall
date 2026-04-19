import { ApexChart } from './ApexChart';
import { industrialChartOptions, cssVar, withAlpha } from '../lib/charts';
import { fmt, truncateMid } from '../lib/format';
import type { ModelAgg } from '../state/types';

export function ModelChart({ byModel }: { byModel: ModelAgg[] }) {
  if (!byModel.length) return null;

  // Collapse the long tail into an "Other" slice so the legend stays short
  // enough to leave room for the donut in a 240px-tall chart card.
  const sorted = [...byModel].sort((a, b) => (b.input + b.output) - (a.input + a.output));
  const TOP_N = 4;
  const top = sorted.slice(0, TOP_N);
  const rest = sorted.slice(TOP_N);
  const series = top.map(m => m.input + m.output);
  const labels = top.map(m => m.model);
  if (rest.length > 0) {
    const otherTotal = rest.reduce((s, m) => s + m.input + m.output, 0);
    if (otherTotal > 0) {
      series.push(otherTotal);
      labels.push(`Other (${rest.length})`);
    }
  }

  // Token-opacity ladder keeps the donut monochrome (industrial canvas
  // rule) while giving every slice a distinguishable grey. The previous
  // categorical palette collapsed to near-pure-black when one model
  // dominated ~80% of tokens because slice #0 used --text-display raw.
  const OPACITY_LADDER = [1.0, 0.55, 0.4, 0.28, 0.18];
  const sliceColors = labels.map((_, i) =>
    withAlpha('--text-display', OPACITY_LADDER[Math.min(i, OPACITY_LADDER.length - 1)] ?? 0.18),
  );

  const base = industrialChartOptions('donut');
  const options = {
    ...base,
    chart: { ...base.chart, type: 'donut' },
    series,
    labels,
    colors: sliceColors,
    stroke: { width: 2, colors: [cssVar('--surface')] },
    // Filter-based hover cue preserves the donut's colour palette. Without
    // this, ApexCharts swaps the total label's colour to the hovered slice
    // (see apexcharts/apexcharts.js#3264). The filter is deliberately
    // gentle — on dark themes the low-opacity tail slices are near-black
    // and a stronger lighten produces distracting white flashes.
    states: { hover: { filter: { type: 'lighten', value: 0.06 } } },
    legend: {
      ...base.legend,
      itemMargin: { horizontal: 10, vertical: 2 },
      onItemHover: { highlightDataSeries: false },
      formatter: (label: string) => truncateMid(label, 18, 6),
    },
    // Anchor the tooltip below the ring via bottomLeft so it never covers
    // the card's "BY MODEL" title. bottomRight would push the tooltip past
    // the card's right edge where overflow:hidden would clip it.
    tooltip: {
      ...base.tooltip,
      fixed: { enabled: true, position: 'bottomLeft', offsetX: 0, offsetY: 0 },
      y: { formatter: (v: number) => fmt(v) + ' tokens' },
    },
    plotOptions: {
      pie: {
        donut: {
          size: '64%',
          labels: {
            show: true,
            total: {
              show: true,
              // Keep the resting TOTAL visible while a slice is hovered.
              showAlways: true,
              label: 'TOTAL',
              fontFamily: 'var(--font-mono), "Space Mono", monospace',
              fontSize: '11px',
              color: cssVar('--text-secondary'),
              formatter: (w: any) =>
                fmt(w.globals.seriesTotals.reduce((a: number, b: number) => a + b, 0)),
            },
            value: {
              fontFamily: 'var(--font-mono), "Space Mono", monospace',
              fontSize: '20px',
              color: cssVar('--text-display'),
              formatter: (val: string) => fmt(Number(val)),
            },
            name: {
              fontFamily: 'var(--font-mono), "Space Mono", monospace',
              fontSize: '11px',
              color: cssVar('--text-display'),
            },
          },
        },
      },
    },
  };

  return <ApexChart options={options} id="chart-model" />;
}
