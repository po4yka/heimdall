/**
 * AddWidgetPicker — modal that lists hidden widgets and lets the user
 * re-add them to the active grid. Style matches AgentRegistryModal.
 */
import { useEffect } from 'preact/hooks';
import type { WidgetDef } from '../../widgets/registry';

interface AddWidgetPickerProps {
  availableWidgets: WidgetDef[];
  onAdd: (widgetId: string) => void;
  onClose: () => void;
}

export function AddWidgetPicker({ availableWidgets, onAdd, onClose }: AddWidgetPickerProps) {
  // Close on Escape.
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [onClose]);

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

        {availableWidgets.length === 0 ? (
          <div class="modal-empty">
            All widgets are visible. Remove a widget first to add it back.
          </div>
        ) : (
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
        )}
      </div>
    </div>
  );
}
