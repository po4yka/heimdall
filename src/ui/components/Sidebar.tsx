import { useEffect } from 'preact/hooks';
import { activeDashboardTab, sidebarCollapsed, syncDashboardUrl, backupModalOpen, settingsModalOpen, type DashboardTab } from '../state/store';

const NAV_ITEMS: Array<{ key: DashboardTab; label: string; abbr: string }> = [
  { key: 'overview',   label: 'Overview',      abbr: 'OV' },
  { key: 'today',      label: 'Today',          abbr: 'TD' },
  { key: 'activity',   label: 'Activity',       abbr: 'AC' },
  { key: 'agents',     label: 'Agents',         abbr: 'AG' },
  { key: 'breakdowns', label: 'Cost & Models',  abbr: 'C$' },
  { key: 'tables',     label: 'Sessions',       abbr: 'SS' },
  { key: 'projects',   label: 'Projects',       abbr: 'PR' },
];

export function Sidebar() {
  const collapsed = sidebarCollapsed.value;
  const activeTab = activeDashboardTab.value;

  // Sync body data attribute so CSS grid can respond to collapsed state.
  useEffect(() => {
    if (collapsed) {
      document.documentElement.dataset['sidebarCollapsed'] = '';
    } else {
      delete document.documentElement.dataset['sidebarCollapsed'];
    }
  }, [collapsed]);

  const handleNavClick = (tab: DashboardTab) => {
    activeDashboardTab.value = tab;
    syncDashboardUrl();
  };

  const toggleCollapsed = () => {
    sidebarCollapsed.value = !collapsed;
    try {
      localStorage.setItem('heimdall:sidebarCollapsed', String(!collapsed));
    } catch {
      // ignored
    }
  };

  return (
    <nav
      class={`sidebar${collapsed ? ' sidebar--collapsed' : ''}`}
      aria-label="Dashboard navigation"
    >
      <ul class="sidebar__nav" role="list">
        {NAV_ITEMS.map(item => {
          const active = activeTab === item.key;
          return (
            <li key={item.key}>
              <button
                type="button"
                class={`sidebar__item${active ? ' sidebar__item--active' : ''}`}
                aria-current={active ? 'page' : undefined}
                title={collapsed ? item.label : undefined}
                onClick={() => handleNavClick(item.key)}
              >
                <span class="sidebar__abbr" aria-hidden="true">{item.abbr}</span>
                {!collapsed && <span class="sidebar__label">{item.label}</span>}
              </button>
            </li>
          );
        })}
      </ul>

      <div class="sidebar__footer">
        <button
          type="button"
          class="sidebar__icon-btn"
          aria-label="Settings"
          title="Settings"
          onClick={() => { settingsModalOpen.value = true; }}
        >
          <span aria-hidden="true">⚙</span>
          {!collapsed && <span class="sidebar__label">Settings</span>}
        </button>
        <button
          type="button"
          class="sidebar__icon-btn"
          aria-label="Backup"
          title="Backup"
          onClick={() => { backupModalOpen.value = true; }}
        >
          <span aria-hidden="true">⊙</span>
          {!collapsed && <span class="sidebar__label">Backup</span>}
        </button>
        <button
          type="button"
          class="sidebar__collapse-btn"
          aria-label={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
          title={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
          onClick={toggleCollapsed}
        >
          <span aria-hidden="true">{collapsed ? '»' : '«'}</span>
        </button>
      </div>
    </nav>
  );
}
