import { useMemo } from 'preact/hooks';
import {
  type CellContext,
  type ColumnDef,
  type SortingState,
} from '@tanstack/table-core';
import { fmt, anyHasCredits } from '../../lib/format';
import type { ProjectAgg } from '../../state/types';

import { DataTable } from './DataTable';
import { renderActionCell, renderCostCell, renderCreditsCell, renderNumberCell } from './cells';

const defaultSort: SortingState = [{ id: 'cost', desc: true }];

function useProjectColumns(
  showCredits: boolean,
  onSelectProject?: ((project: ProjectAgg) => void) | undefined,
): ColumnDef<ProjectAgg, unknown>[] {
  return useMemo(
    () => [
      {
        id: 'project',
        accessorKey: 'project',
        header: 'Project',
        enableSorting: false,
        cell: (info: CellContext<ProjectAgg, unknown>) => {
          const row = info.row.original as ProjectAgg;
          const label = row.display_name || row.project;
          return renderActionCell(label, row.project, onSelectProject ? () => onSelectProject(row) : undefined);
        },
      },
      {
        id: 'sessions',
        accessorKey: 'sessions',
        header: 'Sessions',
        cell: (info: CellContext<ProjectAgg, unknown>) =>
          renderNumberCell(Number(info.getValue() ?? 0), value => String(value)),
      },
      {
        id: 'turns',
        accessorKey: 'turns',
        header: 'Turns',
        cell: (info: CellContext<ProjectAgg, unknown>) =>
          renderNumberCell(Number(info.getValue() ?? 0), fmt),
      },
      {
        id: 'input',
        accessorKey: 'input',
        header: 'Input',
        cell: (info: CellContext<ProjectAgg, unknown>) =>
          renderNumberCell(Number(info.getValue() ?? 0), fmt),
      },
      {
        id: 'output',
        accessorKey: 'output',
        header: 'Output',
        cell: (info: CellContext<ProjectAgg, unknown>) =>
          renderNumberCell(Number(info.getValue() ?? 0), fmt),
      },
      {
        id: 'cost',
        accessorKey: 'cost',
        header: 'Est. Cost',
        cell: (info: CellContext<ProjectAgg, unknown>) =>
          renderCostCell(Number(info.getValue() ?? 0)),
      },
      ...(showCredits ? [{
        id: 'credits',
        accessorFn: (row: ProjectAgg) => row.credits ?? null,
        header: 'Credits',
        sortUndefined: 'last' as const,
        cell: (info: CellContext<ProjectAgg, unknown>) => {
          const v = info.getValue() as number | null;
          return renderCreditsCell(v);
        },
      }] : []),
    ],
    [showCredits, onSelectProject]
  );
}

export function ProjectCostTable({
  byProject,
  onExportCSV,
  onSelectProject,
}: {
  byProject: ProjectAgg[];
  onExportCSV: () => void;
  onSelectProject?: (project: ProjectAgg) => void;
}) {
  const showCredits = anyHasCredits(byProject);
  const columns = useProjectColumns(showCredits, onSelectProject);

  return (
    <DataTable
      columns={columns}
      data={byProject}
      title="Cost by Project"
      sectionKey="project-cost-mount"
      exportFn={onExportCSV}
      defaultSort={defaultSort}
      costRows
    />
  );
}
