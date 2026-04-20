import { describe, expect, it } from 'vitest';
import type { WindowInfo } from '../state/types';
import { SegmentedProgressBar } from './SegmentedProgressBar';
import { BudgetCard, RateWindowCard, RateWindowUnavailable } from './RateWindowCard';

function collectText(node: unknown): string[] {
  if (typeof node === 'string' || typeof node === 'number') return [String(node)];
  if (Array.isArray(node)) return node.flatMap(collectText);
  if (!node || typeof node !== 'object') return [];
  const vnode = node as { props?: { children?: unknown } };
  return collectText(vnode.props?.children);
}

function findByType(node: unknown, type: unknown): Array<{ props: Record<string, unknown> }> {
  if (Array.isArray(node)) return node.flatMap(entry => findByType(entry, type));
  if (!node || typeof node !== 'object') return [];
  const vnode = node as { type?: unknown; props?: Record<string, unknown> };
  const own = vnode.type === type ? [{ props: vnode.props ?? {} }] : [];
  return [...own, ...findByType(vnode.props?.['children'], type)];
}

describe('RateWindowCard', () => {
  it('renders rate window usage and reset timing', () => {
    const windowInfo = {
      used_percent: 87.6,
      resets_in_minutes: 95,
    } as WindowInfo;

    const vnode = RateWindowCard({ label: 'Claude', window: windowInfo }) as {
      props: Record<string, unknown>;
    };
    const text = collectText(vnode).join(' ');
    const bars = findByType(vnode, SegmentedProgressBar);

    expect(text).toContain('87.6');
    expect(text).toContain('Resets in 1h 35m');
    expect(bars[0]?.props['aria-label']).toBe('Claude usage');
  });

  it('renders budget and unavailable fallback cards', () => {
    const budget = BudgetCard({
      used: 12.34,
      limit: 50,
      currency: 'USD',
      utilization: 24.68,
    });
    const unavailable = RateWindowUnavailable({ error: 'API offline' });
    const budgetText = collectText(budget).join(' ');

    expect(budgetText).toContain('12.34');
    expect(budgetText).toContain('50.00');
    expect(collectText(budget)).toContain('USD');
    expect(collectText(unavailable)).toContain('Unavailable');
    expect(collectText(unavailable)).toContain('API offline');
  });
});
