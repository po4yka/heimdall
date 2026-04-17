import {
  rawData,
  selectedModels,
  selectedRange,
  selectedProvider,
  projectSearchQuery,
  type ProviderFilter,
} from '../state/store';
import type { RangeKey } from '../state/types';

const RANGES: RangeKey[] = ['7d', '30d', '90d', 'all'];
const PROVIDERS: ProviderFilter[] = ['both', 'claude', 'codex'];
const PROVIDER_LABEL: Record<ProviderFilter, string> = {
  both: 'Both',
  claude: 'Claude',
  codex: 'Codex',
};

function modelPriority(m: string): number {
  const ml = m.toLowerCase();
  if (ml.includes('opus')) return 0;
  if (ml.includes('sonnet')) return 1;
  if (ml.includes('haiku')) return 2;
  return 3;
}

interface FilterBarProps {
  onFilterChange: () => void;
  onURLUpdate: () => void;
}

export function FilterBar({ onFilterChange, onURLUpdate }: FilterBarProps) {
  const allModels = rawData.value?.all_models ?? [];
  const sortedModels = [...allModels].sort((a, b) => {
    const pa = modelPriority(a);
    const pb = modelPriority(b);
    return pa !== pb ? pa - pb : a.localeCompare(b);
  });

  const toggleModel = (model: string, checked: boolean) => {
    const next = new Set(selectedModels.value);
    if (checked) next.add(model);
    else next.delete(model);
    selectedModels.value = next;
    onURLUpdate();
    onFilterChange();
  };

  const selectAll = () => {
    selectedModels.value = new Set(sortedModels);
    onURLUpdate();
    onFilterChange();
  };

  const clearAll = () => {
    selectedModels.value = new Set();
    onURLUpdate();
    onFilterChange();
  };

  const setRange = (range: RangeKey) => {
    selectedRange.value = range;
    onURLUpdate();
    onFilterChange();
  };

  const setProvider = (provider: ProviderFilter) => {
    selectedProvider.value = provider;
    onURLUpdate();
    onFilterChange();
  };

  const hasCodexData = rawData.value?.provider_breakdown?.some(p => p.provider === 'codex') ?? false;

  const onSearchInput = (e: Event) => {
    const value = (e.currentTarget as HTMLInputElement).value;
    projectSearchQuery.value = value.toLowerCase().trim();
    onURLUpdate();
    onFilterChange();
  };

  const clearSearch = () => {
    projectSearchQuery.value = '';
    onURLUpdate();
    onFilterChange();
  };

  return (
    <div id="filter-bar" role="toolbar" aria-label="Filters">
      <div class="filter-label">Models</div>
      <div id="model-checkboxes" role="group" aria-label="Model filters">
        {sortedModels.map(model => {
          const checked = selectedModels.value.has(model);
          return (
            <label key={model} class={`model-cb-label${checked ? ' checked' : ''}`} data-model={model}>
              <input
                type="checkbox"
                value={model}
                checked={checked}
                onChange={(e) => toggleModel(model, (e.currentTarget as HTMLInputElement).checked)}
                aria-label={model}
              />
              {model}
            </label>
          );
        })}
      </div>
      <button class="filter-btn" type="button" onClick={selectAll}>All</button>
      <button class="filter-btn" type="button" onClick={clearAll}>None</button>
      <div class="filter-sep"></div>
      <div class="filter-label">Range</div>
      <div class="range-group" role="group" aria-label="Date range">
        {RANGES.map(range => (
          <button
            key={range}
            class={`range-btn${selectedRange.value === range ? ' active' : ''}`}
            type="button"
            data-range={range}
            onClick={() => setRange(range)}
          >
            {range}
          </button>
        ))}
      </div>
      {hasCodexData && (
        <>
          <div class="filter-sep"></div>
          <div class="filter-label">Provider</div>
          <div class="range-group" role="group" aria-label="Provider">
            {PROVIDERS.map(provider => (
              <button
                key={provider}
                class={`range-btn${selectedProvider.value === provider ? ' active' : ''}`}
                type="button"
                data-provider={provider}
                onClick={() => setProvider(provider)}
              >
                {PROVIDER_LABEL[provider]}
              </button>
            ))}
          </div>
        </>
      )}
      <div class="filter-sep"></div>
      <label for="project-search" class="filter-label">Project</label>
      <input
        type="text"
        id="project-search"
        placeholder="Search..."
        aria-label="Filter by project"
        value={projectSearchQuery.value}
        onInput={onSearchInput}
        style={{
          background: 'transparent',
          border: '1px solid var(--border-visible)',
          color: 'var(--text-primary)',
          padding: '3px 10px',
          borderRadius: '4px',
          fontFamily: 'var(--font-mono)',
          fontSize: '11px',
          letterSpacing: '0.04em',
          width: '160px',
          outline: 'none',
        }}
      />
      {projectSearchQuery.value && (
        <button class="filter-btn" id="project-clear-btn" type="button" onClick={clearSearch}>
          Clear
        </button>
      )}
    </div>
  );
}
