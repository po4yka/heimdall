import { loadConfig, saveConfig } from '../storage';

const $ = (id: string) => document.getElementById(id) as HTMLInputElement;
const status = document.getElementById('status') as HTMLDivElement;

async function render(): Promise<void> {
  const cfg = await loadConfig();
  $('heimdallUrl').value = cfg.heimdallUrl;
  $('companionToken').value = cfg.companionToken ?? '';
  $('syncIntervalMinutes').value = String(cfg.syncIntervalMinutes);
  $('vendor-claude.ai').checked = cfg.vendors['claude.ai']?.enabled ?? true;
  $('vendor-chatgpt.com').checked = cfg.vendors['chatgpt.com']?.enabled ?? true;
}

document.getElementById('save')!.addEventListener('click', async () => {
  const cfg = await loadConfig();
  cfg.heimdallUrl = $('heimdallUrl').value.trim() || cfg.heimdallUrl;
  const tok = $('companionToken').value.trim();
  cfg.companionToken = tok.length === 64 ? tok : null;
  const n = parseInt($('syncIntervalMinutes').value, 10);
  if (Number.isFinite(n) && n >= 15) cfg.syncIntervalMinutes = n;
  const claude = cfg.vendors['claude.ai'];
  if (claude) claude.enabled = $('vendor-claude.ai').checked;
  const chatgpt = cfg.vendors['chatgpt.com'];
  if (chatgpt) chatgpt.enabled = $('vendor-chatgpt.com').checked;
  await saveConfig(cfg);
  status.textContent = '[SAVED]';
  setTimeout(() => (status.textContent = ''), 2000);
});

document.getElementById('syncNow')!.addEventListener('click', async () => {
  status.textContent = 'syncing...';
  try {
    const r = await chrome.runtime.sendMessage({ type: 'syncNow' });
    status.textContent = JSON.stringify(r, null, 2);
  } catch (e) {
    status.textContent = `[ERROR: ${(e as Error).message}]`;
  }
});

void render();
