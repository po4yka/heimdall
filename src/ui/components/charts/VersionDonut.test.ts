import { describe, expect, it, vi } from 'vitest';

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

import type { VersionSummary } from '../../state/types';
import { VersionDonut } from './VersionDonut';

describe('VersionDonut', () => {
  it('normalizes unknown versions and computes total metric labels', () => {
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
    };

    expect(chartVNode.props['id']).toBe('chart-version-donut');
    expect(options.labels).toEqual(['(unknown)', '1.2.3']);
    expect(options.plotOptions.pie.donut.labels.total.formatter()).toBe('$2.0000');
  });
});
