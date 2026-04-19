import { render } from 'preact';
import { DashboardTabs } from './components/DashboardTabs';
import { FilterBar } from './components/FilterBar';
import { Footer } from './components/Footer';
import { Header } from './components/Header';
import { InlineStatus } from './components/InlineStatus';
import {
  refreshSectionVisibility,
  renderActivityHeatmap,
  renderAgentStatus,
  renderClaudeUsage,
  renderCostReconciliation,
  renderDashboardView,
  renderUsageWindows,
} from './dashboardView';
import { downloadCSV } from './lib/csv';
import { applyTheme, getTheme } from './lib/theme';
import { clearStatus, setStatus } from './lib/status';
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
} from './state/types';
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
} from './state/store';

applyTheme(getTheme());

function toggleTheme(): void {
  const current =
    document.documentElement.getAttribute('data-theme') === 'light'
      ? 'light'
      : 'dark';
  const next: 'light' | 'dark' = current === 'light' ? 'dark' : 'light';
  localStorage.setItem('theme', next);
  applyTheme(next);
  if (rawData.value) applyFilter();
}

let previousSessionPercent: number | null = null;
let loadDataInFlight = false;
let loadUsageWindowsInFlight = false;
let loadClaudeUsageInFlight = false;
let loadHeatmapInFlight = false;
let loadAgentStatusInFlight = false;
let loadCommunitySignalInFlight = false;
let lastCommunitySignal: CommunitySignal | null = null;
let lastAgentStatusSnapshot: AgentStatusSnapshot | null = null;
let loadBillingBlocksInFlight = false;
let loadContextWindowInFlight = false;
let loadCostReconciliationInFlight = false;

function handleDashboardTabChange(tab: DashboardTab): void {
  if (activeDashboardTab.value === tab) return;
  activeDashboardTab.value = tab;
  syncDashboardUrl();
  refreshSectionVisibility();
}

function applyFilter(): void {
  if (!rawData.value) return;
  renderDashboardView(
    rawData.value,
    focusSingleModel,
    focusProjectQuery,
    exportSessionsCSV,
    exportProjectsCSV
  );
}

function focusSingleModel(model: string): void {
  if (!rawData.value) return;
  const isSoleSelection =
    selectedModels.value.size === 1 && selectedModels.value.has(model);
  selectedModels.value = isSoleSelection
    ? new Set(rawData.value.all_models)
    : new Set([model]);
  syncDashboardUrl();
  applyFilter();
}

function focusProjectQuery(project: string): void {
  const normalized = project.toLowerCase().trim();
  projectSearchQuery.value =
    projectSearchQuery.value === normalized ? '' : normalized;
  syncDashboardUrl();
  applyFilter();
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

function exportProjectsCSV(): void {
  const byProject = rawData.value
    ? rawData.value.sessions_all.reduce<Map<string, ProjectAgg>>((acc, session) => {
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

  exportProjectRowsCSV(
    'projects',
    Array.from(byProject.values()).sort(
      (left, right) => (right.input + right.output) - (left.input + left.output)
    )
  );
}

async function loadUsageWindows(): Promise<void> {
  if (loadUsageWindowsInFlight) return;
  loadUsageWindowsInFlight = true;
  try {
    const resp = await fetch('/api/usage-windows');
    if (!resp.ok) return;
    const data: UsageWindowsResponse = await resp.json();
    renderUsageWindows(
      data,
      previousSessionPercent,
      value => {
        previousSessionPercent = value;
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
  } finally {
    loadUsageWindowsInFlight = false;
  }
}

async function loadClaudeUsage(): Promise<void> {
  if (loadClaudeUsageInFlight) return;
  loadClaudeUsageInFlight = true;
  try {
    const resp = await fetch('/api/claude-usage');
    if (!resp.ok) return;
    const data: ClaudeUsageResponse = await resp.json();
    renderClaudeUsage(data);
  } catch {
    // Silent by design.
  } finally {
    loadClaudeUsageInFlight = false;
  }
}

async function loadAgentStatus(): Promise<void> {
  if (loadAgentStatusInFlight) return;
  loadAgentStatusInFlight = true;
  try {
    const resp = await fetch('/api/agent-status');
    if (!resp.ok) return;
    const data: AgentStatusSnapshot = await resp.json();
    lastAgentStatusSnapshot = data;
    renderAgentStatus(data, lastCommunitySignal);
  } catch {
    // Silent by design.
  } finally {
    loadAgentStatusInFlight = false;
  }
}

async function loadCommunitySignal(): Promise<void> {
  if (loadCommunitySignalInFlight) return;
  loadCommunitySignalInFlight = true;
  try {
    const resp = await fetch('/api/community-signal');
    if (!resp.ok) return;
    const data: CommunitySignal = await resp.json();
    lastCommunitySignal = data.enabled ? data : null;
    if (lastAgentStatusSnapshot) {
      renderAgentStatus(lastAgentStatusSnapshot, lastCommunitySignal);
    }
  } catch {
    // Silent by design.
  } finally {
    loadCommunitySignalInFlight = false;
  }
}

async function loadBillingBlocks(): Promise<void> {
  if (loadBillingBlocksInFlight) return;
  loadBillingBlocksInFlight = true;
  try {
    const resp = await fetch('/api/billing-blocks');
    if (!resp.ok) {
      billingBlocksData.value = null;
      return;
    }
    const data: BillingBlocksResponse = await resp.json();
    billingBlocksData.value = data;
    if (rawData.value) applyFilter();
  } catch {
    billingBlocksData.value = null;
  } finally {
    loadBillingBlocksInFlight = false;
  }
}

async function loadContextWindow(): Promise<void> {
  if (loadContextWindowInFlight) return;
  loadContextWindowInFlight = true;
  try {
    const resp = await fetch('/api/context-window');
    if (!resp.ok) {
      contextWindowData.value = null;
      return;
    }
    const data: ContextWindowResponse = await resp.json();
    contextWindowData.value = data;
    if (rawData.value) applyFilter();
  } catch {
    contextWindowData.value = null;
  } finally {
    loadContextWindowInFlight = false;
  }
}

async function loadCostReconciliation(): Promise<void> {
  if (loadCostReconciliationInFlight) return;
  loadCostReconciliationInFlight = true;
  try {
    const resp = await fetch('/api/cost-reconciliation?period=month');
    if (!resp.ok) {
      costReconciliationData.value = null;
      return;
    }
    const data: CostReconciliationResponse = await resp.json();
    costReconciliationData.value = data;
    renderCostReconciliation();
  } catch {
    costReconciliationData.value = null;
  } finally {
    loadCostReconciliationInFlight = false;
  }
}

async function loadHeatmap(period = 'month'): Promise<void> {
  if (loadHeatmapInFlight) return;
  loadHeatmapInFlight = true;
  try {
    const tzOffset =
      typeof window !== 'undefined' && typeof window.Date !== 'undefined'
        ? new Date().getTimezoneOffset() * -1
        : 0;
    const resp = await fetch(
      `/api/heatmap?period=${encodeURIComponent(period)}&tz_offset_min=${tzOffset}`
    );
    if (!resp.ok) return;
    const data: HeatmapData = await resp.json();
    renderActivityHeatmap(data);
  } catch {
    // Silent by design.
  } finally {
    loadHeatmapInFlight = false;
  }
}

async function loadData(force = false): Promise<void> {
  if (loadDataInFlight && !force) return;
  loadDataInFlight = true;

  const isSubsequentFetch = rawData.value !== null;
  if (isSubsequentFetch) {
    loadState.value = 'refreshing';
    setStatus('header-refresh', 'loading', 'REFRESHING');
  }

  try {
    const resp = await fetch('/api/data');
    if (!resp.ok) {
      setStatus('global', 'error', `Failed to load data: HTTP ${resp.status}`);
      return;
    }

    const data: DashboardData = await resp.json();
    if (data.error) {
      setStatus('global', 'error', data.error);
      return;
    }

    clearStatus('global');
    clearStatus('header-refresh');
    rawData.value = data;

    if (isSubsequentFetch === false) {
      restoreDashboardStateFromUrl(data.all_models);
    }

    applyFilter();
  } catch {
    setStatus('global', 'error', 'Network error loading data');
    clearStatus('header-refresh');
  } finally {
    loadState.value = 'idle';
    loadDataInFlight = false;
  }
}

const headerMount = document.getElementById('header-mount');
if (headerMount) {
  render(<Header onDataReload={loadData} onThemeToggle={toggleTheme} />, headerMount);
}

const filterBarMount = document.getElementById('filter-bar-mount');
if (filterBarMount) {
  render(
    <FilterBar onFilterChange={applyFilter} onURLUpdate={syncDashboardUrl} />,
    filterBarMount
  );
}

const dashboardTabsMount = document.getElementById('dashboard-tabs-mount');
if (dashboardTabsMount) {
  render(<DashboardTabs onTabChange={handleDashboardTabChange} />, dashboardTabsMount);
}

const footerEl = document.querySelector('footer');
if (footerEl?.parentElement) {
  render(<Footer />, footerEl.parentElement, footerEl);
}

const globalStatusMount = document.getElementById('inline-status-global');
if (globalStatusMount) {
  render(<InlineStatus placement="global" />, globalStatusMount);
}

window.addEventListener('popstate', () => {
  if (!rawData.value) return;
  restoreDashboardStateFromUrl(rawData.value.all_models);
  applyFilter();
});

loadData();
setInterval(loadData, 30000);
loadUsageWindows();
loadClaudeUsage();
loadAgentStatus();
loadCommunitySignal();
setInterval(() => {
  loadUsageWindows();
  loadClaudeUsage();
  loadAgentStatus();
  loadCommunitySignal();
}, 60000);
loadHeatmap('all');
setInterval(() => loadHeatmap('all'), 30000);
loadBillingBlocks();
setInterval(loadBillingBlocks, 30000);
loadContextWindow();
setInterval(loadContextWindow, 30000);
loadCostReconciliation();
setInterval(loadCostReconciliation, 30000);
