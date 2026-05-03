import { render } from 'preact';
import type { ToolErrorsResponse } from '../state/types';
import {
  PAGE_SIZE,
  errorMessage,
  loadState,
  pageOffset,
  providerFilter,
  rangeFilter,
  readUrlParams,
  rows,
  syncUrl,
  toolName,
  total,
} from './store';
import { ToolErrorsPage } from './ToolErrorsPage';

function rangeToDateBounds(range: string): { start?: string; end?: string } {
  if (range === 'all') return {};
  const days = range === '7d' ? 7 : range === '90d' ? 90 : 30;
  const end = new Date();
  const start = new Date();
  start.setDate(start.getDate() - days);
  const fmt = (d: Date) => d.toISOString().slice(0, 10);
  return { start: fmt(start), end: fmt(end) };
}

function renderPage(): void {
  const mount = document.getElementById('main-content');
  if (mount) render(<ToolErrorsPage onLoad={loadData} />, mount);
}

export async function loadData(): Promise<void> {
  loadState.value = 'loading';
  errorMessage.value = null;
  syncUrl();

  try {
    const tzOffset = new Date().getTimezoneOffset() * -1;
    const p = new URLSearchParams();
    p.set('tool', toolName.value);
    if (providerFilter.value) p.set('provider', providerFilter.value);
    p.set('limit', String(PAGE_SIZE));
    p.set('offset', String(pageOffset.value));
    p.set('tz_offset_min', String(tzOffset));

    const { start, end } = rangeToDateBounds(rangeFilter.value);
    if (start) p.set('start', start);
    if (end) p.set('end', end);

    const resp = await fetch(`/api/tool-errors?${p.toString()}`);
    if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
    const data = await resp.json() as ToolErrorsResponse;
    rows.value = data.rows;
    total.value = data.total;
    loadState.value = 'idle';
  } catch (err) {
    errorMessage.value = err instanceof Error ? err.message : 'Failed to load errors';
    loadState.value = 'error';
  }

  renderPage();
}

export function startToolErrorsPage(): void {
  readUrlParams();

  // Hide dashboard-only chrome.
  const filterBar = document.getElementById('filter-bar-mount');
  const tabsMount = document.getElementById('dashboard-tabs-mount');
  if (filterBar) filterBar.style.display = 'none';
  if (tabsMount) tabsMount.style.display = 'none';

  document.title = toolName.value ? `${toolName.value} Errors` : 'Tool Errors';
  renderPage();
  void loadData();
}
