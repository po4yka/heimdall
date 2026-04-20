import { describe, expect, it } from 'vitest';
import type { OpenAiReconciliation } from '../state/types';
import { ReconciliationBlock } from './ReconciliationBlock';

function collectText(node: unknown): string[] {
  if (typeof node === 'string' || typeof node === 'number') return [String(node)];
  if (Array.isArray(node)) return node.flatMap(collectText);
  if (!node || typeof node !== 'object') return [];
  const vnode = node as { props?: { children?: unknown } };
  return collectText(vnode.props?.children);
}

describe('ReconciliationBlock', () => {
  it('renders reconciliation metrics when org usage is available', () => {
    const reconciliation: OpenAiReconciliation = {
      available: true,
      error: null,
      lookback_days: 7,
      start_date: '2026-04-13',
      end_date: '2026-04-20',
      estimated_local_cost: 1.23,
      api_usage_cost: 1.28,
      delta_cost: 0.05,
      api_input_tokens: 1000,
      api_output_tokens: 500,
      api_cached_input_tokens: 100,
      api_requests: 25,
    };

    const vnode = ReconciliationBlock({ reconciliation });
    const text = collectText(vnode).join(' ');

    expect(text).toContain('OpenAI Org Usage Reconciliation');
    expect(text).toContain('2026-04-13');
    expect(text).toContain('2026-04-20');
    expect(text).toContain('1.2300');
    expect(text).toContain('1.2800');
    expect(text).toContain('0.0500');
    expect(text).toContain('1,000');
    expect(text).toContain('500');
  });

  it('renders the unavailable state when the API response is missing', () => {
    const reconciliation = {
      available: false,
      error: 'Org usage unavailable',
      lookback_days: 7,
      start_date: '2026-04-13',
      end_date: '2026-04-20',
      estimated_local_cost: 0,
      api_usage_cost: 0,
      delta_cost: 0,
      api_input_tokens: 0,
      api_output_tokens: 0,
      api_cached_input_tokens: 0,
      api_requests: 0,
    } as OpenAiReconciliation;

    expect(collectText(ReconciliationBlock({ reconciliation }))).toContain('Org usage unavailable');
  });
});
