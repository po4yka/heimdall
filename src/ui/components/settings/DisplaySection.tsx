import { settingsDraft } from '../../state/store';
import type { SettingsResponse } from '../../state/types';

// NOTE(M2): currency options are hardcoded for the first cut. The real list
// should come from `/api/currencies` (server can derive from Frankfurter).
// Track in follow-up; the same enum mirrors what menubar.rs accepts today.
const CURRENCY_OPTIONS = ['USD', 'EUR', 'GBP', 'JPY', 'KRW', 'CNY'] as const;

function patchDisplay(patch: Partial<SettingsResponse['display']>): void {
  const draft = settingsDraft.value;
  if (!draft) return;
  settingsDraft.value = { ...draft, display: { ...draft.display, ...patch } };
}

export function DisplaySection() {
  const draft = settingsDraft.value;
  if (!draft) return null;
  const { currency, locale, compact } = draft.display;

  return (
    <div class="settings-section">
      <div class="settings-row">
        <label class="settings-label" for="settings-display-currency">Currency</label>
        <select
          id="settings-display-currency"
          class="settings-input num"
          value={currency ?? 'USD'}
          onChange={(e) => patchDisplay({ currency: (e.target as HTMLSelectElement).value })}
        >
          {CURRENCY_OPTIONS.map((c) => (
            <option key={c} value={c}>{c}</option>
          ))}
        </select>
      </div>

      <div class="settings-row">
        <label class="settings-label" for="settings-display-locale">Locale</label>
        <input
          id="settings-display-locale"
          type="text"
          class="settings-input"
          value={locale ?? ''}
          placeholder="auto"
          onInput={(e) => {
            const v = (e.target as HTMLInputElement).value.trim();
            patchDisplay({ locale: v.length === 0 ? null : v });
          }}
        />
      </div>

      <div class="settings-row settings-row--toggle">
        <label class="settings-label" for="settings-display-compact">Compact mode</label>
        <input
          id="settings-display-compact"
          type="checkbox"
          checked={compact ?? false}
          onChange={(e) => patchDisplay({ compact: (e.target as HTMLInputElement).checked })}
        />
      </div>
    </div>
  );
}
