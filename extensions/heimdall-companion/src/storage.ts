import type { ExtensionConfig, VendorState } from './types';
import { DEFAULT_CONFIG } from './types';

export async function loadConfig(): Promise<ExtensionConfig> {
  const stored = await chrome.storage.local.get(['config']);
  return mergeWithDefaults(stored['config']);
}

export async function saveConfig(c: ExtensionConfig): Promise<void> {
  await chrome.storage.local.set({ config: c });
}

export function mergeWithDefaults(raw: unknown): ExtensionConfig {
  if (raw === null || typeof raw !== 'object') return structuredClone(DEFAULT_CONFIG);
  const obj = raw as Partial<ExtensionConfig>;
  const merged: ExtensionConfig = structuredClone(DEFAULT_CONFIG);
  if (typeof obj.heimdallUrl === 'string') merged.heimdallUrl = obj.heimdallUrl;
  if (typeof obj.companionToken === 'string') merged.companionToken = obj.companionToken;
  if (typeof obj.syncIntervalMinutes === 'number' && obj.syncIntervalMinutes > 0) {
    merged.syncIntervalMinutes = obj.syncIntervalMinutes;
  }
  if (obj.vendors && typeof obj.vendors === 'object') {
    for (const [vendor, state] of Object.entries(obj.vendors)) {
      const v = state as Partial<VendorState>;
      merged.vendors[vendor] = {
        enabled: v?.enabled ?? true,
        lastSyncAt: v?.lastSyncAt ?? null,
        lastSeenUpdatedAt: { ...(v?.lastSeenUpdatedAt ?? {}) },
      };
    }
  }
  if (obj.telemetry) {
    merged.telemetry = {
      totalCaptures: obj.telemetry.totalCaptures ?? 0,
      totalErrors: obj.telemetry.totalErrors ?? 0,
    };
  }
  return merged;
}
