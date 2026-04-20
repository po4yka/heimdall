import { afterEach, describe, expect, it, vi } from 'vitest';
import type { HourlyRow } from '../../state/types';

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
          '--border': '#222222',
          '--text-secondary': '#999999',
        })[name] ?? '',
    }),
    configurable: true,
  });
});

import { HourlyChart } from './HourlyChart';

function collectTitles(node: unknown): string[] {
  if (Array.isArray(node)) return node.flatMap(collectTitles);
  if (!node || typeof node !== 'object') return [];
  const vnode = node as { props?: Record<string, unknown> };
  const own = typeof vnode.props?.['title'] === 'string' ? [vnode.props['title'] as string] : [];
  return [...own, ...collectTitles(vnode.props?.['children'])];
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('HourlyChart', () => {
  it('returns null when there is no activity', () => {
    expect(HourlyChart({ data: [] })).toBeNull();
  });

  it('renders 24 hourly bars with formatted tooltips', () => {
    const data: HourlyRow[] = [
      { hour: 6, turns: 4, provider: 'claude', input: 0, output: 0, reasoning_output: 0 },
      { hour: 12, turns: 8, provider: 'claude', input: 0, output: 0, reasoning_output: 0 },
    ] as HourlyRow[];

    const vnode = HourlyChart({ data }) as { props: { children: unknown[] } };
    const bars = (vnode.props.children[1] as { props: { children: unknown[] } }).props.children;
    const titles = collectTitles(vnode);

    expect(titles).toContain('6:00 -- 4 turns');
    expect(titles).toContain('12:00 -- 8 turns');
    expect((bars as unknown[])).toHaveLength(24);
  });
});
