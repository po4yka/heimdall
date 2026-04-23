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
      contract_version: 1,
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
            recommended_key: 'p90',
            levels: [
              { key: 'p90', label: 'P90', limit_tokens: 800_000 },
              { key: 'p95', label: 'P95', limit_tokens: 900_000 },
              { key: 'max', label: 'Max', limit_tokens: 950_000 },
            ],
            note: 'Based on fewer than 10 completed blocks.',
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
    expect(text).toContain('Based on fewer than 10 completed blocks.');
  });

  it('omits hidden detail panels while keeping provider lanes visible', () => {
    const data: LiveMonitorResponse = {
      contract_version: 1,
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
    liveMonitorHiddenPanels.value = ['active_block', 'warnings'];

    const text = collectText(renderLiveMonitorView()).join(' ');
    expect(text).toContain('Provider Lanes');
    expect(text).toContain('Claude');
    expect(text).not.toContain('Active Block');
    expect(text).not.toContain('Warnings');
    expect(text).toContain('Context Window');
  });
});
