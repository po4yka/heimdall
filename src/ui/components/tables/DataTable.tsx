import type { ComponentChild } from 'preact';
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
  type TableState,
  type Header,
  type Cell,
  type VisibilityState,
  type Updater,
} from '@tanstack/table-core';
import { isSectionCollapsed, setSectionCollapsed, syncDashboardUrl } from '../../state/store';

interface DataTableProps<T> {
  columns: ColumnDef<T, unknown>[];
  data: T[];
  title?: string;
  sectionKey?: string;
  exportFn?: () => void;
  pageSize?: number;
  defaultSort?: SortingState;
  enableColumnVisibility?: boolean;
  costRows?: boolean;
  paginationState?: PaginationState;
  onPaginationChange?: (pagination: PaginationState) => void;
  columnVisibilityState?: VisibilityState;
  onColumnVisibilityChange?: (columnVisibility: VisibilityState) => void;
}

function renderCell<T>(cell: Cell<T, unknown>): ComponentChild {
  const def = cell.column.columnDef.cell;
  if (typeof def === 'function') {
    return def(cell.getContext()) as ComponentChild;
  }
  return cell.getValue() as ComponentChild;
}

function renderHeader<T>(header: Header<T, unknown>): ComponentChild {
  const def = header.column.columnDef.header;
  if (typeof def === 'function') {
    return def(header.getContext()) as ComponentChild;
  }
  return (def ?? header.column.id) as ComponentChild;
}

function resolveUpdater<T>(updater: Updater<T>, prev: T): T {
  return typeof updater === 'function' ? (updater as (old: T) => T)(prev) : updater;
}

export function DataTable<T>({
  columns,
  data,
  title,
  sectionKey,
  exportFn,
  pageSize,
  defaultSort,
  enableColumnVisibility,
  costRows,
  paginationState,
  onPaginationChange,
  columnVisibilityState,
  onColumnVisibilityChange,
}: DataTableProps<T>) {
  const [sorting, setSorting] = useState<SortingState>(defaultSort || []);
  const [localPagination, setLocalPagination] = useState<PaginationState>({
    pageIndex: 0,
    pageSize: pageSize || data.length || 100,
  });
  const [localColumnVisibility, setLocalColumnVisibility] = useState<VisibilityState>({});
  const [, rerender] = useState(0);
  const pagination = paginationState ?? localPagination;
  const columnVisibility = columnVisibilityState ?? localColumnVisibility;

  useEffect(() => {
    if (!pageSize) return;

    const rowsPerPage = pagination.pageSize || pageSize;
    const maxPageIndex = Math.max(Math.ceil(data.length / rowsPerPage) - 1, 0);
    if (pagination.pageIndex <= maxPageIndex) return;

    const nextPagination = { ...pagination, pageIndex: maxPageIndex };
    if (paginationState) {
      onPaginationChange?.(nextPagination);
    } else {
      setLocalPagination(nextPagination);
    }
  }, [data.length, pageSize, pagination, paginationState, onPaginationChange]);

  const tableRef = useRef<Table<T> | null>(null);

  const stateRef = useRef({ sorting, pagination, columnVisibility });
  stateRef.current = { sorting, pagination, columnVisibility };

  const handlePaginationChange = (updater: Updater<PaginationState>) => {
    const nextPagination = resolveUpdater(updater, stateRef.current.pagination);
    if (paginationState) {
      onPaginationChange?.(nextPagination);
    } else {
      setLocalPagination(nextPagination);
    }
    rerender(n => n + 1);
  };

  const handleColumnVisibilityChange = (updater: Updater<VisibilityState>) => {
    const nextVisibility = resolveUpdater(updater, stateRef.current.columnVisibility);
    if (columnVisibilityState) {
      onColumnVisibilityChange?.(nextVisibility);
    } else {
      setLocalColumnVisibility(nextVisibility);
    }
    rerender(n => n + 1);
  };

  if (!tableRef.current) {
    const tableState: Partial<TableState> = {
      sorting,
      pagination,
      columnVisibility,
      columnPinning: { left: [], right: [] },
    };
    tableRef.current = createTable<T>({
      columns,
      data,
      state: tableState,
      onStateChange: () => {},
      onSortingChange: (updater) => {
        setSorting(prev => resolveUpdater(updater, prev));
        rerender(n => n + 1);
      },
      onPaginationChange: handlePaginationChange,
      onColumnVisibilityChange: handleColumnVisibilityChange,
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
  const headingId = title ? `table-heading-${title.toLowerCase().replace(/[^a-z0-9]+/g, '-')}` : undefined;
  const sectionContentId = sectionKey ? `section-content-${sectionKey}` : undefined;
  const collapsed = sectionKey ? isSectionCollapsed(sectionKey) : false;
  const handleToggleCollapse = () => {
    if (!sectionKey) return;
    setSectionCollapsed(sectionKey, !collapsed);
    syncDashboardUrl();
  };

  return (
    <div class="table-card">
      {(title || exportFn) && (
        <div class="section-header">
          {title && (
            <h2 id={headingId} class="section-title" style={{ margin: 0 }}>
              {title}
            </h2>
          )}
          <div class="section-actions">
            {sectionKey && (
              <button
                class="section-toggle"
                type="button"
                aria-expanded={!collapsed}
                aria-controls={sectionContentId}
                onClick={handleToggleCollapse}
              >
                {collapsed ? 'Show' : 'Hide'}
              </button>
            )}
            {exportFn && (
              <button class="export-btn" type="button" onClick={exportFn} title="Export to CSV">
                &#x2913; CSV
              </button>
            )}
          </div>
        </div>
      )}

      <div id={sectionContentId} style={collapsed ? { display: 'none' } : undefined}>
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

        <table aria-labelledby={headingId}>
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
                      style={sorted ? { borderBottom: '2px solid var(--text-display)' } : undefined}
                      tabIndex={canSort ? 0 : undefined}
                      onClick={canSort ? header.column.getToggleSortingHandler() : undefined}
                      onKeyDown={canSort ? (e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); header.column.getToggleSortingHandler()?.(e); } } : undefined}
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
              <tr key={row.id} class={costRows ? 'cost-row' : undefined}>
                {row.getVisibleCells().map(cell => (
                  <td key={cell.id}>{renderCell(cell)}</td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>

        {pageSize && (
          <div class="pagination">
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
    </div>
  );
}
