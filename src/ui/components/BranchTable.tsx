import { type ColumnDef } from '@tanstack/table-core';
import { fmt, fmtCost } from '../lib/format';
import type { BranchSummary } from '../state/types';
import { InlineRankBar } from '../InlineRankBar';
import { DataTable } from './DataTable';

function makeColumns(data: BranchSummary[]): ColumnDef<BranchSummary, unknown>[] {
  const maxSessions = data.reduce((m, r) => Math.max(m, r.sessions), 0);
  return [
    { accessorKey: 'provider', header: 'Provider',
      cell: ({ getValue }) => <span class="model-tag">{String(getValue()).toUpperCase()}</span> },
    { accessorKey: 'branch', header: 'Branch',
      cell: ({ getValue }) => <span class="model-tag">{String(getValue())}</span> },
    { accessorKey: 'sessions', header: 'Sessions',
      cell: ({ getValue }) => (
        <InlineRankBar
          value={getValue() as number}
          max={maxSessions}
          label={String(getValue())}
        />
      ) },
    { accessorKey: 'turns', header: 'Turns',
      cell: ({ getValue }) => <span class="num">{fmt(getValue() as number)}</span> },
    { accessorKey: 'input', header: 'Input',
      cell: ({ getValue }) => <span class="num">{fmt(getValue() as number)}</span> },
    { accessorKey: 'output', header: 'Output',
      cell: ({ getValue }) => <span class="num">{fmt(getValue() as number)}</span> },
    { accessorKey: 'cost', header: 'Est. Cost',
      cell: ({ getValue }) => <span class="cost">{fmtCost(getValue() as number)}</span> },
  ];
}

export function BranchTable({ data }: { data: BranchSummary[] }) {
  if (!data.length) return null;
  return <DataTable columns={makeColumns(data)} data={data} title="Usage by Git Branch" sectionKey="branch-summary" />;
}
