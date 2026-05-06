import { settingsDraft } from '../../state/store';
import type { SettingsResponse } from '../../state/types';

type PollingKey = 'oauth' | 'claude_admin' | 'openai' | 'agent_status' | 'aggregator';

interface RowConfig {
  key: PollingKey;
  label: string;
  /** Lower bound for refresh_interval; aggregator uses 60s, others 30s. */
  minInterval: number;
  /** Whether the row exposes lookback_days. */
  hasLookback: boolean;
}

const ROWS: RowConfig[] = [
  { key: 'oauth', label: 'OAuth (Claude usage windows)', minInterval: 30, hasLookback: false },
  { key: 'claude_admin', label: 'Claude admin', minInterval: 30, hasLookback: true },
  { key: 'openai', label: 'OpenAI', minInterval: 30, hasLookback: true },
  { key: 'agent_status', label: 'Agent status', minInterval: 30, hasLookback: false },
  { key: 'aggregator', label: 'Aggregator', minInterval: 60, hasLookback: false },
];

const SEVERITY_OPTIONS: Array<SettingsResponse['agent_status']['alert_min_severity']> = [
  'minor',
  'major',
  'critical',
];

const MAX_INTERVAL = 86400;
const MIN_LOOKBACK = 1;
const MAX_LOOKBACK = 365;

function patch<K extends PollingKey>(key: K, p: Partial<SettingsResponse[K]>): void {
  const draft = settingsDraft.value;
  if (!draft) return;
  // Cast through `unknown` because TS struggles with the union of group shapes.
  const next = { ...(draft[key] as object), ...(p as object) } as SettingsResponse[K];
  settingsDraft.value = { ...draft, [key]: next } as SettingsResponse;
}

function clampInterval(raw: number, min: number): number {
  if (!Number.isFinite(raw)) return min;
  return Math.max(min, Math.min(MAX_INTERVAL, Math.round(raw)));
}

function clampLookback(raw: number): number {
  if (!Number.isFinite(raw)) return MIN_LOOKBACK;
  return Math.max(MIN_LOOKBACK, Math.min(MAX_LOOKBACK, Math.round(raw)));
}

function intervalLabel(seconds: number): string {
  if (seconds >= 3600 && seconds % 3600 === 0) {
    const h = seconds / 3600;
    return `polled every ${h} hour${h === 1 ? '' : 's'}`;
  }
  if (seconds >= 60 && seconds % 60 === 0) {
    const m = seconds / 60;
    return `polled every ${m} minute${m === 1 ? '' : 's'}`;
  }
  return `polled every ${seconds} seconds`;
}

interface IntervalStepperProps {
  value: number;
  min: number;
  onChange: (next: number) => void;
  ariaLabel: string;
}

function IntervalStepper({ value, min, onChange, ariaLabel }: IntervalStepperProps) {
  return (
    <div class="settings-stepper">
      <button
        type="button"
        class="settings-stepper-btn"
        aria-label={`${ariaLabel} decrease`}
        onClick={() => onChange(clampInterval(value - 30, min))}
      >
        [-]
      </button>
      <input
        type="number"
        class="settings-input num settings-input--narrow"
        value={value}
        min={min}
        max={MAX_INTERVAL}
        step={30}
        aria-label={ariaLabel}
        onInput={(e) => {
          const raw = Number.parseFloat((e.target as HTMLInputElement).value);
          onChange(clampInterval(raw, min));
        }}
      />
      <button
        type="button"
        class="settings-stepper-btn"
        aria-label={`${ariaLabel} increase`}
        onClick={() => onChange(clampInterval(value + 30, min))}
      >
        [+]
      </button>
    </div>
  );
}

export function PollingSection() {
  const draft = settingsDraft.value;
  if (!draft) return null;

  return (
    <div class="settings-section">
      {ROWS.map((row) => {
        const groupAny = draft[row.key] as { enabled: boolean; refresh_interval: number };
        const enabled = groupAny.enabled;
        const interval = groupAny.refresh_interval;

        return (
          <div key={row.key} class="settings-card">
            <div class="settings-row settings-row--toggle">
              <label class="settings-label" for={`settings-polling-${row.key}-enabled`}>
                {row.label}
              </label>
              <input
                id={`settings-polling-${row.key}-enabled`}
                type="checkbox"
                checked={enabled}
                onChange={(e) =>
                  patch(row.key, { enabled: (e.target as HTMLInputElement).checked } as never)
                }
              />
            </div>

            {enabled && (
              <>
                <div class="settings-row">
                  <label class="settings-label">Refresh interval</label>
                  <IntervalStepper
                    value={interval}
                    min={row.minInterval}
                    ariaLabel={`${row.label} refresh interval seconds`}
                    onChange={(next) =>
                      patch(row.key, { refresh_interval: next } as never)
                    }
                  />
                </div>
                <div class="settings-helper">{intervalLabel(interval)}</div>

                {row.hasLookback && (
                  <div class="settings-row">
                    <label class="settings-label" for={`settings-polling-${row.key}-lookback`}>
                      Lookback days
                    </label>
                    <input
                      id={`settings-polling-${row.key}-lookback`}
                      type="number"
                      class="settings-input num settings-input--narrow"
                      value={(groupAny as unknown as { lookback_days: number }).lookback_days}
                      min={MIN_LOOKBACK}
                      max={MAX_LOOKBACK}
                      step={1}
                      onInput={(e) => {
                        const raw = Number.parseFloat((e.target as HTMLInputElement).value);
                        patch(row.key, { lookback_days: clampLookback(raw) } as never);
                      }}
                    />
                  </div>
                )}

                {row.key === 'agent_status' && (
                  <>
                    <div class="settings-row settings-row--toggle">
                      <label class="settings-label" for="settings-polling-agent-status-claude">
                        Claude provider
                      </label>
                      <input
                        id="settings-polling-agent-status-claude"
                        type="checkbox"
                        checked={draft.agent_status.claude_enabled}
                        onChange={(e) =>
                          patch('agent_status', {
                            claude_enabled: (e.target as HTMLInputElement).checked,
                          })
                        }
                      />
                    </div>
                    <div class="settings-row settings-row--toggle">
                      <label class="settings-label" for="settings-polling-agent-status-openai">
                        OpenAI provider
                      </label>
                      <input
                        id="settings-polling-agent-status-openai"
                        type="checkbox"
                        checked={draft.agent_status.openai_enabled}
                        onChange={(e) =>
                          patch('agent_status', {
                            openai_enabled: (e.target as HTMLInputElement).checked,
                          })
                        }
                      />
                    </div>
                    <div class="settings-row">
                      <label class="settings-label" for="settings-polling-agent-status-severity">
                        Alert min severity
                      </label>
                      <select
                        id="settings-polling-agent-status-severity"
                        class="settings-input"
                        value={draft.agent_status.alert_min_severity}
                        onChange={(e) =>
                          patch('agent_status', {
                            alert_min_severity: (e.target as HTMLSelectElement)
                              .value as SettingsResponse['agent_status']['alert_min_severity'],
                          })
                        }
                      >
                        {SEVERITY_OPTIONS.map((s) => (
                          <option key={s} value={s}>
                            {s}
                          </option>
                        ))}
                      </select>
                    </div>
                  </>
                )}

                {row.key === 'aggregator' && (
                  <div class="settings-row settings-row--toggle">
                    <label class="settings-label" for="settings-polling-aggregator-spike">
                      Spike webhook
                    </label>
                    <input
                      id="settings-polling-aggregator-spike"
                      type="checkbox"
                      checked={draft.aggregator.spike_webhook}
                      onChange={(e) =>
                        patch('aggregator', {
                          spike_webhook: (e.target as HTMLInputElement).checked,
                        })
                      }
                    />
                  </div>
                )}
              </>
            )}
          </div>
        );
      })}
    </div>
  );
}
