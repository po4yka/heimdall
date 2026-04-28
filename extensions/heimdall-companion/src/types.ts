export interface VendorState {
  enabled: boolean;
  lastSyncAt: string | null;
  lastSeenUpdatedAt: Record<string, string>;
}

export interface ExtensionConfig {
  version: 1;
  heimdallUrl: string;
  companionToken: string | null;
  syncIntervalMinutes: number;
  vendors: Record<string, VendorState>;
  telemetry: { totalCaptures: number; totalErrors: number };
}

export interface WebConversation {
  vendor: string;
  conversation_id: string;
  captured_at: string;
  schema_fingerprint: string;
  payload: unknown;
}

export const DEFAULT_VENDORS = ['claude.ai', 'chatgpt.com'] as const;

export const DEFAULT_CONFIG: ExtensionConfig = {
  version: 1,
  heimdallUrl: 'http://localhost:8080',
  companionToken: null,
  syncIntervalMinutes: 360,
  vendors: Object.fromEntries(DEFAULT_VENDORS.map(v => [v, {
    enabled: true,
    lastSyncAt: null,
    lastSeenUpdatedAt: {},
  }])),
  telemetry: { totalCaptures: 0, totalErrors: 0 },
};
