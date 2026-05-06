import { signal } from '@preact/signals';

const STORAGE_KEY = 'heimdall.widget-empty-overrides';

function readStorage(): Set<string> {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return new Set();
    const parsed = JSON.parse(raw) as unknown;
    if (!Array.isArray(parsed)) return new Set();
    return new Set(parsed.filter((x): x is string => typeof x === 'string'));
  } catch {
    return new Set();
  }
}

function writeStorage(set: Set<string>): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify([...set]));
  } catch {
    /* ignore quota / disabled storage */
  }
}

/**
 * Per-widget "show anyway" overrides for `hideWhenEmpty: true` widgets.
 * When a widget id is in this set, `setSectionVisibility(false)` keeps
 * the GridStack item visible so the user still sees the empty card and
 * its inline-status / "no data yet" placeholder.
 *
 * Backed by localStorage so the override persists across reloads. The
 * signal lets the picker react to overrides being added/removed.
 */
export const widgetEmptyOverrides = signal<Set<string>>(readStorage());

export function isOverridden(widgetId: string): boolean {
  return widgetEmptyOverrides.value.has(widgetId);
}

export function setOverride(widgetId: string, on: boolean): void {
  const next = new Set(widgetEmptyOverrides.value);
  if (on) next.add(widgetId);
  else next.delete(widgetId);
  widgetEmptyOverrides.value = next;
  writeStorage(next);
  // Reapply visibility immediately so the widget appears/disappears on
  // click rather than waiting for the next data tick. The grid item is
  // hidden via inline `display: none`; flipping it back to '' restores
  // the GridStack-managed display.
  const itemEl = document.querySelector<HTMLElement>(
    `.grid-stack-item[gs-id="${widgetId}"]`
  );
  if (!itemEl) return;
  // If turning on: force visible. If turning off: respect whatever the
  // most recent setSectionVisibility decided — we approximate by reading
  // the host container's hasContent dataset.
  if (on) {
    itemEl.style.display = '';
  } else {
    const inner = itemEl.querySelector<HTMLElement>(
      `[id="${widgetId}"], .widget-body > [id]`
    );
    const hasContent = inner?.dataset['hasContent'] === '1';
    itemEl.style.display = hasContent ? '' : 'none';
  }
}
