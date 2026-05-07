import { statusByPlacement, type StatusPlacement } from '../state/store';
import { clearStatus } from '../lib/status';

const LABEL_MAP = {
  success: 'OK',
  error: 'ERROR',
  loading: 'LOADING',
  info: 'INFO',
} as const;

const COLOR_MAP = {
  success: 'var(--success)',
  error: 'var(--accent)',
  loading: 'var(--text-secondary)',
  info: 'var(--text-secondary)',
} as const;

interface InlineStatusProps {
  placement: StatusPlacement;
  inline?: boolean;
  dismissable?: boolean;
}

export function InlineStatus({ placement, inline = false, dismissable = true }: InlineStatusProps) {
  const entry = statusByPlacement.value[placement];
  if (!entry) return null;

  const label = LABEL_MAP[entry.kind];
  const color = COLOR_MAP[entry.kind];
  const content = entry.message ? `[${label}: ${entry.message}]` : `[${label}]`;

  const baseStyle = {
    fontFamily: 'var(--font-mono)',
    fontSize: '11px',
    letterSpacing: '0.08em',
    textTransform: 'uppercase' as const,
    color,
    animation: 'fadeUp 0.15s ease-out',
    display: inline ? 'inline-flex' : 'flex',
    alignItems: 'center',
    gap: '8px',
    padding: inline ? '0' : '8px 16px',
    border: inline ? 'none' : `1px solid ${color}`,
    borderRadius: inline ? '0' : '4px',
    background: inline ? 'transparent' : 'var(--surface)',
  };

  return (
    <div role={entry.kind === 'error' ? 'alert' : 'status'} style={baseStyle}>
      <span>{content}</span>
      {dismissable && entry.kind !== 'loading' && (
        <button
          type="button"
          onClick={() => clearStatus(placement)}
          aria-label="Dismiss"
          style={{
            background: 'transparent',
            border: 'none',
            color,
            cursor: 'pointer',
            fontFamily: 'inherit',
            fontSize: 'inherit',
            letterSpacing: 'inherit',
            padding: '0 4px',
            opacity: 0.7,
          }}
        >
          Dismiss
        </button>
      )}
    </div>
  );
}
