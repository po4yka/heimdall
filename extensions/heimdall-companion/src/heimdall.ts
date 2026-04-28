import type { ExtensionConfig, WebConversation } from './types';

export async function postConversation(
  cfg: ExtensionConfig,
  conv: WebConversation,
): Promise<{ saved: boolean; unchanged: boolean }> {
  if (!cfg.companionToken) throw new Error('companion token not paired');
  const url = `${cfg.heimdallUrl.replace(/\/$/, '')}/api/archive/web-conversation`;
  const resp = await fetch(url, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${cfg.companionToken}`,
    },
    body: JSON.stringify(conv),
  });
  if (resp.status === 401) throw new Error('401: companion token invalid (re-pair in options page)');
  if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
  const body = await resp.json() as { saved?: boolean; unchanged?: boolean };
  return {
    saved: body.saved === true,
    unchanged: body.unchanged === true,
  };
}

export async function postHeartbeat(
  cfg: ExtensionConfig,
  vendor: string,
): Promise<void> {
  if (!cfg.companionToken) return;
  const url = `${cfg.heimdallUrl.replace(/\/$/, '')}/api/archive/companion-heartbeat`;
  const body = {
    extension_version: chrome.runtime.getManifest().version,
    user_agent: navigator.userAgent,
    vendor,
  };
  await fetch(url, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${cfg.companionToken}`,
    },
    body: JSON.stringify(body),
  });
}
