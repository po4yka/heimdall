import { fmt } from '../lib/format';
import {
  PAGE_SIZE,
  errorMessage,
  loadState,
  pageOffset,
  providerFilter,
  rangeFilter,
  rows,
  toolName,
  total,
} from './store';
import { ToolErrorsTable } from './ToolErrorsTable';

interface Props {
  onLoad: () => Promise<void>;
}

const RANGE_OPTIONS = ['7d', '30d', '90d', 'all'] as const;

export function ToolErrorsPage({ onLoad }: Props) {
  const name = toolName.value;
  const count = total.value;
  const offset = pageOffset.value;
  const state = loadState.value;
  const err = errorMessage.value;
  const data = rows.value;
  const hasNullErrors = data.some(r => r.error_text === null);
  const totalPages = Math.ceil(count / PAGE_SIZE);
  const currentPage = Math.floor(offset / PAGE_SIZE);

  function navigate(newOffset: number): void {
    pageOffset.value = newOffset;
    void onLoad();
  }

  return (
    <div style={{ maxWidth: '1400px', margin: '0 auto', padding: '24px' }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: '16px', marginBottom: '20px', flexWrap: 'wrap' }}>
        <a
          href="/"
          style={{ color: 'var(--color-text-secondary)', textDecoration: 'none', fontSize: '13px' }}
        >
          ← Dashboard
        </a>
        <h1 style={{ margin: 0, fontSize: '18px', fontWeight: 600 }}>
          {name} — error details
        </h1>
        {state !== 'loading' && (
          <span class="muted" style={{ fontSize: '13px' }}>
            {fmt(count)} errors total
          </span>
        )}
        {state === 'loading' && <span class="muted" style={{ fontSize: '13px' }}>[loading…]</span>}
      </div>

      {/* Filter bar */}
      <div style={{ display: 'flex', gap: '12px', alignItems: 'center', marginBottom: '16px', flexWrap: 'wrap' }}>
        <label style={{ fontSize: '13px', color: 'var(--color-text-secondary)' }}>
          Range:
          <select
            value={rangeFilter.value}
            onChange={(e) => {
              rangeFilter.value = (e.target as HTMLSelectElement).value;
              pageOffset.value = 0;
              void onLoad();
            }}
            style={{ marginLeft: '6px', background: 'var(--card-bg)', color: 'var(--color-text-primary)', border: '1px solid var(--border)', borderRadius: '4px', padding: '2px 6px', fontSize: '13px' }}
          >
            {RANGE_OPTIONS.map(r => <option key={r} value={r}>{r}</option>)}
          </select>
        </label>

        <label style={{ fontSize: '13px', color: 'var(--color-text-secondary)' }}>
          Provider:
          <input
            type="text"
            value={providerFilter.value}
            placeholder="all"
            onInput={(e) => {
              providerFilter.value = (e.target as HTMLInputElement).value;
              pageOffset.value = 0;
              void onLoad();
            }}
            style={{ marginLeft: '6px', width: '80px', background: 'var(--card-bg)', color: 'var(--color-text-primary)', border: '1px solid var(--border)', borderRadius: '4px', padding: '2px 6px', fontSize: '13px' }}
          />
        </label>
      </div>

      {/* Error state */}
      {err && (
        <div class="card" style={{ color: 'var(--accent)', padding: '16px', marginBottom: '16px' }}>
          [Error: {err}]
        </div>
      )}

      {/* Backfill notice */}
      {hasNullErrors && (
        <div class="card" style={{ fontSize: '12px', color: 'var(--color-text-secondary)', padding: '10px 16px', marginBottom: '12px' }}>
          [Note: some rows have no error message — run <code>cargo run -- db reset --yes && cargo run -- scan</code> to capture pre-upgrade errors]
        </div>
      )}

      {/* Table */}
      {data.length > 0
        ? <ToolErrorsTable data={data} />
        : state === 'idle' && <p class="muted">No errors found for the selected filters.</p>
      }

      {/* Pagination */}
      {totalPages > 1 && (
        <div style={{ display: 'flex', gap: '8px', alignItems: 'center', marginTop: '16px', fontSize: '13px' }}>
          <button
            type="button"
            class="table-action-btn"
            disabled={currentPage === 0}
            onClick={() => navigate(Math.max(0, offset - PAGE_SIZE))}
          >
            ← Prev
          </button>
          <span class="muted">Page {currentPage + 1} of {totalPages}</span>
          <button
            type="button"
            class="table-action-btn"
            disabled={offset + PAGE_SIZE >= count}
            onClick={() => navigate(offset + PAGE_SIZE)}
          >
            Next →
          </button>
        </div>
      )}
    </div>
  );
}
