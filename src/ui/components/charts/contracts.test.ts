import { afterEach, describe, expect, it, vi } from 'vitest';

vi.hoisted(() => {
  Object.defineProperty(globalThis, 'window', {
    value: { location: { pathname: '/dashboard', search: '' } },
    configurable: true,
  });
  Object.defineProperty(globalThis, 'history', {
    value: { replaceState: vi.fn() },
    configurable: true,
  });
  Object.defineProperty(globalThis, 'document', {
    value: {
      documentElement: {
        getAttribute: () => 'dark',
      },
    },
    configurable: true,
  });
  Object.defineProperty(globalThis, 'getComputedStyle', {
    value: () => ({
      getPropertyValue: () => '#e8e8e8',
    }),
    configurable: true,
  });
});

import { selectedRange } from '../../state/store';
import type { HeatmapData, ProjectAgg, VersionSummary, WeeklyAgg, DailyAgg } from '../../state/types';
import { ActivityHeatmap } from './ActivityHeatmap';
import { DailyChart } from './DailyChart';
import { ProjectChart } from './ProjectChart';
import { VersionDonut } from './VersionDonut';
import { WeeklyChart } from './WeeklyChart';

function collectText(node: unknown): string[] {
  if (typeof node === 'string' || typeof node === 'number') return [String(node)];
  if (Array.isArray(node)) return node.flatMap(collectText);
  if (!node || typeof node !== 'object') return [];
  const vnode = node as { props?: { children?: unknown } };
  return collectText(vnode.props?.['children']);
}

function collectProp(node: unknown, prop: string): string[] {
  if (Array.isArray(node)) return node.flatMap(entry => collectProp(entry, prop));
  if (!node || typeof node !== 'object') return [];
  const vnode = node as { props?: Record<string, unknown> };
  const own = typeof vnode.props?.[prop] === 'string' ? [vnode.props[prop] as string] : [];
  return [...own, ...collectProp(vnode.props?.['children'], prop)];
}

afterEach(() => {
  selectedRange.value = '30d';
});

describe('chart contracts', () => {
  it('builds daily chart options from the selected range and daily rows', () => {
    selectedRange.value = '7d';
    const daily: DailyAgg[] = [
      { day: '2026-04-18', input: 10, output: 5, cache_read: 2, cache_creation: 1, reasoning_output: 0, cost: 1 },
      { day: '2026-04-19', input: 12, output: 6, cache_read: 3, cache_creation: 2, reasoning_output: 0, cost: 2 },
    ];

    const vnode = DailyChart({ daily }) as { props: Record<string, unknown> };
    const options = vnode.props['options'] as Record<string, unknown>;
    const series = options['series'] as Array<{ name: string; data: number[] }>;

    expect(vnode.props['id']).toBe('chart-daily');
    expect(series.map(entry => entry.name)).toEqual([
      'Input',
      'Output',
      'Cached Input',
      'Cache Creation',
    ]);
    expect(series[0]?.data).toEqual([10, 12]);
    expect((options['xaxis'] as { categories: string[] }).categories).toEqual([
      '2026-04-18',
      '2026-04-19',
    ]);
  });

  it('renders weekly chart fallback and chart tooltip contracts', () => {
    const empty = WeeklyChart({ weekly: [] });
    expect(collectText(empty)).toContain('No weekly data available.');

    const weekly: WeeklyAgg[] = [
      {
        week: '2026-15',
        input: 10,
        output: 4,
        cache_read: 2,
        cache_creation: 1,
        reasoning_output: 0,
        cost_nanos: 1234000,
      },
    ];
    const vnode = WeeklyChart({ weekly }) as { props: Record<string, unknown> };
    const options = vnode.props['options'] as {
      tooltip: { custom: ({ dataPointIndex }: { dataPointIndex: number }) => string };
    };

    expect(vnode.props['id']).toBe('chart-weekly');
    expect(options.tooltip.custom({ dataPointIndex: 0 })).toContain('2026-15');
    expect(options.tooltip.custom({ dataPointIndex: 0 })).toContain('Input: 10');
  });

  it('builds project chart interaction and raw-token tooltip behavior', () => {
    const rows: ProjectAgg[] = [
      {
        project: 'alpha/heimdall',
        display_name: 'Alpha Heimdall',
        input: 10,
        output: 5,
        cache_read: 0,
        cache_creation: 0,
        reasoning_output: 0,
        turns: 3,
        sessions: 1,
        cost: 1,
        credits: null,
      },
    ];
    const selected: ProjectAgg[] = [];
    const vnode = ProjectChart({
      byProject: rows,
      onSelectProject: project => selected.push(project),
    }) as { props: Record<string, unknown> };
    const options = vnode.props['options'] as {
      chart: { events: { dataPointSelection: (_a: unknown, _b: unknown, config: { dataPointIndex: number }) => void } };
      tooltip: { y: { formatter: (_v: number, ctx?: { dataPointIndex: number }) => string } };
      xaxis: { categories: string[] };
    };

    options.chart.events.dataPointSelection(null, null, { dataPointIndex: 0 });

    expect(selected).toEqual([rows[0]]);
    expect(options.tooltip.y.formatter(0, { dataPointIndex: 0 })).toBe('15 tokens');
    expect(options.xaxis.categories[0]).toContain('Alpha');
  });

  it('normalizes version donut labels and metric totals', () => {
    const rows: VersionSummary[] = [
      { provider: 'claude', version: '', turns: 2, sessions: 1, cost: 1.25, tokens: 100 },
      { provider: 'codex', version: '1.2.3', turns: 3, sessions: 2, cost: 0.75, tokens: 50 },
    ];
    const vnode = VersionDonut({
      rows,
      metric: 'cost',
      onMetricChange: () => undefined,
    }) as { props: { children: unknown } };

    const children = Array.isArray(vnode.props.children) ? vnode.props.children : [vnode.props.children];
    const chartShell = children[1] as { props: { children: { props: Record<string, unknown> } } };
    const chartVNode = chartShell.props.children;
    const options = chartVNode.props['options'] as {
      labels: string[];
      plotOptions: { pie: { donut: { labels: { total: { formatter: () => string } } } } };
      tooltip: { custom: ({ seriesIndex }: { seriesIndex: number }) => string };
    };

    expect(chartVNode.props['id']).toBe('chart-version-donut');
    expect(options.labels).toEqual(['(unknown)', '1.2.3']);
    expect(options.plotOptions.pie.donut.labels.total.formatter()).toBe('$2.0000');
    expect(options.tooltip.custom({ seriesIndex: 0 })).toContain('(unknown)');
  });

  it('renders heatmap captions and hour-cell tooltips', () => {
    const data: HeatmapData = {
      cells: [{ dow: 1, hour: 6, cost_nanos: 2_000_000_000, call_count: 3 }],
      max_cost_nanos: 2_000_000_000,
      max_call_count: 3,
      active_days: 1,
      total_cost_nanos: 2_000_000_000,
      period: 'month',
      tz_offset_min: 0,
    };

    const vnode = ActivityHeatmap({ data });
    const text = collectText(vnode);
    const titles = collectProp(vnode, 'title');
    const joined = text.join('');

    expect(text).toContain('ACTIVITY / 7x24 / ');
    expect(text).toContain('MONTH');
    expect(joined).toContain('1 active day');
    expect(titles.some(title => title.includes('Mon 06:00'))).toBe(true);
  });
});
