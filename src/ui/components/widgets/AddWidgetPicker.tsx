/**
 * AddWidgetPicker — modal that lists hidden widgets and lets the user
 * re-add them to the active grid. Also surfaces widgets that are
 * currently hidden because their data response was empty so they
 * remain discoverable.
 */
import { useEffect } from 'preact/hooks';
import type { WidgetDef } from '../../widgets/registry';
import { isOverridden, setOverride, widgetEmptyOverrides } from '../../lib/widget-overrides';

interface AddWidgetPickerProps {
  /** Widgets the user previously removed from the layout. */
  availableWidgets: WidgetDef[];
  /** Widgets that are in the layout but currently hidden because their
   *  renderer reported no content. Listed for discoverability with a
   *  "Show anyway" toggle. */
  emptyHiddenWidgets: WidgetDef[];
  onAdd: (widgetId: string) => void;
  onClose: () => void;
}

export function AddWidgetPicker({
  availableWidgets,
  emptyHiddenWidgets,
  onAdd,
  onClose,
}: AddWidgetPickerProps) {
  // Subscribe to override-set changes so the badge text updates live.
  void widgetEmptyOverrides.value;

  // Close on Escape.
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [onClose]);

  const hasAvailable = availableWidgets.length > 0;
  const hasEmpty = emptyHiddenWidgets.length > 0;
  const isEmpty = !hasAvailable && !hasEmpty;

  return (
    <div
      class="modal-overlay"
      role="dialog"
      aria-modal="true"
      aria-label="Add widget"
      onClick={e => { if (e.target === e.currentTarget) onClose(); }}
    >
      <div class="modal-panel">
        <div class="modal-header">
          <h2 class="modal-title">Add widget</h2>
          <button
            type="button"
            class="modal-close-button"
            onClick={onClose}
            aria-label="Close"
          >
            ×
          </button>
        </div>

        {isEmpty && (
          <div class="modal-empty">
            All widgets are visible. Remove a widget first to add it back.
          </div>
        )}

        {hasAvailable && (
          <div class="widget-picker-group">
            {hasEmpty && (
              <div class="widget-picker-group-label">Available</div>
            )}
            <ul class="widget-picker-list">
              {availableWidgets.map(widget => (
                <li key={widget.id} class="widget-picker-item">
                  <div class="widget-picker-info">
                    <span class="widget-picker-title">{widget.title}</span>
                    {widget.description && (
                      <span class="widget-picker-desc">{widget.description}</span>
                    )}
                  </div>
                  <button
                    type="button"
                    class="widget-picker-add-btn"
                    onClick={() => { onAdd(widget.id); onClose(); }}
                  >
                    Add
                  </button>
                </li>
              ))}
            </ul>
          </div>
        )}

        {hasEmpty && (
          <div class="widget-picker-group">
            <div class="widget-picker-group-label">
              Hidden because empty
              <span class="widget-picker-group-hint">
                These widgets reappear automatically when their data is available.
              </span>
            </div>
            <ul class="widget-picker-list">
              {emptyHiddenWidgets.map(widget => {
                const overridden = isOverridden(widget.id);
                return (
                  <li key={widget.id} class="widget-picker-item widget-picker-item--muted">
                    <div class="widget-picker-info">
                      <span class="widget-picker-title">{widget.title}</span>
                      {widget.description && (
                        <span class="widget-picker-desc">{widget.description}</span>
                      )}
                    </div>
                    <button
                      type="button"
                      class={`widget-picker-add-btn${overridden ? ' is-active' : ''}`}
                      onClick={() => setOverride(widget.id, !overridden)}
                      aria-pressed={overridden}
                    >
                      {overridden ? 'Hide' : 'Show anyway'}
                    </button>
                  </li>
                );
              })}
            </ul>
          </div>
        )}
      </div>
    </div>
  );
}
