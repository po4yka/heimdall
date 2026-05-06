import { useState } from 'preact/hooks';
import { patchProject } from '../../lib/projects';
import { setStatus } from '../../lib/status';

interface PinStarProps {
  /** Project UUIDv4 from /api/projects (the PATCH key). */
  projectUuid: string;
  pinned: boolean;
  /** Optional callback fired after a successful PATCH (for parent reload). */
  onChange?: () => void;
  /** Visible label for screen readers. Defaults to slug or "project". */
  label?: string;
}

/**
 * Monochrome pin-star toggle, opacity-differentiated per industrial-design
 * guide: filled `★` at full opacity when pinned, outline `☆` at 0.35 when not.
 * Uses --color-text-primary so it adapts to dark/light themes; never colored.
 */
export function PinStar({ projectUuid, pinned, onChange, label }: PinStarProps) {
  const [busy, setBusy] = useState(false);
  const [optimistic, setOptimistic] = useState<boolean | null>(null);
  const current = optimistic ?? pinned;
  const ariaLabel = `${current ? 'Unpin' : 'Pin'} ${label ?? 'project'}`;

  async function toggle() {
    if (busy) return;
    const next = !current;
    setBusy(true);
    setOptimistic(next);
    try {
      await patchProject(projectUuid, { pinned: next });
      onChange?.();
    } catch (err) {
      setOptimistic(null);
      setStatus(
        'project-registry',
        'error',
        `Pin failed: ${err instanceof Error ? err.message : String(err)}`,
        3000,
      );
    } finally {
      setBusy(false);
    }
  }

  return (
    <button
      type="button"
      class={`pin-star ${current ? 'is-pinned' : 'is-unpinned'}`}
      aria-label={ariaLabel}
      aria-pressed={current}
      title={ariaLabel}
      onClick={toggle}
      disabled={busy}
      style={busy ? { cursor: 'wait' } : undefined}
    >
      {current ? '★' : '☆'}
    </button>
  );
}
