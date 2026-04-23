import { afterEach, describe, expect, it, vi } from 'vitest';
import type { LiveMonitorResponse } from '../state/types';

const baseResponse: LiveMonitorResponse = {
  contract_version: 1,
  generated_at: '2026-04-23T10:00:00Z',
  default_focus: 'claude',
  freshness: {
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
      today_cost_usd: 1.5,
      last_refresh: '2026-04-23T10:00:00Z',
      last_refresh_label: 'Updated just now',
    },
  ],
};

describe('live monitor store', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
    vi.resetModules();
  });

  it('hydrates persisted focus, density, and hidden panels from localStorage', async () => {
    vi.stubGlobal('localStorage', {
      getItem: vi.fn(() => JSON.stringify({
        focus: 'codex',
        density: 'compact',
        hiddenPanels: ['warnings', 'recent_session'],
      })),
      setItem: vi.fn(),
    });

    const store = await import('./store');
    store.hydrateLiveMonitorPreferences();

    expect(store.liveMonitorFocus.value).toBe('codex');
    expect(store.liveMonitorDensity.value).toBe('compact');
    expect(store.liveMonitorHiddenPanels.value).toEqual(['recent_session', 'warnings']);
  });

  it('persists preference changes immediately', async () => {
    const setItem = vi.fn();
    vi.stubGlobal('localStorage', {
      getItem: vi.fn(() => null),
      setItem,
    });

    const store = await import('./store');
    store.hydrateLiveMonitorPreferences();
    store.setLiveMonitorFocus('codex');
    store.setLiveMonitorDensity('compact');
    store.toggleLiveMonitorPanel('warnings');

    expect(setItem).toHaveBeenCalledWith(
      store.LIVE_MONITOR_PREFERENCE_KEY,
      JSON.stringify({
        focus: 'codex',
        density: 'compact',
        hiddenPanels: ['warnings'],
      })
    );
  });

  it('uses saved focus across refreshes and falls back to all when provider becomes unavailable', async () => {
    vi.stubGlobal('localStorage', {
      getItem: vi.fn(() => JSON.stringify({
        focus: 'codex',
        density: 'expanded',
        hiddenPanels: [],
      })),
      setItem: vi.fn(),
    });

    const store = await import('./store');
    store.hydrateLiveMonitorPreferences();
    store.setLiveMonitorData(baseResponse);
    expect(store.liveMonitorFocus.value).toBe('all');

    store.setLiveMonitorFocus('claude');
    store.setLiveMonitorData({
      ...baseResponse,
      default_focus: 'all',
      providers: [
        ...baseResponse.providers,
        {
          provider: 'codex',
          title: 'Codex',
          visual_state: 'healthy',
          source_label: 'Source: cli-rpc',
          warnings: [],
          today_cost_usd: 0.5,
          last_refresh: '2026-04-23T10:00:00Z',
          last_refresh_label: 'Updated just now',
        },
      ],
    });

    expect(store.liveMonitorFocus.value).toBe('claude');
  });
});
