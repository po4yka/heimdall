import { useMemo } from 'preact/hooks';
import { type ColumnDef, type SortingState } from '@tanstack/table-core';
import { fmt, fmtCost } from '../lib/format';
import type { ModelAgg } from '../state/types';
import { DataTable } from './DataTable';

const defaultSort: SortingState = [{ id: 'cost', desc: true }];

function useModelColumns(): ColumnDef<ModelAgg, any>[] {
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
        header: 'Cache Read',
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
    ],
    []
  );
}

export function ModelCostTable({ byModel }: { byModel: ModelAgg[] }) {
  const columns = useModelColumns();

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
