import { describe, expect, it } from 'vitest';
import type { CacheEfficiency } from '../state/types';
import { CacheEfficiencyCard } from './CacheEfficiencyCard';

function collectText(node: unknown): string[] {
  if (typeof node === 'string' || typeof node === 'number') return [String(node)];
  if (Array.isArray(node)) return node.flatMap(collectText);
  if (!node || typeof node !== 'object') return [];
  const vnode = node as { props?: { children?: unknown } };
  return collectText(vnode.props?.children);
}

describe('CacheEfficiencyCard', () => {
  it('renders cache hit rate and savings tooltip details', () => {
    const data: CacheEfficiency = {
      cache_read_tokens: 2_000_000,
      cache_write_tokens: 0,
      input_tokens: 2_000_000,
      output_tokens: 0,
      cache_read_cost_nanos: 0,
      cache_write_cost_nanos: 0,
      input_cost_nanos: 0,
      output_cost_nanos: 0,
      cache_hit_rate: 0.5,
    };

    const vnode = CacheEfficiencyCard({
      data,
      inputRatePerMtok: 3,
      cacheReadRatePerMtok: 0.3,
    }) as { props: Record<string, unknown> };
    const progress = (vnode.props['children'] as unknown[])[1] as {
      props: { children: { props: { style: Record<string, unknown> } } };
    };

    expect(vnode.props['title']).toContain(
      '2.00M tokens cache-read / 4.00M total input-addressable tokens'
    );
    expect(vnode.props['title']).toContain('saved approx $5.40 vs. no-cache');
    expect(collectText(vnode)).toContain('50.0%');
    expect(progress.props.children.props.style['width']).toBe('50.00%');
  });

  it('renders an empty-state tooltip when no cache rate is available', () => {
    const data: CacheEfficiency = {
      cache_read_tokens: 0,
      cache_write_tokens: 0,
      input_tokens: 0,
      output_tokens: 0,
      cache_read_cost_nanos: 0,
      cache_write_cost_nanos: 0,
      input_cost_nanos: 0,
      output_cost_nanos: 0,
      cache_hit_rate: null,
    };

    const vnode = CacheEfficiencyCard({ data }) as { props: Record<string, unknown> };
    expect(vnode.props['title']).toBe('No cache activity recorded');
    expect(collectText(vnode)).toContain('--');
  });
});
