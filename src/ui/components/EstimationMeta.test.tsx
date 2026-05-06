import { describe, expect, it } from 'vitest';
import { EstimationMeta } from './EstimationMeta';

// Walks a vnode tree harvesting plain text plus common label-style props
// (`label`, `value`, `sub`, `subtitle`, `title`, `placeholder`). Components
// such as KpiCard expose their text via props rather than children, so a
// children-only walker would miss them.
function collectText(node: unknown): string[] {
  if (typeof node === 'string' || typeof node === 'number') return [String(node)];
  if (Array.isArray(node)) return node.flatMap(collectText);
  if (!node || typeof node !== 'object') return [];
  const vnode = node as { props?: Record<string, unknown> };
  const props = vnode.props ?? {};
  const labelProps = ['label', 'value', 'sub', 'subtitle', 'title', 'placeholder'] as const;
  const fromLabels = labelProps.flatMap(key => collectText(props[key]));
  return [...fromLabels, ...collectText(props['children'])];
}

describe('EstimationMeta', () => {
  it('renders breakdown summaries and mixed pricing versions', () => {
    const vnode = EstimationMeta({
      confidenceBreakdown: [['high', { sessions: 3, cost: 1.2 }]],
      billingModeBreakdown: [['estimated_local', { sessions: 2, cost: 0.8 }]],
      pricingVersions: ['2026-04-01', '2026-04-15'],
    });
    const text = collectText(vnode).join(' ');

    expect(text).toContain('Cost confidence');
    expect(text).toContain('High: 3');
    expect(text).toContain('Billing mode');
    expect(text).toContain('Estimated Local: 2');
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
