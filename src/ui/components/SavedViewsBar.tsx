/**
 * SavedViewsBar — chip strip below the dashboard tabs.
 *
 * Lets the user switch between named layout presets (Default / Compact /
 * Triage) and any custom views they've saved. The active view chip shows
 * an `--accent-interactive` underline; saving the current layout offers
 * an inline rename input.
 */
import { useEffect, useState } from 'preact/hooks';
import { activeDashboardTab, tabToScreen } from '../state/store';
import { pendingLayoutApply } from '../widgets/apply-layout';
import {
  listViews,
  getActiveViewId,
  setActiveViewId,
  saveView,
  deleteView,
  savedViewsToken,
  activeViewToken,
  type SavedView,
} from '../lib/saved-views';

interface SavedViewsBarProps {
  /** Returns the current grid layout so the user can save it. */
  getCurrentLayout: () => SavedView['layout'] | null;
}

export function SavedViewsBar({ getCurrentLayout }: SavedViewsBarProps) {
  // Re-read storage on changes.
  void savedViewsToken.value;
  void activeViewToken.value;

  const screen = tabToScreen(activeDashboardTab.value);
  const views = listViews(screen);
  const activeId = getActiveViewId(screen);
  const [savingName, setSavingName] = useState<string | null>(null);

  useEffect(() => {
    if (savingName === null) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') setSavingName(null);
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [savingName]);

  const applyView = (view: SavedView) => {
    setActiveViewId(screen, view.id);
    pendingLayoutApply.value = { screen: view.screen, layout: view.layout };
  };

  const onSaveCurrent = () => {
    const layout = getCurrentLayout();
    if (!layout) return;
    setSavingName('My view');
  };

  const commitSave = (name: string) => {
    const layout = getCurrentLayout();
    if (!layout) {
      setSavingName(null);
      return;
    }
    const trimmed = name.trim();
    if (!trimmed) {
      setSavingName(null);
      return;
    }
    const view = saveView(screen, trimmed, layout);
    setActiveViewId(screen, view.id);
    setSavingName(null);
  };

  return (
    <nav class="saved-views-bar" aria-label="Saved views">
      <ul class="saved-views-bar__list">
        {views.map(view => {
          const isActive = view.id === activeId;
          return (
            <li key={view.id} class="saved-views-bar__item">
              <button
                type="button"
                class={`saved-views-bar__chip${isActive ? ' is-active' : ''}${
                  view.isPreset ? ' is-preset' : ''
                }`}
                aria-pressed={isActive}
                onClick={() => applyView(view)}
                title={view.isPreset ? 'Built-in preset' : 'Custom view — click × to delete'}
              >
                {view.name}
                {!view.isPreset && (
                  <span
                    class="saved-views-bar__delete"
                    role="button"
                    aria-label={`Delete ${view.name}`}
                    onClick={(e) => {
                      e.stopPropagation();
                      if (confirm(`Delete view "${view.name}"?`)) {
                        deleteView(screen, view.id);
                      }
                    }}
                  >
                    ×
                  </span>
                )}
              </button>
            </li>
          );
        })}
        <li class="saved-views-bar__item">
          {savingName === null ? (
            <button
              type="button"
              class="saved-views-bar__chip saved-views-bar__chip--ghost"
              onClick={onSaveCurrent}
              aria-label="Save current view"
            >
              + Save view
            </button>
          ) : (
            <input
              autoFocus
              class="saved-views-bar__input"
              type="text"
              value={savingName}
              placeholder="View name"
              onInput={(e) => setSavingName((e.currentTarget as HTMLInputElement).value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') commitSave(savingName ?? '');
                if (e.key === 'Escape') setSavingName(null);
              }}
              onBlur={() => commitSave(savingName ?? '')}
            />
          )}
        </li>
      </ul>
    </nav>
  );
}
