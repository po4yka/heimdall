import { render } from 'preact';
import { DashboardTabs } from './components/DashboardTabs';
import { FilterBar } from './components/FilterBar';
import { Footer } from './components/Footer';
import { Header } from './components/Header';
import { InlineStatus } from './components/InlineStatus';
import { createDashboardRuntime } from './dashboard/runtime';
import { applyTheme, getTheme } from './lib/theme';
import { rawData, syncDashboardUrl } from './state/store';

applyTheme(getTheme());
const runtime = createDashboardRuntime();

function toggleTheme(): void {
  const current =
    document.documentElement.getAttribute('data-theme') === 'light'
      ? 'light'
      : 'dark';
  const next: 'light' | 'dark' = current === 'light' ? 'dark' : 'light';
  localStorage.setItem('theme', next);
  applyTheme(next);
  if (rawData.value) runtime.applyFilter();
}

const headerMount = document.getElementById('header-mount');
if (headerMount) {
  render(<Header onDataReload={runtime.loadData} onThemeToggle={toggleTheme} />, headerMount);
}

const filterBarMount = document.getElementById('filter-bar-mount');
if (filterBarMount) {
  render(
    <FilterBar onFilterChange={runtime.applyFilter} onURLUpdate={syncDashboardUrl} />,
    filterBarMount
  );
}

const dashboardTabsMount = document.getElementById('dashboard-tabs-mount');
if (dashboardTabsMount) {
  render(<DashboardTabs onTabChange={runtime.handleDashboardTabChange} />, dashboardTabsMount);
}

const footerEl = document.querySelector('footer');
if (footerEl?.parentElement) {
  render(<Footer />, footerEl.parentElement, footerEl);
}

const globalStatusMount = document.getElementById('inline-status-global');
if (globalStatusMount) {
  render(<InlineStatus placement="global" />, globalStatusMount);
}

runtime.start();
