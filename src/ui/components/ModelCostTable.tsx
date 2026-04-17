import { useMemo } from 'preact/hooks';
import { type ColumnDef, type SortingState } from '@tanstack/table-core';
import { fmt, fmtCost } from '../lib/format';
import type { ModelAgg } from '../state/types';
import { DataTable } from './DataTable';
import { SegmentedProgressBar } from './SegmentedProgressBar';

const defaultSort: SortingState = [{ id: 'cost', desc: true }];

function useModelColumns(totalCost: number): ColumnDef<ModelAgg, any>[] {
  return useMemo(
    () => [
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
        cell: (info: any) => <span class="num">{fmt(info.getValue())}</span>,
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
        id: 'cache_read',
        accessorKey: 'cache_read',
        header: 'Cached Input',
        cell: (info: any) => <span class="num">{fmt(info.getValue())}</span>,
      },
      {
        id: 'cache_creation',
        accessorKey: 'cache_creation',
        header: 'Cache Creation',
        cell: (info: any) => <span class="num">{fmt(info.getValue())}</span>,
      },
      {
        id: 'cost',
        accessorKey: 'cost',
        header: 'Est. Cost',
        cell: (info: any) => {
          const row = info.row.original as ModelAgg;
          return row.is_billable ? (
            <span class="cost">{fmtCost(info.getValue())}</span>
          ) : (
            <span class="cost-na">n/a</span>
          );
        },
      },
      {
        id: 'share',
        accessorFn: (row: ModelAgg) => row.cost,
        header: 'Share',
        enableSorting: false,
        cell: (info: any) => {
          const row = info.row.original as ModelAgg;
          if (!row.is_billable || totalCost <= 0) {
            return <span class="cost-na">&mdash;</span>;
          }
          const pct = (row.cost / totalCost) * 100;
          return (
            <div style={{ minWidth: '120px', display: 'flex', alignItems: 'center', gap: '8px' }}>
              <div style={{ flex: 1 }}>
                <SegmentedProgressBar
                  value={row.cost}
                  max={totalCost}
                  segments={12}
                  size="compact"
                  status="neutral"
                  aria-label={`${row.model} cost share`}
                />
              </div>
              <span class="num" style={{ fontSize: '11px', color: 'var(--text-secondary)', minWidth: '36px', textAlign: 'right' }}>
                {pct.toFixed(0)}%
              </span>
            </div>
          );
        },
      },
    ],
    [totalCost]
  );
}

export function ModelCostTable({ byModel }: { byModel: ModelAgg[] }) {
  const totalCost = useMemo(
    () => byModel.reduce((s, m) => (m.is_billable ? s + m.cost : s), 0),
    [byModel]
  );
  const columns = useModelColumns(totalCost);

  return (
    <DataTable
      columns={columns}
      data={byModel}
      title="Cost by Model"
      defaultSort={defaultSort}
      costRows
    />
  );
}
