import { type ColumnDef } from '@tanstack/table-core';
import { signal } from '@preact/signals';
import { esc } from '../lib/format';
import type { ToolErrorRow } from '../state/types';
import { DataTable } from '../components/tables/DataTable';

const expandedInputs = signal<Set<string>>(new Set());
const expandedErrors = signal<Set<string>>(new Set());

function toggle(set: typeof expandedInputs, key: string): void {
  const next = new Set(set.value);
  if (next.has(key)) next.delete(key);
  else next.add(key);
  set.value = next;
}

function ExpandableCell({ value, rowKey, store }: { value: string | null; rowKey: string; store: typeof expandedInputs }) {
  if (!value) return <span class="dim">—</span>;
  const PREVIEW = 200;
  const isLong = value.length > PREVIEW;
  const isExpanded = store.value.has(rowKey);
  const display = isLong && !isExpanded ? value.slice(0, PREVIEW) + '…' : value;
  return (
    <div>
      <pre
        style={{
          margin: 0,
          whiteSpace: 'pre-wrap',
          wordBreak: 'break-all',
          fontFamily: 'var(--font-mono)',
          fontSize: '11px',
          color: 'var(--color-text-secondary)',
          maxHeight: isExpanded ? 'none' : '4.5em',
          overflow: 'hidden',
        }}
        // eslint-disable-next-line react/no-danger
        dangerouslySetInnerHTML={{ __html: esc(display) }}
      />
      {isLong && (
        <button
          type="button"
          class="table-action-btn"
          style={{ fontSize: '11px', marginTop: '2px' }}
          onClick={() => toggle(store, rowKey)}
        >
          {isExpanded ? 'show less' : 'show full'}
        </button>
      )}
    </div>
  );
}

function makeColumns(): ColumnDef<ToolErrorRow, unknown>[] {
  return [
    {
      accessorKey: 'timestamp',
      header: 'Timestamp',
      cell: ({ getValue }) => <span class="num muted">{String(getValue())}</span>,
    },
    {
      accessorKey: 'project',
      header: 'Project',
      cell: ({ getValue }) => <span class="muted" style={{ wordBreak: 'break-all' }}>{String(getValue())}</span>,
    },
    {
      accessorKey: 'session_id',
      header: 'Session',
      cell: ({ getValue }) => {
        const v = String(getValue());
        return <span class="num muted" title={v}>{v.slice(-12)}</span>;
      },
    },
    {
      accessorKey: 'model',
      header: 'Model',
      cell: ({ getValue }) => {
        const v = String(getValue());
        return v ? <span class="model-tag">{v}</span> : <span class="dim">—</span>;
      },
    },
    {
      accessorKey: 'mcp_server',
      header: 'MCP Server',
      cell: ({ getValue }) => {
        const v = getValue() as string | null;
        return v ? <span class="muted">{v}</span> : <span class="dim">—</span>;
      },
    },
    {
      id: 'tool_input',
      header: 'Input',
      cell: ({ row }) => (
        <ExpandableCell
          value={row.original.tool_input}
          rowKey={`input-${row.index}`}
          store={expandedInputs}
        />
      ),
    },
    {
      id: 'error_text',
      header: 'Error',
      cell: ({ row }) => (
        <ExpandableCell
          value={row.original.error_text ?? '(no message captured — db reset to backfill)'}
          rowKey={`err-${row.index}`}
          store={expandedErrors}
        />
      ),
    },
  ];
}

export function ToolErrorsTable({ data }: { data: ToolErrorRow[] }) {
  if (!data.length) return null;
  return (
    <DataTable
      columns={makeColumns()}
      data={data}
      title="Error details"
      sectionKey="tool-errors-detail"
    />
  );
}
