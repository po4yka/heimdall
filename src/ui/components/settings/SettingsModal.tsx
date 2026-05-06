import { useEffect, useState, useCallback } from 'preact/hooks';
import {
  settingsModalOpen,
  settingsServer,
  settingsDraft,
  settingsInFlight,
  settingsActiveSection,
} from '../../state/store';
import type { SettingsResponse, SettingsSectionKey, SettingsPatch } from '../../state/types';
import { setStatus } from '../../lib/status';
import { InlineStatus } from '../InlineStatus';
import { DisplaySection } from './DisplaySection';
import { PollingSection } from './PollingSection';
import { StatuslineBlocksSection } from './StatuslineBlocksSection';
import { WebhooksSection } from './WebhooksSection';
import type { UrlIntent } from './WebhooksSection';

interface SettingsModalProps {
  onDataReload: (force?: boolean) => Promise<void>;
}

interface SectionMeta {
  key: SettingsSectionKey;
  label: string;
  description: string;
  /** When true the row renders disabled with [Coming soon]. */
  comingSoon: boolean;
}

const SECTIONS: SectionMeta[] = [
  { key: 'display', label: 'Display', description: 'Currency, locale, and number compaction.', comingSoon: false },
  { key: 'polling', label: 'Polling', description: 'How often live data sources are refreshed.', comingSoon: false },
  { key: 'statusline_blocks', label: 'Statusline & blocks', description: 'Threshold tuning and block sizing.', comingSoon: false },
  { key: 'webhooks', label: 'Webhooks', description: 'Notify external systems on events.', comingSoon: false },
  { key: 'aliases', label: 'Project aliases', description: 'Map project slugs to display names.', comingSoon: true },
  { key: 'pricing', label: 'Pricing overrides', description: 'Custom rates for specific models.', comingSoon: true },
];

function isDirty(server: SettingsResponse | null, draft: SettingsResponse | null): boolean {
  if (!server || !draft) return false;
  return JSON.stringify(server) !== JSON.stringify(draft);
}

/** Compute a sparse SettingsPatch containing only sections whose stringified
 *  form differs between draft and server. Only sections present in M2 may
 *  change today; the diff still iterates the full top-level keyset so future
 *  milestones get this for free.
 *
 *  urlIntent is passed explicitly because the webhook URL is never round-tripped
 *  through the draft (server returns url_present:bool, not the URL string itself).
 *  The intent encodes whether the user set or cleared the URL this session. */
function diffPatch(server: SettingsResponse, draft: SettingsResponse, urlIntent: UrlIntent): SettingsPatch {
  const patch: SettingsPatch = {};
  type SectionKey = keyof SettingsResponse;
  // Iterate keys defensively — `read_only` is server-derived and can't be
  // patched, so we exclude it from the diff up front.
  const keys: SectionKey[] = [
    'display',
    'oauth',
    'claude_admin',
    'openai',
    'agent_status',
    'aggregator',
    'blocks',
    'statusline',
    'webhooks',
    'project_aliases',
    'pricing',
  ];
  for (const key of keys) {
    if (JSON.stringify(server[key]) !== JSON.stringify(draft[key])) {
      // We send the *full* sub-object on diff; the server can interpret it as
      // a Partial since SettingsPatch types each section as Partial<...>.
      // For webhooks we strip `url_present` (server-derived) and inject the
      // URL intent from the section's local state.
      if (key === 'webhooks') {
        const { url_present: _drop, ...rest } = draft.webhooks;
        void _drop;
        const webhookPatch: SettingsPatch['webhooks'] = { ...rest };
        if (urlIntent.kind === 'set') {
          webhookPatch.url = urlIntent.value;
        } else if (urlIntent.kind === 'clear') {
          webhookPatch.url = null;
        }
        // kind === 'unchanged': omit url field entirely
        patch.webhooks = webhookPatch;
      } else {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        (patch as any)[key] = draft[key];
      }
    }
  }
  // Also emit webhooks patch if only the url intent changed (url_present unchanged
  // in draft but user explicitly set/cleared the URL).
  if (!patch.webhooks && urlIntent.kind !== 'unchanged') {
    const { url_present: _drop, ...rest } = draft.webhooks;
    void _drop;
    const webhookPatch: SettingsPatch['webhooks'] = { ...rest };
    if (urlIntent.kind === 'set') {
      webhookPatch.url = urlIntent.value;
    } else if (urlIntent.kind === 'clear') {
      webhookPatch.url = null;
    }
    patch.webhooks = webhookPatch;
  }
  return patch;
}

function closeModal(force = false): void {
  const dirty = isDirty(settingsServer.value, settingsDraft.value);
  if (dirty && !force) {
    const ok = window.confirm('Discard unsaved changes?');
    if (!ok) return;
  }
  settingsModalOpen.value = false;
  settingsDraft.value = settingsServer.value; // reset draft to last clean state
  if (/^#\/settings\b/.test(window.location.hash)) {
    history.replaceState(null, '', window.location.pathname + window.location.search);
  }
}

export function SettingsModal({ onDataReload }: SettingsModalProps) {
  const [loadError, setLoadError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  // URL intent tracks whether the user explicitly set or cleared the webhook URL
  // this session. It is separate from the draft because url_present is a bool
  // (server never returns the URL string) so we cannot diff it normally.
  const [urlIntent, setUrlIntent] = useState<UrlIntent>({ kind: 'unchanged' });

  const fetchSettings = useCallback(async () => {
    setLoading(true);
    setLoadError(null);
    try {
      const r = await fetch('/api/settings');
      if (!r.ok) throw new Error(`HTTP ${r.status}`);
      const body = (await r.json()) as SettingsResponse;
      settingsServer.value = body;
      settingsDraft.value = body;
      setUrlIntent({ kind: 'unchanged' });
    } catch (err) {
      setLoadError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { void fetchSettings(); }, [fetchSettings]);

  // Close on ESC. Mirrors BackupModal/AgentRegistryModal.
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') closeModal();
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, []);

  async function handleSave() {
    const server = settingsServer.value;
    const draft = settingsDraft.value;
    if (!server || !draft) return;
    const patch = diffPatch(server, draft, urlIntent);
    if (Object.keys(patch).length === 0) return;

    settingsInFlight.value = true;
    try {
      const r = await fetch('/api/settings', {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(patch),
      });
      if (!r.ok) {
        let msg = `HTTP ${r.status}`;
        try {
          const body = (await r.json()) as { error?: string };
          if (body.error) msg = body.error;
        } catch {
          // body might not be JSON; keep the status fallback
        }
        setStatus('settings', 'error', msg, 6000);
        return;
      }
      const updated = (await r.json()) as SettingsResponse;
      settingsServer.value = updated;
      settingsDraft.value = updated;
      setUrlIntent({ kind: 'unchanged' });
      setStatus('settings', 'success', 'SAVED', 2500);
      // Refresh dependent panels (currency, costs, etc.) after a save.
      void onDataReload(true);
    } catch (err) {
      setStatus('settings', 'error', err instanceof Error ? err.message : String(err), 6000);
    } finally {
      settingsInFlight.value = false;
    }
  }

  const dirty = isDirty(settingsServer.value, settingsDraft.value) || urlIntent.kind !== 'unchanged';
  const inFlight = settingsInFlight.value;
  const activeKey = settingsActiveSection.value;
  const activeMeta = SECTIONS.find((s) => s.key === activeKey) ?? SECTIONS[0]!;

  function renderSection() {
    if (loading) {
      return (
        <div class="settings-loading">Loading settings…</div>
      );
    }
    if (loadError) {
      return (
        <div class="settings-error-panel">
          <div>[ERROR: {loadError}]</div>
          <button type="button" class="settings-btn" onClick={() => void fetchSettings()}>
            [Retry]
          </button>
        </div>
      );
    }
    if (!settingsDraft.value) return null;
    switch (activeKey) {
      case 'display': return <DisplaySection />;
      case 'polling': return <PollingSection />;
      case 'statusline_blocks': return <StatuslineBlocksSection />;
      case 'webhooks': return (
        <WebhooksSection
          onUrlIntentChange={setUrlIntent}
        />
      );
      default: return (
        <div class="settings-loading">Coming soon.</div>
      );
    }
  }

  return (
    <div class="settings-overlay" onClick={() => closeModal()}>
      <div
        class="settings-modal"
        onClick={(e: Event) => e.stopPropagation()}
        role="dialog"
        aria-modal="true"
        aria-label="Settings"
      >
        <nav class="settings-rail" aria-label="Settings sections">
          <h2 class="settings-rail-title">Settings</h2>
          <ul class="settings-rail-list">
            {SECTIONS.map((s) => {
              const isActive = s.key === activeKey && !s.comingSoon;
              return (
                <li key={s.key}>
                  <button
                    type="button"
                    class={`settings-rail-item${isActive ? ' settings-rail-item--active' : ''}`}
                    disabled={s.comingSoon}
                    aria-current={isActive ? 'page' : undefined}
                    onClick={() => { if (!s.comingSoon) settingsActiveSection.value = s.key; }}
                  >
                    <span>{s.label}</span>
                    {s.comingSoon && <span class="settings-rail-suffix">[Coming soon]</span>}
                  </button>
                </li>
              );
            })}
          </ul>
        </nav>

        <div class="settings-pane">
          <header class="settings-pane-header">
            <div>
              <h3 class="settings-pane-title">{activeMeta.label}</h3>
              <p class="settings-pane-desc">{activeMeta.description}</p>
            </div>
            <button
              type="button"
              class="settings-close"
              aria-label="Close"
              onClick={() => closeModal()}
            >
              [X]
            </button>
          </header>

          <div class="settings-pane-body">
            {renderSection()}
          </div>

          <footer class="settings-pane-footer">
            <button
              type="button"
              class="settings-btn"
              onClick={() => closeModal()}
            >
              [Cancel]
            </button>
            <div class="settings-footer-status">
              <InlineStatus placement="settings" inline />
            </div>
            <button
              type="button"
              class="settings-btn settings-btn--primary"
              disabled={!dirty || inFlight}
              onClick={() => void handleSave()}
            >
              {inFlight ? '[Saving…]' : '[Save]'}
            </button>
          </footer>
        </div>
      </div>
    </div>
  );
}
