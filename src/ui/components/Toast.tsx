import { signal } from '@preact/signals';

interface ToastMessage {
  text: string;
  type: 'error' | 'success';
  id: number;
}

export const toasts = signal<ToastMessage[]>([]);
let toastId = 0;

export function showError(msg: string): void {
  const id = ++toastId;
  toasts.value = [...toasts.value, { text: msg, type: 'error', id }];
  setTimeout(() => { toasts.value = toasts.value.filter(t => t.id !== id); }, 6000);
}

export function showSuccess(msg: string): void {
  const id = ++toastId;
  toasts.value = [...toasts.value, { text: msg, type: 'success', id }];
  setTimeout(() => { toasts.value = toasts.value.filter(t => t.id !== id); }, 6000);
}

export function ToastContainer() {
  return (
    <div style={{
      position: 'fixed',
      top: 56,
      right: 16,
      zIndex: 999,
      display: 'flex',
      flexDirection: 'column',
      gap: '8px',
    }}>
      {toasts.value.map(t => (
        <div key={t.id} style={{
          background: `var(--toast-${t.type === 'error' ? 'error' : 'success'}-bg)`,
          color: `var(--toast-${t.type === 'error' ? 'error' : 'success'}-text)`,
          padding: '10px 16px',
          borderRadius: '8px',
          fontSize: '12px',
          fontWeight: 500,
          maxWidth: '360px',
          border: '1px solid var(--border)',
          animation: 'slideIn 0.2s ease-out',
        }}>
          {t.text}
        </div>
      ))}
    </div>
  );
}
