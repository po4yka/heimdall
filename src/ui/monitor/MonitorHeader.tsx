import { themeMode } from '../state/store';
import {
  isLiveMonitorPanelHidden,
  LIVE_MONITOR_PANEL_OPTIONS,
  liveMonitorData,
  liveMonitorDensity,
  liveMonitorFocus,
  liveMonitorRefreshing,
  setLiveMonitorDensity,
  setLiveMonitorFocus,
  toggleLiveMonitorPanel,
} from './store';

interface MonitorHeaderProps {
  onThemeToggle: () => void;
  onRefresh: () => Promise<void>;
}

export function MonitorHeader({ onThemeToggle, onRefresh }: MonitorHeaderProps) {
  const mode = themeMode.value;
  const icon = mode === 'dark'
    ? <svg aria-hidden="true" focusable="false" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
      </svg>
    : <svg aria-hidden="true" focusable="false" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="5" />
        <line x1="12" y1="1" x2="12" y2="3" />
        <line x1="12" y1="21" x2="12" y2="23" />
        <line x1="4.22" y1="4.22" x2="5.64" y2="5.64" />
        <line x1="18.36" y1="18.36" x2="19.78" y2="19.78" />
        <line x1="1" y1="12" x2="3" y2="12" />
        <line x1="21" y1="12" x2="23" y2="12" />
        <line x1="4.22" y1="19.78" x2="5.64" y2="18.36" />
        <line x1="18.36" y1="5.64" x2="19.78" y2="4.22" />
      </svg>;
  const generatedAt = liveMonitorData.value?.generated_at ?? null;
  const issue = liveMonitorData.value?.global_issue ?? null;

  return (
    <header>
      <div style={{ display: 'flex', alignItems: 'center', gap: '12px', flexWrap: 'wrap' }}>
        <h1 style={{ marginBottom: 0 }}>
          <span style={{ color: 'var(--text-secondary)', fontWeight: 400 }}>Live</span>
          {' '}
          <span style={{ color: 'var(--text-display)', fontWeight: 500 }}>Monitor</span>
        </h1>
        {issue && (
          <span
            style={{
              fontFamily: 'var(--font-mono)',
              fontSize: '10px',
              padding: '2px 8px',
              borderRadius: '999px',
              border: '1px solid var(--border-visible)',
              color: 'var(--accent)',
              letterSpacing: '0.08em',
              textTransform: 'uppercase',
            }}
          >
            {issue}
          </span>
        )}
      </div>
      <div class="meta">
        {generatedAt ? `Updated ${new Date(generatedAt).toLocaleTimeString()}` : 'Waiting for monitor data'}
      </div>
      <div class="header-actions">
        <div style={{ display: 'inline-flex', border: '1px solid var(--border-visible)', borderRadius: '999px', overflow: 'hidden' }}>
          {(['all', 'claude', 'codex'] as const).map(option => (
            <button
              key={option}
              type="button"
              onClick={() => { setLiveMonitorFocus(option); }}
              style={{
                padding: '8px 12px',
                border: 'none',
                borderRight: option === 'codex' ? 'none' : '1px solid var(--border-visible)',
                background: liveMonitorFocus.value === option ? 'var(--text-primary)' : 'transparent',
                color: liveMonitorFocus.value === option ? 'var(--bg)' : 'var(--text-primary)',
                fontSize: '12px',
                letterSpacing: '0.08em',
                textTransform: 'uppercase',
              }}
            >
              {option === 'all' ? 'All' : option === 'claude' ? 'Claude' : 'Codex'}
            </button>
          ))}
        </div>
        <div style={{ display: 'inline-flex', border: '1px solid var(--border-visible)', borderRadius: '999px', overflow: 'hidden' }}>
          {(['expanded', 'compact'] as const).map(option => (
            <button
              key={option}
              type="button"
              onClick={() => { setLiveMonitorDensity(option); }}
              style={{
                padding: '8px 12px',
                border: 'none',
                borderRight: option === 'compact' ? 'none' : '1px solid var(--border-visible)',
                background: liveMonitorDensity.value === option ? 'var(--text-primary)' : 'transparent',
                color: liveMonitorDensity.value === option ? 'var(--bg)' : 'var(--text-primary)',
                fontSize: '12px',
                letterSpacing: '0.08em',
                textTransform: 'uppercase',
              }}
            >
              {option}
            </button>
          ))}
        </div>
        <details
          style={{
            border: '1px solid var(--border-visible)',
            borderRadius: '18px',
            padding: '8px 12px',
            minWidth: '220px',
          }}
        >
          <summary
            style={{
              cursor: 'pointer',
              listStyle: 'none',
              fontSize: '12px',
              letterSpacing: '0.08em',
              textTransform: 'uppercase',
            }}
          >
            Panels
          </summary>
          <div style={{ display: 'grid', gap: '8px', marginTop: '10px' }}>
            {LIVE_MONITOR_PANEL_OPTIONS.map(panel => {
              const visible = !isLiveMonitorPanelHidden(panel.id);
              return (
                <label key={panel.id} style={{ display: 'flex', alignItems: 'center', gap: '8px', fontSize: '12px' }}>
                  <input
                    type="checkbox"
                    checked={visible}
                    onInput={() => { toggleLiveMonitorPanel(panel.id); }}
                  />
                  <span>{panel.label}</span>
                </label>
              );
            })}
          </div>
        </details>
        <a
          href="/"
          style={{
            border: '1px solid var(--border-visible)',
            borderRadius: '999px',
            padding: '8px 12px',
            color: 'var(--text-primary)',
            textDecoration: 'none',
            fontSize: '12px',
            letterSpacing: '0.08em',
            textTransform: 'uppercase',
          }}
        >
          Dashboard
        </a>
        <button
          class="theme-toggle"
          type="button"
          onClick={onThemeToggle}
          aria-label="Toggle theme"
        >
          {icon}
        </button>
        <button type="button" onClick={() => void onRefresh()} disabled={liveMonitorRefreshing.value}>
          {liveMonitorRefreshing.value ? 'Refreshing…' : 'Refresh'}
        </button>
      </div>
    </header>
  );
}
