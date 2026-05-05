import { useMemo } from 'preact/hooks';
import { ApexChart } from '../charts/ApexChart';
import { dashboardChartOptions, withAlpha } from '../../lib/charts';
import { fmtCostBig } from '../../lib/format';
import type { AgentTimelinePoint } from '../../state/types';

interface AgentTimelineProps {
  timeline: AgentTimelinePoint[];
}

const MAX_ROLES = 8;

export function AgentTimeline({ timeline }: AgentTimelineProps) {
  const options = useMemo(() => {
    if (!timeline.length) return null;

    // Aggregate cost per role, pick top MAX_ROLES
    const roleCost = new Map<string, number>();
    for (const pt of timeline) {
      roleCost.set(pt.role, (roleCost.get(pt.role) ?? 0) + pt.cost_usd);
    }
    const sortedRoles = [...roleCost.entries()]
      .sort((a, b) => b[1] - a[1])
      .map(([r]) => r);
    const topRoles = sortedRoles.slice(0, MAX_ROLES);
    const hasOther = sortedRoles.length > MAX_ROLES;

    // Collect all unique buckets (dates), sorted
    const buckets = [...new Set(timeline.map(pt => pt.bucket))].sort();

    // Build series
    const seriesRoles = hasOther ? [...topRoles, 'Other'] : topRoles;
    const seriesData: Record<string, number[]> = {};
    for (const role of seriesRoles) seriesData[role] = new Array(buckets.length).fill(0);

    for (const pt of timeline) {
      const bucketIdx = buckets.indexOf(pt.bucket);
      if (bucketIdx < 0) continue;
      if (topRoles.includes(pt.role)) {
        const arr = seriesData[pt.role];
        if (arr) arr[bucketIdx] = (arr[bucketIdx] ?? 0) + pt.cost_usd;
      } else if (hasOther) {
        const arr = seriesData['Other'];
        if (arr) arr[bucketIdx] = (arr[bucketIdx] ?? 0) + pt.cost_usd;
      }
    }

    // Monochrome opacity ladder: 1.0, 0.7, 0.5, 0.35, 0.25, 0.18, 0.12, 0.08
    const opacityLadder = [1.0, 0.7, 0.5, 0.35, 0.25, 0.18, 0.12, 0.08, 0.06];
    const colors = seriesRoles.map((_, i) =>
      withAlpha('--text-display', opacityLadder[Math.min(i, opacityLadder.length - 1)] ?? 0.06)
    );

    const series = seriesRoles.map(role => ({
      name: role,
      data: seriesData[role]!,
    }));

    const base = dashboardChartOptions('bar');
    return {
      ...base,
      chart: {
        ...base.chart,
        type: 'bar',
        stacked: true,
      },
      series,
      colors,
      xaxis: {
        ...base.xaxis,
        categories: buckets,
        tickAmount: Math.min(buckets.length, 10),
        labels: {
          ...base.xaxis?.labels,
          rotate: -30,
          style: {
            ...(base.xaxis?.labels as { style?: object } | undefined)?.style,
            fontSize: '9px',
          },
        },
      },
      yaxis: {
        ...base.yaxis,
        labels: {
          ...((base.yaxis as { labels?: object } | undefined)?.labels),
          formatter: (v: number) => fmtCostBig(v),
        },
      },
      tooltip: {
        ...base.tooltip,
        y: {
          formatter: (v: number) => fmtCostBig(v),
        },
      },
      plotOptions: {
        bar: { horizontal: false, columnWidth: '60%' },
      },
    };
  }, [timeline]);

  if (!timeline.length) {
    return (
      <div class="table-card agent-timeline-wrap" style={{ padding: '20px' }}>
        <div class="section-title">Agent activity</div>
        <div class="empty-state">No timeline data</div>
      </div>
    );
  }

  return (
    <div class="table-card agent-timeline-wrap">
      <div class="section-header" style={{ padding: '20px 20px 0' }}>
        <div class="section-title" style={{ padding: 0 }}>Agent activity</div>
      </div>
      <div class="chart-wrap tall" style={{ padding: '0 12px 12px' }}>
        {options && <ApexChart options={options} id="agent-timeline-chart" />}
      </div>
    </div>
  );
}
