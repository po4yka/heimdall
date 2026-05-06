/**
 * WidgetGrid — Feature 2: drag/resize/remove-able dashboard grid.
 *
 * Renders a GridStack instance for the given screen. Loads the saved layout
 * from /api/layouts/:screen (or DEFAULT_LAYOUTS when no row exists). Persists
 * changes with a 500ms debounce on every GridStack 'change' event.
 *
 * Edit mode is controlled by the `editMode` signal in store.ts.
 * Mobile (<720px) bypasses GridStack and renders a single-column read-only stack.
 */
import { useEffect, useRef, useCallback } from 'preact/hooks';
import { GridStack } from 'gridstack';
import { render } from 'preact';
import { editMode } from '../state/store';
import { fetchLayout, saveLayout, resetLayout } from '../lib/layouts';
import { setStatus } from '../lib/status';
import { widgetsForScreen, widgetById, WIDGET_CATALOG } from './registry';
import { DEFAULT_LAYOUTS } from './default-layouts';
import { AddWidgetPicker } from '../components/widgets/AddWidgetPicker';
import type { DashboardScreen, PlacedWidget, ScreenLayout } from './registry';
import { pendingLayoutApply, publishCurrentLayout } from './apply-layout';

const MOBILE_BREAKPOINT = 720;
const SAVE_DEBOUNCE_MS = 500;
const GRID_COLUMNS = 4;
const CELL_HEIGHT = 132;
const CELL_MARGIN = 12;

interface WidgetGridProps {
  screen: DashboardScreen;
}

/** Return the next open y position below all placed widgets. */
function nextY(widgets: PlacedWidget[]): number {
  if (!widgets.length) return 0;
  return Math.max(...widgets.map(w => w.y + w.h));
}

/**
 * Reconcile a saved layout against the current widget catalog for `screen`:
 * - Drop placed/hidden widgets whose id is no longer in the catalog.
 * - Clamp placed widget sizes up to the catalog's current minW / minH.
 *   This is how widget-size bumps (e.g. subscription-quota h:3 → h:10)
 *   reach users who already have a saved layout — without forcing them
 *   to manually `[EDIT LAYOUT]` and reset.
 * - Append catalog widgets that are neither placed nor hidden at the bottom.
 */
export function reconcileLayout(saved: ScreenLayout, screen: DashboardScreen): ScreenLayout {
  const catalog = widgetsForScreen(screen);
  const catalogIds = new Set(catalog.map(w => w.id));
  const catalogById = new Map(catalog.map(w => [w.id, w]));

  const widgets = saved.widgets
    .filter(w => catalogIds.has(w.i))
    .map(w => {
      const def = catalogById.get(w.i);
      if (!def) return w;
      const next: PlacedWidget = { ...w };
      if (def.minW !== undefined && next.w < def.minW) next.w = def.minW;
      if (def.minH !== undefined && next.h < def.minH) next.h = def.minH;
      return next;
    });
  const hidden = saved.hidden.filter(id => catalogIds.has(id));

  const placedIds = new Set(widgets.map(w => w.i));
  const hiddenSet = new Set(hidden);
  let y = nextY(widgets);

  for (const def of catalog) {
    if (placedIds.has(def.id) || hiddenSet.has(def.id)) continue;
    const placed: PlacedWidget = { i: def.id, x: 0, y, w: def.defaultSize.w, h: def.defaultSize.h };
    if (def.minW !== undefined) placed.minW = def.minW;
    if (def.minH !== undefined) placed.minH = def.minH;
    widgets.push(placed);
    y += def.defaultSize.h;
  }

  return { widgets, hidden };
}

/** Build a ScreenLayout from the current GridStack grid state + the hidden list. */
function layoutFromGrid(grid: GridStack, hidden: string[]): ScreenLayout {
  const widgets: PlacedWidget[] = grid.getGridItems().map(el => {
    const node = el.gridstackNode;
    const w: PlacedWidget = {
      i: node?.id ?? el.getAttribute('gs-id') ?? '',
      x: node?.x ?? 0,
      y: node?.y ?? 0,
      w: node?.w ?? 1,
      h: node?.h ?? 1,
    };
    if (node?.minW !== undefined) w.minW = node.minW;
    if (node?.minH !== undefined) w.minH = node.minH;
    return w;
  });
  return { widgets, hidden };
}

export function WidgetGrid({ screen }: WidgetGridProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const gridRef = useRef<GridStack | null>(null);
  const hiddenRef = useRef<string[]>([]);
  const saveTimerRef = useRef<number | null>(null);
  const pickerMountRef = useRef<HTMLDivElement | null>(null);
  const isMobileRef = useRef(window.innerWidth < MOBILE_BREAKPOINT);
  const resetBtnRef = useRef<HTMLButtonElement | null>(null);
  const addBtnRef = useRef<HTMLButtonElement | null>(null);

  // ── Debounced save ──────────────────────────────────────────────────
  const scheduleSave = useCallback(() => {
    if (saveTimerRef.current !== null) clearTimeout(saveTimerRef.current);
    saveTimerRef.current = window.setTimeout(async () => {
      if (!gridRef.current) return;
      const layout = layoutFromGrid(gridRef.current, hiddenRef.current);
      // Publish the latest layout so SavedViewsBar can capture it.
      publishCurrentLayout(screen, layout);
      try {
        await saveLayout(screen, layout);
        setStatus('layout-save', 'success', '[SAVED]', 2000);
      } catch {
        setStatus('layout-save', 'error', '[SAVE FAILED]', 4000);
      }
    }, SAVE_DEBOUNCE_MS);
  }, [screen]);

  // ── Picker rendering ───────────────────────────────────────────────
  const renderPicker = useCallback((open: boolean) => {
    if (!pickerMountRef.current) return;
    const screenWidgets = widgetsForScreen(screen);
    const hidden = hiddenRef.current;
    // Empty-hidden widgets: layout-present hideWhenEmpty widgets whose
    // grid item is currently `display: none` because the renderer
    // reported no content. Surface them in the picker so users can
    // discover them and toggle "Show anyway".
    const emptyHiddenWidgets = screenWidgets.filter(w => {
      if (!w.hideWhenEmpty) return false;
      if (hidden.includes(w.id)) return false;
      const itemEl = gridRef.current?.engine.nodes.find(n => n.id === w.id)?.el
        ?? document.querySelector<HTMLElement>(`.grid-stack-item[gs-id="${w.id}"]`);
      if (!itemEl) return false;
      return (itemEl as HTMLElement).style.display === 'none';
    });
    render(
      open ? (
        <AddWidgetPicker
          availableWidgets={screenWidgets.filter(w => hidden.includes(w.id))}
          emptyHiddenWidgets={emptyHiddenWidgets}
          onAdd={(widgetId) => {
            const def = widgetById(widgetId);
            if (!def || !gridRef.current) return;
            hiddenRef.current = hiddenRef.current.filter(id => id !== widgetId);

            const currentWidgets = layoutFromGrid(gridRef.current, hiddenRef.current).widgets;
            const y = nextY(currentWidgets);
            const placed: PlacedWidget = { i: widgetId, x: 0, y, w: def.defaultSize.w, h: def.defaultSize.h };
            if (def.minW !== undefined) placed.minW = def.minW;
            if (def.minH !== undefined) placed.minH = def.minH;
            mountWidgetIntoGrid(gridRef.current, placed);
            scheduleSave();
            updateAddBtnVisibility();
          }}
          onClose={() => renderPicker(false)}
        />
      ) : null,
      pickerMountRef.current
    );
  }, [screen, scheduleSave]);

  function updateAddBtnVisibility() {
    const btn = addBtnRef.current;
    if (!btn) return;
    btn.style.display = editMode.value && hiddenRef.current.length > 0 ? '' : 'none';
  }

  // ── Initialize GridStack ───────────────────────────────────────────
  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;

    if (isMobileRef.current) {
      renderMobileStack(el, screen);
      return;
    }

    let cancelled = false;

    (async () => {
      let layout: ScreenLayout;
      try {
        const saved = await fetchLayout(screen);
        layout = saved
          ? reconcileLayout(saved, screen)
          : reconcileLayout(DEFAULT_LAYOUTS[screen], screen);
      } catch {
        layout = reconcileLayout(DEFAULT_LAYOUTS[screen], screen);
      }

      if (cancelled) return;

      const gridRoot = document.createElement('div');
      gridRoot.className = 'grid-stack';

      const pickerMount = document.createElement('div');
      pickerMount.className = 'widget-picker-mount';
      pickerMountRef.current = pickerMount;

      const controlsBar = document.createElement('div');
      controlsBar.className = 'widget-grid-controls';

      const addBtn = document.createElement('button');
      addBtn.type = 'button';
      addBtn.className = 'widget-add-button header-button';
      addBtn.textContent = '[+] Add widget';
      addBtn.setAttribute('aria-label', 'Add widget');
      addBtn.style.display = 'none';
      addBtn.addEventListener('click', () => renderPicker(true));
      addBtnRef.current = addBtn;

      const resetBtn = document.createElement('button');
      resetBtn.type = 'button';
      resetBtn.className = 'widget-reset-button header-button';
      resetBtn.textContent = '[Reset layout]';
      resetBtn.setAttribute('aria-label', 'Reset layout to defaults');
      resetBtn.style.display = 'none';
      resetBtn.addEventListener('click', async () => {
        if (!confirm('Reset layout to defaults? Your custom positions will be lost.')) return;
        try {
          await resetLayout(screen);
        } catch {
          // ignore — restore locally regardless
        }
        const def = reconcileLayout(DEFAULT_LAYOUTS[screen], screen);
        hiddenRef.current = def.hidden.slice();
        if (gridRef.current) {
          gridRef.current.destroy(false);
          gridRef.current = null;
        }
        gridRoot.innerHTML = '';
        const newGrid = initGrid(gridRoot, def);
        setupGridEvents(newGrid, gridRoot, scheduleSave, hiddenRef, renderPicker, updateAddBtnVisibility);
        gridRef.current = newGrid;
        updateAddBtnVisibility();
        syncEditMode(newGrid, el, editMode.value, resetBtn);
        setStatus('layout-save', 'success', '[RESET]', 2000);
      });
      resetBtnRef.current = resetBtn;

      controlsBar.appendChild(addBtn);
      controlsBar.appendChild(resetBtn);
      el.appendChild(controlsBar);
      el.appendChild(gridRoot);
      el.appendChild(pickerMount);

      hiddenRef.current = layout.hidden.slice();
      const grid = initGrid(gridRoot, layout);
      setupGridEvents(grid, gridRoot, scheduleSave, hiddenRef, renderPicker, updateAddBtnVisibility);
      gridRef.current = grid;

      // Publish the initial layout so SavedViewsBar's "Save view" can
      // capture it without waiting for the user to drag anything.
      publishCurrentLayout(screen, layout);

      updateAddBtnVisibility();
      syncEditMode(grid, el, editMode.value, resetBtn);
    })();

    return () => {
      cancelled = true;
      if (saveTimerRef.current !== null) clearTimeout(saveTimerRef.current);
      if (gridRef.current) {
        gridRef.current.destroy(false);
        gridRef.current = null;
      }
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [screen]);

  // ── React to editMode changes ──────────────────────────────────────
  useEffect(() => {
    const el = containerRef.current;
    if (!el || isMobileRef.current) return;
    const grid = gridRef.current;
    const resetBtn = resetBtnRef.current;
    if (grid && resetBtn) {
      syncEditMode(grid, el, editMode.value, resetBtn);
      updateAddBtnVisibility();
    }
  }, [editMode.value]);

  // ── React to saved-view application ────────────────────────────────
  useEffect(() => {
    const pending = pendingLayoutApply.value;
    if (!pending || pending.screen !== screen) return;
    const el = containerRef.current;
    if (!el || isMobileRef.current) return;
    // Tear down + rebuild grid from the picked view's layout.
    const reconciled = reconcileLayout(pending.layout, screen);
    hiddenRef.current = reconciled.hidden.slice();
    if (gridRef.current) {
      gridRef.current.destroy(false);
      gridRef.current = null;
    }
    const gridRoot = el.querySelector<HTMLElement>('.grid-stack');
    if (!gridRoot) return;
    gridRoot.innerHTML = '';
    const newGrid = initGrid(gridRoot, reconciled);
    setupGridEvents(newGrid, gridRoot, scheduleSave, hiddenRef, renderPicker, updateAddBtnVisibility);
    gridRef.current = newGrid;
    publishCurrentLayout(screen, reconciled);
    updateAddBtnVisibility();
    const resetBtn = resetBtnRef.current;
    if (resetBtn) syncEditMode(newGrid, el, editMode.value, resetBtn);
    // Persist to backend so the picked layout survives without the user
    // having to drag anything (matches the existing reset-layout flow).
    saveLayout(screen, reconciled).catch(() => {
      /* server may not yet have a row — non-fatal */
    });
    setStatus('layout-save', 'success', '[VIEW APPLIED]', 2000);
    pendingLayoutApply.value = null;
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [pendingLayoutApply.value, screen]);

  // ── Mobile resize handler ──────────────────────────────────────────
  useEffect(() => {
    const onResize = () => {
      const nowMobile = window.innerWidth < MOBILE_BREAKPOINT;
      if (nowMobile !== isMobileRef.current) {
        isMobileRef.current = nowMobile;
        const el = containerRef.current;
        if (!el) return;
        if (nowMobile && gridRef.current) {
          gridRef.current.destroy(false);
          gridRef.current = null;
          el.innerHTML = '';
          renderMobileStack(el, screen);
        }
      }
    };
    window.addEventListener('resize', onResize);
    return () => window.removeEventListener('resize', onResize);
  }, [screen]);

  return <div ref={containerRef} class="widget-grid-root" />;
}

// ── Helper: mount a single widget element into the grid ──────────────────────

function mountWidgetIntoGrid(grid: GridStack, placed: PlacedWidget) {
  const def = widgetById(placed.i);
  if (!def) return;

  const itemEl = document.createElement('div');
  itemEl.className = 'grid-stack-item';
  itemEl.setAttribute('gs-id', placed.i);
  itemEl.setAttribute('gs-x', String(placed.x));
  itemEl.setAttribute('gs-y', String(placed.y));
  itemEl.setAttribute('gs-w', String(placed.w));
  itemEl.setAttribute('gs-h', String(placed.h));
  if (placed.minW !== undefined) itemEl.setAttribute('gs-min-w', String(placed.minW));
  if (placed.minH !== undefined) itemEl.setAttribute('gs-min-h', String(placed.minH));

  const contentEl = document.createElement('div');
  contentEl.className = 'grid-stack-item-content widget-card';

  const chromeEl = buildChrome(grid, itemEl);
  const bodyEl = document.createElement('div');
  bodyEl.className = 'widget-body';
  def.render(bodyEl);

  contentEl.appendChild(chromeEl);
  contentEl.appendChild(bodyEl);
  itemEl.appendChild(contentEl);
  grid.makeWidget(itemEl);
  return itemEl;
}

function buildChrome(
  grid: GridStack,
  itemEl: HTMLElement,
): HTMLElement {
  const chromeEl = document.createElement('div');
  chromeEl.className = 'widget-chrome';
  chromeEl.innerHTML =
    '<span class="widget-drag-handle" title="Drag to move" aria-hidden="true">&#x2807;</span>';

  const removeBtn = document.createElement('button');
  removeBtn.className = 'widget-remove-button';
  removeBtn.type = 'button';
  removeBtn.title = 'Hide widget';
  removeBtn.setAttribute('aria-label', 'Hide widget');
  removeBtn.textContent = '×';
  // The remove logic is wired up via the 'widget-hidden' custom event in setupGridEvents.
  removeBtn.addEventListener('click', () => {
    const widgetId = itemEl.getAttribute('gs-id') ?? '';
    grid.removeWidget(itemEl, false);
    itemEl.dispatchEvent(new CustomEvent('widget-hidden', { detail: widgetId, bubbles: true }));
  });
  chromeEl.appendChild(removeBtn);
  return chromeEl;
}

// ── Helper: init GridStack and mount all widgets ──────────────────────────────

function initGrid(gridRoot: HTMLElement, layout: ScreenLayout): GridStack {
  const grid = GridStack.init(
    {
      float: true,
      column: GRID_COLUMNS,
      cellHeight: CELL_HEIGHT,
      margin: CELL_MARGIN,
      draggable: { handle: '.widget-drag-handle' },
      resizable: { handles: 'se' },
      disableDrag: true,
      disableResize: true,
      // Auto-resize every widget to fit its rendered content height,
      // eliminating internal v-scrollbars. Pairs with `.widget-body`
      // using `overflow: visible` (input.css). GridStack still rounds
      // up to whole cellHeight rows.
      sizeToContent: true,
    },
    gridRoot
  );

  grid.batchUpdate(true);

  for (const placed of layout.widgets) {
    const def = widgetById(placed.i);
    if (!def) continue;

    const itemEl = document.createElement('div');
    itemEl.className = 'grid-stack-item';
    itemEl.setAttribute('gs-id', placed.i);
    itemEl.setAttribute('gs-x', String(placed.x));
    itemEl.setAttribute('gs-y', String(placed.y));
    itemEl.setAttribute('gs-w', String(placed.w));
    itemEl.setAttribute('gs-h', String(placed.h));
    if (placed.minW !== undefined) itemEl.setAttribute('gs-min-w', String(placed.minW));
    if (placed.minH !== undefined) itemEl.setAttribute('gs-min-h', String(placed.minH));

    const contentEl = document.createElement('div');
    contentEl.className = 'grid-stack-item-content widget-card';

    const chromeEl = buildChrome(grid, itemEl);
    const bodyEl = document.createElement('div');
    bodyEl.className = 'widget-body';
    def.render(bodyEl);

    contentEl.appendChild(chromeEl);
    contentEl.appendChild(bodyEl);
    itemEl.appendChild(contentEl);
    gridRoot.appendChild(itemEl);
    grid.makeWidget(itemEl);
  }

  grid.batchUpdate(false);
  return grid;
}

// ── Helper: wire up GridStack change events ───────────────────────────────────

function setupGridEvents(
  grid: GridStack,
  gridRoot: HTMLElement,
  scheduleSave: () => void,
  hiddenRef: { current: string[] },
  renderPicker: (open: boolean) => void,
  updateAddBtnVisibility: () => void,
) {
  grid.on('change', () => scheduleSave());
  grid.on('added', () => scheduleSave());

  gridRoot.addEventListener('widget-hidden', (e) => {
    const id = (e as CustomEvent<string>).detail;
    hiddenRef.current = [...hiddenRef.current, id];
    scheduleSave();
    updateAddBtnVisibility();
    renderPicker(false);
  });
}

function syncEditMode(
  grid: GridStack,
  container: HTMLElement,
  editing: boolean,
  resetBtn: HTMLButtonElement,
) {
  if (editing) {
    grid.enable();
    container.classList.add('editing');
  } else {
    grid.disable();
    container.classList.remove('editing');
  }
  resetBtn.style.display = editing ? '' : 'none';
}

// ── Mobile: plain read-only stack ───────────────────────────────────────────

function renderMobileStack(el: HTMLElement, screen: DashboardScreen) {
  el.innerHTML = '';
  const layout = reconcileLayout(DEFAULT_LAYOUTS[screen], screen);
  const sorted = layout.widgets.slice().sort((a, b) => a.y - b.y || a.x - b.x);
  const stack = document.createElement('div');
  stack.className = 'widget-mobile-stack';
  for (const placed of sorted) {
    const def = widgetById(placed.i);
    if (!def) continue;
    const card = document.createElement('div');
    card.className = 'widget-card widget-mobile-card';
    const body = document.createElement('div');
    body.className = 'widget-body';
    def.render(body);
    card.appendChild(body);
    stack.appendChild(card);
  }
  el.appendChild(stack);
}

// Suppress unused-import warning — WIDGET_CATALOG is imported for its side effects.
void WIDGET_CATALOG;
