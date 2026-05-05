import { type ColumnDef } from '@tanstack/table-core';
import { fmt, fmtCostBig, esc } from '../../lib/format';
import type { AgentRoleAggregate } from '../../state/types';
import { DataTable } from '../tables/DataTable';

const columns: ColumnDef<AgentRoleAggregate, unknown>[] = [
  {
    accessorKey: 'role',
    header: 'ROLE',
    cell: ({ row }) => {
      const agg = row.original;
      const display = agg.display_name ?? agg.role;
      return <span title={agg.role}>{esc(display)}</span>;
    },
  },
  {
    accessorKey: 'sessions',
    header: 'SESSIONS',
    cell: ({ getValue }) => <span class="num">{Number(getValue() ?? 0).toLocaleString()}</span>,
  },
  {
    accessorKey: 'total_tokens',
    header: 'TOKENS',
    cell: ({ getValue }) => <span class="num">{fmt(getValue() as number)}</span>,
  },
  {
    accessorKey: 'cost_usd',
    header: 'COST',
    cell: ({ getValue }) => <span class="num">{fmtCostBig(getValue() as number)}</span>,
  },
  {
    accessorKey: 'tool_uses',
    header: 'TOOL USES',
    cell: ({ getValue }) => <span class="num">{Number(getValue() ?? 0).toLocaleString()}</span>,
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
