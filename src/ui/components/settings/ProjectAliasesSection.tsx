import { useEffect, useRef, useState } from 'preact/hooks';
import { settingsDraft, projectsRegistry } from '../../state/store';
import { esc } from '../../lib/format';

type AliasEntry = { slug: string; display_name: string };

function patchAliases(entries: AliasEntry[]): void {
  const draft = settingsDraft.value;
  if (!draft) return;
  settingsDraft.value = {
    ...draft,
    project_aliases: { entries },
  };
}

// ── Autocomplete dropdown ────────────────────────────────────────────────────

interface AutocompleteProps {
  query: string;
  onSelect: (slug: string) => void;
  onClose: () => void;
}

function AutocompleteDropdown({ query, onSelect, onClose }: AutocompleteProps) {
  const ref = useRef<HTMLDivElement>(null);

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

  const allSlugs = projectsRegistry.value.map((r) => r.slug);
  const lower = query.toLowerCase();
  const matches = allSlugs
    .filter((s) => !lower || s.toLowerCase().includes(lower))
    .slice(0, 8);

  return (
    <div class="settings-aliases-autocomplete" ref={ref}>
      {matches.length === 0 ? (
        <div class="settings-aliases-autocomplete-empty">No matching projects</div>
      ) : (
        matches.map((slug) => (
          <button
            key={slug}
            type="button"
            class="settings-aliases-autocomplete-item"
            onMouseDown={(e) => {
              e.preventDefault(); // prevent blur before click
              onSelect(slug);
            }}
          >
            {esc(slug)}
          </button>
        ))
      )}
    </div>
  );
}

// ── Single alias row ─────────────────────────────────────────────────────────

interface AliasRowProps {
  entry: AliasEntry;
  index: number;
  isFocusTarget: boolean;
  isDuplicateSlug: boolean;
  onChange: (index: number, updated: AliasEntry) => void;
  onDelete: (index: number) => void;
}

function AliasRow({ entry, index, isFocusTarget, isDuplicateSlug, onChange, onDelete }: AliasRowProps) {
  const slugRef = useRef<HTMLInputElement>(null);
  const [showAc, setShowAc] = useState(false);

  // Auto-focus new rows.
  useEffect(() => {
    if (isFocusTarget && slugRef.current) {
      slugRef.current.focus();
      slugRef.current.scrollIntoView({ block: 'nearest' });
    }
  }, [isFocusTarget]);

  const isEmpty = !entry.slug.trim() || !entry.display_name.trim();
  const showHint = isEmpty || isDuplicateSlug;

  function handleSlugChange(val: string) {
    onChange(index, { ...entry, slug: val });
  }

  function handleDisplayNameChange(val: string) {
    onChange(index, { ...entry, display_name: val });
  }

  function handleAcSelect(slug: string) {
    onChange(index, { ...entry, slug });
    setShowAc(false);
  }

  return (
    <div class="settings-aliases-row-wrapper">
      <div class="settings-aliases-row">
        {/* Slug column */}
        <div class="settings-aliases-cell settings-aliases-cell--slug">
          <div class="settings-aliases-slug-wrap">
            <input
              ref={slugRef}
              type="text"
              class="settings-input num settings-aliases-input"
              value={entry.slug}
              placeholder="project-slug"
              onInput={(e) => handleSlugChange((e.target as HTMLInputElement).value)}
              onKeyDown={(e) => {
                if (e.key === 'Escape') setShowAc(false);
                if (e.key === '/' && !entry.slug) setShowAc(true);
              }}
              onFocus={() => { /* no auto-open */ }}
            />
            <button
              type="button"
              class="settings-aliases-ac-trigger"
              title="Pick from recent projects"
              onClick={() => setShowAc((v) => !v)}
              tabIndex={-1}
            >
              &#x21A7;
            </button>
          </div>
          {showAc && (
            <AutocompleteDropdown
              query={entry.slug}
              onSelect={handleAcSelect}
              onClose={() => setShowAc(false)}
            />
          )}
        </div>

        {/* Display name column */}
        <div class="settings-aliases-cell settings-aliases-cell--name">
          <input
            type="text"
            class="settings-input settings-aliases-input"
            value={entry.display_name}
            placeholder="Friendly name"
            onInput={(e) => handleDisplayNameChange((e.target as HTMLInputElement).value)}
          />
        </div>

        {/* Delete button */}
        <div class="settings-aliases-cell settings-aliases-cell--action">
          <button
            type="button"
            class="settings-aliases-delete-btn"
            aria-label={`Delete alias for ${esc(entry.slug)}`}
            onClick={() => onDelete(index)}
          >
            [X]
          </button>
        </div>
      </div>

      {showHint && (
        <div class="settings-aliases-validation-hint">
          {isDuplicateSlug
            ? 'Duplicate slug — only the first will save.'
            : 'Both fields are required.'}
        </div>
      )}
    </div>
  );
}

// ── Section ──────────────────────────────────────────────────────────────────

export function ProjectAliasesSection() {
  const draft = settingsDraft.value;
  if (!draft) return null;

  const entries = draft.project_aliases.entries;
  const [filter, setFilter] = useState('');
  const [focusIndex, setFocusIndex] = useState<number | null>(null);

  // Track which slugs appear more than once (by their first occurrence index).
  const slugCounts = new Map<string, number>();
  for (const e of entries) {
    const s = e.slug.trim().toLowerCase();
    if (s) slugCounts.set(s, (slugCounts.get(s) ?? 0) + 1);
  }
  const firstOccurrence = new Map<string, number>();
  for (let i = 0; i < entries.length; i++) {
    const s = entries[i]!.slug.trim().toLowerCase();
    if (s && !firstOccurrence.has(s)) firstOccurrence.set(s, i);
  }

  const hasEmptySlugRow = entries.some((e) => e.slug.trim() === '');

  function handleAdd() {
    if (hasEmptySlugRow) return;
    const newEntries = [...entries, { slug: '', display_name: '' }];
    patchAliases(newEntries);
    setFocusIndex(newEntries.length - 1);
  }

  function handleChange(index: number, updated: AliasEntry) {
    const newEntries = entries.map((e, i) => (i === index ? updated : e));
    patchAliases(newEntries);
  }

  function handleDelete(index: number) {
    patchAliases(entries.filter((_, i) => i !== index));
    setFocusIndex(null);
  }

  // Apply display filter (doesn't mutate state).
  const lowerFilter = filter.toLowerCase();
  const visibleIndices = entries
    .map((e, i) => ({ e, i }))
    .filter(
      ({ e }) =>
        !lowerFilter ||
        e.slug.toLowerCase().includes(lowerFilter) ||
        e.display_name.toLowerCase().includes(lowerFilter),
    );

  const count = entries.length;

  return (
    <div class="settings-section">
      {/* Header strip */}
      <div class="settings-aliases-header">
        <input
          type="text"
          class="settings-input settings-aliases-filter"
          placeholder="Filter aliases..."
          value={filter}
          onInput={(e) => setFilter((e.target as HTMLInputElement).value)}
        />
        <button
          type="button"
          class="settings-btn settings-btn--primary"
          disabled={hasEmptySlugRow}
          onClick={handleAdd}
          title={hasEmptySlugRow ? 'Fill in the empty row first' : undefined}
        >
          [+ Add alias]
        </button>
      </div>

      {/* Table */}
      <div class="settings-aliases-table">
        {/* Column headers */}
        <div class="settings-aliases-thead">
          <div class="settings-aliases-th settings-aliases-th--slug">SLUG</div>
          <div class="settings-aliases-th settings-aliases-th--name">DISPLAY NAME</div>
          <div class="settings-aliases-th settings-aliases-th--action"></div>
        </div>

        {/* Rows */}
        {entries.length === 0 ? (
          <div class="settings-aliases-empty">
            No project aliases configured yet. Click [+ Add alias] to create one.
          </div>
        ) : visibleIndices.length === 0 ? (
          <div class="settings-aliases-empty">
            No aliases match &ldquo;{esc(filter)}&rdquo;.
          </div>
        ) : (
          visibleIndices.map(({ e, i }) => {
            const slugKey = e.slug.trim().toLowerCase();
            const isDup =
              slugKey !== '' &&
              (slugCounts.get(slugKey) ?? 0) > 1 &&
              firstOccurrence.get(slugKey) !== i;
            return (
              <AliasRow
                key={i}
                entry={e}
                index={i}
                isFocusTarget={focusIndex === i}
                isDuplicateSlug={isDup}
                onChange={handleChange}
                onDelete={handleDelete}
              />
            );
          })
        )}
      </div>

      {/* Footer help */}
      <div class="settings-aliases-footer">
        <span class="settings-hint">
          Aliases override the slug shown in the dashboard. The raw slug is always preserved in storage.
        </span>
        <span class="settings-aliases-counter">
          {count} {count === 1 ? 'alias' : 'aliases'}
        </span>
      </div>
    </div>
  );
}
