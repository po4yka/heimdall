import { type ColumnDef } from '@tanstack/table-core';
import { fmt } from '../lib/format';
import type { EntrypointSummary } from '../state/types';
import { DataTable } from './DataTable';

const columns: ColumnDef<EntrypointSummary, unknown>[] = [
  { accessorKey: 'provider', header: 'Provider',
    cell: ({ getValue }) => <span class="model-tag">{String(getValue()).toUpperCase()}</span> },
  { accessorKey: 'entrypoint', header: 'Entrypoint',
    cell: ({ getValue }) => <span class="model-tag">{String(getValue())}</span> },
  { accessorKey: 'sessions', header: 'Sessions',
    cell: ({ getValue }) => <span class="num">{Number(getValue() ?? 0)}</span> },
  { accessorKey: 'turns', header: 'Turns',
    cell: ({ getValue }) => <span class="num">{fmt(getValue() as number)}</span> },
  { accessorKey: 'input', header: 'Input',
    cell: ({ getValue }) => <span class="num">{fmt(getValue() as number)}</span> },
  { accessorKey: 'output', header: 'Output',
    cell: ({ getValue }) => <span class="num">{fmt(getValue() as number)}</span> },
];

export function EntrypointTable({ data }: { data: EntrypointSummary[] }) {
  if (!data.length) return null;
  return <DataTable columns={columns} data={data} title="Usage by Entrypoint" sectionKey="entrypoint-breakdown" />;
}
