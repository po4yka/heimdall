import { useMemo } from 'preact/hooks';
import {
  type CellContext,
  type ColumnDef,
  type PaginationState,
  type SortingState,
  type VisibilityState,
} from '@tanstack/table-core';
import { fmt, anyHasCredits, fmtLabel } from '../../lib/format';
import {
  lastFilteredSessions,
  SESSIONS_PAGE_SIZE,
  sessionsTablePagination,
  sessionsTableColumnVisibility,
  syncDashboardUrl,
} from '../../state/store';
import type { SessionRow } from '../../state/types';
import { DataTable } from './DataTable';
import { renderCostCell, renderCreditsCell, renderNumberCell, renderTagCell } from './cells';

const defaultSort: SortingState = [{ id: 'last', desc: true }];
const primaryOverflowStyle = {
  display: 'block',
  minWidth: 0,
  maxWidth: 'clamp(12rem, 24vw, 20rem)',
  overflow: 'hidden',
  textOverflow: 'ellipsis',
  whiteSpace: 'nowrap',
};
const secondaryOverflowStyle = {
  ...primaryOverflowStyle,
  marginTop: '2px',
  fontSize: 'var(--font-size-tertiary)',
  fontFamily: 'var(--font-mono)',
};
const projectOverflowStyle = {
  ...primaryOverflowStyle,
  maxWidth: 'clamp(10rem, 20vw, 18rem)',
};

function useSessionColumns(
  showCredits: boolean,
  onSelectProject?: ((row: SessionRow) => void) | undefined,
  onSelectModel?: ((model: string) => void) | undefined,
): ColumnDef<SessionRow, unknown>[] {
  return useMemo(
    () => [
      {
        id: 'session',
        accessorKey: 'session_id',
        header: 'Session',
        enableSorting: false,
        cell: (info: CellContext<SessionRow, unknown>) => {
          const row = info.row.original as SessionRow;
          const title = row.title?.trim();
          const sessionId = String(info.getValue());
          const tooltip = title ? `${title}\n${sessionId}` : sessionId;
          return (
            <div style={{ minWidth: 0, maxWidth: 'clamp(14rem, 28vw, 24rem)' }} title={tooltip}>
              <span class="muted" style={{ ...primaryOverflowStyle, fontFamily: 'var(--font-mono)' }}>
                {title || sessionId}
              </span>
              {title && (
                <span class="muted" style={secondaryOverflowStyle}>
                  {sessionId}
                </span>
              )}
            </div>
          );
        },
      },
      {
        id: 'project',
        accessorKey: 'project',
        header: 'Project',
        enableSorting: false,
        cell: (info: CellContext<SessionRow, unknown>) => {
          const row = info.row.original as SessionRow;
          const label = row.display_name || row.project;
          const showProjectPath = label !== row.project;
          const tooltip = showProjectPath ? `${label}\n${row.project}` : row.project;
          const content = (
            <>
              <span style={projectOverflowStyle}>{label}</span>
              {showProjectPath && (
                <span class="muted" style={secondaryOverflowStyle}>
                  {row.project}
                </span>
              )}
            </>
          );
          return (
            <div style={{ minWidth: 0, maxWidth: 'clamp(12rem, 24vw, 22rem)' }} title={tooltip}>
              {onSelectProject ? (
                <button type="button" class="table-action-btn table-action-btn--stack" onClick={() => onSelectProject(row)}>
                  {content}
                </button>
              ) : content}
            </div>
          );
        },
      },
      {
        id: 'provider',
        accessorKey: 'provider',
        header: 'Provider',
        enableSorting: false,
        cell: (info: CellContext<SessionRow, unknown>) => (
          <span class="model-tag">{String(info.getValue()).toUpperCase()}</span>
        ),
      },
      {
        id: 'last',
        accessorKey: 'last',
        header: 'Last Active',
        cell: (info: CellContext<SessionRow, unknown>) => (
          <span class="muted">{String(info.getValue() ?? '')}</span>
        ),
      },
      {
        id: 'duration_min',
        accessorKey: 'duration_min',
        header: 'Duration',
        cell: (info: CellContext<SessionRow, unknown>) => (
          <span class="muted">{Number(info.getValue() ?? 0)}m</span>
        ),
      },
      {
        id: 'model',
        accessorKey: 'model',
        header: 'Model',
        enableSorting: false,
        cell: (info: CellContext<SessionRow, unknown>) => {
          const model = String(info.getValue());
          return renderTagCell(model, onSelectModel ? () => onSelectModel(model) : undefined);
        },
      },
      {
        id: 'turns',
        accessorKey: 'turns',
        header: 'Turns',
        cell: (info: CellContext<SessionRow, unknown>) => {
          const row = info.row.original as SessionRow;
          return (
            <span class="num">
              {fmt(Number(info.getValue() ?? 0))}
              {row.subagent_count > 0 && (
                <span class="muted" style={{ fontSize: '10px' }}>
                  {' '}
                  ({row.subagent_count} agents)
                </span>
              )}
            </span>
          );
        },
      },
      {
        id: 'input',
        accessorKey: 'input',
        header: 'Input',
        cell: (info: CellContext<SessionRow, unknown>) =>
          renderNumberCell(Number(info.getValue() ?? 0), fmt),
      },
      {
        id: 'output',
        accessorKey: 'output',
        header: 'Output',
        cell: (info: CellContext<SessionRow, unknown>) =>
          renderNumberCell(Number(info.getValue() ?? 0), fmt),
      },
      {
        id: 'cost',
        accessorKey: 'cost',
        header: 'Est. Cost',
        cell: (info: CellContext<SessionRow, unknown>) => {
          const row = info.row.original as SessionRow;
          return renderCostCell(Number(info.getValue() ?? 0), row.is_billable);
        },
      },
      ...(showCredits ? [{
        id: 'credits',
        accessorFn: (row: SessionRow) => row.credits ?? null,
        header: 'Credits',
        sortUndefined: 'last' as const,
        cell: (info: CellContext<SessionRow, unknown>) => {
          const v = info.getValue() as number | null;
          return renderCreditsCell(v);
        },
      }] : []),
      {
        id: 'cost_meta',
        accessorKey: 'cost_confidence',
        header: 'Cost Meta',
        enableSorting: false,
        cell: (info: CellContext<SessionRow, unknown>) => {
          const row = info.row.original as SessionRow;
          const pricing = row.pricing_version || 'n/a';
          const shortPricing = pricing.includes('@') ? pricing.split('@')[0] : pricing;
          return (
            <div class="muted" style={{ fontSize: '10px', lineHeight: '1.4' }}>
              <div style={{ whiteSpace: 'nowrap' }}>
                {fmtLabel(row.cost_confidence || 'low')} / {fmtLabel(row.billing_mode || 'estimated_local')}
              </div>
              <div title={pricing} style={{ whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis', maxWidth: '140px' }}>
                {shortPricing}
              </div>
            </div>
          );
        },
      },
      {
        id: 'subagent_delta_cost',
        accessorFn: (row: SessionRow) => {
          const ag = row.agent_sessions_cost_nanos ?? 0;
          const tc = row.subagent_turns_cost_nanos ?? 0;
          if (ag === 0 && tc === 0) return null;
          return (ag - tc) / 1_000_000_000;
        },
        header: 'Subagent Δ',
        sortUndefined: 'last' as const,
        cell: (info: CellContext<SessionRow, unknown>) => {
          const v = info.getValue() as number | null;
          if (v == null) return <span class="muted">--</span>;
          const drift = Math.abs(v) >= 0.01;
          const sign = v >= 0 ? '+' : '';
          return (
            <span
              class="num"
              style={drift ? { color: 'var(--accent)' } : undefined}
              title="agent_sessions cost minus turns(is_subagent=1) cost"
            >
              {sign}${v.toFixed(4)}
            </span>
          );
        },
      },
      {
        id: 'cache_hit_ratio',
        accessorKey: 'cache_hit_ratio',
        header: 'Cache %',
        cell: (info: CellContext<SessionRow, unknown>) => {
          const v = info.getValue() as number | null | undefined;
          if (v == null || !Number.isFinite(v)) return <span class="num">--</span>;
          return <span class="num">{(v * 100).toFixed(0)}%</span>;
        },
      },
      {
        id: 'tokens_per_min',
        accessorKey: 'tokens_per_min',
        header: 'Tok/min',
        cell: (info: CellContext<SessionRow, unknown>) => {
          const v = info.getValue() as number;
          return <span class="num">{v > 0 ? fmt(Math.round(v)) : '--'}</span>;
        },
      },
    ],
    [showCredits, onSelectProject, onSelectModel]
  );
}

export function SessionsTable({
  onExportCSV,
  onSelectProject,
  onSelectModel,
}: {
  onExportCSV: () => void;
  onSelectProject?: (row: SessionRow) => void;
  onSelectModel?: (model: string) => void;
}) {
  const data = lastFilteredSessions.value;
  const showCredits = anyHasCredits(data);
  const columns = useSessionColumns(showCredits, onSelectProject, onSelectModel);
  const pagination = sessionsTablePagination.value;
  const columnVisibility = sessionsTableColumnVisibility.value;

  const handlePaginationChange = (nextPagination: PaginationState) => {
    sessionsTablePagination.value = nextPagination;
    syncDashboardUrl();
  };

  const handleColumnVisibilityChange = (nextColumnVisibility: VisibilityState) => {
    sessionsTableColumnVisibility.value = nextColumnVisibility;
    syncDashboardUrl();
  };

  return (
    <DataTable
      columns={columns}
      data={data}
      title="Recent sessions"
      sectionKey="sessions-mount"
      exportFn={onExportCSV}
      pageSize={SESSIONS_PAGE_SIZE}
      defaultSort={defaultSort}
      enableColumnVisibility
      paginationState={pagination}
      onPaginationChange={handlePaginationChange}
      columnVisibilityState={columnVisibility}
      onColumnVisibilityChange={handleColumnVisibilityChange}
    />
  );
}
