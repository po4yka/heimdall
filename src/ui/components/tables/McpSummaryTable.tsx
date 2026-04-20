import { type ColumnDef } from '@tanstack/table-core';
import { fmt } from '../../lib/format';
import type { McpServerSummary } from '../../state/types';
import { InlineRankBar } from '../shared/InlineRankBar';
import { DataTable } from './DataTable';

function makeColumns(data: McpServerSummary[]): ColumnDef<McpServerSummary, unknown>[] {
  const maxInvocations = data.reduce((m, r) => Math.max(m, r.invocations), 0);
  return [
    { accessorKey: 'provider', header: 'Provider',
      cell: ({ getValue }) => <span class="model-tag">{String(getValue()).toUpperCase()}</span> },
    { accessorKey: 'server', header: 'MCP Server',
      cell: ({ getValue }) => <span class="model-tag mcp">{String(getValue())}</span> },
    { accessorKey: 'tools_used', header: 'Tools',
      cell: ({ getValue }) => <span class="num">{Number(getValue() ?? 0)}</span> },
    { accessorKey: 'invocations', header: 'Calls',
      cell: ({ getValue }) => (
        <InlineRankBar
          value={getValue() as number}
          max={maxInvocations}
          label={fmt(getValue() as number)}
        />
      ) },
    { accessorKey: 'sessions_used', header: 'Sessions',
      cell: ({ getValue }) => <span class="num">{fmt(getValue() as number)}</span> },
  ];
}

export function McpSummaryTable({ data }: { data: McpServerSummary[] }) {
  if (!data.length) return null;
  return <DataTable columns={makeColumns(data)} data={data} title="MCP Server Usage" sectionKey="mcp-summary" />;
}
