import type { ExtensionConfig, WebConversation } from './types';
import { postConversation, postHeartbeat } from './heimdall';
import { saveConfig } from './storage';

export interface SyncResult {
  vendor: string;
  listed: number;
  written: number;
  unchanged: number;
  errors: string[];
}

/** Pure function: given current state and a list of (id, updated_at), return ids to fetch. */
export function pickChanged(
  lastSeen: Record<string, string>,
  observed: Array<{ id: string; updated_at?: string }>,
): string[] {
  const out: string[] = [];
  for (const item of observed) {
    const seen = lastSeen[item.id];
    if (!item.updated_at) {
      // No timestamp from vendor — only fetch if we've never seen this id.
      if (!seen) out.push(item.id);
      continue;
    }
    if (!seen || item.updated_at > seen) out.push(item.id);
  }
  return out;
}

/** SHA-256 hex of the sorted, newline-separated keys of a top-level object. */
export async function schemaFingerprint(value: unknown): Promise<string> {
  if (value === null || typeof value !== 'object') return '';
  const keys = Object.keys(value as Record<string, unknown>).sort().join('\n');
  const buf = new TextEncoder().encode(keys);
  const hash = await crypto.subtle.digest('SHA-256', buf);
  return [...new Uint8Array(hash)].map(b => b.toString(16).padStart(2, '0')).join('');
}

export interface VendorAdapter {
  vendor: string;
  list(): Promise<Array<{ id: string; updated_at?: string }>>;
  fetch(id: string): Promise<unknown>;
}

const MAX_FETCH_CONCURRENCY = 2;
const RATE_LIMIT_BACKOFF_MS = 5_000;
const RATE_LIMIT_RE = /429|rate.?limit/i;

type PoolResult<T> = { ok: true; value: T } | { ok: false; error: Error };

/** Run `fn` over `items` with at most `limit` concurrent calls in flight. */
async function runCapped<T>(
  items: string[],
  limit: number,
  fn: (id: string) => Promise<T>,
): Promise<Array<PoolResult<T>>> {
  if (items.length === 0) return [];
  const out: Array<PoolResult<T>> = new Array(items.length);
  // Queue carries [originalIndex, id] pairs so results land in input order.
  const queue: Array<[number, string]> = items.map((id, i) => [i, id]);
  const worker = async () => {
    let next: [number, string] | undefined;
    while ((next = queue.shift()) !== undefined) {
      const [i, id] = next;
      try {
        out[i] = { ok: true, value: await fn(id) };
      } catch (e) {
        out[i] = { ok: false, error: e instanceof Error ? e : new Error(String(e)) };
      }
    }
  };
  await Promise.all(Array.from({ length: Math.min(limit, items.length) }, worker));
  return out;
}

export async function syncVendor(
  cfg: ExtensionConfig,
  adapter: VendorAdapter,
): Promise<SyncResult> {
  const result: SyncResult = {
    vendor: adapter.vendor, listed: 0, written: 0, unchanged: 0, errors: [],
  };
  const state = cfg.vendors[adapter.vendor];
  if (!state || !state.enabled) return result;

  let observed: Array<{ id: string; updated_at?: string }> = [];
  try {
    observed = await adapter.list();
  } catch (e) {
    result.errors.push(`list: ${(e as Error).message}`);
    return result;
  }
  result.listed = observed.length;
  const changed = pickChanged(state.lastSeenUpdatedAt, observed);

  // Build lookup for updated_at values (O(1) per fetch result).
  const observedMap = new Map(observed.map(o => [o.id, o]));

  const fetchResults = await runCapped(changed, MAX_FETCH_CONCURRENCY, async (id) => {
    const payload = await adapter.fetch(id);
    if (payload === undefined) {
      throw new Error('undefined payload — possible Cloudflare challenge or parse error');
    }
    const fingerprint = await schemaFingerprint(payload);
    const conv: WebConversation = {
      vendor: adapter.vendor,
      conversation_id: id,
      captured_at: new Date().toISOString(),
      schema_fingerprint: fingerprint,
      payload,
    };
    const { saved, unchanged } = await postConversation(cfg, conv);
    return { id, saved, unchanged, observedItem: observedMap.get(id) };
  });

  for (let i = 0; i < fetchResults.length; i++) {
    const r = fetchResults[i]!;
    const id = changed[i]!;
    if (r.ok) {
      if (r.value.saved) result.written++;
      if (r.value.unchanged) result.unchanged++;
      state.lastSeenUpdatedAt[id] = r.value.observedItem?.updated_at ?? new Date().toISOString();
    } else {
      const msg = r.error.message;
      result.errors.push(`${id}: ${msg}`);
      cfg.telemetry.totalErrors++;
      // Back off when rate-limited to avoid hammering the vendor API.
      if (RATE_LIMIT_RE.test(msg)) {
        await new Promise(res => setTimeout(res, RATE_LIMIT_BACKOFF_MS));
      }
    }
  }

  state.lastSyncAt = new Date().toISOString();
  cfg.telemetry.totalCaptures += result.written;

  await postHeartbeat(cfg, adapter.vendor).catch(() => {});
  await saveConfig(cfg);
  return result;
}
