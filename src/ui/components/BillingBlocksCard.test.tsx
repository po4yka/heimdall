import { describe, expect, it } from 'vitest';
import type { BillingBlocksResponse } from '../state/types';
import { SegmentedProgressBar } from './SegmentedProgressBar';
import { BillingBlocksCard } from './BillingBlocksCard';

function collectText(node: unknown): string[] {
  if (typeof node === 'string' || typeof node === 'number') return [String(node)];
  if (Array.isArray(node)) return node.flatMap(collectText);
  if (!node || typeof node !== 'object') return [];
  const vnode = node as { type?: unknown; props?: Record<string, unknown> };
  if (typeof vnode.type === 'function') {
    return collectText(vnode.type(vnode.props ?? {}));
  }
  return collectText(vnode.props?.['children']);
}

function findByType(node: unknown, type: unknown): Array<{ props: Record<string, unknown> }> {
  if (Array.isArray(node)) return node.flatMap(entry => findByType(entry, type));
  if (!node || typeof node !== 'object') return [];
  const vnode = node as { type?: unknown; props?: Record<string, unknown> };
  const own = vnode.type === type ? [{ props: vnode.props ?? {} }] : [];
  if (typeof vnode.type === 'function' && vnode.type !== type) {
    return [...own, ...findByType(vnode.type(vnode.props ?? {}), type)];
  }
  return [...own, ...findByType(vnode.props?.['children'], type)];
}

describe('BillingBlocksCard', () => {
  it('renders historical fallback when there is no active block', () => {
    const data: BillingBlocksResponse = {
      session_length_hours: 5,
      token_limit: null,
      historical_max_tokens: 123_000,
      quota_suggestions: {
        sample_count: 3,
        population_count: 3,
        recommended_key: 'p90',
        sample_strategy: 'completed_blocks',
        sample_label: '3 completed blocks',
        levels: [
          { key: 'p90', label: 'P90', limit_tokens: 500_000 },
          { key: 'p95', label: 'P95', limit_tokens: 600_000 },
          { key: 'max', label: 'Max', limit_tokens: 700_000 },
        ],
        note: 'Based on fewer than 10 completed blocks.',
      },
      blocks: [],
    };

    const vnode = BillingBlocksCard({ data });
    const text = collectText(vnode).join(' ');

    expect(text).toContain('NO ACTIVE BLOCK');
    expect(text).toContain('123.0K');
    expect(text).toContain('SUGGESTED QUOTAS');
    expect(text).toContain('P90');
    expect(text).toContain('[RECOMMENDED]');
  });

  it('renders active block totals, burn rate, and quota summaries', () => {
    const data = {
      session_length_hours: 5,
      token_limit: 1_000_000,
      historical_max_tokens: 0,
      quota_suggestions: {
        sample_count: 12,
        population_count: 16,
        recommended_key: 'p90',
        sample_strategy: 'near_limit_hits',
        sample_label: '12 near-limit completed blocks',
        levels: [
          { key: 'p90', label: 'P90', limit_tokens: 800_000 },
          { key: 'p95', label: 'P95', limit_tokens: 900_000 },
          { key: 'max', label: 'Max', limit_tokens: 950_000 },
        ],
      },
      blocks: [
        {
          is_active: true,
          first_timestamp: '2026-04-20T00:00:00Z',
          last_timestamp: '2026-04-20T01:30:00Z',
          end: '2026-04-20T05:00:00Z',
          entry_count: 12,
          tokens: {
            input: 200_000,
            output: 100_000,
            cache_read: 50_000,
            cache_creation: 10_000,
            reasoning_output: 5_000,
          },
          burn_rate: {
            cost_per_hour_nanos: 2_500_000_000,
            tier: 'moderate',
          },
          projection: {
            projected_cost_nanos: 4_000_000_000,
            projected_tokens: 700_000,
          },
          quota: {
            used_tokens: 300_000,
            limit_tokens: 1_000_000,
            projected_tokens: 700_000,
            current_pct: 30,
            projected_pct: 70,
            current_severity: 'ok',
            projected_severity: 'warn',
          },
        },
      ],
    } as BillingBlocksResponse;

    const vnode = BillingBlocksCard({ data });
    const text = collectText(vnode).join(' ');
    const bars = findByType(vnode, SegmentedProgressBar);

    expect(text).toContain('365.0K');
    expect(text).toContain('1h 30m');
    expect(text).toContain('elapsed');
    expect(text).toContain('05:00 UTC');
    expect(text).toContain('2.5000');
    expect(text).toContain('[WARN]');
    expect(text).toContain('Configured');
    expect(text).toContain('1.0M');
    expect(text).toContain('P95');
    expect(text).toContain('Projects');
    expect(text).toContain('12 near-limit completed blocks');
    expect(bars).toHaveLength(2);
    expect(bars[0]?.props['value']).toBe(300_000);
    expect(bars[0]?.props['status']).toBe('success');
    expect(bars[1]?.props['value']).toBe(700_000);
    expect(bars[1]?.props['status']).toBe('warning');
  });
});
