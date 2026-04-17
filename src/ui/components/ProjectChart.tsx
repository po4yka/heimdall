import { ApexChart } from './ApexChart';
import { industrialChartOptions, tokenSeriesColors } from '../lib/charts';
import { fmt } from '../lib/format';
import type { ProjectAgg } from '../state/types';

export function ProjectChart({ byProject }: { byProject: ProjectAgg[] }) {
  const top = byProject.slice(0, 10);
  if (!top.length) return null;

  const base = industrialChartOptions('bar');
  const colors = tokenSeriesColors();
  const options = {
    ...base,
    chart: { ...base.chart, type: 'bar' },
    series: [
      { name: 'Input',  data: top.map(p => p.input) },
      { name: 'Output', data: top.map(p => p.output) },
    ],
    colors: [colors[0], colors[1]],
    fill: { type: 'solid' },
    plotOptions: { bar: { horizontal: true, barHeight: '60%', borderRadius: 0 } },
    xaxis: {
      ...base.xaxis,
      categories: top.map(p => p.project.length > 22 ? '\u2026' + p.project.slice(-20) : p.project),
      labels: { ...base.xaxis.labels, formatter: (v: number) => fmt(v) },
    },
    yaxis: {
      ...base.yaxis,
      labels: { ...base.yaxis.labels, maxWidth: 160 },
    },
    tooltip: { ...base.tooltip, y: { formatter: (v: number) => fmt(v) + ' tokens' } },
  };

  return <ApexChart options={options} id="chart-project" />;
}
