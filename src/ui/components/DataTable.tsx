import { useState, useRef, useEffect } from 'preact/hooks';
import {
  createTable,
  getCoreRowModel,
  getSortedRowModel,
  getPaginationRowModel,
  type ColumnDef,
  type SortingState,
  type PaginationState,
  type Table,
  type Header,
  type Cell,
  type VisibilityState,
  type Updater,
  type TableState,
} from '@tanstack/table-core';

interface DataTableProps<T> {
  columns: ColumnDef<T, any>[];
  data: T[];
  title?: string;
  exportFn?: () => void;
  pageSize?: number;
  defaultSort?: SortingState;
  enableColumnVisibility?: boolean;
}

function renderCell<T>(cell: Cell<T, unknown>): any {
  const def = cell.column.columnDef.cell;
  if (typeof def === 'function') {
    return def(cell.getContext());
  }
  return cell.getValue();
}

function renderHeader<T>(header: Header<T, unknown>): any {
  const def = header.column.columnDef.header;
  if (typeof def === 'function') {
    return def(header.getContext());
  }
  return def;
}

function resolveUpdater<T>(updater: Updater<T>, prev: T): T {
  return typeof updater === 'function' ? (updater as (old: T) => T)(prev) : updater;
}

export function DataTable<T>({
  columns,
  data,
  title,
  exportFn,
  pageSize,
  defaultSort,
  enableColumnVisibility,
}: DataTableProps<T>) {
  const [sorting, setSorting] = useState<SortingState>(defaultSort || []);
  const [pagination, setPagination] = useState<PaginationState>({
    pageIndex: 0,
    pageSize: pageSize || data.length || 100,
  });
  const [columnVisibility, setColumnVisibility] = useState<VisibilityState>({});
  const [, rerender] = useState(0);

  // Reset pagination to page 0 when data changes
  useEffect(() => {
    setPagination(prev => ({ ...prev, pageIndex: 0 }));
  }, [data]);

  const tableRef = useRef<Table<T> | null>(null);

  const stateRef = useRef({ sorting, pagination, columnVisibility });
  stateRef.current = { sorting, pagination, columnVisibility };

  if (!tableRef.current) {
    tableRef.current = createTable<T>({
      columns,
      data,
      state: { sorting, pagination, columnVisibility, columnPinning: { left: [], right: [] } } as any,
      onStateChange: (updater: Updater<TableState>) => {
        const newState = resolveUpdater(updater, tableRef.current!.getState());
        if (newState.sorting !== stateRef.current.sorting) setSorting(newState.sorting);
        if (newState.pagination !== stateRef.current.pagination) setPagination(newState.pagination);
        if (newState.columnVisibility !== stateRef.current.columnVisibility) setColumnVisibility(newState.columnVisibility);
        rerender(n => n + 1);
      },
      onSortingChange: (updater) => setSorting(prev => resolveUpdater(updater, prev)),
      onPaginationChange: (updater) => setPagination(prev => resolveUpdater(updater, prev)),
      onColumnVisibilityChange: (updater) => setColumnVisibility(prev => resolveUpdater(updater, prev)),
      getCoreRowModel: getCoreRowModel(),
      getSortedRowModel: getSortedRowModel(),
      ...(pageSize ? { getPaginationRowModel: getPaginationRowModel() } : {}),
      renderFallbackValue: '',
    });
  }

  // Update table options on every render
  tableRef.current.setOptions(prev => ({
    ...prev,
    columns,
    data,
    state: { ...tableRef.current!.getState(), sorting, pagination, columnVisibility },
  }));

  const table = tableRef.current;

  const headerGroups = table.getHeaderGroups();
  const rows = table.getRowModel().rows;

  return (
    <div class="table-card">
      {(title || exportFn) && (
        <div class={exportFn ? 'section-header' : ''}>
          {title && <div class="section-title">{title}</div>}
          {exportFn && (
            <button class="export-btn" onClick={exportFn} title="Export to CSV">
              &#x2913; CSV
            </button>
          )}
        </div>
      )}

      {enableColumnVisibility && (
        <div class="column-toggle">
          {table.getAllLeafColumns().map(column => (
            <label key={column.id}>
              <input
                type="checkbox"
                checked={column.getIsVisible()}
                onChange={column.getToggleVisibilityHandler()}
              />
              {typeof column.columnDef.header === 'string'
                ? column.columnDef.header
                : column.id}
            </label>
          ))}
        </div>
      )}

      <table>
        <thead>
          {headerGroups.map(headerGroup => (
            <tr key={headerGroup.id}>
              {headerGroup.headers.map(header => {
                const canSort = header.column.getCanSort();
                const sorted = header.column.getIsSorted();
                return (
                  <th
                    key={header.id}
                    scope="col"
                    class={canSort ? 'sortable' : undefined}
                    aria-sort={sorted === 'asc' ? 'ascending' : sorted === 'desc' ? 'descending' : undefined}
                    onClick={canSort ? header.column.getToggleSortingHandler() : undefined}
                  >
                    {renderHeader(header)}
                    {canSort && (
                      <span class="sort-icon">
                        {sorted === 'desc' ? ' \u25bc' : sorted === 'asc' ? ' \u25b2' : ''}
                      </span>
                    )}
                  </th>
                );
              })}
            </tr>
          ))}
        </thead>
        <tbody>
          {rows.map(row => (
            <tr key={row.id}>
              {row.getVisibleCells().map(cell => (
                <td key={cell.id}>{renderCell(cell)}</td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>

      {pageSize && (
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            marginTop: '12px',
            fontSize: '12px',
            color: 'var(--muted)',
          }}
        >
          <span>
            {table.getRowCount() > 0
              ? `Showing ${pagination.pageIndex * pagination.pageSize + 1}\u2013${Math.min(
                  (pagination.pageIndex + 1) * pagination.pageSize,
                  table.getRowCount()
                )} of ${table.getRowCount()}`
              : 'No sessions'}
          </span>
          <div style={{ display: 'flex', gap: '6px' }}>
            <button
              class="filter-btn"
              disabled={!table.getCanPreviousPage()}
              onClick={() => table.previousPage()}
            >
              &laquo; Prev
            </button>
            <button
              class="filter-btn"
              disabled={!table.getCanNextPage()}
              onClick={() => table.nextPage()}
            >
              Next &raquo;
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
