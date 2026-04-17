import { render } from 'preact';
import { Footer } from './components/Footer';
import { Header } from './components/Header';
import { FilterBar } from './components/FilterBar';
import { RateWindowCard, BudgetCard, RateWindowUnavailable } from './components/RateWindowCard';
import { EstimationMeta } from './components/EstimationMeta';
import { ReconciliationBlock } from './components/ReconciliationBlock';
import { StatsCards } from './components/StatsCards';
import { showError, showSuccess, ToastContainer } from './components/Toast';
import { SubagentSummary as SubagentSummaryComponent } from './components/SubagentSummary';
import { EntrypointTable } from './components/EntrypointTable';
import { ServiceTiersTable } from './components/ServiceTiers';
import { ToolUsageTable } from './components/ToolUsageTable';
import { McpSummaryTable } from './components/McpSummaryTable';
import { BranchTable } from './components/BranchTable';
import { VersionTable } from './components/VersionTable';
import { HourlyChart } from './components/HourlyChart';
import { SessionsTable } from './components/SessionsTable';
import { ModelCostTable } from './components/ModelCostTable';
import { ProjectCostTable } from './components/ProjectCostTable';
import { DailyChart } from './components/DailyChart';
import { ModelChart } from './components/ModelChart';
import { ProjectChart } from './components/ProjectChart';

import type {
  UsageWindowsResponse,
  SubagentSummary,
  EntrypointSummary,
  ServiceTierSummary,
  ToolSummary,
  McpServerSummary,
  HourlyRow,
  BranchSummary,
  VersionSummary,
  DashboardData,
  DailyModelRow,
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
  metaText,
  planBadge,
} from './state/store';
import { $ } from './lib/format';
import { downloadCSV } from './lib/csv';
import { RANGE_LABELS } from './lib/charts';
import { applyTheme, getTheme } from './lib/theme';

// ── Theme bootstrap ──────────────────────────────────────────────────
applyTheme(getTheme());

function toggleTheme(): void {
  const current = document.documentElement.getAttribute('data-theme') === 'light' ? 'light' : 'dark';
  const next: 'light' | 'dark' = current === 'light' ? 'dark' : 'light';
  localStorage.setItem('theme', next);
  applyTheme(next);
  if (rawData.value) applyFilter();
}

// ── Local-only state ─────────────────────────────────────────────────
let previousSessionPercent: number | null = null;
let loadDataInFlight = false;
let loadUsageWindowsInFlight = false;

// ── URL persistence ──────────────────────────────────────────────────
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

function readURLModels(allModels: string[]): Set<string> {
  const param = new URLSearchParams(window.location.search).get('models');
  if (!param) return new Set(allModels);
  const fromURL = new Set(param.split(',').map(s => s.trim()).filter(Boolean));
  return new Set(allModels.filter(m => fromURL.has(m)));
}

function isDefaultModelSelection(allModels: string[]): boolean {
  if (selectedModels.value.size !== allModels.length) return false;
  return allModels.every(m => selectedModels.value.has(m));
}

function updateURL(): void {
  const allModels = rawData.value?.all_models ?? [];
  const params = new URLSearchParams();
  if (selectedRange.value !== '30d') params.set('range', selectedRange.value);
  if (!isDefaultModelSelection(allModels)) params.set('models', Array.from(selectedModels.value).join(','));
  if (projectSearchQuery.value) params.set('project', projectSearchQuery.value);
  const search = params.toString() ? '?' + params.toString() : '';
  history.replaceState(null, '', window.location.pathname + search);
}

function matchesProjectSearch(project: string): boolean {
  if (!projectSearchQuery.value) return true;
  return project.toLowerCase().includes(projectSearchQuery.value);
}

// ── Aggregations ─────────────────────────────────────────────────────
function buildAggregations(filteredDaily: DailyModelRow[], filteredSessions: typeof lastFilteredSessions.value) {
  const dailyMap: Record<string, DailyAgg> = {};
  for (const r of filteredDaily) {
    if (!dailyMap[r.day]) {
      dailyMap[r.day] = {
        day: r.day,
        input: 0,
        output: 0,
        cache_read: 0,
        cache_creation: 0,
        reasoning_output: 0,
      };
    }
    const d = dailyMap[r.day];
    d.input += r.input;
    d.output += r.output;
    d.cache_read += r.cache_read;
    d.cache_creation += r.cache_creation;
    d.reasoning_output += r.reasoning_output;
  }
  const daily = Object.values(dailyMap).sort((a, b) => a.day.localeCompare(b.day));

  const modelMap: Record<string, ModelAgg> = {};
  for (const r of filteredDaily) {
    if (!modelMap[r.model]) {
      modelMap[r.model] = {
        model: r.model,
        input: 0,
        output: 0,
        cache_read: 0,
        cache_creation: 0,
        reasoning_output: 0,
        turns: 0,
        sessions: 0,
        cost: 0,
        is_billable: r.cost > 0,
      };
    }
    const m = modelMap[r.model];
    m.input += r.input;
    m.output += r.output;
    m.cache_read += r.cache_read;
    m.cache_creation += r.cache_creation;
    m.reasoning_output += r.reasoning_output;
    m.turns += r.turns;
    m.cost += r.cost;
    if (r.cost > 0) m.is_billable = true;
  }

  for (const s of filteredSessions) {
    if (modelMap[s.model]) modelMap[s.model].sessions++;
  }
  const byModel = Object.values(modelMap).sort((a, b) => (b.input + b.output) - (a.input + a.output));

  const projMap: Record<string, ProjectAgg> = {};
  for (const s of filteredSessions) {
    if (!projMap[s.project]) {
      projMap[s.project] = {
        project: s.project,
        input: 0,
        output: 0,
        cache_read: 0,
        cache_creation: 0,
        reasoning_output: 0,
        turns: 0,
        sessions: 0,
        cost: 0,
      };
    }
    const p = projMap[s.project];
    p.input += s.input;
    p.output += s.output;
    p.cache_read += s.cache_read;
    p.cache_creation += s.cache_creation;
    p.reasoning_output += s.reasoning_output;
    p.turns += s.turns;
    p.sessions++;
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
    reasoning_output: byModel.reduce((s, m) => s + m.reasoning_output, 0),
    cost: filteredSessions.reduce((s, sess) => s + sess.cost, 0),
  };

  const confidenceBreakdown = Object.entries(
    filteredSessions.reduce<Record<string, { sessions: number; cost: number }>>((acc, session) => {
      const key = session.cost_confidence || 'low';
      if (!acc[key]) acc[key] = { sessions: 0, cost: 0 };
      acc[key].sessions += 1;
      acc[key].cost += session.cost;
      return acc;
    }, {})
  ).sort(([a], [b]) => confidenceRank(a) - confidenceRank(b));

  const billingModeBreakdown = Object.entries(
    filteredSessions.reduce<Record<string, { sessions: number; cost: number }>>((acc, session) => {
      const key = session.billing_mode || 'estimated_local';
      if (!acc[key]) acc[key] = { sessions: 0, cost: 0 };
      acc[key].sessions += 1;
      acc[key].cost += session.cost;
      return acc;
    }, {})
  ).sort((a, b) => b[1].sessions - a[1].sessions);

  const pricingVersions = Array.from(
    new Set(filteredSessions.map(session => session.pricing_version).filter(Boolean))
  );

  return { daily, byModel, byProject, totals, confidenceBreakdown, billingModeBreakdown, pricingVersions };
}

function confidenceRank(confidence: string): number {
  switch (confidence) {
    case 'low': return 0;
    case 'medium': return 1;
    case 'high': return 2;
    default: return 3;
  }
}

// ── Renderers (Preact into existing mount points) ────────────────────
function renderEstimationMeta(
  confidenceBreakdown: Array<[string, { sessions: number; cost: number }]>,
  billingModeBreakdown: Array<[string, { sessions: number; cost: number }]>,
  pricingVersions: string[]
): void {
  const container = $('estimation-meta');
  if (!container) return;

  if (!confidenceBreakdown.length && !billingModeBreakdown.length && !pricingVersions.length) {
    container.style.display = 'none';
    render(null, container);
    return;
  }

  container.style.display = 'grid';
  render(
    <EstimationMeta
      confidenceBreakdown={confidenceBreakdown}
      billingModeBreakdown={billingModeBreakdown}
      pricingVersions={pricingVersions}
    />,
    container
  );
}

function renderOpenAiReconciliation(reconciliation: DashboardData['openai_reconciliation']): void {
  const container = $('openai-reconciliation');
  if (!container) return;
  if (!reconciliation) {
    container.style.display = 'none';
    render(null, container);
    return;
  }
  container.style.display = '';
  render(<ReconciliationBlock reconciliation={reconciliation} />, container);
}

function renderPlaceholder(containerId: string, title: string, message: string): void {
  const container = $(containerId);
  if (!container) return;
  render(
    <div class="card">
      <h2>{title}</h2>
      <div class="muted">{message}</div>
    </div>,
    container
  );
}

function renderCodexSection(filteredDaily: DailyModelRow[], filteredSessions: typeof lastFilteredSessions.value): void {
  const section = $('codex-section');
  if (!section || !rawData.value) return;

  const codexDailyRows = filteredDaily.filter(r => r.provider === 'codex');
  const codexSessions = filteredSessions.filter(s => s.provider === 'codex');
  const hasCodex =
    codexDailyRows.length > 0 ||
    codexSessions.length > 0 ||
    rawData.value.provider_breakdown.some(row => row.provider === 'codex');
  section.style.display = hasCodex ? '' : 'none';
  if (!hasCodex) return;

  const { daily, byModel, byProject, totals } = buildAggregations(codexDailyRows, codexSessions);
  $('codex-daily-chart-title').textContent = 'Codex Daily Usage - ' + RANGE_LABELS[selectedRange.value];

  render(<StatsCards totals={totals} daily={daily} />, $('codex-stats-row'));
  render(<DailyChart daily={daily} />, $('codex-chart-daily'));
  render(<ModelChart byModel={byModel} />, $('codex-chart-model'));
  render(<ProjectChart byProject={byProject} />, $('codex-chart-project'));
  render(<ModelCostTable byModel={byModel} />, $('codex-model-cost-mount'));
  render(
    <ProjectCostTable
      byProject={byProject.slice(0, 30)}
      onExportCSV={() => exportProjectRowsCSV('codex-projects', byProject)}
    />,
    $('codex-project-cost-mount')
  );

  const codexTools = rawData.value.tool_summary.filter(row => row.provider === 'codex');
  if (codexTools.length) render(<ToolUsageTable data={codexTools} />, $('codex-tool-summary'));
  else renderPlaceholder('codex-tool-summary', 'Codex Tool Usage', 'No Codex tool calls were recorded.');

  const codexMcp = rawData.value.mcp_summary.filter(row => row.provider === 'codex');
  if (codexMcp.length) render(<McpSummaryTable data={codexMcp} />, $('codex-mcp-summary'));
  else renderPlaceholder('codex-mcp-summary', 'Codex MCP Servers', 'No MCP usage was recorded for Codex sessions.');

  const codexBranches = rawData.value.git_branch_summary.filter(row => row.provider === 'codex');
  if (codexBranches.length) render(<BranchTable data={codexBranches} />, $('codex-branch-summary'));
  else renderPlaceholder('codex-branch-summary', 'Codex Branches', 'Git branch metadata was not present in the recorded Codex logs.');

  const codexVersions = rawData.value.version_summary.filter(row => row.provider === 'codex');
  if (codexVersions.length) render(<VersionTable data={codexVersions} />, $('codex-version-summary'));
  else renderPlaceholder('codex-version-summary', 'Codex Versions', 'CLI version metadata was not available for the current Codex sessions.');

  const codexHourly = rawData.value.hourly_distribution.filter(row => row.provider === 'codex');
  if (codexHourly.length) render(<HourlyChart data={codexHourly} />, $('codex-hourly-chart'));
  else renderPlaceholder('codex-hourly-chart', 'Codex Hourly Distribution', 'No hourly token distribution is available for Codex in the current selection.');
}

// ── Filter driver ────────────────────────────────────────────────────
function applyFilter(): void {
  if (!rawData.value) return;
  const cutoff = getRangeCutoff(selectedRange.value);

  const filteredDaily = rawData.value.daily_by_model.filter(r =>
    selectedModels.value.has(r.model) && (!cutoff || r.day >= cutoff)
  );

  const filteredSessions = rawData.value.sessions_all.filter(s =>
    selectedModels.value.has(s.model) && (!cutoff || s.last_date >= cutoff) && matchesProjectSearch(s.project)
  );
  const { daily, byModel, byProject, totals, confidenceBreakdown, billingModeBreakdown, pricingVersions } =
    buildAggregations(filteredDaily, filteredSessions);

  $('daily-chart-title').textContent = 'Daily Token Usage - ' + RANGE_LABELS[selectedRange.value];

  render(<StatsCards totals={totals} daily={daily} />, $('stats-row'));
  renderEstimationMeta(confidenceBreakdown, billingModeBreakdown, pricingVersions);
  renderOpenAiReconciliation(rawData.value.openai_reconciliation);
  render(<DailyChart daily={daily} />, $('chart-daily'));
  render(<ModelChart byModel={byModel} />, $('chart-model'));
  render(<ProjectChart byProject={byProject} />, $('chart-project'));

  lastFilteredSessions.value = filteredSessions;
  lastByProject.value = byProject;

  render(<ModelCostTable byModel={byModel} />, $('model-cost-mount'));
  render(<SessionsTable onExportCSV={exportSessionsCSV} />, $('sessions-mount'));
  render(<ProjectCostTable byProject={lastByProject.value.slice(0, 30)} onExportCSV={exportProjectsCSV} />, $('project-cost-mount'));
  renderCodexSection(filteredDaily, filteredSessions);
}

// ── CSV Export ───────────────────────────────────────────────────────
function exportSessionsCSV(): void {
  const header = ['Session', 'Provider', 'Project', 'Last Active', 'Duration (min)', 'Model', 'Turns', 'Input', 'Output', 'Cached Input', 'Cache Creation', 'Reasoning Output', 'Est. Cost'];
  const rows = lastFilteredSessions.value.map(s => {
    const cost = s.cost;
    return [s.session_id, s.provider, s.project, s.last, s.duration_min, s.model, s.turns, s.input, s.output, s.cache_read, s.cache_creation, s.reasoning_output, cost.toFixed(4)];
  });
  downloadCSV('sessions', header, rows);
}

function exportProjectRowsCSV(filename: string, rowsData: ProjectAgg[]): void {
  const header = ['Project', 'Sessions', 'Turns', 'Input', 'Output', 'Cached Input', 'Cache Creation', 'Reasoning Output', 'Est. Cost'];
  const rows = rowsData.map(p =>
    [p.project, p.sessions, p.turns, p.input, p.output, p.cache_read, p.cache_creation, p.reasoning_output, p.cost.toFixed(4)]
  );
  downloadCSV(filename, header, rows);
}

function exportProjectsCSV(): void {
  exportProjectRowsCSV('projects', lastByProject.value);
}

// ── Usage windows ────────────────────────────────────────────────────
function renderUsageWindows(data: UsageWindowsResponse): void {
  const container = $('usage-windows');
  if (!container) return;

  if (!data.available) {
    planBadge.value = '';
    if (data.error) {
      container.style.display = 'grid';
      render(<RateWindowUnavailable error={data.error} />, container);
    } else {
      container.style.display = 'none';
      render(null, container);
    }
    return;
  }

  container.style.display = 'grid';
  render(
    <>
      {data.session && <RateWindowCard label="Session (5h)" window={data.session} />}
      {data.weekly && <RateWindowCard label="Weekly" window={data.weekly} />}
      {data.weekly_opus && <RateWindowCard label="Weekly Opus" window={data.weekly_opus} />}
      {data.weekly_sonnet && <RateWindowCard label="Weekly Sonnet" window={data.weekly_sonnet} />}
      {data.budget && (
        <BudgetCard
          used={data.budget.used}
          limit={data.budget.limit}
          currency={data.budget.currency}
          utilization={data.budget.utilization}
        />
      )}
    </>,
    container
  );

  // Session depletion inline alert (still via Toast for now; replaced by inline status in C4)
  if (data.session) {
    const currentPercent = 100 - data.session.used_percent;
    if (previousSessionPercent !== null) {
      if (previousSessionPercent > 0.01 && currentPercent <= 0.01) {
        showError('Session depleted \u2014 resets in ' + (data.session.resets_in_minutes ?? 0) + 'm');
      } else if (previousSessionPercent <= 0.01 && currentPercent > 0.01) {
        showSuccess('Session restored');
      }
    }
    previousSessionPercent = currentPercent;
  }

  planBadge.value = data.identity?.plan
    ? data.identity.plan.charAt(0).toUpperCase() + data.identity.plan.slice(1)
    : '';
}

// ── Secondary tables ─────────────────────────────────────────────────
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

function renderToolSummary(data: ToolSummary[]): void {
  const container = $('tool-summary');
  if (!container) return;
  if (!data.length) {
    container.style.display = 'none';
    render(null, container);
    return;
  }
  container.style.display = '';
  render(<ToolUsageTable data={data} />, container);
}

function renderMcpSummary(data: McpServerSummary[]): void {
  const container = $('mcp-summary');
  if (!container) return;
  if (!data.length) {
    container.style.display = 'none';
    render(null, container);
    return;
  }
  container.style.display = '';
  render(<McpSummaryTable data={data} />, container);
}

function renderBranchSummary(data: BranchSummary[]): void {
  const container = $('branch-summary');
  if (!container) return;
  if (!data.length) {
    container.style.display = 'none';
    render(null, container);
    return;
  }
  container.style.display = '';
  render(<BranchTable data={data} />, container);
}

function renderVersionSummary(data: VersionSummary[]): void {
  const container = $('version-summary');
  if (!container) return;
  if (!data.length) {
    container.style.display = 'none';
    render(null, container);
    return;
  }
  container.style.display = '';
  render(<VersionTable data={data} />, container);
}

function renderHourlyChart(data: HourlyRow[]): void {
  const container = $('hourly-chart');
  if (!container) return;
  if (!data.length) {
    container.style.display = 'none';
    render(null, container);
    return;
  }
  container.style.display = '';
  render(<HourlyChart data={data} />, container);
}

async function loadUsageWindows(): Promise<void> {
  if (loadUsageWindowsInFlight) return;
  loadUsageWindowsInFlight = true;
  try {
    const resp = await fetch('/api/usage-windows');
    if (!resp.ok) return;
    const data: UsageWindowsResponse = await resp.json();
    renderUsageWindows(data);
  } catch { /* silent */ }
  finally {
    loadUsageWindowsInFlight = false;
  }
}

// ── Data loading ─────────────────────────────────────────────────────
async function loadData(force = false): Promise<void> {
  if (loadDataInFlight && !force) return;
  loadDataInFlight = true;
  try {
    const resp = await fetch('/api/data');
    if (!resp.ok) {
      showError(`Failed to load data: HTTP ${resp.status}`);
      return;
    }
    const d: DashboardData = await resp.json();
    if (d.error) {
      showError(d.error);
      return;
    }
    metaText.value = 'Updated: ' + d.generated_at + ' \u00b7 Auto-refresh 30s';

    const isFirstLoad = rawData.value === null;
    rawData.value = d;

    if (isFirstLoad) {
      selectedRange.value = readURLRange();
      selectedModels.value = readURLModels(d.all_models);
      const urlProject = new URLSearchParams(window.location.search).get('project');
      if (urlProject) projectSearchQuery.value = urlProject;
    }

    applyFilter();
    if (rawData.value.subagent_summary) renderSubagentSummary(rawData.value.subagent_summary);
    if (rawData.value.entrypoint_breakdown) renderEntrypointBreakdown(rawData.value.entrypoint_breakdown);
    if (rawData.value.service_tiers) renderServiceTiers(rawData.value.service_tiers);
    if (rawData.value.tool_summary) renderToolSummary(rawData.value.tool_summary);
    if (rawData.value.mcp_summary) renderMcpSummary(rawData.value.mcp_summary);
    if (rawData.value.git_branch_summary) renderBranchSummary(rawData.value.git_branch_summary);
    if (rawData.value.version_summary) renderVersionSummary(rawData.value.version_summary);
    if (rawData.value.hourly_distribution) renderHourlyChart(rawData.value.hourly_distribution);
  } catch (e) {
    console.error(e);
  }
  finally {
    loadDataInFlight = false;
  }
}

// ── Preact mounts ────────────────────────────────────────────────────
const headerMount = document.getElementById('header-mount');
if (headerMount) {
  render(<Header onDataReload={loadData} onThemeToggle={toggleTheme} />, headerMount);
}

const filterBarMount = document.getElementById('filter-bar-mount');
if (filterBarMount) {
  render(<FilterBar onFilterChange={applyFilter} onURLUpdate={updateURL} />, filterBarMount);
}

const footerEl = document.querySelector('footer');
if (footerEl && footerEl.parentElement) {
  render(<Footer />, footerEl.parentElement, footerEl);
}

const toastRoot = document.createElement('div');
document.body.appendChild(toastRoot);
render(<ToastContainer />, toastRoot);

// ── Boot ─────────────────────────────────────────────────────────────
loadData();
setInterval(loadData, 30000);
loadUsageWindows();
setInterval(loadUsageWindows, 60000);
