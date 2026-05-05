import { render } from 'preact';
import { DatePicker } from '../components/today/DatePicker';
import { DaysHoursHeatmap } from '../components/today/DaysHoursHeatmap';
import { HourHeatstrip } from '../components/today/HourHeatstrip';
import { HourTimeline } from '../components/today/HourTimeline';
import { TodayKpis } from '../components/today/TodayKpis';
import { WeekdayHourHeatmap } from '../components/today/WeekdayHourHeatmap';
import { ActivityHeatmap } from '../components/charts/ActivityHeatmap';
import { AgentDistribution } from '../components/agents/AgentDistribution';
import { AgentKpis } from '../components/agents/AgentKpis';
import { AgentSetupBanner } from '../components/agents/AgentSetupBanner';
import { AgentSpawnBatches } from '../components/agents/AgentSpawnBatches';
import { AgentTimeline } from '../components/agents/AgentTimeline';
import { AgentToolSpectrum } from '../components/agents/AgentToolSpectrum';
import { AgentTopSessions } from '../components/agents/AgentTopSessions';
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
import { RateWindowCard, BudgetCard, ClaudeAdminFallbackGrid, RateWindowUnavailable } from '../components/RateWindowCard';
import { ReconciliationBlock } from '../components/ReconciliationBlock';
import { ServiceTiersTable } from '../components/tables/ServiceTiers';
import { SessionsTable } from '../components/tables/SessionsTable';
import { StatsCards } from '../components/StatsCards';
import { SubagentSummary as SubagentSummaryComponent } from '../components/SubagentSummary';
import { CodexPlanHistory } from '../components/CodexPlanHistory';
import { CodexPlanKpi } from '../components/CodexPlanKpi';
import { SubscriptionQuotaCard } from '../components/SubscriptionQuotaCard';
import { SubscriptionHistoryChart } from '../components/SubscriptionHistoryChart';
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
  heatmapMetric,
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
  'subscription-quota': 'overview',
  'claude-usage': 'overview',
  'agent-status': 'overview',
  'estimation-meta': 'overview',
  'official-sync': 'overview',
  'openai-reconciliation': 'overview',
  'stats-row': 'overview',
  'codex-plan-kpi-mount': 'overview',
  'codex-plan-history-mount': 'activity',
  'daily-chart-card': 'activity',
  'model-chart-card': 'activity',
  'project-chart-card': 'activity',
  'hourly-chart': 'activity',
  'activity-heatmap': 'activity',
  'subagent-summary': 'breakdowns',
  'agent-setup-banner': 'breakdowns',
  'agent-kpis-row': 'breakdowns',
  'agent-timeline': 'breakdowns',
  'agent-distribution': 'breakdowns',
  'agent-top-sessions': 'breakdowns',
  'agent-spawn-batches': 'breakdowns',
  'agent-tool-spectrum': 'breakdowns',
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
  'backup-panel': 'backup',
  'today-date-picker-mount': 'today',
  'today-kpis-mount': 'today',
  'today-hour-timeline-mount': 'today',
  'today-hour-heatstrip-mount': 'today',
  'today-days-hours-30-mount': 'today',
  'today-days-hours-7-mount': 'today',
  'today-weekday-hour-mount': 'today',
};

const SECTION_DISPLAY_MODE: Record<string, string> = {
  'usage-windows': 'grid',
  'subscription-quota': 'block',
  'agent-status': 'grid',
  'estimation-meta': 'grid',
  'stats-row': 'grid',
  'agent-kpis-row': 'grid',
  'codex-plan-kpi-mount': 'grid',
  'today-kpis-mount': 'grid',
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

function renderSection(
  mountId: string,
  hasContent: boolean,
  element: import('preact').VNode,
  displayMode?: string,
): void {
  const container = $(mountId);
  if (!container) return;
  setSectionVisibility(mountId, hasContent, displayMode ?? '');
  render(hasContent ? element : null, container);
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
  renderSection(
    'openai-reconciliation',
    !!reconciliation,
    <ReconciliationBlock reconciliation={reconciliation!} />,
  );
}

function renderOfficialSync(summary: DashboardData['official_sync']): void {
  renderSection(
    'official-sync',
    !!summary?.available,
    <OfficialSyncPanel summary={summary!} providerFilter={selectedProvider.value} />,
  );
}

function renderAgentTelemetry(data: DashboardData): void {
  const { agent_telemetry } = data;
  const totalCostUsd = data.provider_breakdown.reduce((s, p) => s + p.cost, 0);
  const hasAgentActivity = agent_telemetry.totals.sessions > 0;

  // Setup banner (always render — component decides whether to show)
  const bannerContainer = $('agent-setup-banner');
  if (bannerContainer) {
    setSectionVisibility('agent-setup-banner', true);
    render(<AgentSetupBanner telemetry={agent_telemetry} />, bannerContainer);
  }

  // KPI cards
  renderSection(
    'agent-kpis-row',
    hasAgentActivity,
    <AgentKpis telemetry={agent_telemetry} totalCostUsd={totalCostUsd} />,
    'grid',
  );

  // Timeline
  renderSection(
    'agent-timeline',
    agent_telemetry.timeline.length > 0,
    <AgentTimeline timeline={agent_telemetry.timeline} />,
  );

  // Distribution
  renderSection(
    'agent-distribution',
    agent_telemetry.distribution.length > 0,
    <AgentDistribution data={agent_telemetry.distribution} />,
  );

  // Top sessions
  renderSection(
    'agent-top-sessions',
    agent_telemetry.top_sessions.length > 0,
    <AgentTopSessions data={agent_telemetry.top_sessions} />,
  );

  // Spawn batches
  renderSection(
    'agent-spawn-batches',
    agent_telemetry.spawn_batches.length > 0,
    <AgentSpawnBatches data={agent_telemetry.spawn_batches} />,
  );

  // Tool spectrum
  renderSection(
    'agent-tool-spectrum',
    agent_telemetry.tool_spectrum.length > 0,
    <AgentToolSpectrum data={agent_telemetry.tool_spectrum} />,
  );
}

function renderSubagentSummary(summary: DashboardData['subagent_summary']): void {
  renderSection(
    'subagent-summary',
    summary.subagent_turns > 0,
    <SubagentSummaryComponent summary={summary} />,
  );
}

function renderEntrypointBreakdown(data: DashboardData['entrypoint_breakdown']): void {
  renderSection('entrypoint-breakdown', data.length > 0, <EntrypointTable data={data} />);
}

function renderServiceTiers(data: DashboardData['service_tiers']): void {
  renderSection('service-tiers', data.length > 0, <ServiceTiersTable data={data} />);
}

function renderToolSummary(data: DashboardData['tool_summary']): void {
  renderSection('tool-summary', data.length > 0, <ToolUsageTable data={data} />);
}

function renderMcpSummary(data: DashboardData['mcp_summary']): void {
  renderSection('mcp-summary', data.length > 0, <McpSummaryTable data={data} />);
}

function renderBranchSummary(data: DashboardData['git_branch_summary']): void {
  renderSection('branch-summary', data.length > 0, <BranchTable data={data} />);
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
          <div style={{ flex: '1 1 260px', minWidth: '220px' }}>
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
  renderSection('hourly-chart', data.length > 0, <HourlyChart data={data} />);
}

function renderSubscriptionQuota(
  section: DashboardData['subscription_quota']
): void {
  const container = $('subscription-quota');
  if (!container) return;
  const hasContent =
    !!section &&
    (section.providers.length > 0 ||
      section.history.length > 0 ||
      section.changelog.length > 0);
  if (!hasContent) {
    setSectionVisibility('subscription-quota', false, 'block');
    render(null, container);
    return;
  }
  setSectionVisibility('subscription-quota', true, 'block');
  render(
    <div class="subscription-quota-section">
      <div class="subscription-quota-grid">
        {section!.providers.map(snap => (
          <SubscriptionQuotaCard key={snap.provider} snapshot={snap} />
        ))}
      </div>
      <SubscriptionHistoryChart
        history={section!.history}
        changelog={section!.changelog}
      />
    </div>,
    container
  );
}

function renderCodexPlan(section: DashboardData['codex_plan']): void {
  const hasToday = !!(section?.today);
  const hasHistory = !!(section?.history && section.history.length > 0);
  const hasAny = hasToday || hasHistory;

  // KPI tile
  if (hasToday) {
    renderSection(
      'codex-plan-kpi-mount',
      true,
      <CodexPlanKpi today={section!.today!} />,
      'grid',
    );
  } else {
    setSectionVisibility('codex-plan-kpi-mount', false, 'grid');
    render(null, $('codex-plan-kpi-mount'));
  }

  // History chart
  if (hasHistory) {
    renderSection(
      'codex-plan-history-mount',
      true,
      <CodexPlanHistory history={section!.history} />,
    );
  } else {
    setSectionVisibility('codex-plan-history-mount', false);
    render(null, $('codex-plan-history-mount'));
  }

  void hasAny; // used implicitly via individual section visibility above
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
  if (data.source === 'admin' && data.admin_fallback) {
    render(
      <>
        <ClaudeAdminFallbackGrid summary={data.admin_fallback} />
        <div style={{ gridColumn: '1 / -1' }}>
          <InlineStatus placement="rate-windows" />
        </div>
      </>,
      container
    );
    setPreviousSessionPercent(null);
    clearStatusMessage();
    return;
  }
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
  renderSection('claude-usage', !!(data.last_run || data.latest_snapshot), <ClaudeUsagePanel data={data} />);
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

  const handleMetricChange = (next: import('../state/store').HeatmapMetric) => {
    heatmapMetric.value = next;
    syncDashboardUrl();
    renderActivityHeatmap(data);
  };

  render(
    <ActivityHeatmap
      data={data}
      metric={heatmapMetric.value}
      onMetricChange={handleMetricChange}
    />,
    container
  );
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

export function renderTodayView(
  data: import('../state/types').TodayResponse,
  onDateChange: (date: string | null) => void
): void {
  // Date picker (always shown when Today tab is active)
  const pickerContainer = $('today-date-picker-mount');
  if (pickerContainer) {
    setSectionVisibility('today-date-picker-mount', true);
    render(<DatePicker onDateChange={onDateChange} />, pickerContainer);
  }

  // KPI grid
  renderSection(
    'today-kpis-mount',
    true,
    <TodayKpis totals={data.totals} day={data.day} />,
    'grid'
  );

  // Hour timeline
  renderSection(
    'today-hour-timeline-mount',
    true,
    <div style={{ padding: '20px' }}>
      <div class="section-title" style={{ marginBottom: '12px' }}>Hour timeline — {data.day}</div>
      <HourTimeline hours={data.hours} />
    </div>
  );

  // Hour heatstrip
  renderSection(
    'today-hour-heatstrip-mount',
    true,
    <div style={{ padding: '20px' }}>
      <div class="section-title" style={{ marginBottom: '12px' }}>Hour heatstrip</div>
      <HourHeatstrip hours={data.hours} />
    </div>
  );

  // 30-day × 24-hour grid
  renderSection(
    'today-days-hours-30-mount',
    data.days_hours_30.length > 0,
    <div style={{ padding: '20px', overflowX: 'auto' }}>
      <DaysHoursHeatmap
        cells={data.days_hours_30}
        daysCount={30}
        title="30 days × 24 hours"
        onDayClick={onDateChange}
      />
    </div>
  );

  // 7-day × 24-hour grid
  renderSection(
    'today-days-hours-7-mount',
    data.days_hours_7.length > 0,
    <div style={{ padding: '20px', overflowX: 'auto' }}>
      <DaysHoursHeatmap
        cells={data.days_hours_7}
        daysCount={7}
        title="7 days × 24 hours"
        onDayClick={onDateChange}
      />
    </div>
  );

  // 7×24 weekday-hour pattern (90 days)
  renderSection(
    'today-weekday-hour-mount',
    data.weekday_hour_90.length > 0,
    <div style={{ padding: '20px', overflowX: 'auto' }}>
      <div class="section-title" style={{ marginBottom: '12px' }}>
        Weekday × hour pattern (90-day window)
      </div>
      <WeekdayHourHeatmap cells={data.weekday_hour_90} />
    </div>
  );

  refreshSectionVisibility();
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
  renderSubscriptionQuota(data.subscription_quota);
  renderCodexPlan(data.codex_plan);
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
  renderAgentTelemetry(data);
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
