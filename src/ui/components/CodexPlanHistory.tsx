import { useMemo } from 'preact/hooks';
import type { ApexOptions } from '../lib/apex';
import type { CodexPlanDailyRow } from '../state/dashboard-types';
import { ApexChart } from './charts/ApexChart';

interface Props {
  history: CodexPlanDailyRow[];
}

// Monochrome opacity ladder: one segment per plan type.
const OPACITY_LADDER = [1.0, 0.65, 0.35, 0.2];

function buildOptions(history: CodexPlanDailyRow[]): ApexOptions | null {
  // History arrives DESC from the server; reverse to oldest-first for chart.
  const rows = [...history].reverse();
  if (rows.length === 0) return null;

  // Collect all unique plan types across all rows.
  const planSet = new Set<string>();
  for (const row of rows) {
    for (const plan of Object.keys(row.by_plan)) {
      planSet.add(plan);
    }
    // If no by_plan entries but there is a primary_pct, use snapshot plan.
    if (Object.keys(row.by_plan).length === 0 && row.primary_pct > 0) {
      const pt = row.snapshot?.plan_type ?? 'unknown';
      planSet.add(pt);
    }
  }
  const plans = Array.from(planSet).sort();

  const categories = rows.map(r => r.day);

  // Stacked bar series: one per plan type.
  const barSeries = plans.map((plan, i) => {
    const opacity = OPACITY_LADDER[i % OPACITY_LADDER.length];
    return {
      name: plan.charAt(0).toUpperCase() + plan.slice(1),
      type: 'bar' as const,
      data: rows.map(r => {
        const v = r.by_plan[plan];
        return v != null ? Math.min(100, Math.max(0, v)) : 0;
      }),
      color: `rgba(var(--text-primary-rgb, 232, 232, 232), ${opacity})`,
    };
  });

  // Dashed line for secondary (7d) window — nullable, use null for missing.
  const secondarySeries = {
    name: '7d window',
    type: 'line' as const,
    data: rows.map(r => (r.secondary_pct != null ? Math.min(100, Math.max(0, r.secondary_pct)) : null)),
    color: 'var(--text-secondary, #888)',
  };

  // Red strip markers for days with limit hits: small annotation on top.
  const limitHitAnnotations = rows
    .filter(r => r.limit_hit_count > 0)
    .map(r => ({
      x: r.day,
      strokeDashArray: 0,
      borderColor: 'var(--accent)',
      label: {
        text: `limit x${r.limit_hit_count}`,
        style: {
          color: 'var(--accent)',
          background: 'transparent',
          fontFamily: 'var(--font-mono)',
          fontSize: '10px',
        },
      },
    }));

  const allSeries = [...barSeries, secondarySeries];

  // ApexCharts supports mixed bar+line series and array-form stroke/fill/yaxis,
  // but our local ApexOptions type only covers the common single-series shape.
  // Cast through unknown to pass the richer config without widening the type.
  const opts = {
    chart: {
      type: 'bar',
      stacked: true,
      toolbar: { show: false },
      animations: { enabled: false },
      fontFamily: 'var(--font-mono)',
    },
    theme: { mode: 'dark' },
    series: allSeries,
    colors: allSeries.map(s => s.color),
    stroke: {
      width: allSeries.map((_, i) => (i < barSeries.length ? 0 : 2)),
      curve: 'smooth',
      dashArray: allSeries.map((_, i) => (i < barSeries.length ? 0 : 4)),
    },
    fill: {
      type: allSeries.map(() => 'solid'),
      opacity: allSeries.map(() => 1),
    },
    plotOptions: {
      bar: { columnWidth: '70%' },
    },
    grid: {
      borderColor: 'var(--border)',
      strokeDashArray: 2,
      xaxis: { lines: { show: false } },
      yaxis: { lines: { show: true } },
    },
    legend: {
      position: 'top',
      labels: { colors: 'var(--text-secondary)', fontFamily: 'var(--font-mono)' },
      itemMargin: { horizontal: 12, vertical: 4 },
    },
    xaxis: {
      categories,
      labels: {
        rotate: -45,
        style: {
          colors: 'var(--text-secondary)',
          fontFamily: 'var(--font-mono)',
          fontSize: '10px',
        },
      },
      axisBorder: { show: false },
      axisTicks: { show: false },
    },
    yaxis: [
      {
        min: 0,
        max: 100,
        labels: {
          style: {
            colors: 'var(--text-secondary)',
            fontFamily: 'var(--font-mono)',
            fontSize: '11px',
          },
          formatter: (v: number) => `${v.toFixed(0)}%`,
        },
      },
      {
        opposite: true,
        min: 0,
        max: 100,
        show: false,
      },
    ],
    tooltip: {
      theme: 'dark',
      style: { fontFamily: 'var(--font-mono)', fontSize: '11px' },
      y: {
        formatter: (val: number | null) =>
          val != null && Number.isFinite(val) ? `${val.toFixed(1)}%` : '—',
      },
    },
    dataLabels: { enabled: false },
    markers: { size: 0, strokeWidth: 0, hover: { size: 3 } },
    ...(limitHitAnnotations.length > 0
      ? { annotations: { xaxis: limitHitAnnotations } }
      : {}),
  } as unknown as ApexOptions;

  return opts;
}

export function CodexPlanHistory({ history }: Props) {
  const options = useMemo(() => buildOptions(history), [history]);

  if (!options) return null;

  return (
    <div class="card codex-plan-history-card">
      <div class="codex-plan-history-header">
        <div class="codex-plan-history-title">Codex plan utilisation (30 days)</div>
      </div>
      <div class="codex-plan-history-body">
        <div class="chart-wrap tall">
          <ApexChart options={options} id="codex-plan-history-chart" />
        </div>
      </div>
    </div>
  );
}
