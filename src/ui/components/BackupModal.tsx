import { useEffect } from 'preact/hooks';
import { backupModalOpen } from '../state/store';
import { BackupPanel } from './BackupPanel';

interface BackupModalProps {
  onSnapshot: () => Promise<void>;
  onReload: () => Promise<void>;
}

function closeModal(): void {
  backupModalOpen.value = false;
  // Strip the #/backup hash if present so the URL bar reflects the closed state.
  if (/^#\/backup\b/.test(window.location.hash)) {
    history.replaceState(null, '', window.location.pathname + window.location.search);
  }
}

export function BackupModal({ onSnapshot, onReload }: BackupModalProps) {
  // Close on ESC; mirrors AgentRegistryModal.
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') closeModal();
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, []);

  // Re-fetch the snapshot list every time the modal opens so the panel
  // never shows stale data after a snapshot was taken in another tab.
  useEffect(() => {
    void onReload();
  }, [onReload]);

  return (
    <div class="agent-registry-overlay" onClick={closeModal}>
      <div
        class="agent-registry-modal"
        onClick={(e: Event) => e.stopPropagation()}
        role="dialog"
        aria-modal="true"
        aria-label="Backup and snapshots"
      >
        <div class="agent-registry-header">
          <h2 class="agent-registry-title">Backup &amp; snapshots</h2>
          <button
            type="button"
            class="agent-registry-close"
            aria-label="Close"
            onClick={closeModal}
          >
            [X]
          </button>
        </div>
        <div style={{ padding: '0 20px 20px' }}>
          <BackupPanel onSnapshot={onSnapshot} onReload={onReload} />
        </div>
      </div>
    </div>
  );
}
