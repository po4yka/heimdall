import { afterEach, describe, expect, it, vi } from 'vitest';
import type {
  AgentStatusSnapshot,
  BillingBlocksResponse,
  ClaudeUsageResponse,
  CommunitySignal,
  ContextWindowResponse,
  CostReconciliationResponse,
  DashboardData,
  HeatmapData,
  UsageWindowsResponse,
} from '../state/types';

const viewSpies = vi.hoisted(() => ({
  refreshSectionVisibility: vi.fn(),
  renderActivityHeatmap: vi.fn(),
  renderAgentStatus: vi.fn(),
  renderClaudeUsage: vi.fn(),
  renderCostReconciliation: vi.fn(),
  renderDashboardView: vi.fn(),
  renderUsageWindows: vi.fn(),
}));

vi.mock('./view', () => viewSpies);

type StoreModule = typeof import('../state/store');
type RuntimeModule = typeof import('./runtime');

interface LoadedRuntimeContext {
  store: StoreModule;
  runtimeModule: RuntimeModule;
  addEventListener: ReturnType<typeof vi.fn>;
  intervals: Array<{ handler: TimerHandler; delay: number }>;
}

function makeDashboardData(): DashboardData {
  return {
    all_models: ['sonnet'],
    provider_breakdown: [],
    confidence_breakdown: [],
    billing_mode_breakdown: [],
    daily_by_model: [],
    sessions_all: [],
    subagent_summary: {
      parent_turns: 0,
      parent_input: 0,
      parent_output: 0,
      subagent_turns: 0,
      subagent_input: 0,
      subagent_output: 0,
      unique_agents: 0,
    },
    entrypoint_breakdown: [],
    service_tiers: [],
    tool_summary: [],
    mcp_summary: [],
    hourly_distribution: [],
    git_branch_summary: [],
    version_summary: [],
    daily_by_project: [],
    weekly_by_model: [],
    openai_reconciliation: null,
    official_sync: {
      available: false,
      last_sync_at: null,
      latest_success_at: null,
      total_runs: 0,
      total_records: 0,
      sources_success: 0,
      sources_error: 0,
      sources_skipped: 0,
      sources: [],
      record_counts: [],
    },
    generated_at: '2026-04-19T12:00:00Z',
    cache_efficiency: {
      cache_read_tokens: 0,
      cache_write_tokens: 0,
      input_tokens: 0,
      output_tokens: 0,
      cache_read_cost_nanos: 0,
      cache_write_cost_nanos: 0,
      input_cost_nanos: 0,
      output_cost_nanos: 0,
      cache_hit_rate: null,
    },
  };
}

const dashboardData = makeDashboardData();
const usageWindows: UsageWindowsResponse = {
  available: true,
  source: 'oauth',
};
const claudeUsage: ClaudeUsageResponse = {
  available: true,
  last_run: {
    id: 1,
    captured_at: '2026-04-19T12:00:00Z',
    status: 'ok',
    exit_code: 0,
    invocation_mode: 'manual',
    period: 'day',
    parser_version: 'v1',
  },
  latest_snapshot: null,
};
const agentStatus: AgentStatusSnapshot = {
  claude: null,
  openai: null,
  fetched_at: '2026-04-19T12:00:00Z',
};
const communitySignal: CommunitySignal = {
  fetched_at: '2026-04-19T12:00:00Z',
  claude: [],
  openai: [],
  enabled: true,
};
const heatmap: HeatmapData = {
  cells: [],
  max_cost_nanos: 0,
  max_call_count: 0,
  active_days: 0,
  total_cost_nanos: 0,
  period: 'all',
  tz_offset_min: 0,
};
const billingBlocks: BillingBlocksResponse = {
  session_length_hours: 5,
  token_limit: null,
  historical_max_tokens: 0,
  blocks: [],
};
const contextWindow: ContextWindowResponse = {
  enabled: false,
};
const reconciliation: CostReconciliationResponse = {
  enabled: true,
  period: 'month',
  hook_total_nanos: 0,
  local_total_nanos: 0,
  divergence_pct: 0,
  breakdown: [],
};

function makeResponse(data: unknown, ok = true, status = 200): Response {
  return {
    ok,
    status,
    json: async () => data,
  } as Response;
}

async function flushAsyncWork(): Promise<void> {
  await Promise.resolve();
  await Promise.resolve();
}

async function loadRuntimeContext(url = 'http://localhost/dashboard'): Promise<LoadedRuntimeContext> {
  vi.resetModules();
  Object.values(viewSpies).forEach(spy => spy.mockReset());

  const current = new URL(url);
  const addEventListener = vi.fn();
  const intervals: Array<{ handler: TimerHandler; delay: number }> = [];

  vi.stubGlobal('window', {
    location: { pathname: current.pathname, search: current.search },
    addEventListener,
    Date,
  });
  vi.stubGlobal('history', { replaceState: vi.fn() });
  vi.stubGlobal('setInterval', vi.fn((handler: TimerHandler, delay?: number) => {
    intervals.push({ handler, delay: delay ?? 0 });
    return intervals.length;
  }));

  const store = await import('../state/store');
  const runtimeModule = await import('./runtime');

  return { store, runtimeModule, addEventListener, intervals };
}

afterEach(() => {
  vi.unstubAllGlobals();
  vi.resetModules();
  Object.values(viewSpies).forEach(spy => spy.mockReset());
});

describe('dashboard runtime', () => {
  it('loads dashboard data and delegates rendering on first fetch', async () => {
    const { store, runtimeModule } = await loadRuntimeContext();
    const restoreStateSpy = vi.spyOn(store, 'restoreDashboardStateFromUrl');

    vi.stubGlobal('fetch', vi.fn(async (input: RequestInfo | URL) => {
      if (String(input) === '/api/data') return makeResponse(dashboardData);
      throw new Error(`Unexpected fetch ${String(input)}`);
    }));

    const runtime = runtimeModule.createDashboardRuntime();
    await runtime.loadData();

    expect(store.rawData.value).toEqual(dashboardData);
    expect(restoreStateSpy).toHaveBeenCalledWith(['sonnet']);
    expect(viewSpies.renderDashboardView).toHaveBeenCalledWith(
      dashboardData,
      expect.any(Function),
      expect.any(Function),
      expect.any(Function),
      expect.any(Function)
    );
  });

  it('starts polling all dashboard feeds and refreshes section visibility on tab changes', async () => {
    const { store, runtimeModule, addEventListener, intervals } = await loadRuntimeContext();
    const fetchSpy = vi.fn(async (input: RequestInfo | URL) => {
      const url = String(input);
      if (url === '/api/data') return makeResponse(dashboardData);
      if (url === '/api/usage-windows') return makeResponse(usageWindows);
      if (url === '/api/claude-usage') return makeResponse(claudeUsage);
      if (url === '/api/agent-status') return makeResponse(agentStatus);
      if (url === '/api/community-signal') return makeResponse(communitySignal);
      if (url.startsWith('/api/heatmap?period=all&tz_offset_min=')) return makeResponse(heatmap);
      if (url === '/api/billing-blocks') return makeResponse(billingBlocks);
      if (url === '/api/context-window') return makeResponse(contextWindow);
      if (url === '/api/cost-reconciliation?period=month') return makeResponse(reconciliation);
      throw new Error(`Unexpected fetch ${url}`);
    });

    vi.stubGlobal('fetch', fetchSpy);

    const runtime = runtimeModule.createDashboardRuntime();
    runtime.start();
    await flushAsyncWork();

    expect(addEventListener).toHaveBeenCalledWith('popstate', expect.any(Function));
    expect(intervals).toHaveLength(6);
    expect(fetchSpy).toHaveBeenCalledWith('/api/data');
    expect(fetchSpy).toHaveBeenCalledWith('/api/usage-windows');
    expect(fetchSpy).toHaveBeenCalledWith('/api/claude-usage');
    expect(fetchSpy).toHaveBeenCalledWith('/api/agent-status');
    expect(fetchSpy).toHaveBeenCalledWith('/api/community-signal');
    expect(fetchSpy).toHaveBeenCalledWith('/api/billing-blocks');
    expect(fetchSpy).toHaveBeenCalledWith('/api/context-window');
    expect(fetchSpy).toHaveBeenCalledWith('/api/cost-reconciliation?period=month');
    expect(
      fetchSpy.mock.calls.some(([url]) =>
        String(url).startsWith('/api/heatmap?period=all&tz_offset_min=')
      )
    ).toBe(true);
    expect(viewSpies.renderUsageWindows).toHaveBeenCalledWith(
      usageWindows,
      null,
      expect.any(Function),
      expect.any(Function),
      expect.any(Function)
    );
    expect(viewSpies.renderClaudeUsage).toHaveBeenCalledWith(claudeUsage);
    expect(viewSpies.renderActivityHeatmap).toHaveBeenCalledWith(heatmap);
    expect(viewSpies.renderCostReconciliation).toHaveBeenCalled();

    runtime.handleDashboardTabChange('tables');

    expect(store.activeDashboardTab.value).toBe('tables');
    expect(viewSpies.refreshSectionVisibility).toHaveBeenCalled();
  });
});
