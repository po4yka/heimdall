import { type ColumnDef } from '@tanstack/table-core';
import { fmt, fmtCostBig, fmtRelativeTime, esc } from '../../lib/format';
import type { AgentSessionRow } from '../../state/types';
import { DataTable } from '../tables/DataTable';

function fmtDuration(seconds: number): string {
  if (seconds < 60) return `${Math.round(seconds)}s`;
  const m = Math.floor(seconds / 60);
  const s = Math.round(seconds % 60);
  return `${m}m ${s}s`;
}

function StopReasonBadge({ reason }: { reason: string | null }) {
  if (!reason) return <span class="num" style={{ color: 'var(--text-disabled)' }}>—</span>;
  let cls = 'agent-stop-reason-badge';
  if (reason === 'end_turn') cls += ' agent-stop-reason--success';
  else if (reason === 'max_tokens') cls += ' agent-stop-reason--warning';
  else cls += ' agent-stop-reason--error';
  return <span class={cls}>{esc(reason)}</span>;
}

const columns: ColumnDef<AgentSessionRow, unknown>[] = [
  {
    accessorKey: 'ts_start',
    header: 'STARTED',
    cell: ({ getValue }) => (
      <span class="num" title={String(getValue() ?? '')}>
        {fmtRelativeTime(String(getValue() ?? ''))}
      </span>
    ),
  },
  {
    accessorKey: 'role',
    header: 'ROLE',
    cell: ({ getValue }) => <span>{esc(String(getValue() ?? ''))}</span>,
  },
  {
    accessorKey: 'description',
    header: 'DESCRIPTION',
    cell: ({ getValue }) => {
      const raw = String(getValue() ?? '');
      const truncated = raw.length > 60 ? raw.slice(0, 60) + '…' : raw;
      return <span title={raw}>{esc(truncated)}</span>;
    },
  },
  {
    accessorKey: 'model',
    header: 'MODEL',
    cell: ({ getValue }) => <span class="model-tag">{esc(String(getValue() ?? ''))}</span>,
  },
  {
    accessorKey: 'duration_s',
    header: 'DURATION',
    cell: ({ getValue }) => (
      <span class="num">{fmtDuration(getValue() as number)}</span>
    ),
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
    accessorKey: 'stop_reason',
    header: 'STOP',
    cell: ({ getValue }) => <StopReasonBadge reason={getValue() as string | null} />,
  },
];

export function AgentTopSessions({ data }: { data: AgentSessionRow[] }) {
  if (!data.length) return null;
  return (
    <DataTable
      columns={columns}
      data={data.slice(0, 25)}
      title="Top agent sessions"
      sectionKey="agent-top-sessions"
      defaultSort={[{ id: 'cost_usd', desc: true }]}
    />
  );
}
