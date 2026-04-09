// ── External declarations ──────────────────────────────────────────────
declare const ApexCharts: any;

import type {
  WindowInfo,
  BudgetInfo,
  IdentityInfo,
  UsageWindowsResponse,
  SubagentSummary,
  EntrypointSummary,
  ServiceTierSummary,
  DashboardData,
  DailyModelRow,
  SessionRow,
  DailyAgg,
  ModelAgg,
  ProjectAgg,
  Totals,
  StatCard,
  SortDir,
  RangeKey,
} from './state/types';
import {
  rawData,
  selectedModels,
  selectedRange,
  projectSearchQuery,
  sessionSortCol,
  sessionSortDir,
  modelSortCol,
  modelSortDir,
  projectSortCol,
  projectSortDir,
  sessionsCurrentPage,
  SESSIONS_PAGE_SIZE,
  lastFilteredSessions,
  lastByProject,
} from './state/store';
import { esc, $, fmt, fmtCost, fmtCostBig, fmtResetTime, progressColor } from './lib/format';
import { csvField, csvTimestamp, downloadCSV } from './lib/csv';
import { TOKEN_COLORS, MODEL_COLORS, RANGE_LABELS, RANGE_TICKS, apexThemeMode, cssVar } from './lib/charts';
import { getTheme } from './lib/theme';

// ── Theme (app-level, depends on state) ───────────────────────────────
function applyTheme(theme: 'light' | 'dark'): void {
  if (theme === 'dark') {
    document.documentElement.setAttribute('data-theme', 'dark');
  } else {
    document.documentElement.removeAttribute('data-theme');
  }
  const icon = document.getElementById('theme-icon');
  if (icon) icon.innerHTML = theme === 'dark' ? '&#x2600;' : '&#x263E;';
  // Re-render charts with new theme colors
  if (rawData.value) applyFilter();
}

function toggleTheme(): void {
  const current = document.documentElement.getAttribute('data-theme') === 'dark' ? 'dark' : 'light';
  const next = current === 'dark' ? 'light' : 'dark';
  localStorage.setItem('theme', next);
  applyTheme(next);
}

// Apply theme immediately before render
applyTheme(getTheme());

// ── Local-only state (not reactive) ───────────────────────────────────
let charts: Record<string, any> = {};
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

function sessionsPage(delta: number): void {
  const maxPage = Math.max(0, Math.ceil(lastFilteredSessions.value.length / SESSIONS_PAGE_SIZE) - 1);
  sessionsCurrentPage.value = Math.max(0, Math.min(maxPage, sessionsCurrentPage.value + delta));
  renderSessionsPage();
}

function renderSessionsPage(): void {
  const start = sessionsCurrentPage.value * SESSIONS_PAGE_SIZE;
  const page = lastFilteredSessions.value.slice(start, start + SESSIONS_PAGE_SIZE);
  renderSessionsTable(page);

  const total = lastFilteredSessions.value.length;
  const maxPage = Math.max(0, Math.ceil(total / SESSIONS_PAGE_SIZE) - 1);
  $('sessions-page-info').textContent = total > 0
    ? `Showing ${start + 1}\u2013${Math.min(start + SESSIONS_PAGE_SIZE, total)} of ${total}`
    : 'No sessions';
  ($('sessions-prev') as HTMLButtonElement).disabled = sessionsCurrentPage.value <= 0;
  ($('sessions-next') as HTMLButtonElement).disabled = sessionsCurrentPage.value >= maxPage;
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

// ── Sort helpers ───────────────────────────────────────────────────────
function setSessionSort(col: string): void {
  if (sessionSortCol.value === col) sessionSortDir.value = sessionSortDir.value === 'desc' ? 'asc' : 'desc';
  else { sessionSortCol.value = col; sessionSortDir.value = 'desc'; }
  updateSortIcons(); applyFilter();
}

function updateSortIcons(): void {
  document.querySelectorAll('.sort-icon').forEach(el => el.textContent = '');
  const icon = document.getElementById('sort-icon-' + sessionSortCol.value);
  if (icon) icon.textContent = sessionSortDir.value === 'desc' ? ' \u25bc' : ' \u25b2';
}

function sortSessions(sessions: SessionRow[]): SessionRow[] {
  return [...sessions].sort((a, b) => {
    let av: number | string, bv: number | string;
    if (sessionSortCol.value === 'cost') {
      av = a.cost;
      bv = b.cost;
    } else if (sessionSortCol.value === 'duration_min') {
      av = a.duration_min || 0;
      bv = b.duration_min || 0;
    } else {
      av = (a as any)[sessionSortCol.value] ?? 0;
      bv = (b as any)[sessionSortCol.value] ?? 0;
    }
    if (av < bv) return sessionSortDir.value === 'desc' ? 1 : -1;
    if (av > bv) return sessionSortDir.value === 'desc' ? -1 : 1;
    return 0;
  });
}

function setModelSort(col: string): void {
  if (modelSortCol.value === col) modelSortDir.value = modelSortDir.value === 'desc' ? 'asc' : 'desc';
  else { modelSortCol.value = col; modelSortDir.value = 'desc'; }
  updateModelSortIcons(); applyFilter();
}

function updateModelSortIcons(): void {
  document.querySelectorAll('[id^="msort-"]').forEach(el => el.textContent = '');
  const icon = document.getElementById('msort-' + modelSortCol.value);
  if (icon) icon.textContent = modelSortDir.value === 'desc' ? ' \u25bc' : ' \u25b2';
}

function sortModels(byModel: ModelAgg[]): ModelAgg[] {
  return [...byModel].sort((a, b) => {
    let av: number, bv: number;
    if (modelSortCol.value === 'cost') {
      av = a.cost;
      bv = b.cost;
    } else {
      av = (a as any)[modelSortCol.value] ?? 0;
      bv = (b as any)[modelSortCol.value] ?? 0;
    }
    if (av < bv) return modelSortDir.value === 'desc' ? 1 : -1;
    if (av > bv) return modelSortDir.value === 'desc' ? -1 : 1;
    return 0;
  });
}

function setProjectSort(col: string): void {
  if (projectSortCol.value === col) projectSortDir.value = projectSortDir.value === 'desc' ? 'asc' : 'desc';
  else { projectSortCol.value = col; projectSortDir.value = 'desc'; }
  updateProjectSortIcons(); applyFilter();
}

function updateProjectSortIcons(): void {
  document.querySelectorAll('[id^="psort-"]').forEach(el => el.textContent = '');
  const icon = document.getElementById('psort-' + projectSortCol.value);
  if (icon) icon.textContent = projectSortDir.value === 'desc' ? ' \u25bc' : ' \u25b2';
}

function sortProjects(byProject: ProjectAgg[]): ProjectAgg[] {
  return [...byProject].sort((a, b) => {
    const av = (a as any)[projectSortCol.value] ?? 0;
    const bv = (b as any)[projectSortCol.value] ?? 0;
    if (av < bv) return projectSortDir.value === 'desc' ? 1 : -1;
    if (av > bv) return projectSortDir.value === 'desc' ? -1 : 1;
    return 0;
  });
}

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
  lastFilteredSessions.value = sortSessions(filteredSessions);
  lastByProject.value = sortProjects(byProject);
  sessionsCurrentPage.value = 0;
  renderSessionsPage();
  renderModelCostTable(byModel);
  renderProjectCostTable(lastByProject.value.slice(0, 30));
}

// ── Renderers ──────────────────────────────────────────────────────────
function renderStats(t: Totals): void {
  const rangeLabel = RANGE_LABELS[selectedRange.value].toLowerCase();
  const stats: StatCard[] = [
    { label: 'Sessions',       value: t.sessions.toLocaleString(), sub: rangeLabel },
    { label: 'Turns',          value: fmt(t.turns),                sub: rangeLabel },
    { label: 'Input Tokens',   value: fmt(t.input),                sub: rangeLabel },
    { label: 'Output Tokens',  value: fmt(t.output),               sub: rangeLabel },
    { label: 'Cache Read',     value: fmt(t.cache_read),           sub: 'from prompt cache' },
    { label: 'Cache Creation', value: fmt(t.cache_creation),       sub: 'writes to prompt cache' },
    { label: 'Est. Cost',      value: fmtCostBig(t.cost),          sub: 'API pricing estimate', color: '#4ade80' },
  ];
  $('stats-row').innerHTML = stats.map(s => `
    <div class="stat-card">
      <div class="label">${s.label}</div>
      <div class="value" style="${s.color ? 'color:' + s.color : ''}">${esc(s.value)}</div>
      ${s.sub ? `<div class="sub">${esc(s.sub)}</div>` : ''}
    </div>
  `).join('');
}

function renderDailyChart(daily: DailyAgg[]): void {
  const el = document.getElementById('chart-daily')!;
  if (charts.daily) charts.daily.destroy();
  charts.daily = new ApexCharts(el, {
    chart: { type: 'bar', height: '100%', stacked: true, background: 'transparent',
             toolbar: { show: false }, fontFamily: 'inherit' },
    theme: { mode: apexThemeMode() },
    series: [
      { name: 'Input',          data: daily.map(d => d.input) },
      { name: 'Output',         data: daily.map(d => d.output) },
      { name: 'Cache Read',     data: daily.map(d => d.cache_read) },
      { name: 'Cache Creation', data: daily.map(d => d.cache_creation) },
    ],
    colors: [TOKEN_COLORS.input, TOKEN_COLORS.output, TOKEN_COLORS.cache_read, TOKEN_COLORS.cache_creation],
    xaxis: { categories: daily.map(d => d.day),
             labels: { rotate: -45, maxHeight: 60 },
             tickAmount: Math.min(daily.length, RANGE_TICKS[selectedRange.value]) },
    yaxis: { labels: { formatter: (v: number) => fmt(v) } },
    legend: { position: 'top', fontSize: '11px' },
    dataLabels: { enabled: false },
    tooltip: { y: { formatter: (v: number) => fmt(v) + ' tokens' } },
    grid: { borderColor: cssVar('--chart-grid') },
    plotOptions: { bar: { columnWidth: '70%' } },
  });
  charts.daily.render();
}

function renderModelChart(byModel: ModelAgg[]): void {
  const el = document.getElementById('chart-model')!;
  if (charts.model) charts.model.destroy();
  if (!byModel.length) { charts.model = null; el.innerHTML = ''; return; }
  charts.model = new ApexCharts(el, {
    chart: { type: 'donut', height: '100%', background: 'transparent', fontFamily: 'inherit' },
    theme: { mode: apexThemeMode() },
    series: byModel.map(m => m.input + m.output),
    labels: byModel.map(m => m.model),
    colors: MODEL_COLORS.slice(0, byModel.length),
    legend: { position: 'bottom', fontSize: '11px' },
    dataLabels: { enabled: false },
    tooltip: { y: { formatter: (v: number) => fmt(v) + ' tokens' } },
    stroke: { width: 2, colors: [cssVar('--card')] },
    plotOptions: { pie: { donut: { size: '60%' } } },
  });
  charts.model.render();
}

function renderProjectChart(byProject: ProjectAgg[]): void {
  const top = byProject.slice(0, 10);
  const el = document.getElementById('chart-project')!;
  if (charts.project) charts.project.destroy();
  if (!top.length) { charts.project = null; el.innerHTML = ''; return; }
  charts.project = new ApexCharts(el, {
    chart: { type: 'bar', height: '100%', background: 'transparent',
             toolbar: { show: false }, fontFamily: 'inherit' },
    theme: { mode: apexThemeMode() },
    series: [
      { name: 'Input',  data: top.map(p => p.input) },
      { name: 'Output', data: top.map(p => p.output) },
    ],
    colors: [TOKEN_COLORS.input, TOKEN_COLORS.output],
    plotOptions: { bar: { horizontal: true, barHeight: '60%' } },
    xaxis: { categories: top.map(p => p.project.length > 22 ? '\u2026' + p.project.slice(-20) : p.project),
             labels: { formatter: (v: number) => fmt(v) } },
    yaxis: { labels: { maxWidth: 160 } },
    legend: { position: 'top', fontSize: '11px' },
    dataLabels: { enabled: false },
    tooltip: { y: { formatter: (v: number) => fmt(v) + ' tokens' } },
    grid: { borderColor: cssVar('--chart-grid') },
  });
  charts.project.render();
}

function renderSessionsTable(sessions: SessionRow[]): void {
  $('sessions-body').innerHTML = sessions.map(s => {
    const cost = s.cost;
    const costCell = s.is_billable
      ? `<td class="cost">${fmtCost(cost)}</td>`
      : `<td class="cost-na">n/a</td>`;
    return `<tr>
      <td class="muted" style="font-family:monospace">${esc(s.session_id)}&hellip;</td>
      <td>${esc(s.project)}</td>
      <td class="muted">${esc(s.last)}</td>
      <td class="muted">${esc(s.duration_min)}m</td>
      <td><span class="model-tag">${esc(s.model)}</span></td>
      <td class="num">${s.turns}${s.subagent_count > 0 ? `<span class="muted" style="font-size:10px"> (${s.subagent_count} agents)</span>` : ''}</td>
      <td class="num">${fmt(s.input)}</td>
      <td class="num">${fmt(s.output)}</td>
      ${costCell}
    </tr>`;
  }).join('');
}

function renderModelCostTable(byModel: ModelAgg[]): void {
  $('model-cost-body').innerHTML = sortModels(byModel).map(m => {
    const cost = m.cost;
    const costCell = m.is_billable
      ? `<td class="cost">${fmtCost(cost)}</td>`
      : `<td class="cost-na">n/a</td>`;
    return `<tr>
      <td><span class="model-tag">${esc(m.model)}</span></td>
      <td class="num">${fmt(m.turns)}</td>
      <td class="num">${fmt(m.input)}</td>
      <td class="num">${fmt(m.output)}</td>
      <td class="num">${fmt(m.cache_read)}</td>
      <td class="num">${fmt(m.cache_creation)}</td>
      ${costCell}
    </tr>`;
  }).join('');
}

function renderProjectCostTable(byProject: ProjectAgg[]): void {
  $('project-cost-body').innerHTML = sortProjects(byProject).map(p => `<tr>
      <td>${esc(p.project)}</td>
      <td class="num">${p.sessions}</td>
      <td class="num">${fmt(p.turns)}</td>
      <td class="num">${fmt(p.input)}</td>
      <td class="num">${fmt(p.output)}</td>
      <td class="cost">${fmtCost(p.cost)}</td>
    </tr>`).join('');
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
      <div class="sub">${b.currency}</div>
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

function showSuccess(msg: string): void {
  const el = document.createElement('div');
  el.style.cssText = 'position:fixed;top:16px;right:16px;background:var(--toast-success-bg);color:var(--toast-success-text);padding:12px 20px;border-radius:8px;font-size:13px;z-index:999;max-width:400px;box-shadow:0 4px 12px rgba(0,0,0,0.15)';
  el.textContent = msg;
  document.body.appendChild(el);
  setTimeout(() => el.remove(), 6000);
}

function renderSubagentSummary(summary: SubagentSummary): void {
  const container = $('subagent-summary');
  if (!container) return;

  if (summary.subagent_turns === 0) {
    container.style.display = 'none';
    return;
  }

  container.style.display = '';
  const totalInput = summary.parent_input + summary.subagent_input;
  const totalOutput = summary.parent_output + summary.subagent_output;
  const subPctInput = totalInput > 0 ? (summary.subagent_input / totalInput * 100) : 0;
  const subPctOutput = totalOutput > 0 ? (summary.subagent_output / totalOutput * 100) : 0;

  container.innerHTML = `
    <div class="section-title">Subagent Breakdown</div>
    <div style="display:grid;grid-template-columns:1fr 1fr 1fr;gap:16px">
      <div>
        <div class="label" style="color:var(--muted);font-size:11px;text-transform:uppercase;margin-bottom:4px">Turns</div>
        <div style="font-size:15px">Parent: <strong>${fmt(summary.parent_turns)}</strong></div>
        <div style="font-size:15px">Subagent: <strong>${fmt(summary.subagent_turns)}</strong></div>
        <div class="sub">${summary.unique_agents} unique agents</div>
      </div>
      <div>
        <div class="label" style="color:var(--muted);font-size:11px;text-transform:uppercase;margin-bottom:4px">Input Tokens</div>
        <div style="font-size:15px">Parent: <strong>${fmt(summary.parent_input)}</strong></div>
        <div style="font-size:15px">Subagent: <strong>${fmt(summary.subagent_input)}</strong> (${subPctInput.toFixed(1)}%)</div>
      </div>
      <div>
        <div class="label" style="color:var(--muted);font-size:11px;text-transform:uppercase;margin-bottom:4px">Output Tokens</div>
        <div style="font-size:15px">Parent: <strong>${fmt(summary.parent_output)}</strong></div>
        <div style="font-size:15px">Subagent: <strong>${fmt(summary.subagent_output)}</strong> (${subPctOutput.toFixed(1)}%)</div>
      </div>
    </div>
  `;
}

function renderEntrypointBreakdown(data: EntrypointSummary[]): void {
  const container = $('entrypoint-breakdown');
  if (!container) return;
  if (!data.length) { container.style.display = 'none'; return; }
  container.style.display = '';
  container.innerHTML = `
    <div class="section-title">Usage by Entrypoint</div>
    <table><thead><tr>
      <th>Entrypoint</th><th>Sessions</th><th>Turns</th><th>Input</th><th>Output</th>
    </tr></thead><tbody>${data.map(e => `<tr>
      <td><span class="model-tag">${esc(e.entrypoint)}</span></td>
      <td class="num">${e.sessions}</td>
      <td class="num">${fmt(e.turns)}</td>
      <td class="num">${fmt(e.input)}</td>
      <td class="num">${fmt(e.output)}</td>
    </tr>`).join('')}</tbody></table>`;
}

function renderServiceTiers(data: ServiceTierSummary[]): void {
  const container = $('service-tiers');
  if (!container) return;
  if (!data.length) { container.style.display = 'none'; return; }
  container.style.display = '';
  container.innerHTML = `
    <div class="section-title">Service Tiers</div>
    <table><thead><tr>
      <th>Tier</th><th>Region</th><th>Turns</th>
    </tr></thead><tbody>${data.map(s => `<tr>
      <td>${esc(s.service_tier)}</td>
      <td>${esc(s.inference_geo)}</td>
      <td class="num">${fmt(s.turns)}</td>
    </tr>`).join('')}</tbody></table>`;
}

function renderCostSparkline(daily: DailyAgg[]): void {
  const container = $('cost-sparkline');
  if (!container) return;
  const last7 = daily.slice(-7);
  if (last7.length < 2) { container.style.display = 'none'; return; }
  container.style.display = '';
  container.innerHTML = '<div class="sub" style="margin-bottom:4px">7-day trend</div><div id="sparkline-chart"></div>';

  if (charts.sparkline) charts.sparkline.destroy();
  charts.sparkline = new ApexCharts(document.getElementById('sparkline-chart')!, {
    chart: { type: 'line', height: 30, width: 120, sparkline: { enabled: true },
             background: 'transparent', fontFamily: 'inherit' },
    series: [{ data: last7.map(d => d.input + d.output) }],
    stroke: { width: 1.5, curve: 'smooth' },
    colors: [cssVar('--accent')],
    tooltip: { enabled: false },
  });
  charts.sparkline.render();
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
function showError(msg: string): void {
  const el = document.createElement('div');
  el.style.cssText = 'position:fixed;top:16px;right:16px;background:var(--toast-error-bg);color:var(--toast-error-text);padding:12px 20px;border-radius:8px;font-size:13px;z-index:999;max-width:400px;box-shadow:0 4px 12px rgba(0,0,0,0.15)';
  el.textContent = msg;
  document.body.appendChild(el);
  setTimeout(() => el.remove(), 6000);
}

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
      updateSortIcons();
      updateModelSortIcons();
      updateProjectSortIcons();

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
  setSessionSort, setModelSort, setProjectSort,
  exportSessionsCSV, exportProjectsCSV, triggerRescan,
  onProjectSearch, clearProjectSearch, sessionsPage, toggleTheme,
});

loadData();
setInterval(loadData, 30000);
loadUsageWindows();
setInterval(loadUsageWindows, 60000);
