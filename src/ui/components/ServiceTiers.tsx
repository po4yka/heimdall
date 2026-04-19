import { type ColumnDef } from '@tanstack/table-core';
import { fmt } from '../lib/format';
import type { ServiceTierSummary } from '../state/types';
import { DataTable } from './DataTable';

const columns: ColumnDef<ServiceTierSummary, unknown>[] = [
  { accessorKey: 'provider', header: 'Provider',
    cell: ({ getValue }) => <span class="model-tag">{String(getValue()).toUpperCase()}</span> },
  { accessorKey: 'service_tier', header: 'Tier' },
  { accessorKey: 'inference_geo', header: 'Region' },
  { accessorKey: 'turns', header: 'Turns',
    cell: ({ getValue }) => <span class="num">{fmt(getValue() as number)}</span> },
];

export function ServiceTiersTable({ data }: { data: ServiceTierSummary[] }) {
  if (!data.length) return null;
  return <DataTable columns={columns} data={data} title="Service Tiers" sectionKey="service-tiers" />;
}
