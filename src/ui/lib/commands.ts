/**
 * Command registry for the Cmd-K palette.
 *
 * Each Command is a single user-invokable entry: navigation (jump to a
 * tab, scroll to a widget), search (sessions / projects / models /
 * branches — these are computed at open time from the live data), or
 * an action (rescan, toggle theme, open settings, save view, etc.).
 *
 * The palette filters by `searchTerms` — a flat list of keywords plus
 * the visible label. Sentence-case throughout.
 */
import {
  activeDashboardTab,
  themeMode,
  editMode,
  backupModalOpen,
  settingsModalOpen,
  rawData,
  selectedModels,
  type DashboardTab,
} from '../state/store';
import { widgetsForScreen, type DashboardScreen, type WidgetDef } from '../widgets/registry';

export type CommandGroup =
  | 'navigate'
  | 'widget'
  | 'session'
  | 'project'
  | 'model'
  | 'action';

export interface Command {
  id: string;
  group: CommandGroup;
  label: string;
  hint?: string | undefined;
  /** Lowercased keyword set used for filtering. Includes the label. */
  searchTerms: string;
  run: () => void;
}

const TAB_LABELS: Record<DashboardTab, string> = {
  overview: 'Overview',
  activity: 'Activity',
  breakdowns: 'Breakdowns',
  tables: 'Sessions',
  projects: 'Projects',
};

function navigateToTab(tab: DashboardTab): void {
  activeDashboardTab.value = tab;
  // The dashboard runtime listens for activeDashboardTab changes via
  // a signal subscriber wired up in app.tsx; switching the signal alone
  // is enough to swap the visible widget grid.
}

function scrollToWidget(widgetId: string): void {
  const el = document.querySelector<HTMLElement>(`.grid-stack-item[gs-id="${widgetId}"]`);
  if (!el) return;
  el.scrollIntoView({ behavior: 'smooth', block: 'center' });
  el.classList.add('widget-flash');
  window.setTimeout(() => el.classList.remove('widget-flash'), 1200);
}

function widgetCommands(): Command[] {
  const screen: DashboardScreen = activeDashboardTab.value;
  const defs = widgetsForScreen(screen);
  return defs.map((def: WidgetDef) => ({
    id: `widget:${def.id}`,
    group: 'widget',
    label: def.title,
    hint: def.description,
    searchTerms: `${def.title} ${def.description ?? ''} ${def.id}`.toLowerCase(),
    run: () => scrollToWidget(def.id),
  }));
}

function sessionCommands(): Command[] {
  const sessions = rawData.value?.sessions_all ?? [];
  return sessions.slice(0, 50).map(s => {
    const label = s.title || s.display_name || s.session_id;
    const subtitle = `${s.model} · ${s.project || '—'}`;
    return {
      id: `session:${s.session_id}`,
      group: 'session',
      label,
      hint: subtitle,
      searchTerms: `${label} ${subtitle} ${s.session_id}`.toLowerCase(),
      run: () => {
        navigateToTab('tables');
        window.setTimeout(() => {
          const row = document.querySelector<HTMLElement>(
            `tr[data-session-id="${s.session_id}"]`
          );
          if (row) {
            row.scrollIntoView({ behavior: 'smooth', block: 'center' });
            row.classList.add('widget-flash');
            window.setTimeout(() => row.classList.remove('widget-flash'), 1200);
          }
        }, 80);
      },
    };
  });
}

function projectCommands(): Command[] {
  // Derive a unique project list from session rows since the dashboard
  // payload doesn't expose a dedicated project_breakdown collection.
  const sessions = rawData.value?.sessions_all ?? [];
  const seen = new Map<string, { project: string; display: string; sessions: number; cost: number }>();
  for (const s of sessions) {
    const existing = seen.get(s.project);
    if (existing) {
      existing.sessions += 1;
      existing.cost += s.cost;
    } else {
      seen.set(s.project, {
        project: s.project,
        display: s.custom_label || s.display_name || s.project,
        sessions: 1,
        cost: s.cost,
      });
    }
  }
  return [...seen.values()]
    .sort((a, b) => b.cost - a.cost)
    .slice(0, 50)
    .map(p => ({
      id: `project:${p.project}`,
      group: 'project',
      label: p.display,
      hint: `${p.sessions.toLocaleString()} sessions · $${p.cost.toFixed(2)}`,
      searchTerms: `${p.project} ${p.display}`.toLowerCase(),
      run: () => {
        navigateToTab('projects');
      },
    }));
}

function modelCommands(): Command[] {
  const models = rawData.value?.all_models ?? [];
  return models.map(m => ({
    id: `model:${m}`,
    group: 'model',
    label: m,
    hint: 'Toggle model filter',
    searchTerms: m.toLowerCase(),
    run: () => {
      const next = new Set(selectedModels.value);
      if (next.has(m)) next.delete(m);
      else next.add(m);
      selectedModels.value = next;
    },
  }));
}

interface ActionContext {
  /** Triggers a rescan via the same path the header button uses. */
  triggerRescan: () => void | Promise<void>;
  /** Switches dark <-> light. */
  toggleTheme: () => void;
}

function actionCommands(ctx: ActionContext): Command[] {
  return [
    {
      id: 'action:rescan',
      group: 'action',
      label: 'Rescan',
      hint: 'Re-scan transcripts and refresh dashboard',
      searchTerms: 'rescan refresh sync reload',
      run: () => { void ctx.triggerRescan(); },
    },
    {
      id: 'action:settings',
      group: 'action',
      label: 'Open settings',
      searchTerms: 'settings preferences config',
      run: () => { settingsModalOpen.value = true; },
    },
    {
      id: 'action:backup',
      group: 'action',
      label: 'Open backup and snapshots',
      searchTerms: 'backup snapshot export',
      run: () => { backupModalOpen.value = true; },
    },
    {
      id: 'action:edit-layout',
      group: 'action',
      label: editMode.value ? 'Exit edit layout' : 'Edit layout',
      searchTerms: 'edit layout customize widgets rearrange',
      run: () => { editMode.value = !editMode.value; },
    },
    {
      id: 'action:theme',
      group: 'action',
      label: themeMode.value === 'dark' ? 'Switch to light theme' : 'Switch to dark theme',
      searchTerms: 'theme dark light mode',
      run: () => ctx.toggleTheme(),
    },
    {
      id: 'action:open-monitor',
      group: 'action',
      label: 'Open live monitor',
      hint: 'Real-time provider lanes',
      searchTerms: 'live monitor real-time provider',
      run: () => { window.location.href = '/monitor'; },
    },
  ];
}

function navigationCommands(): Command[] {
  return (Object.keys(TAB_LABELS) as DashboardTab[]).map(tab => ({
    id: `nav:${tab}`,
    group: 'navigate',
    label: `Go to ${TAB_LABELS[tab]}`,
    searchTerms: `${TAB_LABELS[tab]} tab navigate go to`.toLowerCase(),
    run: () => navigateToTab(tab),
  }));
}

export function buildCommands(ctx: ActionContext): Command[] {
  return [
    ...navigationCommands(),
    ...actionCommands(ctx),
    ...widgetCommands(),
    ...sessionCommands(),
    ...projectCommands(),
    ...modelCommands(),
  ];
}

/** Returns commands whose searchTerms (or label) match the query. */
export function filterCommands(commands: Command[], query: string): Command[] {
  const q = query.trim().toLowerCase();
  if (!q) return commands;
  const tokens = q.split(/\s+/).filter(Boolean);
  return commands.filter(c => {
    const hay = `${c.label} ${c.searchTerms}`.toLowerCase();
    return tokens.every(t => hay.includes(t));
  });
}
