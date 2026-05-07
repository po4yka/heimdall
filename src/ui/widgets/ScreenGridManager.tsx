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
import { activeDashboardTab, tabToScreen, loadState, rawData } from '../state/store';
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
  const isLoading = loadState.value === 'refreshing' && rawData.value === null;
  const isError = loadState.value === 'idle' && rawData.value === null;

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
          {screen === activeScreen && isError && (
            <div role="alert" aria-live="assertive" style={{ padding: '24px', fontFamily: 'var(--font-mono)', fontSize: '12px', color: 'var(--text-secondary)' }}>
              [ERROR: FAILED TO LOAD DATA — CHECK SERVER LOGS]
            </div>
          )}
          <WidgetGrid screen={screen} />
        </div>
      ))}
    </>
  );
}
