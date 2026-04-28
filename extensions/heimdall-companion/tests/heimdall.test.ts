import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import { postConversation } from '../src/heimdall';
import { DEFAULT_CONFIG } from '../src/types';

beforeAll(() => {
  // happy-dom doesn't define `chrome`; stub the bits postConversation needs
  // (none directly — only postHeartbeat uses `chrome.runtime.getManifest`).
  (globalThis as unknown as { chrome: unknown }).chrome = {
    runtime: { getManifest: () => ({ version: '0.1.0' }) },
  };
});

afterEach(() => vi.restoreAllMocks());

describe('postConversation', () => {
  it('attaches the bearer header and returns saved=true', async () => {
    const fetchMock = vi.spyOn(globalThis as { fetch: typeof fetch }, 'fetch').mockImplementation(
      async (...args: Parameters<typeof fetch>) => {
        const init = args[1] as RequestInit;
        const headers = init.headers as Record<string, string>;
        expect(headers['Authorization']).toBe('Bearer abc');
        return new Response(JSON.stringify({ saved: true }), { status: 200 });
      },
    );
    const cfg = { ...DEFAULT_CONFIG, companionToken: 'abc' };
    const conv = { vendor: 'claude.ai', conversation_id: 'x', captured_at: 't', schema_fingerprint: 'f', payload: {} };
    const out = await postConversation(cfg, conv);
    expect(out).toEqual({ saved: true, unchanged: false });
    expect(fetchMock).toHaveBeenCalledOnce();
  });

  it('throws on 401 with re-pair hint', async () => {
    vi.spyOn(globalThis as { fetch: typeof fetch }, 'fetch').mockResolvedValue(
      new Response('', { status: 401 }),
    );
    const cfg = { ...DEFAULT_CONFIG, companionToken: 'abc' };
    const conv = { vendor: 'x', conversation_id: 'y', captured_at: 't', schema_fingerprint: 'f', payload: {} };
    await expect(postConversation(cfg, conv)).rejects.toThrow(/re-pair/);
  });

  it('throws when no token paired', async () => {
    const cfg = { ...DEFAULT_CONFIG, companionToken: null };
    const conv = { vendor: 'x', conversation_id: 'y', captured_at: 't', schema_fingerprint: 'f', payload: {} };
    await expect(postConversation(cfg, conv)).rejects.toThrow(/not paired/);
  });
});
