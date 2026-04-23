import { render } from 'preact';
import { DashboardTabs } from './components/DashboardTabs';
import { FilterBar } from './components/FilterBar';
import { Footer } from './components/Footer';
import { Header } from './components/Header';
import { InlineStatus } from './components/InlineStatus';
import { createDashboardRuntime } from './dashboard/runtime';
import { MonitorHeader } from './monitor/MonitorHeader';
import { createLiveMonitorRuntime } from './monitor/runtime';
import { hydrateLiveMonitorPreferences } from './monitor/store';
import { applyTheme, getTheme } from './lib/theme';
import { rawData, syncDashboardUrl } from './state/store';

applyTheme(getTheme());
const isMonitorRoute = window.location.pathname === '/monitor';
if (isMonitorRoute) {
  hydrateLiveMonitorPreferences();
}
const dashboardRuntime = !isMonitorRoute ? createDashboardRuntime() : null;
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

if (dashboardRuntime) {
  dashboardRuntime.start();
}
if (monitorRuntime) {
  monitorRuntime.start();
}
