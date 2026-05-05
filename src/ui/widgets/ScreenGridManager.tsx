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
import { useEffect } from 'preact/hooks';
import { WidgetGrid } from './WidgetGrid';
import { activeDashboardTab } from '../state/store';
import type { DashboardScreen } from './registry';

const ALL_SCREENS: DashboardScreen[] = [
  'overview',
  'activity',
  'breakdowns',
  'tables',
  'projects',
];

export function ScreenGridManager() {
  const activeScreen = activeDashboardTab.value as DashboardScreen;

  useEffect(() => {
    // When the tab changes, refresh visibility of all grid roots.
    // The WidgetGrid components are always mounted; visibility is CSS-driven.
    // (The signal subscription triggers a re-render, so activeScreen is current.)
  }, [activeScreen]);

  return (
    <>
      {ALL_SCREENS.map(screen => (
        <div
          key={screen}
          class="screen-grid-wrapper"
          data-screen={screen}
          style={{ display: screen === activeScreen ? '' : 'none' }}
        >
          <WidgetGrid screen={screen} />
        </div>
      ))}
    </>
  );
}
