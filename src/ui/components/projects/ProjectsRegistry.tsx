import { useEffect, useMemo, useState } from 'preact/hooks';
import { useSignalEffect } from '@preact/signals';
import {
  type CellContext,
  type ColumnDef,
  type SortingState,
} from '@tanstack/table-core';
import { fmt, fmtCost, fmtRelativeTime } from '../../lib/format';
import { setStatus, clearStatus } from '../../lib/status';
import {
  fetchProjectsRegistry,
  patchProject,
} from '../../lib/projects';
import {
  projectsRegistry,
  setProjectHash,
} from '../../state/store';
import type { ProjectRegistryRow } from '../../state/dashboard-types';
import { DataTable } from '../tables/DataTable';
import { InlineStatus } from '../InlineStatus';
import { PinStar } from './PinStar';

interface ProjectsRegistryProps {
  /** Called after a successful PATCH so the parent can refresh dashboard data. */
  onReload?: () => void;
}

const defaultSort: SortingState = [
  { id: 'pinned', desc: true },
  { id: 'last_active', desc: true },
];

async function copyText(text: string, label: string) {
  try {
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(text);
      setStatus('project-registry', 'success', `[COPIED ${label}]`, 1500);
    }
  } catch {
    setStatus('project-registry', 'error', `[COPY FAILED]`, 2000);
  }
}

export function ProjectsRegistry({ onReload }: ProjectsRegistryProps) {
  const [rows, setRows] = useState<ProjectRegistryRow[]>(projectsRegistry.value);
  const [loading, setLoading] = useState(rows.length === 0);
  const [query, setQuery] = useState('');
  // Pending edits keyed by uuid so each row's input is locally controlled.
  const [labelEdits, setLabelEdits] = useState<Record<string, string>>({});

  async function load() {
    setLoading(true);
    try {
      const fresh = await fetchProjectsRegistry();
      projectsRegistry.value = fresh;
      setRows(fresh);
    } catch (err) {
      setStatus(
        'project-registry',
        'error',
        `[ERROR: ${err instanceof Error ? err.message : String(err)}]`,
        4000,
      );
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Keep local rows in sync with the global signal (other surfaces may refresh it).
  useSignalEffect(() => {
    setRows(projectsRegistry.value);
  });

  async function handleLabelSave(row: ProjectRegistryRow, raw: string) {
    const trimmed = raw.trim();
    const next = trimmed.length > 0 ? trimmed : null;
    // Skip no-op edits.
    const current = row.custom_label ?? '';
    if ((next ?? '') === current) return;
    clearStatus('project-registry');
    try {
      await patchProject(row.project_uuid, { label: next });
      setStatus('project-registry', 'success', '[SAVED]', 1500);
      await load();
      onReload?.();
    } catch (err) {
      setStatus(
        'project-registry',
        'error',
        `[ERROR: ${err instanceof Error ? err.message : String(err)}]`,
        3000,
      );
    }
  }

  async function handleClearLabel(row: ProjectRegistryRow) {
    if ((row.custom_label ?? null) === null) return;
    clearStatus('project-registry');
    try {
      await patchProject(row.project_uuid, { label: null });
      setLabelEdits(prev => {
        const next = { ...prev };
        delete next[row.project_uuid];
        return next;
      });
      setStatus('project-registry', 'success', '[CLEARED]', 1500);
      await load();
      onReload?.();
    } catch (err) {
      setStatus(
        'project-registry',
        'error',
        `[ERROR: ${err instanceof Error ? err.message : String(err)}]`,
        3000,
      );
    }
  }

  function openProject(row: ProjectRegistryRow) {
    setProjectHash(row.project_uuid);
    // Notify the dashboard runtime via hashchange so charts narrow.
    window.dispatchEvent(new HashChangeEvent('hashchange'));
  }

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return rows;
    return rows.filter(r => {
      return (
        r.slug.toLowerCase().includes(q) ||
        r.raw_name.toLowerCase().includes(q) ||
        (r.custom_label ?? '').toLowerCase().includes(q) ||
        r.project_uuid.toLowerCase().includes(q) ||
        r.display_name.toLowerCase().includes(q)
      );
    });
  }, [rows, query]);

  const columns: ColumnDef<ProjectRegistryRow, unknown>[] = useMemo(
    () => [
      {
        id: 'pinned',
        accessorFn: (row: ProjectRegistryRow) => (row.pinned ? 1 : 0),
        header: 'Pin',
        sortingFn: (a, b) => (a.original.pinned === b.original.pinned ? 0 : a.original.pinned ? -1 : 1),
        cell: (info: CellContext<ProjectRegistryRow, unknown>) => {
          const row = info.row.original;
          return (
            <PinStar
              projectUuid={row.project_uuid}
              pinned={row.pinned}
              label={row.display_name || row.slug}
              onChange={() => {
                void load();
                onReload?.();
              }}
            />
          );
        },
      },
      {
        id: 'label',
        accessorFn: (row: ProjectRegistryRow) => row.custom_label ?? row.display_name ?? row.slug,
        header: 'Label',
        cell: (info: CellContext<ProjectRegistryRow, unknown>) => {
          const row = info.row.original;
          const editValue = labelEdits[row.project_uuid] ?? (row.custom_label ?? '');
          return (
            <input
              type="text"
              class="agent-registry-input"
              value={editValue}
              placeholder={row.raw_name || row.slug}
              onInput={(e) => {
                const v = (e.target as HTMLInputElement).value;
                setLabelEdits(prev => ({ ...prev, [row.project_uuid]: v }));
              }}
              onBlur={(e) => {
                const v = (e.target as HTMLInputElement).value;
                void handleLabelSave(row, v);
              }}
              onKeyDown={(e) => {
                if (e.key === 'Enter') {
                  e.preventDefault();
                  (e.target as HTMLInputElement).blur();
                }
              }}
            />
          );
        },
      },
      {
        id: 'slug',
        accessorKey: 'slug',
        header: 'Slug',
        cell: (info: CellContext<ProjectRegistryRow, unknown>) => {
          const row = info.row.original;
          return (
            <button
              type="button"
              class="table-action-btn"
              title="Click to copy slug"
              style={{ fontFamily: 'var(--font-mono)', color: 'var(--text-secondary)' }}
              onClick={() => void copyText(row.slug, 'SLUG')}
            >
              {row.slug}
              {row.is_cowork && (
                <span style={{ marginLeft: 6, opacity: 0.6 }} title="Cowork session">
                  [cowork]
                </span>
              )}
            </button>
          );
        },
      },
      {
        id: 'uuid',
        accessorKey: 'project_uuid',
        header: 'UUID',
        cell: (info: CellContext<ProjectRegistryRow, unknown>) => {
          const row = info.row.original;
          const link = `#/project/${row.project_uuid}`;
          return (
            <button
              type="button"
              class="table-action-btn"
              title="Click to copy deep link"
              style={{ fontFamily: 'var(--font-mono)', color: 'var(--text-secondary)', fontSize: '0.85em' }}
              onClick={() => void copyText(link, 'LINK')}
            >
              {row.project_uuid.slice(0, 8)}…
            </button>
          );
        },
      },
      {
        id: 'sessions',
        accessorKey: 'sessions',
        header: 'Sessions',
        cell: (info: CellContext<ProjectRegistryRow, unknown>) => (
          <span class="num">{String(Number(info.getValue() ?? 0))}</span>
        ),
      },
      {
        id: 'calls',
        accessorKey: 'calls',
        header: 'Calls',
        cell: (info: CellContext<ProjectRegistryRow, unknown>) => (
          <span class="num">{fmt(Number(info.getValue() ?? 0))}</span>
        ),
      },
      {
        id: 'cost',
        accessorKey: 'cost',
        header: 'Est. Cost',
        cell: (info: CellContext<ProjectRegistryRow, unknown>) => (
          <span class="cost">{fmtCost(Number(info.getValue() ?? 0))}</span>
        ),
      },
      {
        id: 'last_active',
        accessorFn: (row: ProjectRegistryRow) => row.last_active ?? '',
        header: 'Last active',
        cell: (info: CellContext<ProjectRegistryRow, unknown>) => {
          const row = info.row.original;
          return (
            <span style={{ color: 'var(--text-secondary)' }}>
              {fmtRelativeTime(row.last_active)}
            </span>
          );
        },
      },
      {
        id: 'actions',
        header: 'Actions',
        enableSorting: false,
        cell: (info: CellContext<ProjectRegistryRow, unknown>) => {
          const row = info.row.original;
          return (
            <div style={{ display: 'flex', gap: '6px' }}>
              <button
                type="button"
                class="filter-btn"
                style={{ fontSize: '10px', padding: '2px 8px' }}
                title="Open in dashboard with this project pre-filtered"
                onClick={() => openProject(row)}
              >
                [Open]
              </button>
              {row.custom_label != null && (
                <button
                  type="button"
                  class="filter-btn"
                  style={{ fontSize: '10px', padding: '2px 8px' }}
                  title="Clear custom label"
                  onClick={() => void handleClearLabel(row)}
                >
                  [Clear label]
                </button>
              )}
            </div>
          );
        },
      },
    ],
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [labelEdits, onReload],
  );

  return (
    <div class="table-card">
      <div class="section-header" style={{ padding: '20px 20px 12px' }}>
        <h2 class="section-title" style={{ margin: 0 }}>
          Projects
        </h2>
        <div class="section-actions" style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
          <input
            type="search"
            placeholder="Search slug, label, UUID…"
            value={query}
            onInput={(e) => setQuery((e.target as HTMLInputElement).value)}
            class="agent-registry-input"
            style={{ minWidth: 220 }}
          />
          <button
            type="button"
            class="filter-btn"
            onClick={() => void load()}
            disabled={loading}
            title="Refresh registry"
          >
            {loading ? '[REFRESHING]' : 'Refresh'}
          </button>
          <InlineStatus placement="project-registry" inline />
        </div>
      </div>
      <div style={{ padding: '0 20px 20px' }}>
        <DataTable
          columns={columns}
          data={filtered}
          defaultSort={defaultSort}
        />
        {!loading && filtered.length === 0 && (
          <div class="empty-state" style={{ marginTop: 12 }}>
            {query ? 'No projects match the search.' : 'No projects detected yet.'}
          </div>
        )}
      </div>
    </div>
  );
}
