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
  it('renders predictive billing-block cards only when the fields are present', () => {
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
        predictive_insights: {
          rolling_hour_burn: {
            tokens_per_min: 3200,
            cost_per_hour_nanos: 1_500_000_000,
            coverage_minutes: 45,
            tier: 'moderate',
          },
          historical_envelope: {
            sample_count: 8,
            tokens: { average: 600_000, p50: 500_000, p75: 700_000, p90: 900_000, p95: 950_000 },
            cost_usd: { average: 4.2, p50: 3.8, p75: 5.0, p90: 6.1, p95: 6.8 },
            turns: { average: 18, p50: 16, p75: 20, p90: 24, p95: 26 },
          },
          limit_hit_analysis: {
            sample_count: 8,
            hit_count: 3,
            hit_rate: 0.375,
            threshold_tokens: 900_000,
            threshold_percent: 90,
            active_projected_hit: true,
            risk_level: 'high',
            summary_label: '3 of 8 completed blocks reached 90% of the configured limit · active block is on pace to join them',
          },
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
    expect(collectText(withForecast).join(' ')).toContain('Predictive Signals');
    expect(collectText(withForecast).join(' ')).toContain('ROLLING 1H BURN');
    expect(collectText(withForecast).join(' ')).toContain('LIMIT-HIT RISK');
    expect(collectText(withoutForecast).join(' ')).not.toContain('Depletion Forecast');
    expect(collectText(withoutForecast).join(' ')).not.toContain('Predictive Signals');
  });
});
