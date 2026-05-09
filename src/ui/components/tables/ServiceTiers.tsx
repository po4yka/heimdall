import { type ColumnDef } from '@tanstack/table-core';
import { fmt, fmtLabel } from '../../lib/format';
import type { ServiceTierSummary } from '../../state/types';
import { DataTable } from './DataTable';

const columns: ColumnDef<ServiceTierSummary, unknown>[] = [
  { accessorKey: 'provider', header: 'Provider',
    cell: ({ getValue }) => <span class="model-tag">{fmtLabel(String(getValue()))}</span> },
  { accessorKey: 'service_tier', header: 'Tier',
    cell: ({ getValue }) => <span>{fmtLabel(getValue() as string)}</span> },
  { accessorKey: 'inference_geo', header: 'Region',
    cell: ({ getValue }) => <span>{fmtLabel(getValue() as string)}</span> },
  { accessorKey: 'turns', header: 'Turns',
    cell: ({ getValue }) => <span class="num">{fmt(getValue() as number)}</span> },
];

export function ServiceTiersTable({ data }: { data: ServiceTierSummary[] }) {
  if (!data.length) return null;
  return <DataTable columns={columns} data={data} title="Service Tiers" sectionKey="service-tiers" />;
}
