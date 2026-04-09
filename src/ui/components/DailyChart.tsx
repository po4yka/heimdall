import { ApexChart } from './ApexChart';
import { TOKEN_COLORS, RANGE_TICKS, apexThemeMode, cssVar } from '../lib/charts';
import { fmt } from '../lib/format';
import { selectedRange } from '../state/store';
import type { DailyAgg } from '../state/types';

export function DailyChart({ daily }: { daily: DailyAgg[] }) {
  const options = {
    chart: { type: 'area', height: '100%', stacked: true, background: 'transparent',
             toolbar: { show: false }, fontFamily: 'inherit' },
    theme: { mode: apexThemeMode() },
    series: [
      { name: 'Input',          data: daily.map(d => d.input) },
      { name: 'Output',         data: daily.map(d => d.output) },
      { name: 'Cache Read',     data: daily.map(d => d.cache_read) },
      { name: 'Cache Creation', data: daily.map(d => d.cache_creation) },
    ],
    colors: [TOKEN_COLORS.input, TOKEN_COLORS.output, TOKEN_COLORS.cache_read, TOKEN_COLORS.cache_creation],
    fill: {
      type: 'gradient',
      gradient: {
        shadeIntensity: 1,
        opacityFrom: 0.4,
        opacityTo: 0.05,
        stops: [0, 95, 100],
      },
    },
    stroke: { curve: 'smooth' as const, width: 2 },
    xaxis: { categories: daily.map(d => d.day),
             labels: { rotate: -45, maxHeight: 60 },
             tickAmount: Math.min(daily.length, RANGE_TICKS[selectedRange.value]) },
    yaxis: { labels: { formatter: (v: number) => fmt(v) } },
    legend: { position: 'top' as const, fontSize: '11px' },
    dataLabels: { enabled: false },
    tooltip: { y: { formatter: (v: number) => fmt(v) + ' tokens' } },
    grid: { borderColor: cssVar('--chart-grid'), strokeDashArray: 3 },
  };

  return <ApexChart options={options} id="chart-daily" />;
}
