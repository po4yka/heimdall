import { activeDashboardTab, type DashboardTab } from '../state/store';

const TABS: Array<{ key: DashboardTab; label: string }> = [
  { key: 'overview', label: 'Overview' },
  { key: 'activity', label: 'Activity' },
  { key: 'breakdowns', label: 'Breakdowns' },
  { key: 'tables', label: 'Tables' },
];

interface DashboardTabsProps {
  onTabChange: (tab: DashboardTab) => void;
}

export function DashboardTabs({ onTabChange }: DashboardTabsProps) {
  return (
    <nav id="dashboard-tabs" role="tablist" aria-label="Dashboard sections">
      {TABS.map(tab => {
        const active = activeDashboardTab.value === tab.key;
        return (
          <button
            key={tab.key}
            type="button"
            role="tab"
            class={`dashboard-tab${active ? ' active' : ''}`}
            aria-selected={active}
            onClick={() => onTabChange(tab.key)}
          >
            {tab.label}
          </button>
        );
      })}
    </nav>
  );
}
