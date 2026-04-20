import { render } from 'preact';
import { ActivityHeatmap } from '../components/charts/ActivityHeatmap';
import { AgentStatusCard } from '../components/AgentStatusCard';
import { BranchTable } from '../components/tables/BranchTable';
import { ClaudeUsagePanel } from '../components/ClaudeUsagePanel';
import { CostReconciliationPanel } from '../components/CostReconciliationPanel';
import { DailyChart } from '../components/charts/DailyChart';
import { EntrypointTable } from '../components/tables/EntrypointTable';
import { EstimationMeta } from '../components/EstimationMeta';
import { HourlyChart } from '../components/charts/HourlyChart';
import { InlineStatus } from '../components/InlineStatus';
import { McpSummaryTable } from '../components/tables/McpSummaryTable';
import { ModelChart } from '../components/charts/ModelChart';
import { ModelCostTable } from '../components/tables/ModelCostTable';
import { OfficialSyncPanel } from '../components/OfficialSyncPanel';
import { ProjectChart } from '../components/charts/ProjectChart';
import { ProjectCostTable } from '../components/tables/ProjectCostTable';
import { RateWindowCard, BudgetCard, RateWindowUnavailable } from '../components/RateWindowCard';
import { ReconciliationBlock } from '../components/ReconciliationBlock';
import { ServiceTiersTable } from '../components/tables/ServiceTiers';
import { SessionsTable } from '../components/tables/SessionsTable';
import { StatsCards } from '../components/StatsCards';
import { SubagentSummary as SubagentSummaryComponent } from '../components/SubagentSummary';
import { ToolUsageTable } from '../components/tables/ToolUsageTable';
import { VersionDonut } from '../components/charts/VersionDonut';
import { VersionTable } from '../components/tables/VersionTable';
import { WeeklyChart } from '../components/charts/WeeklyChart';
import { $ } from '../lib/format';
import { RANGE_LABELS } from '../lib/charts';
import {
  activeDashboardTab,
  billingBlocksData,
  contextWindowData,
  costReconciliationData,
  isSectionCollapsed,
  lastByProject,
  lastFilteredSessions,
  planBadge,
  projectSearchQuery,
  selectedBucket,
  selectedModels,
  selectedProvider,
  selectedRange,
  setSectionCollapsed,
  syncDashboardUrl,
  versionDonutMetric,
  type DashboardTab,
} from '../state/store';
import type {
  AgentStatusSnapshot,
  ClaudeUsageResponse,
  CommunitySignal,
  DashboardData,
  HeatmapData,
  UsageWindowsResponse,
} from '../state/types';
import { buildAggregations, buildWeeklyAgg, getRangeCutoff } from './aggregation';

const SECTION_TAB_MAP: Record<string, DashboardTab> = {
  'usage-windows': 'overview',
  'claude-usage': 'overview',
  'agent-status': 'overview',
  'estimation-meta': 'overview',
  'official-sync': 'overview',
  'openai-reconciliation': 'overview',
  'stats-row': 'overview',
  'daily-chart-card': 'activity',
  'model-chart-card': 'activity',
  'project-chart-card': 'activity',
  'hourly-chart': 'activity',
  'activity-heatmap': 'activity',
  'subagent-summary': 'breakdowns',
  'entrypoint-breakdown': 'breakdowns',
  'service-tiers': 'breakdowns',
  'tool-summary': 'breakdowns',
  'mcp-summary': 'breakdowns',
  'branch-summary': 'breakdowns',
  'version-summary': 'breakdowns',
  'cost-reconciliation': 'breakdowns',
  'model-cost-mount': 'tables',
  'sessions-mount': 'tables',
  'project-cost-mount': 'tables',
};

const SECTION_DISPLAY_MODE: Record<string, string> = {
  'usage-windows': 'grid',
  'agent-status': 'grid',
  'estimation-meta': 'grid',
  'stats-row': 'grid',
};

function matchesProvider<T extends { provider?: string }>(row: T): boolean {
  if (selectedProvider.value === 'both') return true;
  return row.provider === selectedProvider.value;
}

function matchesProjectSearch(project: string, displayName?: string): boolean {
  const query = projectSearchQuery.value;
  if (!query) return true;
  if (project.toLowerCase().includes(query)) return true;
  if (displayName && displayName.toLowerCase().includes(query)) return true;
  return false;
}

export function setSectionVisibility(
  sectionId: string,
  hasContent: boolean,
  displayMode = ''
): void {
  const container = $(sectionId);
  if (!container) return;
  container.dataset['hasContent'] = hasContent ? '1' : '0';
  const visibleInTab = SECTION_TAB_MAP[sectionId] === activeDashboardTab.value;
  container.style.display = hasContent && visibleInTab ? displayMode : 'none';
}

export function refreshSectionVisibility(): void {
  for (const [sectionId, tab] of Object.entries(SECTION_TAB_MAP)) {
    const container = $(sectionId);
    if (!container) continue;
    const hasContent = container.dataset['hasContent'] !== '0';
    const displayMode = SECTION_DISPLAY_MODE[sectionId] ?? '';
    container.style.display = hasContent && tab === activeDashboardTab.value ? displayMode : 'none';
  }
}

function renderEstimationMeta(
  confidenceBreakdown: ReturnType<typeof buildAggregations>['confidenceBreakdown'],
  billingModeBreakdown: ReturnType<typeof buildAggregations>['billingModeBreakdown'],
  pricingVersions: string[]
): void {
  const container = $('estimation-meta');
  if (!container) return;
  if (!confidenceBreakdown.length && !billingModeBreakdown.length && !pricingVersions.length) {
    setSectionVisibility('estimation-meta', false, 'grid');
    render(null, container);
    return;
  }

  setSectionVisibility('estimation-meta', true, 'grid');
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
    setSectionVisibility('openai-reconciliation', false);
    render(null, container);
    return;
  }
  setSectionVisibility('openai-reconciliation', true);
  render(<ReconciliationBlock reconciliation={reconciliation} />, container);
}

function renderOfficialSync(summary: DashboardData['official_sync']): void {
  const container = $('official-sync');
  if (!container) return;
  if (!summary?.available) {
    setSectionVisibility('official-sync', false);
    render(null, container);
    return;
  }
  setSectionVisibility('official-sync', true);
  render(
    <OfficialSyncPanel summary={summary} providerFilter={selectedProvider.value} />,
    container
  );
}

function renderSubagentSummary(summary: DashboardData['subagent_summary']): void {
  const container = $('subagent-summary');
  if (!container) return;
  if (summary.subagent_turns === 0) {
    setSectionVisibility('subagent-summary', false);
    render(null, container);
    return;
  }
  setSectionVisibility('subagent-summary', true);
  render(<SubagentSummaryComponent summary={summary} />, container);
}

function renderEntrypointBreakdown(data: DashboardData['entrypoint_breakdown']): void {
  const container = $('entrypoint-breakdown');
  if (!container) return;
  if (!data.length) {
    setSectionVisibility('entrypoint-breakdown', false);
    render(null, container);
    return;
  }
  setSectionVisibility('entrypoint-breakdown', true);
  render(<EntrypointTable data={data} />, container);
}

function renderServiceTiers(data: DashboardData['service_tiers']): void {
  const container = $('service-tiers');
  if (!container) return;
  if (!data.length) {
    setSectionVisibility('service-tiers', false);
    render(null, container);
    return;
  }
  setSectionVisibility('service-tiers', true);
  render(<ServiceTiersTable data={data} />, container);
}

function renderToolSummary(data: DashboardData['tool_summary']): void {
  const container = $('tool-summary');
  if (!container) return;
  if (!data.length) {
    setSectionVisibility('tool-summary', false);
    render(null, container);
    return;
  }
  setSectionVisibility('tool-summary', true);
  render(<ToolUsageTable data={data} />, container);
}

function renderMcpSummary(data: DashboardData['mcp_summary']): void {
  const container = $('mcp-summary');
  if (!container) return;
  if (!data.length) {
    setSectionVisibility('mcp-summary', false);
    render(null, container);
    return;
  }
  setSectionVisibility('mcp-summary', true);
  render(<McpSummaryTable data={data} />, container);
}

function renderBranchSummary(data: DashboardData['git_branch_summary']): void {
  const container = $('branch-summary');
  if (!container) return;
  if (!data.length) {
    setSectionVisibility('branch-summary', false);
    render(null, container);
    return;
  }
  setSectionVisibility('branch-summary', true);
  render(<BranchTable data={data} />, container);
}

function renderVersionSummary(data: DashboardData['version_summary']): void {
  const container = $('version-summary');
  if (!container) return;
  if (!data.length) {
    setSectionVisibility('version-summary', false);
    render(null, container);
    return;
  }
  setSectionVisibility('version-summary', true);

  const handleMetricChange = (next: import('../state/store').VersionMetric) => {
    versionDonutMetric.value = next;
    syncDashboardUrl();
    renderVersionSummary(data);
  };
  const collapsed = isSectionCollapsed('version-summary');
  const toggleCollapsed = () => {
    setSectionCollapsed('version-summary', !collapsed);
    syncDashboardUrl();
    renderVersionSummary(data);
  };

  render(
    <div class="table-card">
      <div class="section-header" style={{ padding: '20px 20px 0' }}>
        <h2 class="section-title" style={{ margin: 0 }}>
          CLI Versions
        </h2>
        <div class="section-actions">
          <button
            class="section-toggle"
            type="button"
            aria-expanded={!collapsed}
            aria-controls="version-summary-content"
            onClick={toggleCollapsed}
          >
            {collapsed ? 'Show' : 'Hide'}
          </button>
        </div>
      </div>
      {!collapsed && (
        <div
          id="version-summary-content"
          style={{
            display: 'flex',
            gap: '24px',
            alignItems: 'flex-start',
            flexWrap: 'wrap',
            padding: '20px',
          }}
        >
          <div style={{ flex: '1 1 260px', minWidth: '220px', height: '300px' }}>
            <VersionDonut
              rows={data}
              metric={versionDonutMetric.value}
              onMetricChange={handleMetricChange}
            />
          </div>
          <div style={{ flex: '2 1 320px', minWidth: '280px' }}>
            <VersionTable data={data} title={null} />
          </div>
        </div>
      )}
    </div>,
    container
  );
}

function renderHourlyChart(data: DashboardData['hourly_distribution']): void {
  const container = $('hourly-chart');
  if (!container) return;
  if (!data.length) {
    setSectionVisibility('hourly-chart', false);
    render(null, container);
    return;
  }
  setSectionVisibility('hourly-chart', true);
  render(<HourlyChart data={data} />, container);
}

export function renderUsageWindows(
  data: UsageWindowsResponse,
  previousSessionPercent: number | null,
  setPreviousSessionPercent: (value: number | null) => void,
  setStatusMessage: (message: string, isError?: boolean) => void,
  clearStatusMessage: () => void
): void {
  const container = $('usage-windows');
  if (!container) return;

  if (!data.available) {
    planBadge.value = '';
    if (data.error) {
      setSectionVisibility('usage-windows', true, 'grid');
      render(<RateWindowUnavailable error={data.error} />, container);
    } else {
      setSectionVisibility('usage-windows', false, 'grid');
      render(null, container);
    }
    return;
  }

  setSectionVisibility('usage-windows', true, 'grid');
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
      <div style={{ gridColumn: '1 / -1' }}>
        <InlineStatus placement="rate-windows" />
      </div>
    </>,
    container
  );

  if (data.session) {
    const currentPercent = 100 - data.session.used_percent;
    if (previousSessionPercent !== null) {
      if (previousSessionPercent > 0.01 && currentPercent <= 0.01) {
        setStatusMessage(`Session depleted - resets in ${data.session.resets_in_minutes ?? 0}m`, true);
      } else if (previousSessionPercent <= 0.01 && currentPercent > 0.01) {
        clearStatusMessage();
      }
    }
    setPreviousSessionPercent(currentPercent);
  }

  planBadge.value = data.identity?.plan
    ? data.identity.plan.charAt(0).toUpperCase() + data.identity.plan.slice(1)
    : '';
}

export function renderClaudeUsage(data: ClaudeUsageResponse): void {
  const container = $('claude-usage');
  if (!container) return;
  if (!data.last_run && !data.latest_snapshot) {
    setSectionVisibility('claude-usage', false);
    render(null, container);
    return;
  }
  setSectionVisibility('claude-usage', true);
  render(<ClaudeUsagePanel data={data} />, container);
}

export function renderAgentStatus(
  snapshot: AgentStatusSnapshot,
  communitySignal: CommunitySignal | null
): void {
  const container = $('agent-status');
  if (!container) return;
  setSectionVisibility('agent-status', true, 'grid');
  render(<AgentStatusCard snapshot={snapshot} communitySignal={communitySignal} />, container);
}

export function renderActivityHeatmap(data: HeatmapData | null): void {
  const container = $('activity-heatmap');
  if (!container) return;
  if (!data) {
    setSectionVisibility('activity-heatmap', false);
    render(null, container);
    return;
  }
  setSectionVisibility('activity-heatmap', true);
  render(<ActivityHeatmap data={data} />, container);
}

export function renderCostReconciliation(): void {
  const container = $('cost-reconciliation');
  if (!container) return;
  const data = costReconciliationData.value;
  if (!data || !data.enabled) {
    setSectionVisibility('cost-reconciliation', false);
    render(null, container);
    return;
  }
  setSectionVisibility('cost-reconciliation', true);
  render(<CostReconciliationPanel data={data} />, container);
}

export function renderDashboardView(
  data: DashboardData,
  focusSingleModel: (model: string) => void,
  focusProjectQuery: (project: string) => void,
  exportSessionsCSV: () => void,
  exportProjectsCSV: () => void
): void {
  const cutoff = getRangeCutoff(selectedRange.value);
  const filteredDaily = data.daily_by_model.filter(
    row =>
      selectedModels.value.has(row.model) &&
      (!cutoff || row.day >= cutoff) &&
      matchesProvider(row)
  );
  const filteredSessions = data.sessions_all.filter(
    session =>
      selectedModels.value.has(session.model) &&
      (!cutoff || session.last_date >= cutoff) &&
      matchesProjectSearch(session.project, session.display_name) &&
      matchesProvider(session)
  );
  const {
    daily,
    byModel,
    byProject,
    totals,
    confidenceBreakdown,
    billingModeBreakdown,
    pricingVersions,
  } = buildAggregations(filteredDaily, filteredSessions);

  const providerLabel =
    selectedProvider.value === 'both' ? '' : ` (${selectedProvider.value})`;
  const bucketIsWeek = selectedBucket.value === 'week';
  const activeDays = daily.filter(day => day.cost > 0).length;
  const activeDayCostNanos = Math.round(
    daily.reduce((sum, day) => sum + day.cost, 0) * 1_000_000_000
  );
  const chartTitleEl = $('daily-chart-title');
  if (chartTitleEl) {
    chartTitleEl.textContent =
      (bucketIsWeek ? 'Weekly Token Usage - ' : 'Daily Token Usage - ') +
      RANGE_LABELS[selectedRange.value] +
      providerLabel;
  }

  render(
    <StatsCards
      totals={totals}
      daily={daily}
      activeDays={activeDays}
      activeDayTotalCostNanos={activeDayCostNanos}
      cacheEfficiency={data.cache_efficiency}
      billingBlocks={billingBlocksData.value}
      contextWindow={contextWindowData.value}
    />,
    $('stats-row')
  );
  setSectionVisibility('stats-row', true, 'grid');
  renderEstimationMeta(confidenceBreakdown, billingModeBreakdown, pricingVersions);
  renderOfficialSync(data.official_sync);
  renderOpenAiReconciliation(data.openai_reconciliation);

  if (bucketIsWeek) {
    const weekly = buildWeeklyAgg(data.weekly_by_model, selectedModels.value, selectedRange.value);
    render(<WeeklyChart weekly={weekly} />, $('chart-daily'));
    setSectionVisibility('daily-chart-card', weekly.length > 0);
  } else {
    render(<DailyChart daily={daily} />, $('chart-daily'));
    setSectionVisibility('daily-chart-card', daily.length > 0);
  }

  render(<ModelChart byModel={byModel} onSelectModel={focusSingleModel} />, $('chart-model'));
  render(
    <ProjectChart
      byProject={byProject}
      onSelectProject={project => focusProjectQuery(project.display_name || project.project)}
    />,
    $('chart-project')
  );
  setSectionVisibility('model-chart-card', byModel.length > 0);
  setSectionVisibility('project-chart-card', byProject.length > 0);

  lastFilteredSessions.value = filteredSessions;
  lastByProject.value = byProject;

  render(<ModelCostTable byModel={byModel} onSelectModel={focusSingleModel} />, $('model-cost-mount'));
  render(
    <SessionsTable
      onExportCSV={exportSessionsCSV}
      onSelectProject={session => focusProjectQuery(session.display_name || session.project)}
      onSelectModel={focusSingleModel}
    />,
    $('sessions-mount')
  );
  render(
    <ProjectCostTable
      byProject={lastByProject.value.slice(0, 30)}
      onExportCSV={exportProjectsCSV}
      onSelectProject={project => focusProjectQuery(project.display_name || project.project)}
    />,
    $('project-cost-mount')
  );
  setSectionVisibility('model-cost-mount', byModel.length > 0);
  setSectionVisibility('sessions-mount', filteredSessions.length > 0);
  setSectionVisibility('project-cost-mount', lastByProject.value.length > 0);

  renderSubagentSummary(data.subagent_summary);
  renderEntrypointBreakdown((data.entrypoint_breakdown ?? []).filter(matchesProvider));
  renderServiceTiers((data.service_tiers ?? []).filter(matchesProvider));
  renderToolSummary((data.tool_summary ?? []).filter(matchesProvider));
  renderMcpSummary((data.mcp_summary ?? []).filter(matchesProvider));
  renderBranchSummary((data.git_branch_summary ?? []).filter(matchesProvider));
  renderVersionSummary((data.version_summary ?? []).filter(matchesProvider));
  renderHourlyChart((data.hourly_distribution ?? []).filter(matchesProvider));
  renderCostReconciliation();
  refreshSectionVisibility();
}
