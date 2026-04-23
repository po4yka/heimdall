import { describe, expect, it } from 'vitest';
import { StatsCards } from './StatsCards';

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

describe('StatsCards', () => {
  it('renders the depletion forecast card only when billing-block data carries the field', () => {
    const withForecast = StatsCards({
      totals: {
        sessions: 1,
        turns: 2,
        input: 100,
        output: 50,
        cache_read: 0,
        cache_creation: 0,
        reasoning_output: 0,
        cost: 1.25,
      },
      billingBlocks: {
        session_length_hours: 5,
        token_limit: 1_000_000,
        historical_max_tokens: 320_000,
        depletion_forecast: {
          primary_signal: {
            kind: 'billing_block',
            title: 'Billing block',
            used_percent: 62,
            projected_percent: 91,
            remaining_tokens: 380_000,
            remaining_percent: 38,
            end_time: '2026-04-23T12:00:00Z',
          },
          secondary_signals: [],
          summary_label: 'Billing block projected to reach 91% before reset',
          severity: 'danger',
        },
        blocks: [],
      },
    });
    const withoutForecast = StatsCards({
      totals: {
        sessions: 1,
        turns: 2,
        input: 100,
        output: 50,
        cache_read: 0,
        cache_creation: 0,
        reasoning_output: 0,
        cost: 1.25,
      },
      billingBlocks: {
        session_length_hours: 5,
        token_limit: 1_000_000,
        historical_max_tokens: 320_000,
        blocks: [],
      },
    });

    expect(collectText(withForecast).join(' ')).toContain('Depletion Forecast');
    expect(collectText(withForecast).join(' ')).toContain('Billing block');
    expect(collectText(withoutForecast).join(' ')).not.toContain('Depletion Forecast');
  });
});
