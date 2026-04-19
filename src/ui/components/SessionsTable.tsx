import { useMemo } from 'preact/hooks';
import { type ColumnDef, type SortingState } from '@tanstack/table-core';
import { fmt, fmtCost, anyHasCredits, fmtCredits } from '../lib/format';
import { lastFilteredSessions, SESSIONS_PAGE_SIZE } from '../state/store';
import type { SessionRow } from '../state/types';
import { DataTable } from './DataTable';

const defaultSort: SortingState = [{ id: 'last', desc: true }];

function useSessionColumns(showCredits: boolean): ColumnDef<SessionRow, any>[] {
  return useMemo(
    () => [
      {
        id: 'session',
        accessorKey: 'session_id',
        header: 'Session',
        enableSorting: false,
        cell: (info: any) => {
          const row = info.row.original as SessionRow;
          const title = row.title;
          return (
            <span class="muted" style={{ fontFamily: 'monospace' }} title={title || undefined}>
              {title || <>{info.getValue()}&hellip;</>}
            </span>
          );
        },
      },
      {
        id: 'project',
        accessorKey: 'project',
        header: 'Project',
        enableSorting: false,
        cell: (info: any) => {
          const row = info.row.original as SessionRow;
          const label = row.display_name || row.project;
          return <span title={row.project}>{label}</span>;
        },
      },
      {
        id: 'provider',
        accessorKey: 'provider',
        header: 'Provider',
        enableSorting: false,
        cell: (info: any) => (
          <span class="model-tag">{String(info.getValue()).toUpperCase()}</span>
        ),
      },
      {
        id: 'last',
        accessorKey: 'last',
        header: 'Last Active',
        cell: (info: any) => <span class="muted">{info.getValue()}</span>,
      },
      {
        id: 'duration_min',
        accessorKey: 'duration_min',
        header: 'Duration',
        cell: (info: any) => <span class="muted">{info.getValue()}m</span>,
      },
      {
        id: 'model',
        accessorKey: 'model',
        header: 'Model',
        enableSorting: false,
        cell: (info: any) => <span class="model-tag">{info.getValue()}</span>,
      },
      {
        id: 'turns',
        accessorKey: 'turns',
        header: 'Turns',
        cell: (info: any) => {
          const row = info.row.original as SessionRow;
          return (
            <span class="num">
              {fmt(info.getValue())}
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
        cell: (info: any) => <span class="num">{fmt(info.getValue())}</span>,
      },
      {
        id: 'output',
        accessorKey: 'output',
        header: 'Output',
        cell: (info: any) => <span class="num">{fmt(info.getValue())}</span>,
      },
      {
        id: 'cost',
        accessorKey: 'cost',
        header: 'Est. Cost',
        cell: (info: any) => {
          const row = info.row.original as SessionRow;
          return row.is_billable ? (
            <span class="cost">{fmtCost(info.getValue())}</span>
          ) : (
            <span class="cost-na">n/a</span>
          );
        },
      },
      ...(showCredits ? [{
        id: 'credits',
        accessorFn: (row: SessionRow) => row.credits ?? null,
        header: 'Credits',
        sortUndefined: 'last' as const,
        cell: (info: any) => {
          const v = info.getValue() as number | null;
          return <span class="num">{fmtCredits(v)}</span>;
        },
      }] : []),
      {
        id: 'cost_meta',
        accessorKey: 'cost_confidence',
        header: 'Cost Meta',
        enableSorting: false,
        cell: (info: any) => {
          const row = info.row.original as SessionRow;
          return (
            <div class="muted" style={{ fontSize: '10px', lineHeight: '1.35' }}>
              <div>{row.cost_confidence || 'low'} / {row.billing_mode || 'estimated_local'}</div>
              <div>{row.pricing_version || 'n/a'}</div>
            </div>
          );
        },
      },
      {
        id: 'cache_hit_ratio',
        accessorKey: 'cache_hit_ratio',
        header: 'Cache %',
        cell: (info: any) => {
          const v = info.getValue() as number | null | undefined;
          if (v == null || !Number.isFinite(v)) return <span class="num">--</span>;
          return <span class="num">{(v * 100).toFixed(0)}%</span>;
        },
      },
      {
        id: 'tokens_per_min',
        accessorKey: 'tokens_per_min',
        header: 'Tok/min',
        cell: (info: any) => {
          const v = info.getValue() as number;
          return <span class="num">{v > 0 ? fmt(Math.round(v)) : '--'}</span>;
        },
      },
    ],
    [showCredits]
  );
}

export function SessionsTable({ onExportCSV }: { onExportCSV: () => void }) {
  const data = lastFilteredSessions.value;
  const showCredits = anyHasCredits(data);
  const columns = useSessionColumns(showCredits);

  return (
    <DataTable
      columns={columns}
      data={data}
      title="Recent Sessions"
      exportFn={onExportCSV}
      pageSize={SESSIONS_PAGE_SIZE}
      defaultSort={defaultSort}
      enableColumnVisibility
    />
  );
}
