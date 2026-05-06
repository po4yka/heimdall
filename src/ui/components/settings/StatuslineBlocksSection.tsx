import { settingsDraft } from '../../state/store';
import type { SettingsResponse } from '../../state/types';

function patchStatusline(p: Partial<SettingsResponse['statusline']>): void {
  const draft = settingsDraft.value;
  if (!draft) return;
  settingsDraft.value = { ...draft, statusline: { ...draft.statusline, ...p } };
}

function patchBlocks(p: Partial<SettingsResponse['blocks']>): void {
  const draft = settingsDraft.value;
  if (!draft) return;
  settingsDraft.value = { ...draft, blocks: { ...draft.blocks, ...p } };
}

function thresholdHint(value: number, kind: 'fraction' | 'positive'): string | null {
  if (kind === 'fraction') {
    if (value < 0) return 'must be >= 0';
    if (value > 1) return 'must be <= 1.0';
  } else {
    if (value <= 0) return 'must be > 0';
  }
  return null;
}

interface ThresholdInputProps {
  id: string;
  label: string;
  value: number;
  kind: 'fraction' | 'positive';
  step: number;
  onChange: (next: number) => void;
}

function ThresholdInput({ id, label, value, kind, step, onChange }: ThresholdInputProps) {
  const hint = thresholdHint(value, kind);
  return (
    <div class="settings-threshold">
      <label class="settings-label" for={id}>{label}</label>
      <input
        id={id}
        type="number"
        class="settings-input num"
        value={value}
        step={step}
        onInput={(e) => {
          const raw = Number.parseFloat((e.target as HTMLInputElement).value);
          if (Number.isFinite(raw)) onChange(raw);
        }}
      />
      {hint && <div class="settings-hint settings-hint--error">{hint}</div>}
    </div>
  );
}

export function StatuslineBlocksSection() {
  const draft = settingsDraft.value;
  if (!draft) return null;
  const sl = draft.statusline;
  const bl = draft.blocks;

  return (
    <div class="settings-section">
      <div class="settings-card">
        <h3 class="settings-subtitle">Statusline thresholds</h3>
        <div class="settings-grid-2x2">
          <ThresholdInput
            id="settings-statusline-context-low"
            label="Context low"
            value={sl.context_low_threshold}
            kind="fraction"
            step={0.01}
            onChange={(v) => patchStatusline({ context_low_threshold: v })}
          />
          <ThresholdInput
            id="settings-statusline-context-medium"
            label="Context medium"
            value={sl.context_medium_threshold}
            kind="fraction"
            step={0.01}
            onChange={(v) => patchStatusline({ context_medium_threshold: v })}
          />
          <ThresholdInput
            id="settings-statusline-burn-normal"
            label="Burn-rate normal max"
            value={sl.burn_rate_normal_max}
            kind="positive"
            step={0.1}
            onChange={(v) => patchStatusline({ burn_rate_normal_max: v })}
          />
          <ThresholdInput
            id="settings-statusline-burn-moderate"
            label="Burn-rate moderate max"
            value={sl.burn_rate_moderate_max}
            kind="positive"
            step={0.1}
            onChange={(v) => patchStatusline({ burn_rate_moderate_max: v })}
          />
        </div>
      </div>

      <div class="settings-card">
        <h3 class="settings-subtitle">Blocks</h3>
        <div class="settings-row">
          <label class="settings-label" for="settings-blocks-token-limit">Token limit</label>
          <div class="settings-input-group">
            <input
              id="settings-blocks-token-limit"
              type="number"
              class="settings-input num"
              value={bl.token_limit ?? ''}
              placeholder="auto"
              min={0}
              step={1}
              onInput={(e) => {
                const v = (e.target as HTMLInputElement).value.trim();
                if (v === '') {
                  patchBlocks({ token_limit: null });
                  return;
                }
                const parsed = Number.parseInt(v, 10);
                if (Number.isFinite(parsed) && parsed >= 0) {
                  patchBlocks({ token_limit: parsed });
                }
              }}
            />
            <button
              type="button"
              class="settings-clear-btn"
              aria-label="Clear token limit"
              disabled={bl.token_limit == null}
              onClick={() => patchBlocks({ token_limit: null })}
            >
              [CLEAR]
            </button>
          </div>
        </div>

        <div class="settings-row">
          <label class="settings-label" for="settings-blocks-session-length">Session length (hours)</label>
          <div class="settings-input-group">
            <input
              id="settings-blocks-session-length"
              type="number"
              class="settings-input num"
              value={bl.session_length_hours ?? ''}
              placeholder="auto"
              min={0}
              step={0.5}
              onInput={(e) => {
                const v = (e.target as HTMLInputElement).value.trim();
                if (v === '') {
                  patchBlocks({ session_length_hours: null });
                  return;
                }
                const parsed = Number.parseFloat(v);
                if (Number.isFinite(parsed) && parsed >= 0) {
                  patchBlocks({ session_length_hours: parsed });
                }
              }}
            />
            <button
              type="button"
              class="settings-clear-btn"
              aria-label="Clear session length"
              disabled={bl.session_length_hours == null}
              onClick={() => patchBlocks({ session_length_hours: null })}
            >
              [CLEAR]
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
