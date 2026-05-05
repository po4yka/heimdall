import { type ColumnDef } from '@tanstack/table-core';
import { fmt, fmtCostBig, fmtRelativeTime, esc } from '../../lib/format';
import type { SpawnBatch } from '../../state/types';
import { DataTable } from '../tables/DataTable';

const columns: ColumnDef<SpawnBatch, unknown>[] = [
  {
    accessorKey: 'spawned_at',
    header: 'SPAWNED',
    cell: ({ getValue }) => (
      <span class="num" title={String(getValue() ?? '')}>
        {fmtRelativeTime(String(getValue() ?? ''))}
      </span>
    ),
  },
  {
    accessorKey: 'project',
    header: 'PROJECT',
    cell: ({ getValue }) => <span>{esc(String(getValue() ?? ''))}</span>,
  },
  {
    accessorKey: 'size',
    header: 'SIZE',
    cell: ({ getValue }) => (
      <span class="num">{Number(getValue() ?? 0)}</span>
    ),
  },
  {
    accessorKey: 'roles',
    header: 'ROLES',
    cell: ({ getValue }) => {
      const roles = getValue() as string[];
      const sorted = [...roles].sort();
      const joined = sorted.join(', ');
      const truncated = joined.length > 60 ? joined.slice(0, 60) + '…' : joined;
      return <span title={joined}>{esc(truncated)}</span>;
    },
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
];

export function AgentSpawnBatches({ data }: { data: SpawnBatch[] }) {
  if (!data.length) return null;
  return (
    <DataTable
      columns={columns}
      data={data}
      title="Parallel spawn batches"
      sectionKey="agent-spawn-batches"
      defaultSort={[{ id: 'spawned_at', desc: true }]}
    />
  );
}
