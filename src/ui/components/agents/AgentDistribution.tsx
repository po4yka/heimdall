import { type ColumnDef } from '@tanstack/table-core';
import { fmt, fmtCostBig, fmtLabel, esc } from '../../lib/format';
import type { AgentRoleAggregate } from '../../state/types';
import { DataTable } from '../tables/DataTable';

const columns: ColumnDef<AgentRoleAggregate, unknown>[] = [
  {
    accessorKey: 'role',
    header: 'Role',
    cell: ({ row }) => {
      const agg = row.original;
      const display = agg.display_name ?? fmtLabel(agg.role);
      return <span title={agg.role}>{esc(display)}</span>;
    },
  },
  {
    accessorKey: 'sessions',
    header: 'Sessions',
    cell: ({ getValue }) => <span class="num">{fmt(Number(getValue() ?? 0))}</span>,
  },
  {
    accessorKey: 'total_tokens',
    header: 'Tokens',
    cell: ({ getValue }) => <span class="num">{fmt(getValue() as number)}</span>,
  },
  {
    accessorKey: 'cost_usd',
    header: 'Cost',
    cell: ({ getValue }) => <span class="num">{fmtCostBig(getValue() as number)}</span>,
  },
  {
    accessorKey: 'tool_uses',
    header: 'Tool uses',
    cell: ({ getValue }) => <span class="num">{fmt(Number(getValue() ?? 0))}</span>,
  },
];

export function AgentDistribution({ data }: { data: AgentRoleAggregate[] }) {
  if (!data.length) return null;
  return (
    <DataTable
      columns={columns}
      data={data}
      title="Role distribution"
      sectionKey="agent-distribution"
      defaultSort={[{ id: 'cost_usd', desc: true }]}
    />
  );
}
