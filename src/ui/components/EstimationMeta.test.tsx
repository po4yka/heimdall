import { describe, expect, it } from 'vitest';
import { EstimationMeta } from './EstimationMeta';

function collectText(node: unknown): string[] {
  if (typeof node === 'string' || typeof node === 'number') return [String(node)];
  if (Array.isArray(node)) return node.flatMap(collectText);
  if (!node || typeof node !== 'object') return [];
  const vnode = node as { props?: { children?: unknown } };
  return collectText(vnode.props?.children);
}

describe('EstimationMeta', () => {
  it('renders breakdown summaries and mixed pricing versions', () => {
    const vnode = EstimationMeta({
      confidenceBreakdown: [['high', { sessions: 3, cost: 1.2 }]],
      billingModeBreakdown: [['estimated_local', { sessions: 2, cost: 0.8 }]],
      pricingVersions: ['2026-04-01', '2026-04-15'],
    });
    const text = collectText(vnode).join(' ');

    expect(text).toContain('Cost Confidence');
    expect(text).toContain('high 3');
    expect(text).toContain('Billing Mode');
    expect(text).toContain('estimated_local 2');
    expect(text).toContain('mixed (2)');
  });

  it('falls back to n/a when no metadata is available', () => {
    const text = collectText(
      EstimationMeta({
        confidenceBreakdown: [],
        billingModeBreakdown: [],
        pricingVersions: [],
      })
    ).join(' ');

    expect(text).toContain('n/a');
  });
});
