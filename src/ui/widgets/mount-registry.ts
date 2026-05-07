/**
 * Mount registry — bridges the WidgetGrid's `render(el)` callbacks with the
 * app-level Preact components that need to be mounted into grid-managed elements.
 *
 * Pattern:
 *   1. `app.tsx` calls `registerMountCallback('backup-panel', el => render(<BackupPanel />, el))`
 *      before the grids mount.
 *   2. The WidgetGrid registry's `render(el)` calls `invokeMountCallback('backup-panel', el)`.
 *   3. The BackupPanel is mounted into the GridStack-managed element.
 *
 * For widgets that the existing view.tsx / runtime.ts render via getElementById,
 * no mount callback is needed — setting el.id is sufficient because the runtime
 * fires AFTER the grids have mounted.
 */

type MountCallback = (el: HTMLElement) => void;

const callbacks = new Map<string, MountCallback>();

/** Register a callback to be invoked when the element with `mountId` is created. */
export function registerMountCallback(mountId: string, cb: MountCallback): void {
  callbacks.set(mountId, cb);
}

/** Invoke the registered callback for `mountId`, if any. */
export function invokeMountCallback(mountId: string, el: HTMLElement): void {
  const cb = callbacks.get(mountId);
  if (cb) {
    cb(el);
    delete el.dataset['loading'];
  }
}
