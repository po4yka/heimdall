import { useEffect, useRef, useState } from 'preact/hooks';
import { settingsDraft } from '../../state/store';
import { esc } from '../../lib/format';

type PricingOverride = {
  model: string;
  input: number;
  output: number;
  cache_write: number | null;
  cache_read: number | null;
};

type PricingModel = {
  model: string;
  family: string;
  default_input: number;
  default_output: number;
  default_cache_write: number | null;
  default_cache_read: number | null;
};

function patchOverrides(overrides: PricingOverride[]): void {
  const draft = settingsDraft.value;
  if (!draft) return;
  settingsDraft.value = {
    ...draft,
    pricing: { overrides },
  };
}

function fmtRate(v: number | null): string {
  if (v === null) return '?';
  return `$${v.toFixed(2)}`;
}

// ── Model picker popover ─────────────────────────────────────────────────────

interface ModelPickerProps {
  models: PricingModel[];
  existingModels: Set<string>;
  onSelect: (m: PricingModel) => void;
  onCustom: (name: string) => void;
  onClose: () => void;
}

function ModelPicker({ models, existingModels, onSelect, onCustom, onClose }: ModelPickerProps) {
  const ref = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const [query, setQuery] = useState('');

  // Auto-focus search input.
  useEffect(() => {
    if (inputRef.current) inputRef.current.focus();
  }, []);

  // Close on click-outside.
  useEffect(() => {
    function handler(e: MouseEvent) {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        onClose();
      }
    }
    document.addEventListener('mousedown', handler);
    return () => document.removeEventListener('mousedown', handler);
  }, [onClose]);

  // Close on Escape.
  useEffect(() => {
    function handler(e: KeyboardEvent) {
      if (e.key === 'Escape') onClose();
    }
    document.addEventListener('keydown', handler);
    return () => document.removeEventListener('keydown', handler);
  }, [onClose]);

  const lower = query.toLowerCase();

  // Filter and group by family (up to 50 total).
  const filtered = models
    .filter((m) => !lower || m.model.toLowerCase().includes(lower) || m.family.toLowerCase().includes(lower))
    .slice(0, 50);

  const grouped = new Map<string, PricingModel[]>();
  for (const m of filtered) {
    const group = grouped.get(m.family) ?? [];
    group.push(m);
    grouped.set(m.family, group);
  }

  // Show "Use custom" row when the user typed something not in the list.
  const typedNotInList =
    query.trim().length > 0 &&
    !models.some((m) => m.model.toLowerCase() === query.trim().toLowerCase());

  return (
    <div class="settings-pricing-picker" ref={ref}>
      <input
        ref={inputRef}
        type="text"
        class="settings-input settings-pricing-picker-search"
        placeholder="Search models..."
        value={query}
        onInput={(e) => setQuery((e.target as HTMLInputElement).value)}
      />
      <div class="settings-pricing-picker-list">
        {filtered.length === 0 && !typedNotInList && (
          <div class="settings-pricing-picker-empty">
            {models.length === 0
              ? 'No model catalog available. Type a model name below.'
              : 'No models match.'}
          </div>
        )}
        {Array.from(grouped.entries()).map(([family, rows]) => (
          <div key={family}>
            <div class="settings-pricing-picker-group">{esc(family)}</div>
            {rows.map((m) => {
              const alreadyAdded = existingModels.has(m.model);
              return (
                <button
                  key={m.model}
                  type="button"
                  class={`settings-pricing-picker-row${alreadyAdded ? ' settings-pricing-picker-row--disabled' : ''}`}
                  disabled={alreadyAdded}
                  onMouseDown={(e) => {
                    e.preventDefault();
                    if (!alreadyAdded) onSelect(m);
                  }}
                >
                  <span class="num">{esc(m.model)}</span>
                  <span class="settings-pricing-picker-rates">
                    {alreadyAdded
                      ? <em>Already overridden</em>
                      : `${fmtRate(m.default_input)} / ${fmtRate(m.default_output)}`}
                  </span>
                </button>
              );
            })}
          </div>
        ))}
        {typedNotInList && (
          <button
            type="button"
            class="settings-pricing-picker-row settings-pricing-picker-row--custom"
            onMouseDown={(e) => {
              e.preventDefault();
              onCustom(query.trim());
            }}
          >
            <span class="num">Use custom: {esc(query.trim())}</span>
            <span class="settings-pricing-picker-rates">rates: 0 / 0</span>
          </button>
        )}
      </div>
    </div>
  );
}

// ── Single override row ──────────────────────────────────────────────────────

interface OverrideRowProps {
  entry: PricingOverride;
  index: number;
  defaultRates: PricingModel | null;
  onChange: (index: number, updated: PricingOverride) => void;
  onDelete: (index: number) => void;
}

function OverrideRow({ entry, index, defaultRates, onChange, onDelete }: OverrideRowProps) {
  function setField<K extends keyof PricingOverride>(k: K, v: PricingOverride[K]) {
    onChange(index, { ...entry, [k]: v });
  }

  function parseNum(raw: string): number {
    const v = parseFloat(raw);
    return isFinite(v) && v >= 0 ? v : 0;
  }

  function parseOptNum(raw: string): number | null {
    if (raw.trim() === '') return null;
    const v = parseFloat(raw);
    return isFinite(v) && v >= 0 ? v : null;
  }

  return (
    <div class="settings-pricing-row">
      {/* Model — read-only */}
      <div class="settings-pricing-cell settings-pricing-cell--model">
        <span class="num">{esc(entry.model)}</span>
      </div>

      {/* Input */}
      <div class="settings-pricing-cell settings-pricing-cell--rate">
        <div class="settings-pricing-input-wrap">
          <input
            type="number"
            class="settings-input num settings-pricing-num-input"
            value={entry.input}
            min="0"
            step="0.01"
            onInput={(e) => setField('input', parseNum((e.target as HTMLInputElement).value))}
          />
          <span class="settings-pricing-suffix">/ 1M</span>
        </div>
        <small class="settings-pricing-default">
          {defaultRates !== null ? `default: ${fmtRate(defaultRates.default_input)}` : 'default: ?'}
        </small>
      </div>

      {/* Output */}
      <div class="settings-pricing-cell settings-pricing-cell--rate">
        <div class="settings-pricing-input-wrap">
          <input
            type="number"
            class="settings-input num settings-pricing-num-input"
            value={entry.output}
            min="0"
            step="0.01"
            onInput={(e) => setField('output', parseNum((e.target as HTMLInputElement).value))}
          />
          <span class="settings-pricing-suffix">/ 1M</span>
        </div>
        <small class="settings-pricing-default">
          {defaultRates !== null ? `default: ${fmtRate(defaultRates.default_output)}` : 'default: ?'}
        </small>
      </div>

      {/* Cache write */}
      <div class="settings-pricing-cell settings-pricing-cell--rate">
        <div class="settings-pricing-input-wrap">
          <input
            type="number"
            class="settings-input num settings-pricing-num-input"
            value={entry.cache_write ?? ''}
            placeholder="—"
            min="0"
            step="0.01"
            onInput={(e) => setField('cache_write', parseOptNum((e.target as HTMLInputElement).value))}
          />
          {entry.cache_write !== null && (
            <button
              type="button"
              class="settings-pricing-clear-btn"
              onClick={() => setField('cache_write', null)}
            >
              [CLEAR]
            </button>
          )}
        </div>
        <small class="settings-pricing-default">
          {defaultRates !== null ? `default: ${fmtRate(defaultRates.default_cache_write)}` : 'default: ?'}
        </small>
      </div>

      {/* Cache read */}
      <div class="settings-pricing-cell settings-pricing-cell--rate">
        <div class="settings-pricing-input-wrap">
          <input
            type="number"
            class="settings-input num settings-pricing-num-input"
            value={entry.cache_read ?? ''}
            placeholder="—"
            min="0"
            step="0.01"
            onInput={(e) => setField('cache_read', parseOptNum((e.target as HTMLInputElement).value))}
          />
          {entry.cache_read !== null && (
            <button
              type="button"
              class="settings-pricing-clear-btn"
              onClick={() => setField('cache_read', null)}
            >
              [CLEAR]
            </button>
          )}
        </div>
        <small class="settings-pricing-default">
          {defaultRates !== null ? `default: ${fmtRate(defaultRates.default_cache_read)}` : 'default: ?'}
        </small>
      </div>

      {/* Delete */}
      <div class="settings-pricing-cell settings-pricing-cell--action">
        <button
          type="button"
          class="settings-aliases-delete-btn"
          aria-label={`Delete override for ${esc(entry.model)}`}
          onClick={() => onDelete(index)}
        >
          [X]
        </button>
      </div>
    </div>
  );
}

// ── Section ──────────────────────────────────────────────────────────────────

export function PricingSection() {
  const draft = settingsDraft.value;
  if (!draft) return null;

  const overrides = draft.pricing.overrides;

  const [filter, setFilter] = useState('');
  const [showPicker, setShowPicker] = useState(false);
  const [pricingModels, setPricingModels] = useState<PricingModel[]>([]);
  const [modelsFetched, setModelsFetched] = useState(false);
  // Cache default rates keyed by model name, populated when user picks a model.
  const [defaultsCache, setDefaultsCache] = useState<Map<string, PricingModel>>(new Map());

  const addBtnRef = useRef<HTMLButtonElement>(null);

  async function ensureModelsFetched() {
    if (modelsFetched) return;
    try {
      const r = await fetch('/api/pricing-models');
      if (!r.ok) throw new Error(`HTTP ${r.status}`);
      const body = (await r.json()) as { models: PricingModel[] };
      setPricingModels(body.models ?? []);
      // Seed defaults cache from catalog.
      setDefaultsCache((prev) => {
        const next = new Map(prev);
        for (const m of (body.models ?? [])) next.set(m.model, m);
        return next;
      });
    } catch {
      setPricingModels([]);
    }
    setModelsFetched(true);
  }

  function handleAddClick() {
    void ensureModelsFetched().then(() => setShowPicker(true));
    if (modelsFetched) setShowPicker(true);
  }

  function handlePickerSelect(m: PricingModel) {
    const newEntry: PricingOverride = {
      model: m.model,
      input: m.default_input,
      output: m.default_output,
      cache_write: m.default_cache_write,
      cache_read: m.default_cache_read,
    };
    setDefaultsCache((prev) => new Map(prev).set(m.model, m));
    patchOverrides([...overrides, newEntry]);
    setShowPicker(false);
  }

  function handlePickerCustom(name: string) {
    const newEntry: PricingOverride = {
      model: name,
      input: 0,
      output: 0,
      cache_write: null,
      cache_read: null,
    };
    patchOverrides([...overrides, newEntry]);
    setShowPicker(false);
  }

  function handleChange(index: number, updated: PricingOverride) {
    patchOverrides(overrides.map((e, i) => (i === index ? updated : e)));
  }

  function handleDelete(index: number) {
    patchOverrides(overrides.filter((_, i) => i !== index));
  }

  const lowerFilter = filter.toLowerCase();
  const visibleIndices = overrides
    .map((e, i) => ({ e, i }))
    .filter(({ e }) => !lowerFilter || e.model.toLowerCase().includes(lowerFilter));

  const existingModels = new Set(overrides.map((o) => o.model));
  const count = overrides.length;

  return (
    <div class="settings-section">
      {/* Header strip */}
      <div class="settings-aliases-header">
        <input
          type="text"
          class="settings-input settings-aliases-filter"
          placeholder="Filter overrides..."
          value={filter}
          onInput={(e) => setFilter((e.target as HTMLInputElement).value)}
        />
        <div class="settings-pricing-add-wrap">
          <button
            ref={addBtnRef}
            type="button"
            class="settings-btn settings-btn--primary"
            onClick={handleAddClick}
          >
            [+ Add override]
          </button>
          {showPicker && (
            <ModelPicker
              models={pricingModels}
              existingModels={existingModels}
              onSelect={handlePickerSelect}
              onCustom={handlePickerCustom}
              onClose={() => setShowPicker(false)}
            />
          )}
        </div>
      </div>

      {/* Table */}
      <div class="settings-pricing-table">
        {/* Column headers */}
        <div class="settings-pricing-thead">
          <div class="settings-pricing-th settings-pricing-th--model">MODEL</div>
          <div class="settings-pricing-th settings-pricing-th--rate">INPUT</div>
          <div class="settings-pricing-th settings-pricing-th--rate">OUTPUT</div>
          <div class="settings-pricing-th settings-pricing-th--rate">CACHE WRITE</div>
          <div class="settings-pricing-th settings-pricing-th--rate">CACHE READ</div>
          <div class="settings-pricing-th settings-pricing-th--action"></div>
        </div>

        {/* Rows */}
        {overrides.length === 0 ? (
          <div class="settings-pricing-empty">
            No pricing overrides configured yet. Click [+ Add override] to override a model's price.
          </div>
        ) : visibleIndices.length === 0 ? (
          <div class="settings-pricing-empty">
            No overrides match &ldquo;{esc(filter)}&rdquo;.
          </div>
        ) : (
          visibleIndices.map(({ e, i }) => (
            <OverrideRow
              key={i}
              entry={e}
              index={i}
              defaultRates={defaultsCache.get(e.model) ?? null}
              onChange={handleChange}
              onDelete={handleDelete}
            />
          ))
        )}
      </div>

      {/* Footer help */}
      <div class="settings-aliases-footer">
        <span class="settings-hint">
          Overrides replace Heimdall's built-in pricing for matching model names. Values are USD per million tokens.
        </span>
        <span class="settings-aliases-counter">
          {count} {count === 1 ? 'override' : 'overrides'}
        </span>
      </div>
    </div>
  );
}
