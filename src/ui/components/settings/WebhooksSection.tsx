import { useState } from 'preact/hooks';
import { settingsDraft } from '../../state/store';
import type { SettingsResponse } from '../../state/types';
import { esc } from '../../lib/format';

/** Tracks what the user intends to do with the webhook URL this session. */
type UrlIntent =
  | { kind: 'unchanged' }
  | { kind: 'set'; value: string }
  | { kind: 'clear' };

function patchWebhooks(p: Partial<Omit<SettingsResponse['webhooks'], 'url_present'>>): void {
  const draft = settingsDraft.value;
  if (!draft) return;
  settingsDraft.value = {
    ...draft,
    webhooks: { ...draft.webhooks, ...p },
  };
}

interface EventToggleProps {
  id: string;
  label: string;
  help: string;
  checked: boolean;
  onChange: (v: boolean) => void;
  disabled?: boolean;
}

function EventToggle({ id, label, help, checked, onChange, disabled }: EventToggleProps) {
  return (
    <div class={`settings-card settings-toggle-stack-item${disabled ? ' settings-toggle-stack-item--disabled' : ''}`}>
      <div class="settings-row settings-row--toggle">
        <label class="settings-label" for={id}>{label}</label>
        <input
          id={id}
          type="checkbox"
          checked={checked}
          disabled={disabled}
          onChange={(e) => onChange((e.target as HTMLInputElement).checked)}
        />
      </div>
      <div class="settings-helper">{help}</div>
    </div>
  );
}

interface WebhooksSectionProps {
  /** Called by the modal to extract the URL intent for diffPatch. */
  onUrlIntentChange: (intent: UrlIntent) => void;
}

export function WebhooksSection({ onUrlIntentChange }: WebhooksSectionProps) {
  const draft = settingsDraft.value;
  if (!draft) return null;

  const { webhooks } = draft;
  const urlPresent = webhooks.url_present;

  // Local state for the inline URL editor.
  const [editing, setEditing] = useState(false);
  const [urlInput, setUrlInput] = useState('');

  function handleSetUrl() {
    setUrlInput('');
    setEditing(true);
  }

  function handleSaveUrl() {
    const trimmed = urlInput.trim();
    if (!trimmed) return;
    // Never display the URL in status text — write only into the intent.
    onUrlIntentChange({ kind: 'set', value: trimmed });
    // Flip url_present in the draft so isDirty() picks up the change.
    const current = settingsDraft.value;
    if (current) {
      settingsDraft.value = {
        ...current,
        webhooks: { ...current.webhooks, url_present: true },
      };
    }
    setEditing(false);
    setUrlInput('');
  }

  function handleCancelUrl() {
    setEditing(false);
    setUrlInput('');
  }

  function handleClearUrl() {
    onUrlIntentChange({ kind: 'clear' });
    const current = settingsDraft.value;
    if (current) {
      settingsDraft.value = {
        ...current,
        webhooks: { ...current.webhooks, url_present: false },
      };
    }
  }

  // Cost threshold helpers.
  const costThreshold = webhooks.cost_threshold;
  const costInputVal = costThreshold !== null && costThreshold !== undefined
    ? String(costThreshold)
    : '';

  function handleCostInput(raw: string) {
    const trimmed = raw.trim();
    if (trimmed === '') {
      patchWebhooks({ cost_threshold: null });
      return;
    }
    const n = parseFloat(trimmed);
    if (Number.isFinite(n) && n > 0) {
      patchWebhooks({ cost_threshold: n });
    }
  }

  // Stop-reason filter helpers.
  const filterArr = webhooks.agent_stop_reason_filter ?? [];
  const filterStr = filterArr.join(', ');
  const agentStopEnabled = webhooks.agent_stop_reason;

  function handleFilterInput(raw: string) {
    const parts = raw.split(',').map((s) => s.trim()).filter(Boolean);
    patchWebhooks({ agent_stop_reason_filter: parts.length === 0 ? null : parts });
  }

  return (
    <div class="settings-section">

      {/* ── URL row ─────────────────────────────────────────────── */}
      <div class="settings-card">
        <div class="settings-row">
          <label class="settings-label">Webhook URL</label>
          <div class="settings-input-group">
            <span class="settings-helper" style="margin-top:0">
              URL configured: {urlPresent ? 'yes' : 'no'}
            </span>
            {!editing && (
              <>
                <button
                  type="button"
                  class="settings-clear-btn"
                  onClick={handleSetUrl}
                >
                  [Set URL]
                </button>
                <button
                  type="button"
                  class="settings-clear-btn settings-clear-btn--destructive"
                  disabled={!urlPresent}
                  onClick={handleClearUrl}
                >
                  [Clear URL]
                </button>
              </>
            )}
          </div>
        </div>

        {editing && (
          <div class="settings-webhook-url-editor">
            <input
              type="url"
              class="settings-input"
              placeholder="https://hooks.example.com/..."
              value={urlInput}
              // eslint-disable-next-line @typescript-eslint/no-unused-vars
              onInput={(e) => {
                // Never echo the URL into any status or display string.
                setUrlInput((e.target as HTMLInputElement).value);
              }}
              onKeyDown={(e) => {
                if (e.key === 'Enter') handleSaveUrl();
                if (e.key === 'Escape') handleCancelUrl();
              }}
              // Auto-focus the input when the inline editor opens.
              ref={(el) => { if (el) el.focus(); }}
            />
            <button type="button" class="settings-btn settings-btn--primary" onClick={handleSaveUrl}>
              [Save]
            </button>
            <button type="button" class="settings-btn" onClick={handleCancelUrl}>
              [Cancel]
            </button>
          </div>
        )}

        <div class="settings-helper">
          {esc('Webhook events POST a JSON body to this URL. Heimdall never sends the configured URL back to the browser; only “URL configured: yes/no” is exposed. Set or clear it from this Mac to update.')}
        </div>
      </div>

      {/* ── Cost threshold ──────────────────────────────────────── */}
      <div class="settings-card">
        <div class="settings-row">
          <label class="settings-label" for="settings-webhook-cost-threshold">
            Cost threshold (USD)
          </label>
          <div class="settings-input-group">
            <input
              id="settings-webhook-cost-threshold"
              type="number"
              class="settings-input num settings-input--narrow"
              value={costInputVal}
              min="0.01"
              step="0.01"
              placeholder="—"
              onInput={(e) => handleCostInput((e.target as HTMLInputElement).value)}
            />
            <button
              type="button"
              class="settings-clear-btn"
              disabled={costThreshold === null || costThreshold === undefined}
              onClick={() => patchWebhooks({ cost_threshold: null })}
            >
              [CLEAR]
            </button>
          </div>
        </div>
        <div class="settings-helper">Fire a webhook when daily cost crosses this amount.</div>
      </div>

      {/* ── Event toggles ───────────────────────────────────────── */}
      <div class="settings-toggle-stack">
        <EventToggle
          id="settings-webhook-session-depleted"
          label="Session depleted"
          help="Fire when an active session crosses the rate-limit ceiling."
          checked={webhooks.session_depleted}
          onChange={(v) => patchWebhooks({ session_depleted: v })}
        />
        <EventToggle
          id="settings-webhook-agent-status"
          label="Agent status"
          help="Fire on Claude / OpenAI status transitions."
          checked={webhooks.agent_status}
          onChange={(v) => patchWebhooks({ agent_status: v })}
        />
        <EventToggle
          id="settings-webhook-spike"
          label="Community spike"
          help="Fire when a third-party aggregator reports a Claude or OpenAI outage spike (only when official status is below Major)."
          checked={webhooks.spike_webhook}
          onChange={(v) => patchWebhooks({ spike_webhook: v })}
        />
        <EventToggle
          id="settings-webhook-cap-changes"
          label="Subscription cap changes"
          help="Fire when estimated Claude Code or Codex caps shift materially."
          checked={webhooks.cap_changes}
          onChange={(v) => patchWebhooks({ cap_changes: v })}
        />
        <EventToggle
          id="settings-webhook-agent-stop-reason"
          label="Subagent stop reason"
          help="Fire when a subagent exits with a stop_reason on the allowlist below."
          checked={webhooks.agent_stop_reason}
          onChange={(v) => patchWebhooks({ agent_stop_reason: v })}
        />
      </div>

      {/* ── Stop-reason filter ──────────────────────────────────── */}
      <div class={`settings-card${!agentStopEnabled ? ' settings-card--muted' : ''}`}>
        <div class="settings-row">
          <label class="settings-label" for="settings-webhook-stop-reason-filter">
            Stop-reason allowlist
          </label>
          <input
            id="settings-webhook-stop-reason-filter"
            type="text"
            class="settings-input"
            value={filterStr}
            placeholder="max_tokens, refusal"
            disabled={!agentStopEnabled}
            onInput={(e) => handleFilterInput((e.target as HTMLInputElement).value)}
          />
        </div>
        <div class="settings-helper">
          Comma-separated. Empty = use default (<span class="settings-mono">max_tokens, refusal</span>).
        </div>
      </div>

    </div>
  );
}

/** Export the UrlIntent type so SettingsModal can reference it. */
export type { UrlIntent };
