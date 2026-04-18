import { ApexChart } from './ApexChart';
import { industrialChartOptions, tokenSeriesColors } from '../lib/charts';
import { fmt } from '../lib/format';
import type { WeeklyAgg } from '../state/types';

export function WeeklyChart({ weekly }: { weekly: WeeklyAgg[] }) {
  if (!weekly?.length) {
    return (
      <div style={{ padding: '24px', color: 'var(--text-muted)', fontFamily: 'var(--font-mono)', fontSize: '12px' }}>
        No weekly data available.
      </div>
    );
  }

  const base = industrialChartOptions('bar');
  const options = {
    ...base,
    chart: { ...base.chart, type: 'bar', stacked: true },
    series: [
      { name: 'Input',          data: weekly.map(w => w.input) },
      { name: 'Output',         data: weekly.map(w => w.output) },
      { name: 'Cached Input',   data: weekly.map(w => w.cache_read) },
      { name: 'Cache Creation', data: weekly.map(w => w.cache_creation) },
    ],
    colors: tokenSeriesColors(),
    fill: { type: 'solid' },
    plotOptions: { bar: { columnWidth: '70%', borderRadius: 0 } },
    xaxis: {
      ...base.xaxis,
      categories: weekly.map(w => w.week),
      labels: { ...base.xaxis.labels, rotate: -45, maxHeight: 60 },
      tickAmount: Math.min(weekly.length, 26),
    },
    yaxis: {
      ...base.yaxis,
      labels: { ...base.yaxis.labels, formatter: (v: number) => fmt(v) },
    },
    tooltip: {
      ...base.tooltip,
      y: { formatter: (v: number) => fmt(v) + ' tokens' },
      custom: ({ dataPointIndex }: { dataPointIndex: number }) => {
        const w = weekly[dataPointIndex];
        if (!w) return '';
        const total = w.input + w.output + w.cache_read + w.cache_creation;
        const costUsd = w.cost_nanos / 1e9;
        const costStr = costUsd < 0.0001 ? '<$0.0001' : '$' + costUsd.toFixed(4);
        return (
          '<div style="padding:8px 12px;font-family:var(--font-mono);font-size:12px;background:var(--color-bg-secondary);border:1px solid var(--color-border)">' +
          '<div style="margin-bottom:4px;font-weight:600">' + w.week + '</div>' +
          '<div>Input: ' + fmt(w.input) + '</div>' +
          '<div>Output: ' + fmt(w.output) + '</div>' +
          '<div>Cached Input: ' + fmt(w.cache_read) + '</div>' +
          '<div>Cache Creation: ' + fmt(w.cache_creation) + '</div>' +
          '<div style="margin-top:4px;border-top:1px solid var(--color-border);padding-top:4px">Total: ' + fmt(total) + ' tokens</div>' +
          '<div>Cost: ' + costStr + '</div>' +
          '</div>'
        );
      },
    },
  };

  return <ApexChart options={options} id="chart-weekly" />;
}

export default WeeklyChart;
