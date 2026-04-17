import { ApexChart } from './ApexChart';
import { industrialChartOptions, tokenSeriesColors, RANGE_TICKS } from '../lib/charts';
import { fmt } from '../lib/format';
import { selectedRange } from '../state/store';
import type { DailyAgg } from '../state/types';

export function DailyChart({ daily }: { daily: DailyAgg[] }) {
  const base = industrialChartOptions('bar');
  const options = {
    ...base,
    chart: { ...base.chart, type: 'bar', stacked: true },
    series: [
      { name: 'Input',          data: daily.map(d => d.input) },
      { name: 'Output',         data: daily.map(d => d.output) },
      { name: 'Cached Input',   data: daily.map(d => d.cache_read) },
      { name: 'Cache Creation', data: daily.map(d => d.cache_creation) },
    ],
    colors: tokenSeriesColors(),
    fill: { type: 'solid' },
    plotOptions: { bar: { columnWidth: '70%', borderRadius: 0 } },
    xaxis: {
      ...base.xaxis,
      categories: daily.map(d => d.day),
      labels: { ...base.xaxis.labels, rotate: -45, maxHeight: 60 },
      tickAmount: Math.min(daily.length, RANGE_TICKS[selectedRange.value]),
    },
    yaxis: {
      ...base.yaxis,
      labels: { ...base.yaxis.labels, formatter: (v: number) => fmt(v) },
    },
    tooltip: { ...base.tooltip, y: { formatter: (v: number) => fmt(v) + ' tokens' } },
  };

  return <ApexChart options={options} id="chart-daily" />;
}
