import { useMemo } from 'preact/hooks';
import {
  type CellContext,
  type ColumnDef,
  type SortingState,
} from '@tanstack/table-core';
import { fmt, fmtCost, anyHasCredits, fmtCredits } from '../lib/format';
import type { ModelAgg } from '../state/types';
import { DataTable } from './DataTable';
import { SegmentedProgressBar } from './SegmentedProgressBar';

const defaultSort: SortingState = [{ id: 'cost', desc: true }];

/** Inline share-of-cost micro-bar matching the existing share column pattern. */
function CostShareBar({ value, max, label }: { value: number; max: number; label: string }) {
  if (max <= 0 || value <= 0) return <span class="cost-na">&mdash;</span>;
  const pct = (value / max) * 100;
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: '6px', minWidth: '100px' }}>
      <span class="num" style={{ fontSize: '13px', minWidth: '52px', textAlign: 'right' }}>
        {fmtCost(value)}
      </span>
      <div
        role="img"
        style={{
          flex: 1,
          height: '4px',
          background: 'rgba(var(--text-primary-rgb,232,232,232),0.12)',
          borderRadius: '2px',
          overflow: 'hidden',
        }}
        aria-label={label}
      >
        <div
          style={{
            height: '100%',
            width: `${Math.min(100, pct).toFixed(1)}%`,
            background: 'rgba(var(--text-primary-rgb,232,232,232),0.65)',
            borderRadius: '2px',
          }}
        />
      </div>
    </div>
  );
}

function useModelColumns(
  totalCost: number,
  totalCacheReadCost: number,
  totalCacheWriteCost: number,
  showCredits: boolean,
  onSelectModel?: ((model: string) => void) | undefined,
): ColumnDef<ModelAgg, unknown>[] {
  return useMemo(
    () => [
      {
        id: 'model',
        accessorKey: 'model',
        header: 'Model',
        enableSorting: false,
        cell: (info: CellContext<ModelAgg, unknown>) => {
          const model = String(info.getValue());
          if (!onSelectModel) return <span class="model-tag">{model}</span>;
          return (
            <button type="button" class="table-action-btn table-action-btn--tag" onClick={() => onSelectModel(model)}>
              <span class="model-tag">{model}</span>
            </button>
          );
        },
      },
      {
        id: 'turns',
        accessorKey: 'turns',
        header: 'Turns',
        cell: (info: CellContext<ModelAgg, unknown>) => (
          <span class="num">{fmt(Number(info.getValue() ?? 0))}</span>
        ),
      },
      {
        id: 'input',
        accessorKey: 'input',
        header: 'Input',
        cell: (info: CellContext<ModelAgg, unknown>) => (
          <span class="num">{fmt(Number(info.getValue() ?? 0))}</span>
        ),
      },
      {
        id: 'output',
        accessorKey: 'output',
        header: 'Output',
        cell: (info: CellContext<ModelAgg, unknown>) => (
          <span class="num">{fmt(Number(info.getValue() ?? 0))}</span>
        ),
      },
      {
        id: 'cache_read',
        accessorKey: 'cache_read',
        header: 'Cached Input',
        cell: (info: CellContext<ModelAgg, unknown>) => (
          <span class="num">{fmt(Number(info.getValue() ?? 0))}</span>
        ),
      },
      {
        id: 'cache_creation',
        accessorKey: 'cache_creation',
        header: 'Cache Creation',
        cell: (info: CellContext<ModelAgg, unknown>) => (
          <span class="num">{fmt(Number(info.getValue() ?? 0))}</span>
        ),
      },
      {
        id: 'cost',
        accessorKey: 'cost',
        header: 'Est. Cost',
        cell: (info: CellContext<ModelAgg, unknown>) => {
          const row = info.row.original as ModelAgg;
          return row.is_billable ? (
            <span class="cost">{fmtCost(Number(info.getValue() ?? 0))}</span>
          ) : (
            <span class="cost-na">n/a</span>
          );
        },
      },
      {
        id: 'share',
        accessorFn: (row: ModelAgg) => row.cost,
        header: 'Share',
        enableSorting: false,
        cell: (info: CellContext<ModelAgg, unknown>) => {
          const row = info.row.original as ModelAgg;
          if (!row.is_billable || totalCost <= 0) {
            return <span class="cost-na">&mdash;</span>;
          }
          const pct = (row.cost / totalCost) * 100;
          return (
            <div style={{ minWidth: '120px', display: 'flex', alignItems: 'center', gap: '8px' }}>
              <div style={{ flex: 1 }}>
                <SegmentedProgressBar
                  value={row.cost}
                  max={totalCost}
                  segments={12}
                  size="compact"
                  status="neutral"
                  aria-label={`${row.model} cost share`}
                />
              </div>
              <span class="num" style={{ fontSize: '11px', color: 'var(--text-secondary)', minWidth: '36px', textAlign: 'right' }}>
                {pct.toFixed(0)}%
              </span>
            </div>
          );
        },
      },
      // Phase 21: cache-read cost column with inline micro-bar
      {
        id: 'cache_read_cost',
        accessorFn: (row: ModelAgg) => row.cache_read_cost ?? 0,
        header: 'Cache Read',
        cell: (info: CellContext<ModelAgg, unknown>) => {
          const row = info.row.original as ModelAgg;
          if (!row.is_billable) return <span class="cost-na">&mdash;</span>;
          return (
            <CostShareBar
              value={row.cache_read_cost ?? 0}
              max={totalCacheReadCost}
              label={`${row.model} cache-read cost share`}
            />
          );
        },
      },
      // Phase 21: cache-write cost column with inline micro-bar
      {
        id: 'cache_write_cost',
        accessorFn: (row: ModelAgg) => row.cache_write_cost ?? 0,
        header: 'Cache Write',
        cell: (info: CellContext<ModelAgg, unknown>) => {
          const row = info.row.original as ModelAgg;
          if (!row.is_billable) return <span class="cost-na">&mdash;</span>;
          return (
            <CostShareBar
              value={row.cache_write_cost ?? 0}
              max={totalCacheWriteCost}
              label={`${row.model} cache-write cost share`}
            />
          );
        },
      },
      // Phase 12: credits column (hidden when no Amp rows in view)
      ...(showCredits ? [{
        id: 'credits',
        accessorFn: (row: ModelAgg) => row.credits ?? null,
        header: 'Credits',
        sortUndefined: 'last' as const,
        cell: (info: CellContext<ModelAgg, unknown>) => {
          const v = info.getValue() as number | null;
          return <span class="num">{fmtCredits(v)}</span>;
        },
      }] : []),
    ],
    [totalCost, totalCacheReadCost, totalCacheWriteCost, showCredits, onSelectModel]
  );
}

export function ModelCostTable({
  byModel,
  onSelectModel,
}: {
  byModel: ModelAgg[];
  onSelectModel?: (model: string) => void;
}) {
  const totalCost = useMemo(
    () => byModel.reduce((s, m) => (m.is_billable ? s + m.cost : s), 0),
    [byModel]
  );
  const totalCacheReadCost = useMemo(
    () => byModel.reduce((s, m) => s + (m.cache_read_cost ?? 0), 0),
    [byModel]
  );
  const totalCacheWriteCost = useMemo(
    () => byModel.reduce((s, m) => s + (m.cache_write_cost ?? 0), 0),
    [byModel]
  );
  const showCredits = anyHasCredits(byModel);
  const columns = useModelColumns(totalCost, totalCacheReadCost, totalCacheWriteCost, showCredits, onSelectModel);

  return (
    <DataTable
      columns={columns}
      data={byModel}
      title="Cost by Model"
      sectionKey="model-cost-mount"
      defaultSort={defaultSort}
      costRows
    />
  );
}
