/**
 * Saved views — per-screen layout presets persisted to localStorage.
 *
 * A "view" bundles a `ScreenLayout` (placed widgets + hidden ids) under a
 * user-friendly name. Three presets ship per screen — Default, Compact,
 * and Triage — and the user can save their own. Switching a view applies
 * its layout via the existing `applyLayout` event, which WidgetGrid
 * listens for.
 */
import { signal } from '@preact/signals';
import type { DashboardScreen, PlacedWidget, ScreenLayout } from '../widgets/registry';
import { WIDGET_CATALOG, widgetsForScreen } from '../widgets/registry';
import { DEFAULT_LAYOUTS } from '../widgets/default-layouts';

export interface SavedView {
  id: string;
  name: string;
  screen: DashboardScreen;
  layout: ScreenLayout;
  isPreset: boolean;
}

const STORAGE_KEY_PREFIX = 'heimdall.saved-views.';
const ACTIVE_KEY_PREFIX = 'heimdall.active-view.';

// ── Presets ───────────────────────────────────────────────────────────

/** Triage subset per screen — alert-relevant widgets only. */
const TRIAGE_WIDGET_IDS: Record<DashboardScreen, string[]> = {
  overview: ['usage-windows', 'subscription-quota', 'agent-status', 'claude-usage', 'stats-row'],
  activity: ['today-date-picker-mount', 'today-kpis-mount', 'today-hour-heatstrip-mount'],
  breakdowns: ['subagent-summary', 'cost-reconciliation'],
  tables: ['sessions-mount'],
  projects: ['projects-registry'],
};

function compactLayout(screen: DashboardScreen): ScreenLayout {
  const widgets = widgetsForScreen(screen);
  let y = 0;
  const placed: PlacedWidget[] = widgets.map(def => {
    const w = def.minW ?? def.defaultSize.w;
    const h = def.minH ?? def.defaultSize.h;
    const item: PlacedWidget = { i: def.id, x: 0, y, w, h };
    if (def.minW !== undefined) item.minW = def.minW;
    if (def.minH !== undefined) item.minH = def.minH;
    y += h;
    return item;
  });
  return { widgets: placed, hidden: [] };
}

function triageLayout(screen: DashboardScreen): ScreenLayout {
  const want = new Set(TRIAGE_WIDGET_IDS[screen]);
  const allDefs = WIDGET_CATALOG.filter(d => d.screens.includes(screen));
  const visible = allDefs.filter(d => want.has(d.id));
  const hidden = allDefs.filter(d => !want.has(d.id)).map(d => d.id);
  let y = 0;
  const widgets: PlacedWidget[] = visible.map(def => {
    const item: PlacedWidget = {
      i: def.id,
      x: 0,
      y,
      w: def.defaultSize.w,
      h: def.defaultSize.h,
    };
    if (def.minW !== undefined) item.minW = def.minW;
    if (def.minH !== undefined) item.minH = def.minH;
    y += def.defaultSize.h;
    return item;
  });
  return { widgets, hidden };
}

function presetsFor(screen: DashboardScreen): SavedView[] {
  return [
    {
      id: 'preset-default',
      name: 'Default',
      screen,
      layout: DEFAULT_LAYOUTS[screen],
      isPreset: true,
    },
    {
      id: 'preset-compact',
      name: 'Compact',
      screen,
      layout: compactLayout(screen),
      isPreset: true,
    },
    {
      id: 'preset-triage',
      name: 'Triage',
      screen,
      layout: triageLayout(screen),
      isPreset: true,
    },
  ];
}

// ── Storage ───────────────────────────────────────────────────────────

function readCustom(screen: DashboardScreen): SavedView[] {
  try {
    const raw = localStorage.getItem(`${STORAGE_KEY_PREFIX}${screen}`);
    if (!raw) return [];
    const parsed = JSON.parse(raw) as unknown;
    if (!Array.isArray(parsed)) return [];
    return parsed.filter((v): v is SavedView =>
      typeof v === 'object' &&
      v !== null &&
      typeof (v as SavedView).id === 'string' &&
      typeof (v as SavedView).name === 'string' &&
      typeof (v as SavedView).screen === 'string' &&
      typeof (v as SavedView).layout === 'object'
    );
  } catch {
    return [];
  }
}

function writeCustom(screen: DashboardScreen, views: SavedView[]): void {
  try {
    localStorage.setItem(`${STORAGE_KEY_PREFIX}${screen}`, JSON.stringify(views));
  } catch {
    /* ignore */
  }
}

// ── Public API ────────────────────────────────────────────────────────

/** Returns presets first, then user-saved views. */
export function listViews(screen: DashboardScreen): SavedView[] {
  return [...presetsFor(screen), ...readCustom(screen)];
}

export function saveView(
  screen: DashboardScreen,
  name: string,
  layout: ScreenLayout
): SavedView {
  const id = `view-${Date.now().toString(36)}`;
  const view: SavedView = { id, name, screen, layout, isPreset: false };
  const next = [...readCustom(screen), view];
  writeCustom(screen, next);
  savedViewsToken.value++;
  return view;
}

export function deleteView(screen: DashboardScreen, viewId: string): void {
  const next = readCustom(screen).filter(v => v.id !== viewId);
  writeCustom(screen, next);
  if (getActiveViewId(screen) === viewId) {
    setActiveViewId(screen, 'preset-default');
  }
  savedViewsToken.value++;
}

export function getActiveViewId(screen: DashboardScreen): string {
  try {
    return localStorage.getItem(`${ACTIVE_KEY_PREFIX}${screen}`) ?? 'preset-default';
  } catch {
    return 'preset-default';
  }
}

export function setActiveViewId(screen: DashboardScreen, viewId: string): void {
  try {
    localStorage.setItem(`${ACTIVE_KEY_PREFIX}${screen}`, viewId);
  } catch {
    /* ignore */
  }
  activeViewToken.value++;
}

/**
 * Bumped when custom views change. Subscribers re-read storage. Keeps the
 * library free of in-memory list state so writers can't desync.
 */
export const savedViewsToken = signal(0);

/** Bumped when the active view changes for any screen. */
export const activeViewToken = signal(0);
