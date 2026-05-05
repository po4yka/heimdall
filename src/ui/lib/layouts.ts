import type { DashboardScreen, ScreenLayout } from '../widgets/registry';

/**
 * Fetch the saved layout for `screen` from the server.
 * Server returns `200 null` when no layout has been saved yet — that's
 * indistinguishable from "saved" in the DevTools network panel (no false
 * console error). Throws on non-OK responses.
 */
export async function fetchLayout(screen: DashboardScreen): Promise<ScreenLayout | null> {
  const res = await fetch(`/api/layouts/${encodeURIComponent(screen)}`);
  if (!res.ok) throw new Error(`fetchLayout: HTTP ${res.status}`);
  return (await res.json()) as ScreenLayout | null;
}

/**
 * Persist `layout` for `screen`. Returns the layout as echoed back by the server.
 * Throws on non-OK responses.
 */
export async function saveLayout(
  screen: DashboardScreen,
  layout: ScreenLayout
): Promise<ScreenLayout> {
  const res = await fetch(`/api/layouts/${encodeURIComponent(screen)}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(layout),
  });
  if (!res.ok) throw new Error(`saveLayout: HTTP ${res.status}`);
  return res.json() as Promise<ScreenLayout>;
}

/**
 * Delete the saved layout for `screen`, causing the client to fall back to
 * its embedded DEFAULT_LAYOUTS on the next render.
 * Throws on non-OK responses.
 */
export async function resetLayout(screen: DashboardScreen): Promise<void> {
  const res = await fetch(`/api/layouts/${encodeURIComponent(screen)}`, {
    method: 'DELETE',
  });
  if (!res.ok) throw new Error(`resetLayout: HTTP ${res.status}`);
}
