import { signal } from '@preact/signals';
import type { DashboardScreen, ScreenLayout } from './registry';

export interface PendingLayoutApply {
  screen: DashboardScreen;
  layout: ScreenLayout;
}

/**
 * Cross-component signal: when the user picks a saved view, the
 * SavedViewsBar publishes the {screen, layout} pair here. WidgetGrid
 * consumes it on the matching screen, rebuilds its grid, and clears the
 * signal. Decouples the views bar from the grid implementation.
 */
export const pendingLayoutApply = signal<PendingLayoutApply | null>(null);

/**
 * Latest known layout per screen — published by WidgetGrid after every
 * GridStack change event. SavedViewsBar reads from here when the user
 * saves the current layout as a new view.
 */
export const currentLayoutByScreen = signal<Partial<Record<DashboardScreen, ScreenLayout>>>({});

export function publishCurrentLayout(screen: DashboardScreen, layout: ScreenLayout): void {
  currentLayoutByScreen.value = { ...currentLayoutByScreen.value, [screen]: layout };
}
