import { backupSnapshots, backupLoadState, type SnapshotMeta } from '../state/store';
import { setStatus } from '../lib/status';
import { esc } from '../lib/format';
import { TableSkeleton } from './_primitives/Skeleton';

export interface BackupPanelProps {
  /** POSTs to /api/archive/snapshot. */
  onSnapshot: () => Promise<void>;
  /** GETs /api/archive and updates the snapshot signal. */
  onReload: () => Promise<void>;
}

export function BackupPanel({ onSnapshot, onReload }: BackupPanelProps) {

  const snapshots = backupSnapshots.value;
  const state = backupLoadState.value;

  return (
    <section class="backup-panel">
      <header class="backup-panel-header">
        <h2>Snapshots</h2>
        <button
          type="button"
          class="primary"
          disabled={state === 'loading'}
          onClick={async () => {
            setStatus('snapshot', 'loading', 'snapshotting...');
            try {
              await onSnapshot();
              await onReload();
              setStatus('snapshot', 'success', 'done', 3000);
            } catch (err) {
              setStatus('snapshot', 'error', `error: ${err instanceof Error ? err.message : String(err)}`);
            }
          }}
        >
          Snapshot now
        </button>
      </header>
      {state === 'error' && (
        <p class="backup-panel-error">Failed to load snapshots.</p>
      )}
      {state === 'loading' && snapshots.length === 0 && (
        <TableSkeleton rows={5} columns={4} />
      )}
      {snapshots.length === 0 && state === 'idle' && (
        <p class="backup-panel-empty">No snapshots yet — click "Snapshot now" to create one.</p>
      )}
      {snapshots.length > 0 && (
        <table class="data-table">
          <thead>
            <tr>
              <th>SNAPSHOT</th>
              <th>CREATED</th>
              <th>FILES</th>
              <th>BYTES</th>
            </tr>
          </thead>
          <tbody>
            {snapshots.map((s: SnapshotMeta) => (
              <tr key={s.snapshot_id}>
                <td>{esc(s.snapshot_id)}</td>
                <td>{esc(s.created_at)}</td>
                <td>{s.total_files}</td>
                <td>{s.total_bytes}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}
