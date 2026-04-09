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
      top: 16,
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
          padding: '12px 20px',
          borderRadius: '8px',
          fontSize: '13px',
          maxWidth: '400px',
          boxShadow: '0 4px 12px rgba(0,0,0,0.15)',
        }}>
          {t.text}
        </div>
      ))}
    </div>
  );
}
