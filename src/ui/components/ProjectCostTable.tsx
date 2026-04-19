import { useMemo } from 'preact/hooks';
import { type ColumnDef, type SortingState } from '@tanstack/table-core';
import { fmt, fmtCost } from '../lib/format';
import type { ProjectAgg } from '../state/types';

import { DataTable } from './DataTable';

const defaultSort: SortingState = [{ id: 'cost', desc: true }];

function useProjectColumns(): ColumnDef<ProjectAgg, any>[] {
  return useMemo(
    () => [
      {
        id: 'project',
        accessorKey: 'project',
        header: 'Project',
        enableSorting: false,
        cell: (info: any) => {
          const row = info.row.original as ProjectAgg;
          const label = row.display_name || row.project;
          return <span title={row.project}>{label}</span>;
        },
      },
      {
        id: 'sessions',
        accessorKey: 'sessions',
        header: 'Sessions',
        cell: (info: any) => <span class="num">{info.getValue()}</span>,
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
        id: 'cost',
        accessorKey: 'cost',
        header: 'Est. Cost',
        cell: (info: any) => <span class="cost">{fmtCost(info.getValue())}</span>,
      },
    ],
    []
  );
}

export function ProjectCostTable({
  byProject,
  onExportCSV,
}: {
  byProject: ProjectAgg[];
  onExportCSV: () => void;
}) {
  const columns = useProjectColumns();

  return (
    <DataTable
      columns={columns}
      data={byProject}
      title="Cost by Project"
      exportFn={onExportCSV}
      defaultSort={defaultSort}
      costRows
    />
  );
}
