import { render } from 'preact';
import { BackupPanel } from './components/BackupPanel';
import { ImportsPanel } from './components/ImportsPanel';
import { WebCapturesPanel } from './components/WebCapturesPanel';
import { AgentRegistryModal } from './components/agents/AgentRegistryModal';
import { ProjectsRegistry } from './components/projects/ProjectsRegistry';
import { DashboardTabs } from './components/DashboardTabs';
import { FilterBar } from './components/FilterBar';
import { Footer } from './components/Footer';
import { Header } from './components/Header';
import { InlineStatus } from './components/InlineStatus';
import { createDashboardRuntime } from './dashboard/runtime';
import { MonitorHeader } from './monitor/MonitorHeader';
import { createLiveMonitorRuntime } from './monitor/runtime';
import { hydrateLiveMonitorPreferences } from './monitor/store';
import { startToolErrorsPage } from './tool_errors/runtime';
import { applyTheme, getTheme } from './lib/theme';
import { startVersionPoll } from './lib/version-poll';
import {
  activeDashboardTab,
  backupSnapshots,
  backupLoadState,
  archiveImports,
  webConversations,
  companionHeartbeat,
  rawData,
  registryModalOpen,
  syncDashboardUrl,
  type WebConversationSummary,
  type CompanionHeartbeat,
} from './state/store';
import { ScreenGridManager } from './widgets/ScreenGridManager';
import { registerMountCallback } from './widgets/mount-registry';

async function loadBackupSnapshots(): Promise<void> {
  backupLoadState.value = 'loading';
  try {
    const r = await fetch('/api/archive');
    if (!r.ok) throw new Error(`HTTP ${r.status}`);
    backupSnapshots.value = (await r.json()) as typeof backupSnapshots.value;
    backupLoadState.value = 'idle';
  } catch {
    backupLoadState.value = 'error';
  }
}

async function triggerSnapshot(): Promise<void> {
  const r = await fetch('/api/archive/snapshot', { method: 'POST' });
  if (!r.ok) throw new Error(`HTTP ${r.status}`);
}

applyTheme(getTheme());
startVersionPoll();
const isMonitorRoute = window.location.pathname === '/monitor';
const isToolErrorsRoute = window.location.pathname === '/tool-errors';
if (isMonitorRoute) {
  hydrateLiveMonitorPreferences();
}
const dashboardRuntime = (!isMonitorRoute && !isToolErrorsRoute) ? createDashboardRuntime() : null;
const monitorRuntime = isMonitorRoute ? createLiveMonitorRuntime() : null;

function toggleTheme(): void {
  const current =
    document.documentElement.getAttribute('data-theme') === 'light'
      ? 'light'
      : 'dark';
  const next: 'light' | 'dark' = current === 'light' ? 'dark' : 'light';
  localStorage.setItem('theme', next);
  applyTheme(next);
  if (rawData.value && dashboardRuntime) dashboardRuntime.applyFilter();
}

const headerMount = document.getElementById('header-mount');
if (headerMount) {
  if (isMonitorRoute && monitorRuntime) {
    render(<MonitorHeader onThemeToggle={toggleTheme} onRefresh={monitorRuntime.loadData} />, headerMount);
  } else if (isToolErrorsRoute) {
    render(
      <Header
        onDataReload={async () => { /* no-op: tool-errors page manages its own refresh */ }}
        onThemeToggle={toggleTheme}
        navigationHref="/"
        navigationLabel="Dashboard"
      />,
      headerMount
    );
  } else if (dashboardRuntime) {
    render(
      <Header
        onDataReload={dashboardRuntime.loadData}
        onThemeToggle={toggleTheme}
        navigationHref="/monitor"
        navigationLabel="Live Monitor"
      />,
      headerMount
    );
  }
}

const filterBarMount = document.getElementById('filter-bar-mount');
if (filterBarMount && dashboardRuntime) {
  render(
    <FilterBar onFilterChange={dashboardRuntime.applyFilter} onURLUpdate={syncDashboardUrl} />,
    filterBarMount
  );
}

const dashboardTabsMount = document.getElementById('dashboard-tabs-mount');
if (dashboardTabsMount && dashboardRuntime) {
  render(<DashboardTabs onTabChange={dashboardRuntime.handleDashboardTabChange} />, dashboardTabsMount);
}

const footerEl = document.querySelector('footer');
if (footerEl?.parentElement) {
  render(<Footer />, footerEl.parentElement, footerEl);
}

const globalStatusMount = document.getElementById('inline-status-global');
if (globalStatusMount && dashboardRuntime) {
  render(<InlineStatus placement="global" />, globalStatusMount);
}

// ── Feature 2: Widget grid mount ────────────────────────────────────────────
// Register mount callbacks for components that require Preact render + callbacks.
// These are called by the WidgetGrid when it mounts the respective widget elements.
if (dashboardRuntime) {
  registerMountCallback('backup-panel', (el) => {
    render(
      <BackupPanel onSnapshot={triggerSnapshot} onReload={loadBackupSnapshots} />,
      el,
    );
    void loadBackupSnapshots();
  });
  registerMountCallback('projects-registry', (el) => {
    render(<ProjectsRegistry onReload={dashboardRuntime.loadData} />, el);
  });
}

// Mount all screen grids into #widget-grid-mount.
// The ScreenGridManager shows only the active screen's grid; others are hidden.
const widgetGridMount = document.getElementById('widget-grid-mount');
if (widgetGridMount && dashboardRuntime) {
  function renderGridManager() {
    render(<ScreenGridManager />, widgetGridMount!);
  }
  renderGridManager();
  // Re-render on tab change so the active screen updates.
  activeDashboardTab.subscribe(() => renderGridManager());
}

// ── Imports and web captures (not in grid — they have their own static divs) ─
async function loadArchiveImports(): Promise<void> {
  try {
    const r = await fetch('/api/archive/imports');
    if (!r.ok) throw new Error(`HTTP ${r.status}`);
    archiveImports.value = (await r.json()) as typeof archiveImports.value;
  } catch (err) {
    console.error('failed to load imports:', err);
  }
}

const importsPanelMount = document.getElementById('imports-panel');
if (importsPanelMount && dashboardRuntime) {
  render(<ImportsPanel onReload={loadArchiveImports} />, importsPanelMount);
  void loadArchiveImports();
}

async function loadWebConversations(): Promise<void> {
  try {
    const r = await fetch('/api/archive/web-conversations');
    if (!r.ok) throw new Error(`HTTP ${r.status}`);
    const body = await r.json() as { conversations: WebConversationSummary[]; heartbeat: CompanionHeartbeat | null };
    webConversations.value = body.conversations;
    companionHeartbeat.value = body.heartbeat;
  } catch (err) {
    console.error('failed to load web captures:', err);
  }
}

const webCapturesPanelMount = document.getElementById('web-captures-panel');
if (webCapturesPanelMount && dashboardRuntime) {
  render(<WebCapturesPanel onReload={loadWebConversations} />, webCapturesPanelMount);
  void loadWebConversations();
}

if (dashboardRuntime) {
  dashboardRuntime.start();
}
if (monitorRuntime) {
  monitorRuntime.start();
}
if (isToolErrorsRoute) {
  startToolErrorsPage();
}

// Agent registry modal — reactive: re-renders whenever registryModalOpen signal changes.
const registryModalMount = document.getElementById('agent-registry-modal-mount');
if (registryModalMount && dashboardRuntime) {
  function RegistryModalRoot() {
    const modalState = registryModalOpen.value;
    const data = rawData.value;
    if (!modalState || !data) return null;
    return (
      <AgentRegistryModal
        project={modalState.project}
        telemetry={data.agent_telemetry}
        onReload={dashboardRuntime!.loadData}
      />
    );
  }
  registryModalOpen.subscribe(() => {
    render(<RegistryModalRoot />, registryModalMount);
  });
  rawData.subscribe(() => {
    render(<RegistryModalRoot />, registryModalMount);
  });
}
