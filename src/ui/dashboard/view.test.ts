import { afterEach, describe, expect, it, vi } from 'vitest';
import type { DashboardData, UsageWindowsResponse } from '../state/types';

const renderSpy = vi.fn();

vi.mock('preact', () => ({
  render: renderSpy,
}));

type StoreModule = typeof import('../state/store');
type ViewModule = typeof import('./view');

interface FakeElement {
  id: string;
  dataset: Record<string, string>;
  style: Record<string, string>;
  textContent: string;
  closest: (selector: string) => Element | null;
}

interface LoadedViewContext {
  store: StoreModule;
  view: ViewModule;
  elements: Record<string, FakeElement>;
}

const DASHBOARD_ELEMENT_IDS = [
  'usage-windows',
  'claude-usage',
  'agent-status',
  'estimation-meta',
  'official-sync',
  'openai-reconciliation',
  'stats-row',
  'daily-chart-card',
  'chart-daily',
  'daily-chart-title',
  'model-chart-card',
  'chart-model',
  'project-chart-card',
  'chart-project',
  'subagent-summary',
  'entrypoint-breakdown',
  'service-tiers',
  'tool-summary',
  'mcp-summary',
  'branch-summary',
  'version-summary',
  'hourly-chart',
  'activity-heatmap',
  'cost-reconciliation',
  'model-cost-mount',
  'sessions-mount',
  'project-cost-mount',
] as const;

function makeFakeElement(id: string): FakeElement {
  return {
    id,
    dataset: {},
    style: {},
    textContent: '',
    // setSectionVisibility now walks up via .closest('.widget-body') →
    // .closest('.grid-stack-item'); the test stubs aren't inside a grid
    // so neither selector matches and we short-circuit at the first
    // call. Returning null here is correct behaviour.
    closest: () => null,
  };
}

// Walk a vnode tree returned to render() and collect every user-visible
// string. preact's render is mocked, so the fake elements never get a
// real textContent — we synthesize it from the vnode passed to render.
// When we hit a vnode whose `type` is a function (a presentational
// component without hooks), we invoke it with its props and recurse
// into the resulting tree, so labels created inside the component
// body are visible to the walker.
function vnodeText(node: unknown, depth = 0): string[] {
  if (typeof node === 'string' || typeof node === 'number') return [String(node)];
  if (Array.isArray(node)) return node.flatMap(item => vnodeText(item, depth));
  if (!node || typeof node !== 'object') return [];
  if (depth > 20) return [];
  const vnode = node as { type?: unknown; props?: Record<string, unknown> };
  const props = vnode.props ?? {};
  if (typeof vnode.type === 'function') {
    try {
      const rendered = (vnode.type as (props: Record<string, unknown>) => unknown)(props);
      return vnodeText(rendered, depth + 1);
    } catch {
      return vnodeText(props['children'], depth + 1);
    }
  }
  const labelProps = ['label', 'value', 'subtitle', 'title', 'placeholder'] as const;
  const fromLabels = labelProps.flatMap(key => vnodeText(props[key], depth + 1));
  return [...fromLabels, ...vnodeText(props['children'], depth + 1)];
}

function lastRenderedTextFor(elements: Record<string, FakeElement>, id: string): string {
  const target = elements[id];
  for (let i = renderSpy.mock.calls.length - 1; i >= 0; i--) {
    const call = renderSpy.mock.calls[i];
    if (!call) continue;
    if (call[1] === target) {
      return vnodeText(call[0]).join(' ');
    }
  }
  return elements[id]?.textContent ?? '';
}

function makeDashboardData(): DashboardData {
  return {
    all_models: ['sonnet', 'haiku'],
    provider_breakdown: [],
    confidence_breakdown: [],
    billing_mode_breakdown: [],
    daily_by_model: [
      {
        day: '2026-04-19',
        provider: 'claude',
        model: 'sonnet',
        input: 10,
        output: 5,
        cache_read: 2,
        cache_creation: 1,
        reasoning_output: 0,
        turns: 3,
        cost: 1.25,
        input_cost: 0.6,
        output_cost: 0.45,
        cache_read_cost: 0.1,
        cache_write_cost: 0.1,
        credits: null,
      },
      {
        day: '2026-04-19',
        provider: 'codex',
        model: 'haiku',
        input: 99,
        output: 50,
        cache_read: 0,
        cache_creation: 0,
        reasoning_output: 0,
        turns: 9,
        cost: 9.5,
        input_cost: 4,
        output_cost: 4,
        cache_read_cost: 0.75,
        cache_write_cost: 0.75,
        credits: null,
      },
    ],
    sessions_all: [
      {
        session_id: 'claude-1',
        provider: 'claude',
        project: 'heimdall',
        display_name: 'Heimdall UI',
        last: '2026-04-19 12:00',
        last_date: '2026-04-19',
        duration_min: 30,
        model: 'sonnet',
        turns: 3,
        input: 10,
        output: 5,
        cache_read: 2,
        cache_creation: 1,
        reasoning_output: 0,
        cost: 1.25,
        is_billable: true,
        pricing_version: 'v1',
        billing_mode: 'estimated_local',
        cost_confidence: 'medium',
        subagent_count: 0,
        subagent_turns: 0,
        title: null,
        cache_hit_ratio: 0.2,
        tokens_per_min: 0.5,
        credits: null,
      },
      {
        session_id: 'codex-1',
        provider: 'codex',
        project: 'other',
        display_name: 'Other Project',
        last: '2026-04-19 12:30',
        last_date: '2026-04-19',
        duration_min: 45,
        model: 'haiku',
        turns: 9,
        input: 99,
        output: 50,
        cache_read: 0,
        cache_creation: 0,
        reasoning_output: 0,
        cost: 9.5,
        is_billable: true,
        pricing_version: 'v1',
        billing_mode: 'estimated_local',
        cost_confidence: 'high',
        subagent_count: 0,
        subagent_turns: 0,
        title: null,
        cache_hit_ratio: 0,
        tokens_per_min: 1,
        credits: null,
      },
    ],
    subagent_summary: {
      parent_turns: 1,
      parent_input: 1,
      parent_output: 1,
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
      cache_read_tokens: 2,
      cache_write_tokens: 1,
      input_tokens: 109,
      output_tokens: 55,
      cache_read_cost_nanos: 100,
      cache_write_cost_nanos: 100,
      input_cost_nanos: 4600,
      output_cost_nanos: 4450,
      cache_hit_rate: 0.2,
    },
    agent_telemetry: {
      totals: {
        sessions: 0,
        total_tokens: 0,
        cost_usd: 0,
        input_tokens: 0,
        cache_create_tokens: 0,
        cache_read_tokens: 0,
        output_tokens: 0,
        tool_uses: 0,
        duration_s: 0,
      },
      timeline: [],
      distribution: [],
      top_sessions: [],
      spawn_batches: [],
      spawn_batches_summary: {
        batch_count: 0,
        avg_size: 0,
        max_size: 0,
        batched_agents: 0,
      },
      tool_spectrum: [],
      detected: [],
    },
  };
}

async function loadViewContext(url = 'http://localhost/dashboard'): Promise<LoadedViewContext> {
  vi.resetModules();
  renderSpy.mockClear();

  const current = new URL(url);
  const elements = Object.fromEntries(
    DASHBOARD_ELEMENT_IDS.map(id => [id, makeFakeElement(id)])
  ) as Record<string, FakeElement>;

  vi.stubGlobal('window', {
    location: { pathname: current.pathname, search: current.search },
  });
  vi.stubGlobal('history', { replaceState: vi.fn() });
  vi.stubGlobal('document', {
    getElementById: (id: string) => elements[id] ?? null,
  });

  const store = await import('../state/store');
  const view = await import('./view');

  return { store, view, elements };
}

afterEach(() => {
  vi.unstubAllGlobals();
  vi.resetModules();
  renderSpy.mockClear();
});

describe('dashboard view', () => {
  it('filters dashboard data into the active project/model slices', async () => {
    const { store, view, elements } = await loadViewContext();
    const data = makeDashboardData();

    store.selectedModels.value = new Set(['sonnet']);
    store.selectedProvider.value = 'claude';
    store.projectSearchQuery.value = 'heimdall';
    store.selectedRange.value = 'all';
    store.selectedBucket.value = 'day';

    view.renderDashboardView(data, vi.fn(), vi.fn(), vi.fn(), vi.fn());

    expect(store.lastFilteredSessions.value).toHaveLength(1);
    expect(store.lastFilteredSessions.value[0]?.session_id).toBe('claude-1');
    expect(store.lastByProject.value).toHaveLength(1);
    expect(store.lastByProject.value[0]?.display_name).toBe('Heimdall UI');
    expect(elements['daily-chart-title']?.textContent).toBe(
      'Daily token usage – All Time (claude)'
    );
    expect(elements['stats-row']?.dataset['hasContent']).toBe('1');
    expect(elements['model-cost-mount']?.dataset['hasContent']).toBe('1');
    expect(renderSpy).toHaveBeenCalled();
  });

  it('renders usage windows and updates the plan badge from identity data', async () => {
    const { store, view, elements } = await loadViewContext();
    const statusMessages: Array<{ message: string; isError: boolean }> = [];
    const previousValues: Array<number | null> = [];
    const usage: UsageWindowsResponse = {
      available: true,
      source: 'oauth',
      session: {
        used_percent: 100,
        resets_at: '2026-04-19T13:00:00Z',
        resets_in_minutes: 42,
      },
      identity: {
        plan: 'pro',
        rate_limit_tier: 'standard',
      },
    };

    view.renderUsageWindows(
      usage,
      12,
      value => previousValues.push(value),
      (message, isError = false) => statusMessages.push({ message, isError }),
      vi.fn()
    );

    expect(elements['usage-windows']?.dataset['hasContent']).toBe('1');
    expect(elements['usage-windows']?.style['display']).toBe('grid');
    expect(previousValues).toEqual([0]);
    expect(statusMessages).toEqual([
      { message: 'Session depleted - resets in 42m', isError: true },
    ]);
    expect(store.planBadge.value).toBe('Pro');
  });

  it('renders Claude admin fallback cards in the existing usage lane', async () => {
    const { view, elements } = await loadViewContext();
    const usage: UsageWindowsResponse = {
      available: true,
      source: 'admin',
      admin_fallback: {
        organization_name: 'Acme Org',
        lookback_days: 30,
        start_date: '2026-03-21',
        end_date: '2026-04-19',
        data_latency_note: 'Org-wide · UTC daily aggregation · up to 1 hour delayed',
        today_active_users: 12,
        today_sessions: 34,
        lookback_lines_accepted: 4567,
        lookback_estimated_cost_usd: 89.12,
        lookback_input_tokens: 1000,
        lookback_output_tokens: 500,
        lookback_cache_read_tokens: 250,
        lookback_cache_creation_tokens: 100,
      },
    };

    view.renderUsageWindows(usage, null, () => {}, () => {}, () => {});

    const text = lastRenderedTextFor(elements, 'usage-windows');
    expect(text).toContain('Active users today');
    expect(text).toContain('Sessions today');
    expect(text).toContain('Accepted lines (30d)');
    expect(text).toContain('Estimated spend (30d)');
    expect(text).toContain('Acme Org');
  });
});
