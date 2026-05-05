/**
 * Default widget layouts per dashboard screen.
 *
 * Positions mirror the existing static order in index.html / SECTION_TAB_MAP.
 * The grid is 4 columns wide; row heights use GridStack cellHeight=132px.
 */
import type { DashboardScreen, PlacedWidget, ScreenLayout } from './registry';

interface StackEntry {
  id: string;
  h: number;
  w?: number;
  x?: number;
  minW?: number;
  minH?: number;
}

// Helper: lay out widgets sequentially from top to bottom.
function stack(defs: StackEntry[]): PlacedWidget[] {
  let y = 0;
  const result: PlacedWidget[] = [];
  for (const d of defs) {
    const w = d.w ?? 4;
    const x = d.x ?? 0;
    const p: PlacedWidget = { i: d.id, x, y, w, h: d.h };
    if (d.minW !== undefined) p.minW = d.minW;
    if (d.minH !== undefined) p.minH = d.minH;
    result.push(p);
    y += d.h;
  }
  return result;
}

const OVERVIEW_WIDGETS = stack([
  { id: 'usage-windows',         h: 2 },
  { id: 'subscription-quota',    h: 3 },
  { id: 'claude-usage',          h: 2 },
  { id: 'agent-status',          h: 2 },
  { id: 'estimation-meta',       h: 1 },
  { id: 'official-sync',         h: 2 },
  { id: 'openai-reconciliation', h: 2 },
  { id: 'subagent-reconciliation', h: 2 },
  { id: 'codex-plan-kpi-mount',  h: 1 },
  { id: 'stats-row',             h: 1 },
]);

// Activity: today-drilldown widgets (date picker + KPIs + hour-bucket
// heatmaps) live at the top, followed by the broader range charts.
// Today was a separate tab pre-2026-05-05; it folded into Activity so the
// strip could shrink from 7 → 5 tabs.
function makeActivityWidgets(): PlacedWidget[] {
  const widgets: PlacedWidget[] = [
    // Today block — drilldown for the selected date.
    { i: 'today-date-picker-mount',    x: 0, y: 0,  w: 4, h: 1 },
    { i: 'today-kpis-mount',           x: 0, y: 1,  w: 4, h: 1 },
    { i: 'today-hour-timeline-mount',  x: 0, y: 2,  w: 4, h: 3 },
    { i: 'today-hour-heatstrip-mount', x: 0, y: 5,  w: 4, h: 2 },
    { i: 'today-days-hours-30-mount',  x: 0, y: 7,  w: 4, h: 4 },
    { i: 'today-days-hours-7-mount',   x: 0, y: 11, w: 4, h: 3 },
    { i: 'today-weekday-hour-mount',   x: 0, y: 14, w: 4, h: 3 },
    // Range block — applies the dashboard filter strip.
    // Codex plan history — full width
    { i: 'codex-plan-history-mount',   x: 0, y: 17, w: 4, h: 3 },
    // Charts row: daily (2 wide) | model (1) | project (1)
    { i: 'daily-chart-card',           x: 0, y: 20, w: 2, h: 3, minW: 1, minH: 2 },
    { i: 'model-chart-card',           x: 2, y: 20, w: 1, h: 3, minW: 1, minH: 2 },
    { i: 'project-chart-card',         x: 3, y: 20, w: 1, h: 3, minW: 1, minH: 2 },
    // Hourly chart (2 wide) then activity heatmap full width
    { i: 'hourly-chart',               x: 0, y: 23, w: 2, h: 3, minW: 1, minH: 2 },
    { i: 'activity-heatmap',           x: 0, y: 26, w: 4, h: 2, minW: 2, minH: 2 },
  ];
  return widgets;
}

const BREAKDOWNS_WIDGETS = stack([
  { id: 'subagent-summary',      h: 2 },
  { id: 'agent-setup-banner',    h: 1 },
  { id: 'agent-kpis-row',        h: 1 },
  { id: 'agent-timeline',        h: 3 },
  { id: 'agent-distribution',    h: 3 },
  { id: 'agent-top-sessions',    h: 3 },
  { id: 'agent-spawn-batches',   h: 3 },
  { id: 'agent-tool-spectrum',   h: 3 },
  { id: 'entrypoint-breakdown',  h: 3 },
  { id: 'service-tiers',         h: 3 },
  { id: 'tool-summary',          h: 3 },
  { id: 'mcp-summary',           h: 3 },
  { id: 'branch-summary',        h: 3 },
  { id: 'version-summary',       h: 3, w: 2 },
  { id: 'cost-reconciliation',   h: 2 },
]);

const TABLES_WIDGETS = stack([
  { id: 'model-cost-mount',   h: 4 },
  { id: 'sessions-mount',     h: 5 },
  { id: 'project-cost-mount', h: 4 },
]);

const PROJECTS_WIDGETS = stack([
  { id: 'projects-registry', h: 12 },
]);

export const DEFAULT_LAYOUTS: Record<DashboardScreen, ScreenLayout> = {
  overview:   { widgets: OVERVIEW_WIDGETS,    hidden: [] },
  activity:   { widgets: makeActivityWidgets(), hidden: [] },
  breakdowns: { widgets: BREAKDOWNS_WIDGETS,  hidden: [] },
  tables:     { widgets: TABLES_WIDGETS,      hidden: [] },
  projects:   { widgets: PROJECTS_WIDGETS,    hidden: [] },
};
