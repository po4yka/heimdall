import { ApexChart } from './ApexChart';
import { industrialChartOptions, modelSeriesColors, cssVar } from '../lib/charts';
import { fmt } from '../lib/format';
import type { ModelAgg } from '../state/types';

export function ModelChart({ byModel }: { byModel: ModelAgg[] }) {
  if (!byModel.length) return null;

  const base = industrialChartOptions('donut');
  const options = {
    ...base,
    chart: { ...base.chart, type: 'donut' },
    series: byModel.map(m => m.input + m.output),
    labels: byModel.map(m => m.model),
    colors: modelSeriesColors(byModel.length),
    stroke: { width: 2, colors: [cssVar('--surface')] },
    plotOptions: {
      pie: {
        donut: {
          size: '64%',
          labels: {
            show: true,
            total: {
              show: true,
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
              color: cssVar('--text-secondary'),
            },
          },
        },
      },
    },
    tooltip: { ...base.tooltip, y: { formatter: (v: number) => fmt(v) + ' tokens' } },
  };

  return <ApexChart options={options} id="chart-model" />;
}
