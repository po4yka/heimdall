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
import type { DailyAgg } from '../../state/types';
import { DailyChart } from './DailyChart';

afterEach(() => {
  selectedRange.value = '30d';
});

describe('DailyChart', () => {
  it('builds stacked daily series and category labels', () => {
    selectedRange.value = '7d';
    const daily: DailyAgg[] = [
      { day: '2026-04-18', input: 10, output: 5, cache_read: 2, cache_creation: 1, reasoning_output: 0, cost: 1 },
      { day: '2026-04-19', input: 12, output: 6, cache_read: 3, cache_creation: 2, reasoning_output: 0, cost: 2 },
    ];

    const vnode = DailyChart({ daily }) as { props: Record<string, unknown> };
    const options = vnode.props['options'] as Record<string, unknown>;
    const series = options['series'] as Array<{ data: number[] }>;

    expect(vnode.props['id']).toBe('chart-daily');
    expect(series[0]?.data).toEqual([10, 12]);
    expect((options['xaxis'] as { categories: string[] }).categories).toEqual([
      '2026-04-18',
      '2026-04-19',
    ]);
  });
});
