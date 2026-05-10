import { loadConfig, saveConfig } from './storage';
import { syncVendor, type VendorAdapter } from './sync';
import { listClaude, fetchClaudeConv } from './in-page/claude';
import { getChatgptToken, listChatgpt, fetchChatgptConv } from './in-page/chatgpt';

const ALARM_NAME = 'heimdall-sync';

chrome.runtime.onInstalled.addListener(async () => {
  const cfg = await loadConfig();
  await scheduleAlarm(cfg.syncIntervalMinutes);
});

chrome.runtime.onStartup.addListener(async () => {
  const cfg = await loadConfig();
  await scheduleAlarm(cfg.syncIntervalMinutes);
});

chrome.alarms.onAlarm.addListener(async (alarm) => {
  if (alarm.name !== ALARM_NAME) return;
  await runSyncAll();
});

chrome.runtime.onMessage.addListener((msg, _sender, send) => {
  if (msg?.type === 'syncNow') {
    runSyncAll().then(send).catch(err => send({ error: String(err) }));
    return true;
  }
  if (msg?.type === 'chatgptCitations' && msg.convId && Array.isArray(msg.mapping)) {
    // Store citation mapping from the content-script sidebar scrape.
    // Keyed by convId so fetchChatgptConv can merge it before POSTing.
    const key = `citations:${msg.convId as string}`;
    chrome.storage.session.set({ [key]: msg.mapping }).catch(() => {/* best-effort */});
    send({ ok: true });
    return false;
  }
  return false;
});

async function scheduleAlarm(minutes: number): Promise<void> {
  const period = Math.max(15, minutes); // chrome.alarms minimum 1 min, we keep 15
  await chrome.alarms.clear(ALARM_NAME);
  chrome.alarms.create(ALARM_NAME, { periodInMinutes: period });
}

async function runSyncAll(): Promise<{ results: unknown[] }> {
  const cfg = await loadConfig();
  const results: unknown[] = [];
  for (const vendor of Object.keys(cfg.vendors)) {
    const adapter = await adapterFor(vendor);
    if (!adapter) continue;
    const r = await syncVendor(cfg, adapter);
    results.push(r);
  }
  await saveConfig(cfg);
  return { results };
}

async function adapterFor(vendor: string): Promise<VendorAdapter | null> {
  const origin = vendorOrigin(vendor);
  if (!origin) return null;
  const tabs = await chrome.tabs.query({ url: `${origin}/*` });
  const tab = tabs[0];
  if (!tab?.id) return null;
  const tabId = tab.id;

  if (vendor === 'claude.ai') {
    return {
      vendor,
      async list() {
        const [res] = await chrome.scripting.executeScript({
          target: { tabId },
          func: listClaude,
        });
        return (res?.result as Array<{ id: string; updated_at?: string }>) ?? [];
      },
      async fetch(id: string) {
        const [res] = await chrome.scripting.executeScript({
          target: { tabId },
          func: fetchClaudeConv,
          args: [id],
        });
        return res?.result;
      },
    };
  }

  if (vendor === 'chatgpt.com') {
    // Fetch the bearer token once per sync run; memoized in this closure.
    const [tokenRes] = await chrome.scripting.executeScript({
      target: { tabId },
      func: getChatgptToken,
    });
    const token = (tokenRes?.result as string | undefined) ?? '';
    return {
      vendor,
      async list() {
        const [res] = await chrome.scripting.executeScript({
          target: { tabId },
          func: listChatgpt,
          args: [token],
        });
        return (res?.result as Array<{ id: string; updated_at?: string }>) ?? [];
      },
      async fetch(id: string) {
        const [res] = await chrome.scripting.executeScript({
          target: { tabId },
          func: fetchChatgptConv,
          args: [id, token],
        });
        return res?.result;
      },
    };
  }

  return null;
}

function vendorOrigin(vendor: string): string | null {
  if (vendor === 'claude.ai') return 'https://claude.ai';
  if (vendor === 'chatgpt.com') return 'https://chatgpt.com';
  return null;
}
