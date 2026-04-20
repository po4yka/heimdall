import { type ColumnDef } from '@tanstack/table-core';
import { fmt } from '../../lib/format';
import type { VersionSummary } from '../../state/types';
import { DataTable } from './DataTable';

const columns: ColumnDef<VersionSummary, unknown>[] = [
  { accessorKey: 'provider', header: 'Provider',
    cell: ({ getValue }) => <span class="model-tag">{String(getValue()).toUpperCase()}</span> },
  { accessorKey: 'version', header: 'Version',
    cell: ({ getValue }) => <span class="model-tag">{String(getValue())}</span> },
  { accessorKey: 'turns', header: 'Turns',
    cell: ({ getValue }) => <span class="num">{fmt(getValue() as number)}</span> },
  { accessorKey: 'sessions', header: 'Sessions',
    cell: ({ getValue }) => <span class="num">{Number(getValue() ?? 0)}</span> },
];

export function VersionTable({ data, title = 'CLI Versions' }: { data: VersionSummary[]; title?: string | null }) {
  if (!data.length) return null;
  if (title == null) {
    return <DataTable columns={columns} data={data} />;
  }
  return <DataTable columns={columns} data={data} title={title} />;
}
