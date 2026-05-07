/**
 * ScreenGridManager — mounts one WidgetGrid per dashboard screen.
 *
 * All grids are kept in the DOM at all times (display:none when inactive) so
 * that existing view.tsx render functions can target their widget body-element
 * IDs via document.getElementById at any time without a re-mount cycle.
 *
 * Only the active screen's grid is visible. Tab changes are handled by
 * subscribing to the `activeDashboardTab` signal.
 */
import { WidgetGrid } from './WidgetGrid';
import { activeDashboardTab, tabToScreen, rawData } from '../state/store';
import { ChartSkeleton } from '../components/_primitives/Skeleton';
import type { DashboardScreen } from './registry';

const ALL_SCREENS: DashboardScreen[] = [
  'overview',
  'activity',
  'breakdowns',
  'tables',
  'projects',
];

export function ScreenGridManager() {
  const activeScreen = tabToScreen(activeDashboardTab.value);
  // Show skeleton while rawData hasn't arrived yet (initial page load only).
  // loadState is only set to 'refreshing' when rawData is already non-null,
  // so the correct signal for "no data yet" is rawData itself.
  const isLoading = rawData.value === null;

  return (
    <>
      {ALL_SCREENS.map(screen => (
        <div
          key={screen}
          class="screen-grid-wrapper"
          data-screen={screen}
          style={{ display: screen === activeScreen ? '' : 'none' }}
        >
          {screen === activeScreen && isLoading && (
            <div class="screen-skeleton-overlay" aria-live="polite" aria-busy="true">
              <ChartSkeleton />
              <ChartSkeleton />
              <ChartSkeleton />
            </div>
          )}
          <WidgetGrid screen={screen} />
        </div>
      ))}
    </>
  );
}
