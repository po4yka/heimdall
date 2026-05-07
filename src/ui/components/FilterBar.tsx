import { useEffect, useRef, useState } from 'preact/hooks';
import {
  activeDashboardTab,
  rawData,
  selectedModels,
  selectedRange,
  selectedProvider,
  selectedBucket,
  projectSearchQuery,
  mobile_filters_expanded,
  type DashboardTab,
  type ProviderFilter,
} from '../state/store';
import type { RangeKey, BucketKey } from '../state/types';

export function shortModelName(full: string): string {
  // Strip 'claude-' prefix, then remove long date suffixes like '-20251001'
  return full.replace(/^claude-/, '').replace(/-\d{8}$/, '');
}

type FilterGroup = 'range' | 'bucket' | 'provider' | 'models' | 'project-search';

const SECTION_FILTER_GROUPS: Record<DashboardTab, FilterGroup[]> = {
  overview:   ['range', 'bucket'],
  today:      [],
  activity:   ['range', 'bucket', 'provider', 'models'],
  agents:     ['range', 'provider'],
  breakdowns: ['range', 'bucket', 'provider', 'models'],
  tables:     ['range', 'provider', 'models', 'project-search'],
  projects:   ['project-search'],
};

const RANGES: RangeKey[] = ['7d', '30d', '90d', 'all'];
const RANGE_LABEL: Record<RangeKey, string> = {
  '7d': '7d',
  '30d': '30d',
  '90d': '90d',
  all: 'All',
};
const BUCKETS: BucketKey[] = ['day', 'week'];
const BUCKET_LABEL: Record<BucketKey, string> = { day: 'Day', week: 'Week' };
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

  const [modelsOpen, setModelsOpen] = useState(false);
  const popoverRef = useRef<HTMLDivElement | null>(null);
  const chipRef = useRef<HTMLButtonElement | null>(null);

  // Click-outside / Escape to close popover
  useEffect(() => {
    if (!modelsOpen) return;
    const onDocClick = (e: MouseEvent) => {
      const t = e.target as Node;
      if (popoverRef.current?.contains(t)) return;
      if (chipRef.current?.contains(t)) return;
      setModelsOpen(false);
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') setModelsOpen(false);
    };
    document.addEventListener('mousedown', onDocClick);
    document.addEventListener('keydown', onKey);
    return () => {
      document.removeEventListener('mousedown', onDocClick);
      document.removeEventListener('keydown', onKey);
    };
  }, [modelsOpen]);

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

  const setBucket = (bucket: BucketKey) => {
    selectedBucket.value = bucket;
    onURLUpdate();
    onFilterChange();
  };

  const setProvider = (provider: ProviderFilter) => {
    selectedProvider.value = provider;
    onURLUpdate();
    onFilterChange();
  };

  const hasCodexData = rawData.value?.provider_breakdown?.some(p => p.provider === 'codex') ?? false;
  const activeGroups = SECTION_FILTER_GROUPS[activeDashboardTab.value] ?? Object.values(SECTION_FILTER_GROUPS).flat();
  const show = (group: FilterGroup) => activeGroups.includes(group);

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

  const toggleMobileFilters = () => {
    mobile_filters_expanded.value = !mobile_filters_expanded.value;
    onURLUpdate();
  };

  const selectedModelCount = selectedModels.value.size;
  const totalModels = sortedModels.length;
  const allSelected = selectedModelCount === totalModels;
  const modelChipLabel = totalModels === 0
    ? 'Models'
    : allSelected
      ? `Models · all ${totalModels}`
      : `Models · ${selectedModelCount}/${totalModels}`;
  const providerSummary = hasCodexData ? PROVIDER_LABEL[selectedProvider.value] : null;
  const filterSummary = [
    RANGE_LABEL[selectedRange.value],
    BUCKET_LABEL[selectedBucket.value],
    providerSummary,
    allSelected ? `All ${totalModels} models` : `${selectedModelCount}/${totalModels} models`,
    projectSearchQuery.value ? `Project: ${projectSearchQuery.value}` : null,
  ].filter(Boolean).join(' · ');

  return (
    <div
      id="filter-bar"
      role="toolbar"
      aria-label="Filters"
      class={`filter-dock${mobile_filters_expanded.value ? ' expanded' : ' collapsed'}`}
    >
      <div class="mobile-filter-header">
        <div class="mobile-filter-summary" aria-live="polite">
          <span class="mobile-filter-summary-label">Filters</span>
          <span class="mobile-filter-summary-text">{filterSummary}</span>
        </div>
        <button
          class="mobile-filter-toggle"
          type="button"
          aria-expanded={mobile_filters_expanded.value}
          aria-controls="filter-sections"
          onClick={toggleMobileFilters}
        >
          {mobile_filters_expanded.value ? 'Hide' : 'Show'}
        </button>
      </div>

      <div id="filter-sections" class="filter-sections">
        {show('range') && (
          <div class="filter-group">
            <span class="filter-group__label">Range</span>
            <div class="segmented" role="group" aria-label="Date range">
              {RANGES.map(range => (
                <button
                  key={range}
                  class={`segmented__item${selectedRange.value === range ? ' is-active' : ''}`}
                  type="button"
                  data-range={range}
                  onClick={() => setRange(range)}
                >
                  {RANGE_LABEL[range]}
                </button>
              ))}
            </div>
          </div>
        )}

        {show('bucket') && (
          <div class="filter-group">
            <span class="filter-group__label">Bucket</span>
            <div class="segmented" role="group" aria-label="Chart bucket">
              {BUCKETS.map(bucket => (
                <button
                  key={bucket}
                  class={`segmented__item${selectedBucket.value === bucket ? ' is-active' : ''}`}
                  type="button"
                  data-bucket={bucket}
                  onClick={() => setBucket(bucket)}
                >
                  {BUCKET_LABEL[bucket]}
                </button>
              ))}
            </div>
          </div>
        )}

        {show('provider') && hasCodexData && (
          <div class="filter-group">
            <span class="filter-group__label">Provider</span>
            <div class="segmented" role="group" aria-label="Provider">
              {PROVIDERS.map(provider => (
                <button
                  key={provider}
                  class={`segmented__item${selectedProvider.value === provider ? ' is-active' : ''}`}
                  type="button"
                  data-provider={provider}
                  onClick={() => setProvider(provider)}
                >
                  {PROVIDER_LABEL[provider]}
                </button>
              ))}
            </div>
          </div>
        )}

        {show('models') && <div class="filter-group filter-group--chip">
          <button
            ref={chipRef}
            type="button"
            class={`filter-chip${modelsOpen ? ' is-open' : ''}`}
            aria-expanded={modelsOpen}
            aria-controls="filter-models-popover"
            onClick={() => setModelsOpen(o => !o)}
          >
            {modelChipLabel}
            <span class="filter-chip__caret" aria-hidden="true">▾</span>
          </button>
          {modelsOpen && (
            <div
              ref={popoverRef}
              id="filter-models-popover"
              class="filter-popover"
              role="dialog"
              aria-label="Select models"
            >
              <div class="filter-popover__header">
                <span>{selectedModelCount}/{totalModels} selected</span>
                <div class="filter-popover__actions">
                  <button class="filter-link" type="button" onClick={selectAll}>All</button>
                  <button class="filter-link" type="button" onClick={clearAll}>None</button>
                </div>
              </div>
              <div class="filter-popover__body" role="group" aria-label="Model filters">
                {sortedModels.map(model => {
                  const checked = selectedModels.value.has(model);
                  return (
                    <label
                      key={model}
                      class={`filter-popover__row${checked ? ' is-checked' : ''}`}
                      data-model={model}
                    >
                      <input
                        type="checkbox"
                        value={model}
                        checked={checked}
                        onChange={(e) =>
                          toggleModel(model, (e.currentTarget as HTMLInputElement).checked)
                        }
                        aria-label={model}
                      />
                      <span class="filter-popover__row-text">{shortModelName(model)}</span>
                    </label>
                  );
                })}
              </div>
            </div>
          )}
        </div>}

        {show('project-search') && (
          <div class="filter-group filter-group--search">
            <label for="project-search" class="filter-group__label">Project</label>
            <input
              type="text"
              id="project-search"
              name="project-search"
              placeholder="Search projects…"
              aria-label="Filter by project name"
              autoComplete="off"
              spellcheck={false}
              enterKeyHint="search"
              value={projectSearchQuery.value}
              onInput={onSearchInput}
              class="project-search-input"
            />
            {projectSearchQuery.value && (
              <button class="filter-link" id="project-clear-btn" type="button" onClick={clearSearch}>
                Clear
              </button>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
