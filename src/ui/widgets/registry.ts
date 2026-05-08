/**
 * Widget registry — Feature 2: customizable dashboard layouts.
 *
 * Each WidgetDef represents one drag/resize-able card on the dashboard.
 * The `render(el)` function mounts content into the supplied element;
 * it does this by setting el.id to the existing static mount-div ID so
 * all existing render functions (which call document.getElementById) still work.
 *
 * This lets us adopt GridStack without rewriting every component renderer.
 */
import { invokeMountCallback } from './mount-registry';

export type DashboardScreen =
  | 'overview'
  | 'activity'
  | 'breakdowns'
  | 'tables'
  | 'projects';

export interface PlacedWidget {
  i: string;
  x: number;
  y: number;
  w: number;
  h: number;
  minW?: number | undefined;
  minH?: number | undefined;
}

export interface ScreenLayout {
  widgets: PlacedWidget[];
  hidden: string[];
}

export interface WidgetSize {
  w: number;
  h: number;
}

export type WidgetCategory =
  | 'kpi'
  | 'chart'
  | 'table'
  | 'heatmap'
  | 'agent'
  | 'today'
  | 'codex'
  | 'system';

export interface WidgetDef {
  /** Stable id — used as GridStack item gs-id and as the key in ScreenLayout. */
  id: string;
  /** Sentence-case display name shown in the picker. */
  title: string;
  description?: string;
  category: WidgetCategory;
  screens: DashboardScreen[];
  defaultSize: WidgetSize;
  minW?: number;
  minH?: number;
  /**
   * Mount the widget into `el`. The implementation sets `el.id` to the
   * legacy mount-div id so existing `document.getElementById` calls resolve
   * to the GridStack-managed element.
   */
  render: (el: HTMLElement) => void;
  /** Optional cleanup (e.g. clear timers) when the widget is removed. */
  destroy?: (el: HTMLElement) => void;
  /**
   * If true, the widget is auto-hidden (display: none on its grid item)
   * when its data response reports no content. The widget remains in the
   * saved layout and reappears automatically when content returns. The
   * AddWidgetPicker surfaces it under "Hidden because empty" so the user
   * can still discover it.
   */
  hideWhenEmpty?: boolean;
}

/** Sets el.id so legacy getElementById-based renderers find the right node. */
function mount(id: string): (el: HTMLElement) => void {
  return (el: HTMLElement) => {
    el.id = id;
  };
}

export const WIDGET_CATALOG: WidgetDef[] = [
  // ── Overview tab ─────────────────────────────────────────────────────────
  {
    id: 'usage-windows',
    title: 'Rate windows',
    description: 'Session and weekly rate-limit progress bars',
    category: 'kpi',
    screens: ['overview'],
    defaultSize: { w: 4, h: 2 },
    minW: 2,
    minH: 1,
    render: mount('usage-windows'),
    hideWhenEmpty: true,
  },
  {
    id: 'subscription-quota',
    title: 'Subscription quota',
    description: 'Provider subscription utilization and history chart',
    category: 'kpi',
    screens: ['overview'],
    // Renders six rate-window sub-cards (Session/Weekly/Weekly Sonnet/
    // Weekly Opus/Claude/Codex). Natural content height ≈ 1300 px which at
    // the 132 px GridStack cellHeight is exactly 10 rows.
    defaultSize: { w: 4, h: 10 },
    minW: 2,
    minH: 8,
    render: mount('subscription-quota'),
  },
  {
    id: 'claude-usage',
    title: 'Claude usage',
    description: 'Claude API usage details from the credentials file',
    category: 'kpi',
    screens: ['overview'],
    defaultSize: { w: 4, h: 2 },
    minW: 2,
    minH: 1,
    render: mount('claude-usage'),
  },
  {
    id: 'agent-status',
    title: 'Agent status',
    description: 'Upstream provider health (Claude, OpenAI)',
    category: 'system',
    screens: ['overview'],
    defaultSize: { w: 4, h: 2 },
    minW: 2,
    minH: 1,
    render: mount('agent-status'),
  },
  {
    id: 'estimation-meta',
    title: 'Estimation metadata',
    description: 'Confidence, billing mode, and pricing version breakdown',
    category: 'system',
    screens: ['overview'],
    defaultSize: { w: 4, h: 1 },
    minW: 2,
    minH: 1,
    render: mount('estimation-meta'),
  },
  {
    id: 'official-sync',
    title: 'Official pricing sync',
    description: 'Status of official pricing data synchronization',
    category: 'system',
    screens: ['overview'],
    defaultSize: { w: 4, h: 2 },
    minW: 2,
    minH: 1,
    render: mount('official-sync'),
    hideWhenEmpty: true,
  },
  {
    id: 'openai-reconciliation',
    title: 'OpenAI reconciliation',
    description: 'OpenAI organization usage reconciliation',
    category: 'system',
    screens: ['overview'],
    defaultSize: { w: 4, h: 2 },
    minW: 2,
    minH: 1,
    render: mount('openai-reconciliation'),
    hideWhenEmpty: true,
  },
  {
    id: 'subagent-reconciliation',
    title: 'Subagent reconciliation',
    description: 'agent_sessions vs turns(is_subagent=1) cost diff',
    category: 'system',
    screens: ['overview'],
    defaultSize: { w: 4, h: 2 },
    minW: 2,
    minH: 1,
    render: mount('subagent-reconciliation'),
    hideWhenEmpty: true,
  },
  {
    id: 'codex-plan-kpi-mount',
    title: 'Codex plan',
    description: 'Codex plan utilization KPI tile',
    category: 'codex',
    screens: ['overview'],
    defaultSize: { w: 1, h: 1 },
    minW: 1,
    minH: 1,
    render: mount('codex-plan-kpi-mount'),
  },
  {
    id: 'stats-row',
    title: 'Summary stats',
    description: 'Token counts, cost, cache efficiency, and active-day averages',
    category: 'kpi',
    screens: ['overview'],
    defaultSize: { w: 4, h: 1 },
    minW: 2,
    minH: 1,
    render: mount('stats-row'),
  },

  // ── Activity tab ──────────────────────────────────────────────────────────
  {
    id: 'codex-plan-history-mount',
    title: 'Codex plan history',
    description: '30-day stacked bar chart of Codex plan utilization',
    category: 'codex',
    screens: ['activity'],
    defaultSize: { w: 4, h: 3 },
    minW: 2,
    minH: 2,
    render: mount('codex-plan-history-mount'),
  },
  {
    id: 'daily-chart-card',
    title: 'Daily token usage',
    description: 'Daily or weekly token usage bar chart',
    category: 'chart',
    screens: ['activity'],
    defaultSize: { w: 2, h: 3 },
    minW: 1,
    minH: 2,
    render: (el) => {
      el.id = 'daily-chart-card';
      el.className = 'card bento-2 chart-card';
      // Preserve inner structure so the chart renders into #chart-daily.
      if (!el.querySelector('#daily-chart-title')) {
        // Static literal — no dynamic content, no XSS vector.
        el.innerHTML = '<h2 id="daily-chart-title">Daily Token Usage</h2><div class="chart-wrap tall"><div id="chart-daily"></div></div>';
      }
    },
  },
  {
    id: 'model-chart-card',
    title: 'Model distribution',
    description: 'Token usage donut chart broken down by model',
    category: 'chart',
    screens: ['activity'],
    defaultSize: { w: 1, h: 3 },
    minW: 1,
    minH: 2,
    render: (el) => {
      el.id = 'model-chart-card';
      el.className = 'card chart-card';
      if (!el.querySelector('#chart-model')) {
        // Static literal — no dynamic content, no XSS vector.
        el.innerHTML = '<h2>By Model</h2><div class="chart-wrap model-chart-wrap"><div id="chart-model"></div></div>';
      }
    },
  },
  {
    id: 'project-chart-card',
    title: 'Top projects',
    description: 'Horizontal bar chart of top projects by cost',
    category: 'chart',
    screens: ['activity'],
    defaultSize: { w: 1, h: 3 },
    minW: 1,
    minH: 2,
    render: (el) => {
      el.id = 'project-chart-card';
      el.className = 'card chart-card';
      if (!el.querySelector('#chart-project')) {
        // Static literal — no dynamic content, no XSS vector.
        el.innerHTML = '<h2>Top Projects</h2><div class="chart-wrap"><div id="chart-project"></div></div>';
      }
    },
  },
  {
    id: 'hourly-chart',
    title: 'Activity by hour',
    description: 'Token usage broken down by hour of day',
    category: 'chart',
    screens: ['activity'],
    defaultSize: { w: 2, h: 3 },
    minW: 1,
    minH: 2,
    render: (el) => {
      el.id = 'hourly-chart';
      el.className = 'card card-flat bento-2';
    },
  },
  {
    id: 'activity-heatmap',
    title: 'Activity heatmap',
    description: '7×24 heatmap of token usage or cost',
    category: 'heatmap',
    screens: ['activity'],
    defaultSize: { w: 4, h: 2 },
    minW: 2,
    minH: 2,
    render: (el) => {
      el.id = 'activity-heatmap';
      el.className = 'card card-flat bento-full table-card';
    },
  },

  // ── Breakdowns tab ────────────────────────────────────────────────────────
  {
    id: 'subagent-summary',
    title: 'Subagent summary',
    description: 'Breakdown of subagent vs orchestrator turns and costs',
    category: 'agent',
    screens: ['breakdowns'],
    defaultSize: { w: 4, h: 2 },
    minW: 2,
    minH: 1,
    render: (el) => {
      el.id = 'subagent-summary';
      el.className = 'card card-flat bento-full table-card';
    },
  },
  {
    id: 'agent-setup-banner',
    title: 'Agent setup banner',
    description: 'Setup guidance for agent telemetry',
    category: 'agent',
    screens: ['breakdowns'],
    defaultSize: { w: 4, h: 1 },
    minW: 2,
    minH: 1,
    render: mount('agent-setup-banner'),
    hideWhenEmpty: true,
  },
  {
    id: 'agent-kpis-row',
    title: 'Agent KPIs',
    description: 'Key metrics for agent telemetry (sessions, cost, tokens)',
    category: 'agent',
    screens: ['breakdowns'],
    defaultSize: { w: 4, h: 1 },
    minW: 2,
    minH: 1,
    render: (el) => {
      el.id = 'agent-kpis-row';
      el.style.display = 'none';
      el.style.gridTemplateColumns = 'repeat(3,1fr)';
      el.style.gap = '16px';
    },
  },
  {
    id: 'agent-timeline',
    title: 'Agent timeline',
    description: 'Cost timeline broken down by agent role',
    category: 'agent',
    screens: ['breakdowns'],
    defaultSize: { w: 4, h: 3 },
    minW: 2,
    minH: 2,
    render: (el) => {
      el.id = 'agent-timeline';
      el.className = 'card card-flat bento-full table-card';
    },
  },
  {
    id: 'agent-distribution',
    title: 'Agent distribution',
    description: 'Breakdown of sessions and cost by agent role',
    category: 'agent',
    screens: ['breakdowns'],
    defaultSize: { w: 4, h: 3 },
    minW: 2,
    minH: 2,
    render: (el) => {
      el.id = 'agent-distribution';
      el.className = 'card card-flat bento-full table-card';
    },
  },
  {
    id: 'agent-top-sessions',
    title: 'Top agent sessions',
    description: 'Highest-cost agent sessions',
    category: 'agent',
    screens: ['breakdowns'],
    defaultSize: { w: 4, h: 3 },
    minW: 2,
    minH: 2,
    render: (el) => {
      el.id = 'agent-top-sessions';
      el.className = 'card card-flat bento-full table-card';
    },
  },
  {
    id: 'agent-spawn-batches',
    title: 'Agent spawn batches',
    description: 'Batches of agent spawns grouped by session',
    category: 'agent',
    screens: ['breakdowns'],
    defaultSize: { w: 4, h: 3 },
    minW: 2,
    minH: 2,
    render: (el) => {
      el.id = 'agent-spawn-batches';
      el.className = 'card card-flat bento-full table-card';
    },
  },
  {
    id: 'agent-tool-spectrum',
    title: 'Agent tool spectrum',
    description: 'Tool usage breakdown across agent roles',
    category: 'agent',
    screens: ['breakdowns'],
    defaultSize: { w: 4, h: 3 },
    minW: 2,
    minH: 2,
    render: (el) => {
      el.id = 'agent-tool-spectrum';
      el.className = 'card card-flat bento-full table-card';
    },
  },
  {
    id: 'entrypoint-breakdown',
    title: 'Entrypoint breakdown',
    description: 'Usage broken down by CLI entrypoint',
    category: 'table',
    screens: ['breakdowns'],
    defaultSize: { w: 4, h: 3 },
    minW: 2,
    minH: 2,
    render: (el) => {
      el.id = 'entrypoint-breakdown';
      el.className = 'card card-flat bento-full table-card';
    },
  },
  {
    id: 'service-tiers',
    title: 'Service tiers',
    description: 'Usage and cost split by service tier',
    category: 'table',
    screens: ['breakdowns'],
    defaultSize: { w: 4, h: 3 },
    minW: 2,
    minH: 2,
    render: (el) => {
      el.id = 'service-tiers';
      el.className = 'card card-flat bento-full table-card';
    },
    hideWhenEmpty: true,
  },
  {
    id: 'tool-summary',
    title: 'Tool usage',
    description: 'Tool invocation counts with cost attribution',
    category: 'table',
    screens: ['breakdowns'],
    defaultSize: { w: 4, h: 3 },
    minW: 2,
    minH: 2,
    render: (el) => {
      el.id = 'tool-summary';
      el.className = 'card card-flat bento-full table-card';
    },
  },
  {
    id: 'mcp-summary',
    title: 'MCP server usage',
    description: 'MCP server invocation counts with cost attribution',
    category: 'table',
    screens: ['breakdowns'],
    defaultSize: { w: 4, h: 3 },
    minW: 2,
    minH: 2,
    render: (el) => {
      el.id = 'mcp-summary';
      el.className = 'card card-flat bento-full table-card';
    },
  },
  {
    id: 'branch-summary',
    title: 'Git branch summary',
    description: 'Usage broken down by git branch',
    category: 'table',
    screens: ['breakdowns'],
    defaultSize: { w: 4, h: 3 },
    minW: 2,
    minH: 2,
    render: (el) => {
      el.id = 'branch-summary';
      el.className = 'card card-flat bento-full table-card';
    },
    hideWhenEmpty: true,
  },
  {
    id: 'version-summary',
    title: 'CLI versions',
    description: 'Usage breakdown by Claude CLI version with donut chart',
    category: 'table',
    screens: ['breakdowns'],
    defaultSize: { w: 2, h: 3 },
    minW: 1,
    minH: 2,
    render: (el) => {
      el.id = 'version-summary';
      el.className = 'card card-flat bento-2';
    },
    hideWhenEmpty: true,
  },
  {
    id: 'cost-reconciliation',
    title: 'Cost reconciliation',
    description: 'Hook-measured vs estimated cost comparison',
    category: 'system',
    screens: ['breakdowns'],
    defaultSize: { w: 4, h: 2 },
    minW: 2,
    minH: 1,
    render: mount('cost-reconciliation'),
  },

  // ── Tables tab ────────────────────────────────────────────────────────────
  {
    id: 'model-cost-mount',
    title: 'Cost by model',
    description: 'Per-model cost table with cache breakdown columns',
    category: 'table',
    screens: ['tables'],
    defaultSize: { w: 4, h: 4 },
    minW: 2,
    minH: 2,
    render: mount('model-cost-mount'),
  },
  {
    id: 'sessions-mount',
    title: 'Sessions',
    description: 'All sessions with sorting, pagination, and CSV export',
    category: 'table',
    screens: ['tables'],
    defaultSize: { w: 4, h: 5 },
    minW: 2,
    minH: 3,
    render: mount('sessions-mount'),
  },
  {
    id: 'project-cost-mount',
    title: 'Cost by project',
    description: 'Per-project cost table with CSV export',
    category: 'table',
    screens: ['tables'],
    defaultSize: { w: 4, h: 4 },
    minW: 2,
    minH: 2,
    render: mount('project-cost-mount'),
  },

  // ── Today tab ─────────────────────────────────────────────────────────────
  {
    id: 'today-date-picker-mount',
    title: 'Date picker',
    description: 'Select a specific date to view',
    category: 'today',
    screens: ['activity'],
    defaultSize: { w: 4, h: 1 },
    minW: 2,
    minH: 1,
    render: (el: HTMLElement) => {
      el.id = 'today-date-picker-mount';
      // Notify runtime so it can render today widgets when the grid is ready.
      // This handles the race where /api/today resolves before the async grid
      // layout fetch completes and the widget body elements are created.
      invokeMountCallback('today-date-picker-mount', el);
    },
  },
  {
    id: 'today-kpis-mount',
    title: 'Today KPIs',
    description: 'Key metrics for the selected day',
    category: 'today',
    screens: ['activity'],
    defaultSize: { w: 4, h: 1 },
    minW: 2,
    minH: 1,
    render: (el) => {
      el.id = 'today-kpis-mount';
      el.style.gridTemplateColumns = 'repeat(auto-fit,minmax(180px,1fr))';
      el.style.gap = '16px';
    },
  },
  {
    id: 'today-hour-timeline-mount',
    title: 'Hour timeline',
    description: 'Token usage timeline for each hour of the selected day',
    category: 'today',
    screens: ['activity'],
    defaultSize: { w: 4, h: 3 },
    minW: 2,
    minH: 2,
    render: (el) => {
      el.id = 'today-hour-timeline-mount';
      el.className = 'card card-flat bento-full';
    },
  },
  {
    id: 'today-hour-heatstrip-mount',
    title: 'Hour heatstrip',
    description: 'Single-row heat strip showing hourly intensity',
    category: 'today',
    screens: ['activity'],
    defaultSize: { w: 4, h: 2 },
    minW: 2,
    minH: 1,
    render: (el) => {
      el.id = 'today-hour-heatstrip-mount';
      el.className = 'card card-flat bento-full';
    },
  },
  {
    id: 'today-days-hours-30-mount',
    title: '30-day heat grid',
    description: '30 days × 24 hours usage grid',
    category: 'today',
    screens: ['activity'],
    defaultSize: { w: 4, h: 4 },
    minW: 2,
    minH: 2,
    render: (el) => {
      el.id = 'today-days-hours-30-mount';
      el.className = 'card card-flat bento-full';
    },
  },
  {
    id: 'today-days-hours-7-mount',
    title: '7-day heat grid',
    description: '7 days × 24 hours usage grid',
    category: 'today',
    screens: ['activity'],
    defaultSize: { w: 4, h: 3 },
    minW: 2,
    minH: 2,
    render: (el) => {
      el.id = 'today-days-hours-7-mount';
      el.className = 'card card-flat bento-full';
    },
  },
  {
    id: 'today-weekday-hour-mount',
    title: 'Weekday × hour pattern',
    description: '7×24 behavioral heatmap over a 90-day window',
    category: 'today',
    screens: ['activity'],
    defaultSize: { w: 4, h: 3 },
    minW: 2,
    minH: 2,
    render: (el) => {
      el.id = 'today-weekday-hour-mount';
      el.className = 'card card-flat bento-full';
    },
  },

  // ── Tables tab (skills) ───────────────────────────────────────────────────
  {
    id: 'skills',
    title: 'Skills inventory',
    description: 'Disk and context-budget impact of installed skills',
    category: 'system',
    screens: ['tables'],
    defaultSize: { w: 4, h: 4 },
    minW: 2,
    minH: 2,
    render: (el: HTMLElement) => {
      el.id = 'skills';
      invokeMountCallback('skills', el);
    },
  },
  {
    id: 'instruction-files',
    title: 'Instruction files',
    description: 'CLAUDE.md / AGENTS.md disk + token impact',
    category: 'system',
    screens: ['tables'],
    defaultSize: { w: 4, h: 4 },
    minW: 2,
    minH: 2,
    render: (el: HTMLElement) => {
      el.id = 'instruction-files';
      invokeMountCallback('instruction-files', el);
    },
  },
  {
    id: 'mcp-servers',
    title: 'MCP servers',
    description: 'Configured MCP servers, transports, runtime state, and usage',
    category: 'system',
    screens: ['tables'],
    defaultSize: { w: 4, h: 4 },
    minW: 2,
    minH: 2,
    render: (el: HTMLElement) => {
      el.id = 'mcp-servers';
      invokeMountCallback('mcp-servers', el);
    },
  },
  {
    id: 'context-pressure',
    title: 'Context pressure',
    description: 'Sessions by context-window utilisation — healthy, warm, tight, compacted',
    category: 'system',
    screens: ['tables'],
    defaultSize: { w: 4, h: 4 },
    minW: 2,
    minH: 2,
    render: (el: HTMLElement) => {
      el.id = 'context-pressure';
      invokeMountCallback('context-pressure', el);
    },
  },
  {
    id: 'agent-tree',
    title: 'Subagent cost attribution',
    description: 'Per-session cost breakdown across root agent and subagents',
    category: 'agent',
    screens: ['tables'],
    defaultSize: { w: 4, h: 4 },
    minW: 2,
    minH: 2,
    render: (el: HTMLElement) => {
      el.id = 'agent-tree';
      invokeMountCallback('agent-tree', el);
    },
  },
  {
    id: 'cost-forecast',
    title: 'Cost forecast',
    description: 'Rolling 7/30-day burn rate and projected monthly spend via linear regression',
    category: 'kpi',
    screens: ['tables'],
    defaultSize: { w: 4, h: 3 },
    minW: 2,
    minH: 2,
    render: (el: HTMLElement) => {
      el.id = 'cost-forecast';
      invokeMountCallback('cost-forecast', el);
    },
  },
  {
    id: 'session-quality-card',
    title: 'Session quality distribution',
    description: 'Turn depth histogram and category × depth heatmap for session quality analysis',
    category: 'table',
    screens: ['tables'],
    defaultSize: { w: 4, h: 5 },
    minW: 2,
    minH: 3,
    render: (el: HTMLElement) => {
      el.id = 'session-quality-card';
      invokeMountCallback('session-quality-card', el);
    },
  },
  {
    id: 'hook-telemetry-card',
    title: 'Hook telemetry',
    description: 'PreToolUse hook latency histogram, outcome breakdown, and top bypass ancestors',
    category: 'system',
    screens: ['tables'],
    defaultSize: { w: 4, h: 5 },
    minW: 2,
    minH: 3,
    render: (el: HTMLElement) => {
      el.id = 'hook-telemetry-card';
      invokeMountCallback('hook-telemetry-card', el);
    },
  },
  {
    id: 'claude-md-size-card',
    title: 'CLAUDE.md size over time',
    description: 'Git history of token count per CLAUDE.md file vs. per-session cost correlation',
    category: 'chart',
    screens: ['tables'],
    defaultSize: { w: 6, h: 6 },
    minW: 2,
    minH: 3,
    render: (el: HTMLElement) => {
      el.id = 'claude-md-size-card';
      invokeMountCallback('claude-md-size-card', el);
    },
  },

  // ── Projects tab ──────────────────────────────────────────────────────────
  {
    id: 'projects-registry',
    title: 'Projects',
    description: 'Searchable project registry with pinning, custom labels, and deep links',
    category: 'table',
    screens: ['projects'],
    defaultSize: { w: 4, h: 12 },
    minW: 2,
    minH: 4,
    render: (el: HTMLElement) => {
      el.id = 'projects-registry';
      invokeMountCallback('projects-registry', el);
    },
  },
];

export function widgetById(id: string): WidgetDef | undefined {
  return WIDGET_CATALOG.find(w => w.id === id);
}

export function widgetsForScreen(screen: DashboardScreen): WidgetDef[] {
  return WIDGET_CATALOG.filter(w => w.screens.includes(screen));
}
