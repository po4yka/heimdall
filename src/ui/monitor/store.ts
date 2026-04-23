import { signal } from '@preact/signals';
import type { LiveMonitorFocus, LiveMonitorResponse } from '../state/types';

export type LiveMonitorDensity = 'expanded' | 'compact';
export type LiveMonitorPanelId =
  | 'active_block'
  | 'depletion_forecast'
  | 'quota_suggestions'
  | 'context_window'
  | 'recent_session'
  | 'warnings';

export interface LiveMonitorPreferences {
  focus: LiveMonitorFocus;
  density: LiveMonitorDensity;
  hiddenPanels: LiveMonitorPanelId[];
}

export const LIVE_MONITOR_PREFERENCE_KEY = 'heimdall.live_monitor.preferences.v1';
export const LIVE_MONITOR_PANEL_OPTIONS: Array<{ id: LiveMonitorPanelId; label: string }> = [
  { id: 'active_block', label: 'Active Block' },
  { id: 'depletion_forecast', label: 'Depletion Forecast' },
  { id: 'quota_suggestions', label: 'Suggested Quotas' },
  { id: 'context_window', label: 'Context Window' },
  { id: 'recent_session', label: 'Recent Session' },
  { id: 'warnings', label: 'Warnings' },
];

const LIVE_MONITOR_FOCUS_OPTIONS: LiveMonitorFocus[] = ['all', 'claude', 'codex'];
const LIVE_MONITOR_DENSITY_OPTIONS: LiveMonitorDensity[] = ['expanded', 'compact'];
const LIVE_MONITOR_PANEL_IDS = LIVE_MONITOR_PANEL_OPTIONS.map(option => option.id);

export const liveMonitorData = signal<LiveMonitorResponse | null>(null);
export const liveMonitorFocus = signal<LiveMonitorFocus>('all');
export const liveMonitorDensity = signal<LiveMonitorDensity>('expanded');
export const liveMonitorHiddenPanels = signal<LiveMonitorPanelId[]>([]);
export const liveMonitorRefreshing = signal<boolean>(false);
export const liveMonitorError = signal<string | null>(null);
const liveMonitorPreferencesHydrated = signal<boolean>(false);

export function setLiveMonitorData(data: LiveMonitorResponse): void {
  liveMonitorData.value = data;
  if (liveMonitorPreferencesHydrated.value) {
    const previousFocus = liveMonitorFocus.value;
    const resolvedFocus = normalizeFocusForProviders(previousFocus, data);
    liveMonitorFocus.value = resolvedFocus;
    if (resolvedFocus !== previousFocus) {
      persistLiveMonitorPreferences();
    }
  } else {
    liveMonitorFocus.value = normalizeFocusForProviders(data.default_focus, data);
  }
  liveMonitorError.value = null;
}

export function hydrateLiveMonitorPreferences(): void {
  if (liveMonitorPreferencesHydrated.value) {
    return;
  }

  const saved = readLiveMonitorPreferences();
  if (saved) {
    liveMonitorFocus.value = saved.focus;
    liveMonitorDensity.value = saved.density;
    liveMonitorHiddenPanels.value = saved.hiddenPanels;
  } else {
    liveMonitorFocus.value = 'all';
    liveMonitorDensity.value = 'expanded';
    liveMonitorHiddenPanels.value = [];
  }
  liveMonitorPreferencesHydrated.value = saved != null;
}

export function setLiveMonitorFocus(focus: LiveMonitorFocus): void {
  liveMonitorFocus.value = focus;
  liveMonitorPreferencesHydrated.value = true;
  persistLiveMonitorPreferences();
}

export function setLiveMonitorDensity(density: LiveMonitorDensity): void {
  liveMonitorDensity.value = density;
  liveMonitorPreferencesHydrated.value = true;
  persistLiveMonitorPreferences();
}

export function toggleLiveMonitorPanel(panelId: LiveMonitorPanelId): void {
  const hiddenPanels = new Set(liveMonitorHiddenPanels.value);
  if (hiddenPanels.has(panelId)) {
    hiddenPanels.delete(panelId);
  } else {
    hiddenPanels.add(panelId);
  }
  liveMonitorHiddenPanels.value = [...hiddenPanels].sort();
  liveMonitorPreferencesHydrated.value = true;
  persistLiveMonitorPreferences();
}

export function isLiveMonitorPanelHidden(panelId: LiveMonitorPanelId): boolean {
  return liveMonitorHiddenPanels.value.includes(panelId);
}

function normalizeFocusForProviders(
  focus: LiveMonitorFocus,
  data: LiveMonitorResponse
): LiveMonitorFocus {
  if (focus === 'all') {
    return 'all';
  }
  return data.providers.some(provider => provider.provider === focus) ? focus : 'all';
}

function normalizeLiveMonitorPreferences(value: unknown): LiveMonitorPreferences | null {
  if (!value || typeof value !== 'object') {
    return null;
  }

  const candidate = value as Partial<LiveMonitorPreferences>;
  const focus = LIVE_MONITOR_FOCUS_OPTIONS.includes(candidate.focus as LiveMonitorFocus)
    ? candidate.focus as LiveMonitorFocus
    : 'all';
  const density = LIVE_MONITOR_DENSITY_OPTIONS.includes(candidate.density as LiveMonitorDensity)
    ? candidate.density as LiveMonitorDensity
    : 'expanded';
  const hiddenPanels = Array.isArray(candidate.hiddenPanels)
    ? Array.from(new Set(candidate.hiddenPanels.filter((panel): panel is LiveMonitorPanelId =>
        LIVE_MONITOR_PANEL_IDS.includes(panel as LiveMonitorPanelId)
      ))).sort()
    : [];

  return { focus, density, hiddenPanels };
}

function readLiveMonitorPreferences(): LiveMonitorPreferences | null {
  try {
    const raw = localStorage.getItem(LIVE_MONITOR_PREFERENCE_KEY);
    if (!raw) {
      return null;
    }
    return normalizeLiveMonitorPreferences(JSON.parse(raw));
  } catch {
    return null;
  }
}

function persistLiveMonitorPreferences(): void {
  try {
    localStorage.setItem(
      LIVE_MONITOR_PREFERENCE_KEY,
      JSON.stringify({
        focus: liveMonitorFocus.value,
        density: liveMonitorDensity.value,
        hiddenPanels: liveMonitorHiddenPanels.value,
      } satisfies LiveMonitorPreferences)
    );
  } catch {
    // Ignore local persistence failures and keep the in-memory monitor state usable.
  }
}
