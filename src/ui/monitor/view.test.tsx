import { afterEach, describe, expect, it } from 'vitest';
import { renderLiveMonitorView } from './view';
import { liveMonitorData, liveMonitorDensity, liveMonitorFocus, liveMonitorHiddenPanels } from './store';
import type { LiveMonitorResponse } from '../state/types';

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

describe('renderLiveMonitorView', () => {
  afterEach(() => {
    liveMonitorData.value = null;
    liveMonitorFocus.value = 'all';
    liveMonitorDensity.value = 'expanded';
    liveMonitorHiddenPanels.value = [];
  });

  it('renders quota suggestions and depletion forecast only for providers carrying the field', () => {
    const data: LiveMonitorResponse = {
      contract_version: 2,
      generated_at: '2026-04-23T10:00:00Z',
      default_focus: 'all',
      freshness: {
        newest_provider_refresh: '2026-04-23T10:00:00Z',
        oldest_provider_refresh: '2026-04-23T09:58:00Z',
        stale_providers: [],
        has_stale_providers: false,
        refresh_state: 'current',
      },
      providers: [
        {
          provider: 'claude',
          title: 'Claude',
          visual_state: 'healthy',
          source_label: 'Source: oauth',
          warnings: [],
          today_cost_usd: 3.2,
          last_refresh: '2026-04-23T10:00:00Z',
          last_refresh_label: 'Updated just now',
          claude_admin: null,
          active_block: {
            start: '2026-04-23T07:00:00Z',
            end: '2026-04-23T12:00:00Z',
            first_timestamp: '2026-04-23T07:00:00Z',
            last_timestamp: '2026-04-23T08:00:00Z',
            cost_nanos: 1000,
            entry_count: 5,
            tokens: {
              input: 100_000,
              output: 50_000,
              cache_read: 0,
              cache_creation: 0,
              reasoning_output: 0,
            },
          },
          quota_suggestions: {
            sample_count: 4,
            population_count: 7,
            recommended_key: 'p90',
            sample_strategy: 'near_limit_hits',
            sample_label: '4 near-limit completed blocks',
            levels: [
              { key: 'p90', label: 'P90', limit_tokens: 800_000 },
              { key: 'p95', label: 'P95', limit_tokens: 900_000 },
              { key: 'max', label: 'Max', limit_tokens: 950_000 },
            ],
            note: 'Based on fewer than 10 completed blocks.',
          },
          predictive_insights: {
            rolling_hour_burn: {
              tokens_per_min: 2800,
              cost_per_hour_nanos: 1_200_000_000,
              coverage_minutes: 40,
              tier: 'moderate',
            },
            historical_envelope: {
              sample_count: 7,
              tokens: { average: 640_000, p50: 600_000, p75: 700_000, p90: 900_000, p95: 940_000 },
              cost_usd: { average: 3.8, p50: 3.1, p75: 4.6, p90: 5.4, p95: 5.8 },
              turns: { average: 14, p50: 12, p75: 16, p90: 20, p95: 22 },
            },
            limit_hit_analysis: {
              sample_count: 7,
              hit_count: 2,
              hit_rate: 2 / 7,
              threshold_tokens: 900_000,
              threshold_percent: 90,
              active_projected_hit: true,
              risk_level: 'high',
              summary_label: '2 of 7 completed blocks reached 90% of the configured limit · active block is on pace to join them',
            },
          },
          depletion_forecast: {
            primary_signal: {
              kind: 'billing_block',
              title: 'Billing block',
              used_percent: 58,
              projected_percent: 92,
              remaining_tokens: 420_000,
              remaining_percent: 42,
              end_time: '2026-04-23T12:00:00Z',
            },
            secondary_signals: [
              {
                kind: 'primary_window',
                title: 'Primary window',
                used_percent: 64,
                remaining_percent: 36,
                resets_in_minutes: 40,
                pace_label: 'Steady',
              },
            ],
            summary_label: 'Billing block projected to reach 92% before reset',
            severity: 'danger',
          },
        },
        {
          provider: 'codex',
          title: 'Codex',
          visual_state: 'healthy',
          source_label: 'Source: cli-rpc',
          warnings: [],
          today_cost_usd: 1.1,
          last_refresh: '2026-04-23T10:00:00Z',
          last_refresh_label: 'Updated just now',
        },
      ],
    };

    liveMonitorData.value = data;
    liveMonitorFocus.value = 'all';

    const text = collectText(renderLiveMonitorView()).join(' ');
    expect(text).toContain('Suggested Quotas');
    expect(text).toContain('Depletion Forecast');
    expect(text).toContain('Billing block projected to reach 92% before reset');
    expect(text).toContain('SUPPORTING SIGNALS');
    expect(text).toContain('P90');
    expect(text).toContain('[RECOMMENDED]');
    expect(text).toContain('Predictive Signals');
    expect(text).toContain('ROLLING 1H BURN');
    expect(text).toContain('Based on fewer than 10 completed blocks.');
  });

  it('omits hidden detail panels while keeping provider lanes visible', () => {
    const data: LiveMonitorResponse = {
      contract_version: 2,
      generated_at: '2026-04-23T10:00:00Z',
      default_focus: 'all',
      freshness: {
        newest_provider_refresh: '2026-04-23T10:00:00Z',
        oldest_provider_refresh: '2026-04-23T09:58:00Z',
        stale_providers: [],
        has_stale_providers: false,
        refresh_state: 'current',
      },
      providers: [
        {
          provider: 'claude',
          title: 'Claude',
          visual_state: 'healthy',
          source_label: 'Source: oauth',
          warnings: ['Needs attention'],
          today_cost_usd: 3.2,
          last_refresh: '2026-04-23T10:00:00Z',
          last_refresh_label: 'Updated just now',
          claude_admin: null,
          active_block: {
            start: '2026-04-23T07:00:00Z',
            end: '2026-04-23T12:00:00Z',
            first_timestamp: '2026-04-23T07:00:00Z',
            last_timestamp: '2026-04-23T08:00:00Z',
            cost_nanos: 1000,
            entry_count: 5,
            tokens: {
              input: 100_000,
              output: 50_000,
              cache_read: 0,
              cache_creation: 0,
              reasoning_output: 0,
            },
          },
          context_window: {
            total_input_tokens: 200_000,
            context_window_size: 400_000,
            pct: 0.5,
            severity: 'ok',
          },
        },
      ],
    };

    liveMonitorData.value = data;
    liveMonitorFocus.value = 'all';
    liveMonitorHiddenPanels.value = ['active_block', 'predictive_insights', 'warnings'];

    const text = collectText(renderLiveMonitorView()).join(' ');
    expect(text).toContain('Provider Lanes');
    expect(text).toContain('Claude');
    expect(text).not.toContain('Active Block');
    expect(text).not.toContain('Predictive Signals');
    expect(text).not.toContain('Warnings');
    expect(text).toContain('Context Window');
  });

  it('renders Claude admin fallback metrics in the provider lane', () => {
    const data: LiveMonitorResponse = {
      contract_version: 2,
      generated_at: '2026-04-23T10:00:00Z',
      default_focus: 'claude',
      freshness: {
        newest_provider_refresh: '2026-04-23T10:00:00Z',
        oldest_provider_refresh: '2026-04-23T10:00:00Z',
        stale_providers: [],
        has_stale_providers: false,
        refresh_state: 'current',
      },
      providers: [
        {
          provider: 'claude',
          title: 'Claude',
          visual_state: 'degraded',
          source_label: 'Source: admin',
          warnings: ['Using org-wide Anthropic admin analytics fallback.'],
          identity_label: 'Acme Org',
          today_cost_usd: 1.1,
          last_refresh: '2026-04-23T10:00:00Z',
          last_refresh_label: 'Updated just now',
          claude_admin: {
            organization_name: 'Acme Org',
            lookback_days: 30,
            start_date: '2026-03-21',
            end_date: '2026-04-19',
            data_latency_note: 'Org-wide · UTC daily aggregation · up to 1 hour delayed',
            today_active_users: 9,
            today_sessions: 21,
            lookback_lines_accepted: 3000,
            lookback_estimated_cost_usd: 44.5,
            lookback_input_tokens: 1,
            lookback_output_tokens: 2,
            lookback_cache_read_tokens: 3,
            lookback_cache_creation_tokens: 4,
          },
        },
      ],
    };

    liveMonitorData.value = data;
    liveMonitorFocus.value = 'claude';

    const text = collectText(renderLiveMonitorView()).join(' ');
    expect(text).toContain('Source: admin');
    expect(text).toContain('Active Users Today');
    expect(text).toContain('Accepted Lines');
    expect(text).toContain('Acme Org');
  });
});
