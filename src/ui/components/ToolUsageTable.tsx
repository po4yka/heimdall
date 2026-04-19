import { type ColumnDef } from '@tanstack/table-core';
import { fmt } from '../lib/format';
import type { ToolSummary } from '../state/types';
import { InlineRankBar } from '../InlineRankBar';
import { DataTable } from './DataTable';

function makeColumns(data: ToolSummary[]): ColumnDef<ToolSummary, unknown>[] {
  const maxInvocations = data.reduce((m, r) => Math.max(m, r.invocations), 0);
  return [
    { accessorKey: 'provider', header: 'Provider',
      cell: ({ getValue }) => <span class="model-tag">{String(getValue()).toUpperCase()}</span> },
    { accessorKey: 'tool_name', header: 'Tool',
      cell: ({ row }) => {
        const cat = row.original.category;
        const badge = cat === 'mcp' ? 'mcp' : 'builtin';
        return (
          <span>
            <span class={`model-tag ${badge}`}>{cat}</span>{' '}
            {row.original.tool_name}
          </span>
        );
      } },
    { accessorKey: 'mcp_server', header: 'MCP Server',
      cell: ({ getValue }) => {
        const v = getValue() as string | null;
        return v ? <span class="muted">{v}</span> : <span class="muted">--</span>;
      } },
    { accessorKey: 'invocations', header: 'Calls',
      cell: ({ getValue }) => (
        <InlineRankBar
          value={getValue() as number}
          max={maxInvocations}
          label={fmt(getValue() as number)}
        />
      ) },
    { accessorKey: 'turns_used', header: 'Turns',
      cell: ({ getValue }) => <span class="num">{fmt(getValue() as number)}</span> },
    { accessorKey: 'sessions_used', header: 'Sessions',
      cell: ({ getValue }) => <span class="num">{fmt(getValue() as number)}</span> },
    { accessorKey: 'errors', header: 'Errors',
      cell: ({ row }) => {
        const e = row.original.errors;
        if (!e) return <span class="dim">0</span>;
        const pct = row.original.invocations > 0
          ? ((e / row.original.invocations) * 100).toFixed(1)
          : '0';
        return <span class="num" style={{ color: 'var(--accent)' }}>{e} ({pct}%)</span>;
      } },
  ];
}

export function ToolUsageTable({ data }: { data: ToolSummary[] }) {
  if (!data.length) return null;
  return <DataTable columns={makeColumns(data)} data={data} title="Tool Usage" sectionKey="tool-summary" />;
}
