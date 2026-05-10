import { loadConfig, saveConfig } from './storage';
import { syncVendor, type VendorAdapter } from './sync';
import { listClaude, fetchClaudeConv } from './in-page/claude';
import { getChatgptToken, listChatgpt, fetchChatgptConv } from './in-page/chatgpt';

const ALARM_NAME = 'heimdall-sync';
// Minimum time between tab-focus-triggered syncs to avoid hammering the vendor APIs.
const FOCUS_COOLDOWN_MS = 5 * 60 * 1000;

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

// Tab-focus trigger: sync vendor when user switches to a claude.ai or
// chatgpt.com tab, subject to a 5-minute cooldown per vendor.
chrome.tabs.onActivated.addListener(async (activeInfo) => {
  try {
    const tab = await chrome.tabs.get(activeInfo.tabId);
    const vendor = vendorFromHostname(tab.url ?? '');
    if (!vendor) return;
    const cfg = await loadConfig();
    const state = cfg.vendors[vendor];
    if (!state) return;
    if (state.lastSyncAt) {
      const age = Date.now() - Date.parse(state.lastSyncAt);
      if (age < FOCUS_COOLDOWN_MS) return;
    }
    await runSyncAll();
  } catch {
    // Tab may have been closed or URL inaccessible — ignore.
  }
});

chrome.runtime.onMessage.addListener((msg, _sender, send) => {
  if (msg?.type === 'syncNow') {
    runSyncAll().then(send).catch(err => send({ error: String(err) }));
    return true;
  }
  if (msg?.type === 'forceResyncNow') {
    // Clear lastSeenUpdatedAt so every conversation is re-fetched.
    loadConfig().then(async cfg => {
      for (const state of Object.values(cfg.vendors)) {
        state.lastSeenUpdatedAt = {};
      }
      await saveConfig(cfg);
      return runSyncAll();
    }).then(send).catch(err => send({ error: String(err) }));
    return true;
  }
  if (msg?.type === 'chatgptCitations' && msg.convId && Array.isArray(msg.mapping)) {
    // Store citation mapping from the content-script sidebar scrape.
    // Uses chrome.storage.local (not session) so the mapping survives
    // service-worker restarts between scrape and next sync run.
    const key = `citations:${msg.convId as string}`;
    chrome.storage.local.set({ [key]: msg.mapping }).catch(() => {/* best-effort */});
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

function vendorFromHostname(url: string): string | null {
  try {
    const { hostname } = new URL(url);
    return ['claude.ai', 'chatgpt.com'].includes(hostname) ? hostname : null;
  } catch {
    return null;
  }
}
