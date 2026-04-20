import { downloadCSV } from '../lib/csv';
import { clearStatus, setStatus } from '../lib/status';
import type {
  AgentStatusSnapshot,
  BillingBlocksResponse,
  ClaudeUsageResponse,
  CommunitySignal,
  ContextWindowResponse,
  CostReconciliationResponse,
  DashboardData,
  HeatmapData,
  ProjectAgg,
  UsageWindowsResponse,
} from '../state/types';
import {
  activeDashboardTab,
  billingBlocksData,
  contextWindowData,
  costReconciliationData,
  loadState,
  projectSearchQuery,
  rawData,
  restoreDashboardStateFromUrl,
  selectedModels,
  syncDashboardUrl,
  type DashboardTab,
} from '../state/store';
import {
  refreshSectionVisibility,
  renderActivityHeatmap,
  renderAgentStatus,
  renderClaudeUsage,
  renderCostReconciliation,
  renderDashboardView,
  renderUsageWindows,
} from './view';

export interface DashboardRuntime {
  applyFilter(): void;
  handleDashboardTabChange(tab: DashboardTab): void;
  loadData(force?: boolean): Promise<void>;
  start(): void;
}

type RuntimeGuard =
  | 'loadData'
  | 'loadUsageWindows'
  | 'loadClaudeUsage'
  | 'loadHeatmap'
  | 'loadAgentStatus'
  | 'loadCommunitySignal'
  | 'loadBillingBlocks'
  | 'loadContextWindow'
  | 'loadCostReconciliation';

interface RuntimeState {
  previousSessionPercent: number | null;
  lastCommunitySignal: CommunitySignal | null;
  lastAgentStatusSnapshot: AgentStatusSnapshot | null;
  inFlight: Record<RuntimeGuard, boolean>;
}

function createRuntimeState(): RuntimeState {
  return {
    previousSessionPercent: null,
    lastCommunitySignal: null,
    lastAgentStatusSnapshot: null,
    inFlight: {
      loadData: false,
      loadUsageWindows: false,
      loadClaudeUsage: false,
      loadHeatmap: false,
      loadAgentStatus: false,
      loadCommunitySignal: false,
      loadBillingBlocks: false,
      loadContextWindow: false,
      loadCostReconciliation: false,
    },
  };
}

async function runExclusive(
  state: RuntimeState,
  guard: RuntimeGuard,
  task: () => Promise<void>,
  force = false
): Promise<void> {
  if (state.inFlight[guard] && !force) return;
  state.inFlight[guard] = true;
  try {
    await task();
  } finally {
    state.inFlight[guard] = false;
  }
}

async function fetchJson<T>(url: string): Promise<T | null> {
  const response = await fetch(url);
  if (!response.ok) return null;
  return (await response.json()) as T;
}

function toggleModelSelection(allModels: string[], currentSelection: Set<string>, model: string): Set<string> {
  const isSoleSelection = currentSelection.size === 1 && currentSelection.has(model);
  return isSoleSelection ? new Set(allModels) : new Set([model]);
}

function toggleProjectSearch(currentQuery: string, project: string): string {
  const normalized = project.toLowerCase().trim();
  return currentQuery === normalized ? '' : normalized;
}

function exportProjectRowsCSV(filename: string, rowsData: ProjectAgg[]): void {
  const header = [
    'Project',
    'Sessions',
    'Turns',
    'Input',
    'Output',
    'Cached Input',
    'Cache Creation',
    'Reasoning Output',
    'Est. Cost',
  ];
  const rows = rowsData.map(project => [
    project.project,
    project.sessions,
    project.turns,
    project.input,
    project.output,
    project.cache_read,
    project.cache_creation,
    project.reasoning_output,
    project.cost.toFixed(4),
  ]);
  downloadCSV(filename, header, rows);
}

function buildProjectAggregates(data: DashboardData | null): ProjectAgg[] {
  const byProject = data
    ? data.sessions_all.reduce<Map<string, ProjectAgg>>((acc, session) => {
        if (!selectedModels.value.has(session.model)) return acc;
        const current = acc.get(session.project) ?? {
          project: session.project,
          display_name: session.display_name || session.project,
          input: 0,
          output: 0,
          cache_read: 0,
          cache_creation: 0,
          reasoning_output: 0,
          turns: 0,
          sessions: 0,
          cost: 0,
          credits: null,
        };
        current.input += session.input;
        current.output += session.output;
        current.cache_read += session.cache_read;
        current.cache_creation += session.cache_creation;
        current.reasoning_output += session.reasoning_output;
        current.turns += session.turns;
        current.sessions += 1;
        current.cost += session.cost;
        if (session.credits != null) {
          current.credits = (current.credits ?? 0) + session.credits;
        }
        acc.set(session.project, current);
        return acc;
      }, new Map())
    : new Map<string, ProjectAgg>();

  return Array.from(byProject.values()).sort(
    (left, right) => (right.input + right.output) - (left.input + left.output)
  );
}

function exportSessionsCSV(): void {
  const header = [
    'Session',
    'Provider',
    'Project',
    'Last Active',
    'Duration (min)',
    'Model',
    'Turns',
    'Input',
    'Output',
    'Cached Input',
    'Cache Creation',
    'Reasoning Output',
    'Est. Cost',
  ];
  const rows = rawData.value
    ? rawData.value.sessions_all
        .filter(session => selectedModels.value.has(session.model))
        .map(session => [
          session.session_id,
          session.provider,
          session.project,
          session.last,
          session.duration_min,
          session.model,
          session.turns,
          session.input,
          session.output,
          session.cache_read,
          session.cache_creation,
          session.reasoning_output,
          session.cost.toFixed(4),
        ])
    : [];
  downloadCSV('sessions', header, rows);
}

function exportProjectsCSV(): void {
  exportProjectRowsCSV('projects', buildProjectAggregates(rawData.value));
}

function createUsageWindowsLoader(state: RuntimeState): () => Promise<void> {
  return () =>
    runExclusive(state, 'loadUsageWindows', async () => {
      try {
        const data = await fetchJson<UsageWindowsResponse>('/api/usage-windows');
        if (!data) return;
        renderUsageWindows(
          data,
          state.previousSessionPercent,
          value => {
            state.previousSessionPercent = value;
          },
          (message, isError = false) => {
            setStatus(
              'rate-windows',
              isError ? 'error' : 'success',
              message,
              isError ? undefined : 4000
            );
          },
          () => clearStatus('rate-windows')
        );
      } catch {
        // Silent by design.
      }
    });
}

function createClaudeUsageLoader(state: RuntimeState): () => Promise<void> {
  return () =>
    runExclusive(state, 'loadClaudeUsage', async () => {
      try {
        const data = await fetchJson<ClaudeUsageResponse>('/api/claude-usage');
        if (data) renderClaudeUsage(data);
      } catch {
        // Silent by design.
      }
    });
}

function createAgentStatusLoader(state: RuntimeState): () => Promise<void> {
  return () =>
    runExclusive(state, 'loadAgentStatus', async () => {
      try {
        const data = await fetchJson<AgentStatusSnapshot>('/api/agent-status');
        if (!data) return;
        state.lastAgentStatusSnapshot = data;
        renderAgentStatus(data, state.lastCommunitySignal);
      } catch {
        // Silent by design.
      }
    });
}

function createCommunitySignalLoader(state: RuntimeState): () => Promise<void> {
  return () =>
    runExclusive(state, 'loadCommunitySignal', async () => {
      try {
        const data = await fetchJson<CommunitySignal>('/api/community-signal');
        if (!data) return;
        state.lastCommunitySignal = data.enabled ? data : null;
        if (state.lastAgentStatusSnapshot) {
          renderAgentStatus(state.lastAgentStatusSnapshot, state.lastCommunitySignal);
        }
      } catch {
        // Silent by design.
      }
    });
}

function createBillingBlocksLoader(state: RuntimeState, applyFilter: () => void): () => Promise<void> {
  return () =>
    runExclusive(state, 'loadBillingBlocks', async () => {
      try {
        const data = await fetchJson<BillingBlocksResponse>('/api/billing-blocks');
        billingBlocksData.value = data;
        if (data && rawData.value) applyFilter();
      } catch {
        billingBlocksData.value = null;
      }
    });
}

function createContextWindowLoader(state: RuntimeState, applyFilter: () => void): () => Promise<void> {
  return () =>
    runExclusive(state, 'loadContextWindow', async () => {
      try {
        const data = await fetchJson<ContextWindowResponse>('/api/context-window');
        contextWindowData.value = data;
        if (data && rawData.value) applyFilter();
      } catch {
        contextWindowData.value = null;
      }
    });
}

function createCostReconciliationLoader(state: RuntimeState): () => Promise<void> {
  return () =>
    runExclusive(state, 'loadCostReconciliation', async () => {
      try {
        const data = await fetchJson<CostReconciliationResponse>('/api/cost-reconciliation?period=month');
        costReconciliationData.value = data;
        if (data) renderCostReconciliation();
      } catch {
        costReconciliationData.value = null;
      }
    });
}

function currentTimezoneOffsetMinutes(): number {
  return typeof window !== 'undefined' && typeof window.Date !== 'undefined'
    ? new Date().getTimezoneOffset() * -1
    : 0;
}

function createHeatmapLoader(state: RuntimeState): (period?: string) => Promise<void> {
  return (period = 'month') =>
    runExclusive(state, 'loadHeatmap', async () => {
      try {
        const tzOffset = currentTimezoneOffsetMinutes();
        const data = await fetchJson<HeatmapData>(
          `/api/heatmap?period=${encodeURIComponent(period)}&tz_offset_min=${tzOffset}`
        );
        if (data) renderActivityHeatmap(data);
      } catch {
        // Silent by design.
      }
    });
}

function createDataLoader(state: RuntimeState, applyFilter: () => void): (force?: boolean) => Promise<void> {
  return (force = false) =>
    runExclusive(
      state,
      'loadData',
      async () => {
        const isSubsequentFetch = rawData.value !== null;
        if (isSubsequentFetch) {
          loadState.value = 'refreshing';
          setStatus('header-refresh', 'loading', 'REFRESHING');
        }

        try {
          const response = await fetch('/api/data');
          if (!response.ok) {
            setStatus('global', 'error', `Failed to load data: HTTP ${response.status}`);
            return;
          }

          const data = (await response.json()) as DashboardData;
          if (data.error) {
            setStatus('global', 'error', data.error);
            return;
          }

          clearStatus('global');
          clearStatus('header-refresh');
          rawData.value = data;

          if (!isSubsequentFetch) {
            restoreDashboardStateFromUrl(data.all_models);
          }

          applyFilter();
        } catch {
          setStatus('global', 'error', 'Network error loading data');
          clearStatus('header-refresh');
        } finally {
          loadState.value = 'idle';
        }
      },
      force
    );
}

function startDashboardPolling(loaders: {
  applyFilter: () => void;
  loadAgentStatus: () => Promise<void>;
  loadBillingBlocks: () => Promise<void>;
  loadClaudeUsage: () => Promise<void>;
  loadCommunitySignal: () => Promise<void>;
  loadContextWindow: () => Promise<void>;
  loadCostReconciliation: () => Promise<void>;
  loadData: (force?: boolean) => Promise<void>;
  loadHeatmap: (period?: string) => Promise<void>;
  loadUsageWindows: () => Promise<void>;
}): void {
  window.addEventListener('popstate', () => {
    if (!rawData.value) return;
    restoreDashboardStateFromUrl(rawData.value.all_models);
    loaders.applyFilter();
  });

  void loaders.loadData();
  setInterval(loaders.loadData, 30000);

  void loaders.loadUsageWindows();
  void loaders.loadClaudeUsage();
  void loaders.loadAgentStatus();
  void loaders.loadCommunitySignal();
  setInterval(() => {
    void loaders.loadUsageWindows();
    void loaders.loadClaudeUsage();
    void loaders.loadAgentStatus();
    void loaders.loadCommunitySignal();
  }, 60000);

  void loaders.loadHeatmap('all');
  setInterval(() => void loaders.loadHeatmap('all'), 30000);

  void loaders.loadBillingBlocks();
  setInterval(() => void loaders.loadBillingBlocks(), 30000);

  void loaders.loadContextWindow();
  setInterval(() => void loaders.loadContextWindow(), 30000);

  void loaders.loadCostReconciliation();
  setInterval(() => void loaders.loadCostReconciliation(), 30000);
}

export function createDashboardRuntime(): DashboardRuntime {
  const state = createRuntimeState();

  const applyFilter = (): void => {
    if (!rawData.value) return;
    renderDashboardView(
      rawData.value,
      focusSingleModel,
      focusProjectQuery,
      exportSessionsCSV,
      exportProjectsCSV
    );
  };

  const focusSingleModel = (model: string): void => {
    if (!rawData.value) return;
    selectedModels.value = toggleModelSelection(rawData.value.all_models, selectedModels.value, model);
    syncDashboardUrl();
    applyFilter();
  };

  const focusProjectQuery = (project: string): void => {
    projectSearchQuery.value = toggleProjectSearch(projectSearchQuery.value, project);
    syncDashboardUrl();
    applyFilter();
  };

  const loadUsageWindows = createUsageWindowsLoader(state);
  const loadClaudeUsage = createClaudeUsageLoader(state);
  const loadAgentStatus = createAgentStatusLoader(state);
  const loadCommunitySignal = createCommunitySignalLoader(state);
  const loadBillingBlocks = createBillingBlocksLoader(state, applyFilter);
  const loadContextWindow = createContextWindowLoader(state, applyFilter);
  const loadCostReconciliation = createCostReconciliationLoader(state);
  const loadHeatmap = createHeatmapLoader(state);
  const loadData = createDataLoader(state, applyFilter);

  return {
    applyFilter,
    handleDashboardTabChange(tab: DashboardTab): void {
      if (activeDashboardTab.value === tab) return;
      activeDashboardTab.value = tab;
      syncDashboardUrl();
      refreshSectionVisibility();
    },
    loadData,
    start(): void {
      startDashboardPolling({
        applyFilter,
        loadAgentStatus,
        loadBillingBlocks,
        loadClaudeUsage,
        loadCommunitySignal,
        loadContextWindow,
        loadCostReconciliation,
        loadData,
        loadHeatmap,
        loadUsageWindows,
      });
    },
  };
}
