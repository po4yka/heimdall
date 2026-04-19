import { afterEach, describe, expect, it, vi } from 'vitest';
import type { DashboardData } from './types';

type StoreModule = typeof import('./store');

interface LoadedStore {
  store: StoreModule;
  location: {
    pathname: string;
    search: string;
  };
  replaceState: ReturnType<typeof vi.fn>;
}

function makeDashboardData(allModels: string[]): DashboardData {
  return { all_models: allModels } as unknown as DashboardData;
}

async function loadStore(url: string): Promise<LoadedStore> {
  vi.resetModules();

  const current = new URL(url);
  const location = {
    pathname: current.pathname,
    search: current.search,
  };
  const replaceState = vi.fn((_state: unknown, _title: string, nextUrl: string) => {
    const resolved = new URL(nextUrl, current.origin);
    location.pathname = resolved.pathname;
    location.search = resolved.search;
  });

  vi.stubGlobal('window', { location });
  vi.stubGlobal('history', { replaceState });

  const store = await import('./store');
  return { store, location, replaceState };
}

afterEach(() => {
  vi.unstubAllGlobals();
  vi.resetModules();
});

describe('store url state', () => {
  it('restores filter and table state from the URL', async () => {
    const { store } = await loadStore(
      'http://localhost/dashboard?range=90d&provider=codex&models=zeta,alpha,ignored&project=heimdall&bucket=week&version_metric=tokens&agent_status_expanded=true&sessions_page=3&sessions_hidden=cost,project',
    );

    store.restoreDashboardStateFromUrl(['alpha', 'beta', 'zeta']);

    expect(store.selectedRange.value).toBe('90d');
    expect(store.selectedProvider.value).toBe('codex');
    expect([...store.selectedModels.value]).toEqual(['alpha', 'zeta']);
    expect(store.projectSearchQuery.value).toBe('heimdall');
    expect(store.selectedBucket.value).toBe('week');
    expect(store.versionDonutMetric.value).toBe('tokens');
    expect(store.agent_status_expanded.value).toBe(true);
    expect(store.sessionsTablePagination.value).toEqual({
      pageIndex: 2,
      pageSize: store.SESSIONS_PAGE_SIZE,
    });
    expect(store.sessionsTableColumnVisibility.value).toEqual({
      cost: false,
      project: false,
    });
  });

  it('falls back to defaults for invalid URL params', async () => {
    const { store } = await loadStore(
      'http://localhost/dashboard?range=bogus&provider=nope&bucket=month&version_metric=profit&sessions_page=0',
    );

    store.restoreDashboardStateFromUrl(['alpha', 'beta']);

    expect(store.selectedRange.value).toBe('30d');
    expect(store.selectedProvider.value).toBe('both');
    expect([...store.selectedModels.value]).toEqual(['alpha', 'beta']);
    expect(store.selectedBucket.value).toBe('day');
    expect(store.versionDonutMetric.value).toBe('cost');
    expect(store.agent_status_expanded.value).toBe(false);
    expect(store.sessionsTablePagination.value).toEqual({
      pageIndex: 0,
      pageSize: store.SESSIONS_PAGE_SIZE,
    });
    expect(store.sessionsTableColumnVisibility.value).toEqual({});
  });

  it('syncs non-default state back into a stable URL', async () => {
    const { store, location, replaceState } = await loadStore('http://localhost/dashboard');

    store.rawData.value = makeDashboardData(['alpha', 'beta', 'gamma']);
    store.selectedRange.value = '90d';
    store.selectedProvider.value = 'codex';
    store.selectedModels.value = new Set(['gamma', 'alpha']);
    store.projectSearchQuery.value = 'heimdall';
    store.versionDonutMetric.value = 'tokens';
    store.selectedBucket.value = 'week';
    store.agent_status_expanded.value = true;
    store.sessionsTablePagination.value = {
      pageIndex: 2,
      pageSize: store.SESSIONS_PAGE_SIZE,
    };
    store.sessionsTableColumnVisibility.value = {
      zeta: false,
      alpha: false,
      project: true,
    };

    store.syncDashboardUrl();

    expect(replaceState).toHaveBeenCalledWith(
      null,
      '',
      '/dashboard?range=90d&provider=codex&models=gamma%2Calpha&project=heimdall&version_metric=tokens&bucket=week&agent_status_expanded=1&sessions_page=3&sessions_hidden=alpha%2Czeta',
    );
    expect(location.search).toBe(
      '?range=90d&provider=codex&models=gamma%2Calpha&project=heimdall&version_metric=tokens&bucket=week&agent_status_expanded=1&sessions_page=3&sessions_hidden=alpha%2Czeta',
    );
  });

  it('omits default state from the URL', async () => {
    const { store, location, replaceState } = await loadStore('http://localhost/dashboard?range=7d');

    store.rawData.value = makeDashboardData(['alpha', 'beta']);
    store.selectedRange.value = '30d';
    store.selectedProvider.value = 'both';
    store.selectedModels.value = new Set(['alpha', 'beta']);
    store.projectSearchQuery.value = '';
    store.versionDonutMetric.value = 'cost';
    store.selectedBucket.value = 'day';
    store.agent_status_expanded.value = false;
    store.sessionsTablePagination.value = {
      pageIndex: 0,
      pageSize: store.SESSIONS_PAGE_SIZE,
    };
    store.sessionsTableColumnVisibility.value = {};

    store.syncDashboardUrl();

    expect(replaceState).toHaveBeenCalledWith(null, '', '/dashboard');
    expect(location.search).toBe('');
  });
});
