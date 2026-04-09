import { render } from 'preact';
import { Footer } from './components/Footer';
import { StatsCards } from './components/StatsCards';
import { showError, showSuccess, ToastContainer } from './components/Toast';
import { SubagentSummary as SubagentSummaryComponent } from './components/SubagentSummary';
import { EntrypointTable } from './components/EntrypointTable';
import { ServiceTiersTable } from './components/ServiceTiers';
import { SessionsTable } from './components/SessionsTable';
import { ModelCostTable } from './components/ModelCostTable';
import { ProjectCostTable } from './components/ProjectCostTable';
import { DailyChart } from './components/DailyChart';
import { ModelChart } from './components/ModelChart';
import { ProjectChart } from './components/ProjectChart';
import { Sparkline } from './components/Sparkline';

import type {
  WindowInfo,
  UsageWindowsResponse,
  SubagentSummary,
  EntrypointSummary,
  ServiceTierSummary,
  DashboardData,
  DailyAgg,
  ModelAgg,
  ProjectAgg,
  Totals,
  RangeKey,
} from './state/types';
import {
  rawData,
  selectedModels,
  selectedRange,
  projectSearchQuery,
  lastFilteredSessions,
  lastByProject,
} from './state/store';
import { esc, $, fmtResetTime, progressColor } from './lib/format';
import { downloadCSV } from './lib/csv';
import { RANGE_LABELS } from './lib/charts';
import { getTheme } from './lib/theme';

// ── Theme (app-level, depends on state) ───────────────────────────────
function applyTheme(theme: 'light' | 'dark'): void {
  if (theme === 'light') {
    document.documentElement.setAttribute('data-theme', 'light');
  } else {
    document.documentElement.removeAttribute('data-theme');
  }
  const icon = document.getElementById('theme-icon');
  if (icon) icon.innerHTML = theme === 'dark'
    ? '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="5"/><line x1="12" y1="1" x2="12" y2="3"/><line x1="12" y1="21" x2="12" y2="23"/><line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/><line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/><line x1="1" y1="12" x2="3" y2="12"/><line x1="21" y1="12" x2="23" y2="12"/><line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/><line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/></svg>'
    : '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/></svg>';
  if (rawData.value) applyFilter();
}

function toggleTheme(): void {
  const current = document.documentElement.getAttribute('data-theme') === 'light' ? 'light' : 'dark';
  const next = current === 'light' ? 'dark' : 'light';
  localStorage.setItem('theme', next);
  applyTheme(next);
}

// Apply theme immediately before render
applyTheme(getTheme());

// ── Local-only state (not reactive) ───────────────────────────────────
let previousSessionPercent: number | null = null;

// ── Model classification (for filter defaults only, costs come from server) ──
function isAnthropicModel(model: string): boolean {
  if (!model) return false;
  const m = model.toLowerCase();
  return m.includes('opus') || m.includes('sonnet') || m.includes('haiku');
}

function getRangeCutoff(range: RangeKey): string | null {
  if (range === 'all') return null;
  const days = range === '7d' ? 7 : range === '30d' ? 30 : 90;
  const d = new Date();
  d.setDate(d.getDate() - days);
  return d.toISOString().slice(0, 10);
}

function readURLRange(): RangeKey {
  const p = new URLSearchParams(window.location.search).get('range');
  return (['7d', '30d', '90d', 'all'] as RangeKey[]).includes(p as RangeKey) ? (p as RangeKey) : '30d';
}

function setRange(range: RangeKey): void {
  selectedRange.value = range;
  document.querySelectorAll<HTMLButtonElement>('.range-btn').forEach(btn =>
    btn.classList.toggle('active', btn.dataset.range === range)
  );
  updateURL();
  applyFilter();
}

// ── Model filter ───────────────────────────────────────────────────────
function modelPriority(m: string): number {
  const ml = m.toLowerCase();
  if (ml.includes('opus'))   return 0;
  if (ml.includes('sonnet')) return 1;
  if (ml.includes('haiku'))  return 2;
  return 3;
}

function readURLModels(allModels: string[]): Set<string> {
  const param = new URLSearchParams(window.location.search).get('models');
  if (!param) return new Set(allModels.filter(m => isAnthropicModel(m)));
  const fromURL = new Set(param.split(',').map(s => s.trim()).filter(Boolean));
  return new Set(allModels.filter(m => fromURL.has(m)));
}

function isDefaultModelSelection(allModels: string[]): boolean {
  const billable = allModels.filter(m => isAnthropicModel(m));
  if (selectedModels.value.size !== billable.length) return false;
  return billable.every(m => selectedModels.value.has(m));
}

function buildFilterUI(allModels: string[]): void {
  const sorted = [...allModels].sort((a, b) => {
    const pa = modelPriority(a), pb = modelPriority(b);
    return pa !== pb ? pa - pb : a.localeCompare(b);
  });
  selectedModels.value = readURLModels(allModels);
  const container = $('model-checkboxes');
  container.innerHTML = sorted.map(m => {
    const checked = selectedModels.value.has(m);
    return `<label class="model-cb-label ${checked ? 'checked' : ''}" data-model="${esc(m)}">
      <input type="checkbox" value="${esc(m)}" ${checked ? 'checked' : ''} onchange="onModelToggle(this)">
      ${esc(m)}
    </label>`;
  }).join('');
}

function onModelToggle(cb: HTMLInputElement): void {
  const label = cb.closest('label')!;
  const next = new Set(selectedModels.value);
  if (cb.checked) { next.add(cb.value); label.classList.add('checked'); }
  else            { next.delete(cb.value); label.classList.remove('checked'); }
  selectedModels.value = next;
  updateURL();
  applyFilter();
}

function selectAllModels(): void {
  const next = new Set(selectedModels.value);
  document.querySelectorAll<HTMLInputElement>('#model-checkboxes input').forEach(cb => {
    cb.checked = true; next.add(cb.value); cb.closest('label')!.classList.add('checked');
  });
  selectedModels.value = next;
  updateURL(); applyFilter();
}

function clearAllModels(): void {
  document.querySelectorAll<HTMLInputElement>('#model-checkboxes input').forEach(cb => {
    cb.checked = false; cb.closest('label')!.classList.remove('checked');
  });
  selectedModels.value = new Set();
  updateURL(); applyFilter();
}

// ── Project search ─────────────────────────────────────────────────────
function onProjectSearch(query: string): void {
  projectSearchQuery.value = query.toLowerCase().trim();
  const clearBtn = document.getElementById('project-clear-btn');
  if (clearBtn) clearBtn.style.display = projectSearchQuery.value ? '' : 'none';
  updateURL();
  applyFilter();
}


function clearProjectSearch(): void {
  projectSearchQuery.value = '';
  const input = document.getElementById('project-search') as HTMLInputElement;
  if (input) input.value = '';
  const clearBtn = document.getElementById('project-clear-btn');
  if (clearBtn) clearBtn.style.display = 'none';
  updateURL();
  applyFilter();
}

function matchesProjectSearch(project: string): boolean {
  if (!projectSearchQuery.value) return true;
  return project.toLowerCase().includes(projectSearchQuery.value);
}

// ── URL persistence ────────────────────────────────────────────────────
function updateURL(): void {
  const allModels = Array.from(document.querySelectorAll<HTMLInputElement>('#model-checkboxes input')).map(cb => cb.value);
  const params = new URLSearchParams();
  if (selectedRange.value !== '30d') params.set('range', selectedRange.value);
  if (!isDefaultModelSelection(allModels)) params.set('models', Array.from(selectedModels.value).join(','));
  if (projectSearchQuery.value) params.set('project', projectSearchQuery.value);
  const search = params.toString() ? '?' + params.toString() : '';
  history.replaceState(null, '', window.location.pathname + search);
}

// ── Sort helpers (moved to Preact table components) ───────────────────

// ── Aggregation & filtering ────────────────────────────────────────────
function applyFilter(): void {
  if (!rawData.value) return;
  const cutoff = getRangeCutoff(selectedRange.value);

  const filteredDaily = rawData.value.daily_by_model.filter(r =>
    selectedModels.value.has(r.model) && (!cutoff || r.day >= cutoff)
  );

  const dailyMap: Record<string, DailyAgg> = {};
  for (const r of filteredDaily) {
    if (!dailyMap[r.day]) dailyMap[r.day] = { day: r.day, input: 0, output: 0, cache_read: 0, cache_creation: 0 };
    const d = dailyMap[r.day];
    d.input += r.input; d.output += r.output;
    d.cache_read += r.cache_read; d.cache_creation += r.cache_creation;
  }
  const daily = Object.values(dailyMap).sort((a, b) => a.day.localeCompare(b.day));

  const modelMap: Record<string, ModelAgg> = {};
  for (const r of filteredDaily) {
    if (!modelMap[r.model]) modelMap[r.model] = { model: r.model, input: 0, output: 0, cache_read: 0, cache_creation: 0, turns: 0, sessions: 0, cost: 0, is_billable: r.cost > 0 || isAnthropicModel(r.model) };
    const m = modelMap[r.model];
    m.input += r.input; m.output += r.output;
    m.cache_read += r.cache_read; m.cache_creation += r.cache_creation;
    m.turns += r.turns; m.cost += r.cost;
  }

  const filteredSessions = rawData.value.sessions_all.filter(s =>
    selectedModels.value.has(s.model) && (!cutoff || s.last_date >= cutoff) && matchesProjectSearch(s.project)
  );

  for (const s of filteredSessions) {
    if (modelMap[s.model]) modelMap[s.model].sessions++;
  }

  const byModel = Object.values(modelMap).sort((a, b) => (b.input + b.output) - (a.input + a.output));

  const projMap: Record<string, ProjectAgg> = {};
  for (const s of filteredSessions) {
    if (!projMap[s.project]) projMap[s.project] = { project: s.project, input: 0, output: 0, cache_read: 0, cache_creation: 0, turns: 0, sessions: 0, cost: 0 };
    const p = projMap[s.project];
    p.input += s.input; p.output += s.output;
    p.cache_read += s.cache_read; p.cache_creation += s.cache_creation;
    p.turns += s.turns; p.sessions++;
    p.cost += s.cost;
  }
  const byProject = Object.values(projMap).sort((a, b) => (b.input + b.output) - (a.input + a.output));

  const totals: Totals = {
    sessions: filteredSessions.length,
    turns: byModel.reduce((s, m) => s + m.turns, 0),
    input: byModel.reduce((s, m) => s + m.input, 0),
    output: byModel.reduce((s, m) => s + m.output, 0),
    cache_read: byModel.reduce((s, m) => s + m.cache_read, 0),
    cache_creation: byModel.reduce((s, m) => s + m.cache_creation, 0),
    cost: filteredSessions.reduce((s, sess) => s + sess.cost, 0),
  };

  $('daily-chart-title').textContent = 'Daily Token Usage \u2014 ' + RANGE_LABELS[selectedRange.value];

  renderStats(totals);
  renderCostSparkline(daily);
  renderDailyChart(daily);
  renderModelChart(byModel);
  renderProjectChart(byProject);
  lastFilteredSessions.value = filteredSessions;
  lastByProject.value = byProject;
  render(<ModelCostTable byModel={byModel} />, $('model-cost-mount'));
  render(<SessionsTable onExportCSV={exportSessionsCSV} />, $('sessions-mount'));
  render(<ProjectCostTable byProject={lastByProject.value.slice(0, 30)} onExportCSV={exportProjectsCSV} />, $('project-cost-mount'));
}

// ── Renderers ──────────────────────────────────────────────────────────
function renderStats(t: Totals): void {
  render(<StatsCards totals={t} />, $('stats-row'));
}

function renderDailyChart(daily: DailyAgg[]): void {
  const container = document.getElementById('chart-daily')!;
  render(<DailyChart daily={daily} />, container);
}

function renderModelChart(byModel: ModelAgg[]): void {
  const container = document.getElementById('chart-model')!;
  render(<ModelChart byModel={byModel} />, container);
}

function renderProjectChart(byProject: ProjectAgg[]): void {
  const container = document.getElementById('chart-project')!;
  render(<ProjectChart byProject={byProject} />, container);
}


// ── CSV Export ──────────────────────────────────────────────────────────
function exportSessionsCSV(): void {
  const header = ['Session', 'Project', 'Last Active', 'Duration (min)', 'Model', 'Turns', 'Input', 'Output', 'Cache Read', 'Cache Creation', 'Est. Cost'];
  const rows = lastFilteredSessions.value.map(s => {
    const cost = s.cost;
    return [s.session_id, s.project, s.last, s.duration_min, s.model, s.turns, s.input, s.output, s.cache_read, s.cache_creation, cost.toFixed(4)];
  });
  downloadCSV('sessions', header, rows);
}

function exportProjectsCSV(): void {
  const header = ['Project', 'Sessions', 'Turns', 'Input', 'Output', 'Cache Read', 'Cache Creation', 'Est. Cost'];
  const rows = lastByProject.value.map(p =>
    [p.project, p.sessions, p.turns, p.input, p.output, p.cache_read, p.cache_creation, p.cost.toFixed(4)]
  );
  downloadCSV('projects', header, rows);
}

// ── Usage Windows & Budget ──────────────────────────────────────────────
function renderWindowCard(label: string, w: WindowInfo): string {
  const pct = Math.min(100, w.used_percent);
  const color = progressColor(pct);
  const resetText = w.resets_in_minutes != null ? `Resets in ${fmtResetTime(w.resets_in_minutes)}` : '';
  return `<div class="stat-card">
    <div class="label">${esc(label)}</div>
    <div class="value" style="font-size:18px;color:${color}">${pct.toFixed(1)}%</div>
    <div style="background:var(--border);border-radius:4px;height:6px;margin:6px 0">
      <div style="background:${color};height:100%;border-radius:4px;width:${pct}%;transition:width 0.3s"></div>
    </div>
    <div class="sub">${esc(resetText)}</div>
  </div>`;
}

function renderUsageWindows(data: UsageWindowsResponse): void {
  const container = $('usage-windows');
  if (!container) return;

  if (!data.available) {
    container.innerHTML = '';
    container.style.display = 'none';
    return;
  }

  container.style.display = '';
  let cards = '';
  if (data.session) cards += renderWindowCard('Session (5h)', data.session);
  if (data.weekly) cards += renderWindowCard('Weekly', data.weekly);
  if (data.weekly_opus) cards += renderWindowCard('Weekly Opus', data.weekly_opus);
  if (data.weekly_sonnet) cards += renderWindowCard('Weekly Sonnet', data.weekly_sonnet);

  if (data.budget) {
    const b = data.budget;
    const pct = Math.min(100, b.utilization);
    const color = progressColor(pct);
    cards += `<div class="stat-card">
      <div class="label">Monthly Budget</div>
      <div class="value" style="font-size:18px;color:${color}">$${b.used.toFixed(2)} / $${b.limit.toFixed(2)}</div>
      <div style="background:var(--border);border-radius:4px;height:6px;margin:6px 0">
        <div style="background:${color};height:100%;border-radius:4px;width:${pct}%;transition:width 0.3s"></div>
      </div>
      <div class="sub">${esc(b.currency)}</div>
    </div>`;
  }

  container.innerHTML = cards;

  // Session depletion alert
  if (data.session) {
    const currentPercent = 100 - data.session.used_percent;
    if (previousSessionPercent !== null) {
      if (previousSessionPercent > 0.01 && currentPercent <= 0.01) {
        showError('Session depleted \u2014 resets in ' + fmtResetTime(data.session.resets_in_minutes));
      } else if (previousSessionPercent <= 0.01 && currentPercent > 0.01) {
        showSuccess('Session restored');
      }
    }
    previousSessionPercent = currentPercent;
  }

  // Plan badge
  const badge = $('plan-badge');
  if (badge && data.identity?.plan) {
    badge.textContent = data.identity.plan.charAt(0).toUpperCase() + data.identity.plan.slice(1);
    badge.style.display = '';
  } else if (badge) {
    badge.style.display = 'none';
  }
}



function renderSubagentSummary(summary: SubagentSummary): void {
  const container = $('subagent-summary');
  if (!container) return;
  if (summary.subagent_turns === 0) {
    container.style.display = 'none';
    render(null, container);
    return;
  }
  container.style.display = '';
  render(<SubagentSummaryComponent summary={summary} />, container);
}

function renderEntrypointBreakdown(data: EntrypointSummary[]): void {
  const container = $('entrypoint-breakdown');
  if (!container) return;
  if (!data.length) {
    container.style.display = 'none';
    render(null, container);
    return;
  }
  container.style.display = '';
  render(<EntrypointTable data={data} />, container);
}

function renderServiceTiers(data: ServiceTierSummary[]): void {
  const container = $('service-tiers');
  if (!container) return;
  if (!data.length) {
    container.style.display = 'none';
    render(null, container);
    return;
  }
  container.style.display = '';
  render(<ServiceTiersTable data={data} />, container);
}

function renderCostSparkline(daily: DailyAgg[]): void {
  const container = $('cost-sparkline');
  if (!container) return;
  const last7 = daily.slice(-7);
  if (last7.length < 2) {
    container.style.display = 'none';
    render(null, container);
    return;
  }
  container.style.display = '';
  render(<Sparkline daily={daily} />, container);
}

async function loadUsageWindows(): Promise<void> {
  try {
    const resp = await fetch('/api/usage-windows');
    if (!resp.ok) return;
    const data: UsageWindowsResponse = await resp.json();
    renderUsageWindows(data);
  } catch { /* silent */ }
}

// ── Rescan ──────────────────────────────────────────────────────────────
async function triggerRescan(): Promise<void> {
  const btn = $('rescan-btn') as HTMLButtonElement;
  btn.disabled = true;
  btn.textContent = '\u21bb Scanning...';
  try {
    const resp = await fetch('/api/rescan', { method: 'POST' });
    if (!resp.ok) {
      showError(`Rescan failed: HTTP ${resp.status} ${resp.statusText}`);
      btn.textContent = '\u21bb Rescan (failed)';
      return;
    }
    const d = await resp.json();
    btn.textContent = '\u21bb Rescan (' + d.new + ' new, ' + d.updated + ' updated)';
    await loadData();
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    showError('Rescan failed: ' + msg);
    btn.textContent = '\u21bb Rescan (error)';
    console.error(e);
  }
  setTimeout(() => { btn.textContent = '\u21bb Rescan'; btn.disabled = false; }, 3000);
}

// ── Loading skeleton ──────────────────────────────────────────────────
function renderLoadingSkeleton(): void {
  const statsRow = document.getElementById('stats-row');
  if (statsRow && !rawData.value) {
    statsRow.innerHTML = Array.from({ length: 7 }, () =>
      '<div class="skeleton" style="height:80px"></div>'
    ).join('');
  }
}
renderLoadingSkeleton();

// ── Data loading ───────────────────────────────────────────────────────
async function loadData(): Promise<void> {
  try {
    const resp = await fetch('/api/data');
    if (!resp.ok) {
      showError(`Failed to load data: HTTP ${resp.status}`);
      return;
    }
    const d: DashboardData = await resp.json();
    if (d.error) {
      document.body.innerHTML = '<div style="padding:40px;color:#f87171;font-family:monospace">' + esc(d.error) + '</div>';
      return;
    }
    $('meta').textContent = 'Updated: ' + d.generated_at + ' \u00b7 Auto-refresh 30s';

    const isFirstLoad = rawData.value === null;
    rawData.value = d;

    if (isFirstLoad) {
      selectedRange.value = readURLRange();
      document.querySelectorAll<HTMLButtonElement>('.range-btn').forEach(btn =>
        btn.classList.toggle('active', btn.dataset.range === selectedRange.value)
      );
      buildFilterUI(d.all_models);

      // Restore project search from URL
      const urlProject = new URLSearchParams(window.location.search).get('project');
      if (urlProject) {
        projectSearchQuery.value = urlProject;
        const input = document.getElementById('project-search') as HTMLInputElement;
        if (input) input.value = urlProject;
        const clearBtn = document.getElementById('project-clear-btn');
        if (clearBtn) clearBtn.style.display = '';
      }
    }

    applyFilter();
    if (rawData.value.subagent_summary) renderSubagentSummary(rawData.value.subagent_summary);
    if (rawData.value.entrypoint_breakdown) renderEntrypointBreakdown(rawData.value.entrypoint_breakdown);
    if (rawData.value.service_tiers) renderServiceTiers(rawData.value.service_tiers);
  } catch (e) {
    console.error(e);
  }
}

// Expose functions to global scope for inline HTML event handlers
Object.assign(window, {
  setRange, onModelToggle, selectAllModels, clearAllModels,
  exportSessionsCSV, exportProjectsCSV, triggerRescan,
  onProjectSearch, clearProjectSearch, toggleTheme,
});

loadData();
setInterval(loadData, 30000);
loadUsageWindows();
setInterval(loadUsageWindows, 60000);

// ── Preact mount: replace static footer with component ────────────────
const footerEl = document.querySelector('footer');
if (footerEl && footerEl.parentElement) {
  render(<Footer />, footerEl.parentElement, footerEl);
}

// ── Preact mount: toast container ─────────────────────────────────────
const toastRoot = document.createElement('div');
document.body.appendChild(toastRoot);
render(<ToastContainer />, toastRoot);
