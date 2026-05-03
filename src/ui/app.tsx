import { render } from 'preact';
import { BackupPanel } from './components/BackupPanel';
import { ImportsPanel } from './components/ImportsPanel';
import { WebCapturesPanel } from './components/WebCapturesPanel';
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
import { backupSnapshots, backupLoadState, archiveImports, webConversations, companionHeartbeat, rawData, syncDashboardUrl, type WebConversationSummary, type CompanionHeartbeat } from './state/store';

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

const backupPanelMount = document.getElementById('backup-panel');
if (backupPanelMount && dashboardRuntime) {
  render(
    <BackupPanel onSnapshot={triggerSnapshot} onReload={loadBackupSnapshots} />,
    backupPanelMount,
  );
  void loadBackupSnapshots();
}

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
