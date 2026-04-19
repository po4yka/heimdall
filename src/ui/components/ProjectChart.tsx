import { ApexChart } from './ApexChart';
import type { ApexDataPointSelectionConfig, ApexTooltipFormatterContext } from '../lib/apex';
import { industrialChartOptions, tokenSeriesColors } from '../lib/charts';
import { fmt, truncateMid } from '../lib/format';
import type { ProjectAgg } from '../state/types';

export function ProjectChart({
  byProject,
  onSelectProject,
}: {
  byProject: ProjectAgg[];
  onSelectProject?: (project: ProjectAgg) => void;
}) {
  const top = byProject.slice(0, 10);
  if (!top.length) return null;

  const base = industrialChartOptions('bar');
  const colors = tokenSeriesColors();
  // Collapse Input + Output into a single Total-tokens series. The
  // per-type breakdown already lives in ProjectCostTable below.
  const totals = top.map(p => p.input + p.output);
  // ApexCharts' logarithmic option does not apply to the value axis of a
  // horizontal bar (vue-apexcharts#300). Instead, render each bar as its
  // share of the maximum (0–100%), which rescues tail bars that would be
  // hairlines under a linear scale. The raw token count stays in the
  // tooltip so users do not lose absolute magnitudes.
  const maxTotal = totals.reduce((m, v) => (v > m ? v : m), 0);
  const shares = totals.map(v => (maxTotal > 0 ? (v / maxTotal) * 100 : 0));

  const options = {
    ...base,
    chart: {
      ...base.chart,
      type: 'bar',
      ...(onSelectProject
        ? {
            events: {
              dataPointSelection: (
                _event: unknown,
                _ctx: unknown,
                config: ApexDataPointSelectionConfig
              ) => {
                const row = top[config.dataPointIndex];
                if (row) onSelectProject(row);
              },
            },
          }
        : {}),
    },
    series: [{ name: 'Share of top', data: shares }],
    colors: [colors[0] ?? 'currentColor'],
    fill: { type: 'solid' },
    plotOptions: { bar: { horizontal: true, barHeight: '60%', borderRadius: 0 } },
    xaxis: {
      ...(base.xaxis ?? {}),
      categories: top.map(p => truncateMid(p.display_name || p.project, 18, 8)),
      min: 0,
      max: 100,
      tickAmount: 4,
      labels: {
        ...(base.xaxis?.labels ?? {}),
        formatter: (v: number) => `${Math.round(v)}%`,
        hideOverlappingLabels: true,
      },
    },
    yaxis: {
      ...(base.yaxis ?? {}),
      labels: { ...(base.yaxis?.labels ?? {}), maxWidth: 120 },
    },
    // Anchor the tooltip to the plot's bottom-left so it cannot cover the
    // card's "TOP PROJECTS" title during hover.
    tooltip: {
      ...base.tooltip,
      fixed: { enabled: true, position: 'bottomLeft', offsetX: 0, offsetY: 0 },
      y: {
        // Display the raw token count per project regardless of bar scale.
        formatter: (_v: number, opts?: ApexTooltipFormatterContext) => {
          const raw = totals[opts?.dataPointIndex ?? 0] ?? 0;
          return fmt(raw) + ' tokens';
        },
      },
    },
  };

  return <ApexChart options={options} id="chart-project" />;
}
