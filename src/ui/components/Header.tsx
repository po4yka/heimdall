import { useRef, useEffect } from 'preact/hooks';
import { useSignalEffect } from '@preact/signals';
import { createTriggerRescan } from '../lib/rescan';
import { setStatus } from '../lib/status';
import { InlineStatus } from './InlineStatus';
import {
  metaText,
  planBadge,
  rescanLabel,
  rescanDisabled,
  themeMode,
  editMode,
  backupModalOpen,
  settingsModalOpen,
  versionInfo,
} from '../state/store';

interface HeaderProps {
  onDataReload: (force?: boolean) => Promise<void>;
  onThemeToggle: () => void;
  navigationHref?: string;
  navigationLabel?: string;
}

export function Header({
  onDataReload,
  onThemeToggle,
  navigationHref,
  navigationLabel,
}: HeaderProps) {
  const headerRef = useRef<HTMLElement | null>(null);
  const btnRef = useRef<HTMLButtonElement | null>(null);
  const triggerRef = useRef<(() => Promise<void>) | null>(null);

  useSignalEffect(() => {
    const themeColorMeta = document.querySelector<HTMLMetaElement>('meta[name="theme-color"]');
    if (!themeColorMeta) return;
    themeColorMeta.setAttribute('content', themeMode.value === 'light' ? '#F5F5F5' : '#0A0A0A');
  });

  useEffect(() => {
    if (!headerRef.current) return;
    const root = document.documentElement;
    const updateOffset = () => {
      if (!headerRef.current) return;
      root.style.setProperty('--header-offset', `${Math.ceil(headerRef.current.getBoundingClientRect().height)}px`);
    };

    updateOffset();
    const observer = new ResizeObserver(() => updateOffset());
    observer.observe(headerRef.current);
    window.addEventListener('resize', updateOffset);

    return () => {
      observer.disconnect();
      window.removeEventListener('resize', updateOffset);
    };
  }, []);

  useEffect(() => {
    if (!btnRef.current) return;
    const proxy = {
      get disabled() { return rescanDisabled.value; },
      set disabled(v: boolean) { rescanDisabled.value = v; },
      get textContent() { return rescanLabel.value; },
      set textContent(v: string | null) { rescanLabel.value = v ?? ''; },
    };
    triggerRef.current = createTriggerRescan({
      button: proxy,
      fetchImpl: (input, init) => fetch(input, init),
      loadData: onDataReload,
      showError: (msg) => setStatus('rescan', 'error', msg, 6000),
      setTimer: (cb, ms) => window.setTimeout(cb, ms),
      logError: (e) => console.error(e),
    });
  }, [onDataReload]);

  const isEditing = editMode.value;
  const isMobile = typeof window !== 'undefined' && window.innerWidth < 720;

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

  const info = versionInfo.value;
  const versionTitle = info ? `Heimdall · v${info.current}` : 'Heimdall';
  const updateUrl = info?.update_available ? info.latest_url : null;
  const planLabel = planBadge.value;

  return (
    <header ref={headerRef}>
      <h1 title={versionTitle} aria-label="Heimdall">
        <span class="header-logo-mark" aria-hidden="true">
          <span class="header-logo-mark__pre">Code</span>
          {' '}
          <span class="header-logo-mark__post">Usage</span>
        </span>
        {planLabel && (
          <span class="header-plan-badge" aria-hidden="true">
            {planLabel}
          </span>
        )}
      </h1>
      <div class="meta">{metaText}</div>
      <div class="header-actions">
        {navigationHref && navigationLabel && (
          <a class="header-button header-button--link" href={navigationHref}>
            [{navigationLabel}]
          </a>
        )}
        {updateUrl && (
          <a
            class="header-button header-button--link header-button--update"
            href={updateUrl}
            target="_blank"
            rel="noopener noreferrer"
            title={`Latest: v${info?.latest} (current: v${info?.current})`}
          >
            [Update v{info?.latest} →]
          </a>
        )}
        <button
          class="theme-toggle"
          type="button"
          onClick={onThemeToggle}
          aria-label="Toggle theme"
        >
          {icon}
        </button>
        {!isMobile && (
          <button
            type="button"
            class={`header-button${isEditing ? ' header-button--active' : ''}`}
            onClick={() => { editMode.value = !editMode.value; }}
            aria-pressed={isEditing}
            aria-label={isEditing ? 'Done editing layout' : 'Edit layout'}
          >
            {isEditing ? '[Done]' : '[Edit layout]'}
          </button>
        )}
        {!isMobile && (
          <button
            type="button"
            class="header-button"
            onClick={() => {
              backupModalOpen.value = true;
              if (!/^#\/backup\b/.test(window.location.hash)) {
                history.replaceState(null, '', `${window.location.pathname}${window.location.search}#/backup`);
              }
            }}
            aria-label="Open backup and snapshots"
          >
            [Backup]
          </button>
        )}
        {!isMobile && (
          <button
            type="button"
            class="header-button"
            onClick={() => {
              settingsModalOpen.value = true;
              if (!/^#\/settings\b/.test(window.location.hash)) {
                history.replaceState(null, '', `${window.location.pathname}${window.location.search}#/settings`);
              }
            }}
            aria-label="Open settings"
          >
            [Settings]
          </button>
        )}
        <button
          id="rescan-btn"
          ref={btnRef}
          type="button"
          disabled={rescanDisabled.value}
          onClick={() => triggerRef.current?.()}
          aria-label="Rescan database"
        >
          {rescanLabel}
        </button>
        <InlineStatus placement="rescan" inline />
        <InlineStatus placement="header-refresh" inline dismissable={false} />
      </div>
    </header>
  );
}
