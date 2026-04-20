import { describe, expect, it, vi } from 'vitest';

vi.hoisted(() => {
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
      getPropertyValue: (name: string) =>
        ({
          '--text-display': '#eeeeee',
          '--surface': '#000000',
          '--text-secondary': '#999999',
          '--border': '#222222',
        })[name] ?? '',
    }),
    configurable: true,
  });
});

vi.mock('preact/hooks', async () => {
  const actual = await vi.importActual<typeof import('preact/hooks')>('preact/hooks');
  return {
    ...actual,
    useState: <T,>(value: T) => [value, vi.fn()] as const,
  };
});

import type { ModelAgg } from '../../state/types';
import { ApexChart } from './ApexChart';
import { ModelChart } from './ModelChart';

function findByType(node: unknown, type: unknown): Array<{ props: Record<string, unknown> }> {
  if (Array.isArray(node)) return node.flatMap(entry => findByType(entry, type));
  if (!node || typeof node !== 'object') return [];
  const vnode = node as { type?: unknown; props?: Record<string, unknown> };
  const own = vnode.type === type ? [{ props: vnode.props ?? {} }] : [];
  return [...own, ...findByType(vnode.props?.['children'], type)];
}

function collectText(node: unknown): string[] {
  if (typeof node === 'string' || typeof node === 'number') return [String(node)];
  if (Array.isArray(node)) return node.flatMap(collectText);
  if (!node || typeof node !== 'object') return [];
  const vnode = node as { props?: { children?: unknown } };
  return collectText(vnode.props?.children);
}

describe('ModelChart', () => {
  it('returns null when the metric totals are empty', () => {
    const rows = [
      {
        model: 'empty',
        input: 0,
        output: 0,
        cache_read: 0,
        cache_creation: 0,
        reasoning_output: 0,
        turns: 0,
        sessions: 0,
        cost: 0,
        is_billable: true,
        input_cost: 0,
        output_cost: 0,
        cache_read_cost: 0,
        cache_write_cost: 0,
        credits: null,
      },
    ] as ModelAgg[];

    expect(ModelChart({ byModel: rows })).toBeNull();
  });

  it('builds donut options and filter actions for model rows', () => {
    const rows: ModelAgg[] = [
      {
        model: 'sonnet-4',
        input: 100,
        output: 50,
        cache_read: 10,
        cache_creation: 5,
        reasoning_output: 0,
        turns: 3,
        sessions: 1,
        cost: 2.5,
        is_billable: true,
        input_cost: 1,
        output_cost: 1,
        cache_read_cost: 0.25,
        cache_write_cost: 0.25,
        credits: null,
      },
      {
        model: 'haiku-3',
        input: 40,
        output: 20,
        cache_read: 0,
        cache_creation: 0,
        reasoning_output: 0,
        turns: 1,
        sessions: 1,
        cost: 0.5,
        is_billable: true,
        input_cost: 0.2,
        output_cost: 0.2,
        cache_read_cost: 0.05,
        cache_write_cost: 0.05,
        credits: null,
      },
    ];
    const selected: string[] = [];

    const vnode = ModelChart({
      byModel: rows,
      onSelectModel: model => selected.push(model),
    }) as { props: { children: unknown[] } };
    const chart = findByType(vnode, ApexChart)[0]!;
    const interactiveRows = findByType(vnode, 'button');
    const options = chart.props['options'] as {
      labels: string[];
      series: number[];
      tooltip: { custom: ({ seriesIndex }: { seriesIndex: number }) => string };
      chart: { events: { dataPointSelection: (_a: unknown, _b: unknown, config: { dataPointIndex: number }) => void } };
    };

    options.chart.events.dataPointSelection(null, null, { dataPointIndex: 0 });
    (interactiveRows[interactiveRows.length - 1]?.props['onClick'] as () => void)();

    expect(chart.props['id']).toBe('chart-model-apex');
    expect(options.labels).toEqual(['sonnet-4', 'haiku-3']);
    expect(options.series).toEqual([2.5, 0.5]);
    expect(options.tooltip.custom({ seriesIndex: 0 })).toContain('Cost: $2.5000');
    expect(collectText(vnode)).toContain('Cost');
    expect(selected).toEqual(['sonnet-4', 'haiku-3']);
  });
});
