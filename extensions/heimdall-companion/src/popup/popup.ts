import { loadConfig } from '../storage';

async function render(): Promise<void> {
  const cfg = await loadConfig();
  const pairedEl = document.getElementById('paired');
  if (pairedEl) {
    pairedEl.textContent = cfg.companionToken ? 'yes' : 'no';
    pairedEl.className = cfg.companionToken ? 'ok' : 'warn';
  }
  const lastSyncs = Object.values(cfg.vendors)
    .map(v => v.lastSyncAt)
    .filter((s): s is string => !!s)
    .sort();
  const lastSyncEl = document.getElementById('lastSync');
  if (lastSyncEl) {
    lastSyncEl.textContent = lastSyncs[lastSyncs.length - 1] ?? '—';
  }
  const capturesEl = document.getElementById('captures');
  if (capturesEl) {
    capturesEl.textContent = String(cfg.telemetry.totalCaptures);
  }
}

document.getElementById('syncNow')!.addEventListener('click', () => {
  chrome.runtime.sendMessage({ type: 'syncNow' });
});
document.getElementById('options')!.addEventListener('click', () => {
  chrome.runtime.openOptionsPage();
});

void render();
