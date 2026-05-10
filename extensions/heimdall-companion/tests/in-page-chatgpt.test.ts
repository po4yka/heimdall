import { afterEach, describe, expect, it, vi } from 'vitest';
import { getChatgptToken, listChatgpt, fetchChatgptConv } from '../src/in-page/chatgpt';
import sessionFixture from './fixtures/chatgpt-session.json';
import convsFixture from './fixtures/chatgpt-conversations.json';
import convFixture from './fixtures/chatgpt-conversation.json';

function jsonResp(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'content-type': 'application/json' },
  });
}

function cfResp(status: number): Response {
  return new Response('<html>cf challenge</html>', {
    status,
    headers: { 'content-type': 'text/html', 'cf-mitigated': 'challenge' },
  });
}

function htmlResp(status: number): Response {
  return new Response('<html>blocked</html>', {
    status,
    headers: { 'content-type': 'text/html' },
  });
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('getChatgptToken', () => {
  it('returns the accessToken from /api/auth/session', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValueOnce(jsonResp(sessionFixture)));
    const token = await getChatgptToken();
    expect(token).toBe(sessionFixture.accessToken);
  });

  it('throws when accessToken is missing from session response', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValueOnce(jsonResp({ user: {} })));
    await expect(getChatgptToken()).rejects.toThrow('no accessToken');
  });

  it('throws CLOUDFLARE_CHALLENGE on cf-mitigated response', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValueOnce(cfResp(403)));
    await expect(getChatgptToken()).rejects.toThrow('CLOUDFLARE_CHALLENGE');
  });

  it('throws on HTTP error', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValueOnce(
      new Response('Unauthorized', { status: 401, headers: { 'content-type': 'text/plain' } }),
    ));
    await expect(getChatgptToken()).rejects.toThrow('HTTP 401');
  });
});

describe('listChatgpt', () => {
  const token = sessionFixture.accessToken;

  it('fetches conversations and maps update_time to updated_at string', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValueOnce(jsonResp(convsFixture)));
    const result = await listChatgpt(token);

    expect(result).toHaveLength(convsFixture.items.length);
    expect(result[0]?.id).toBe(convsFixture.items[0]?.id);
    expect(result[0]?.updated_at).toBe(String(convsFixture.items[0]?.update_time));
  });

  it('sends Authorization header with the provided token', async () => {
    const fetchMock = vi.fn().mockResolvedValueOnce(jsonResp(convsFixture));
    vi.stubGlobal('fetch', fetchMock);
    await listChatgpt(token);

    const headers = fetchMock.mock.calls[0]?.[1] as RequestInit;
    expect((headers.headers as Record<string, string>)?.['Authorization']).toBe(`Bearer ${token}`);
  });

  it('paginates until fewer than limit items returned', async () => {
    const page1 = {
      items: Array.from({ length: 28 }, (_, i) => ({ id: `id-${i}`, update_time: i })),
      total: 30,
    };
    const page2 = { items: [{ id: 'id-28', update_time: 28 }, { id: 'id-29', update_time: 29 }], total: 30 };

    const fetchMock = vi.fn()
      .mockResolvedValueOnce(jsonResp(page1))
      .mockResolvedValueOnce(jsonResp(page2));
    vi.stubGlobal('fetch', fetchMock);

    const result = await listChatgpt(token);
    expect(result).toHaveLength(30);
    expect(fetchMock).toHaveBeenCalledTimes(2);
    // Second call should use offset=28
    const url2 = fetchMock.mock.calls[1]?.[0] as string;
    expect(url2).toContain('offset=28');
  });

  it('stops pagination when offset reaches total', async () => {
    const full = { items: convsFixture.items, total: 2 };
    const fetchMock = vi.fn().mockResolvedValueOnce(jsonResp(full));
    vi.stubGlobal('fetch', fetchMock);
    await listChatgpt(token);
    expect(fetchMock).toHaveBeenCalledTimes(1);
  });

  it('throws CLOUDFLARE_CHALLENGE on html cf-mitigated response', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValueOnce(cfResp(403)));
    await expect(listChatgpt(token)).rejects.toThrow('CLOUDFLARE_CHALLENGE');
  });

  it('throws CLOUDFLARE_CHALLENGE on html 403 without cf-mitigated header', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValueOnce(htmlResp(403)));
    await expect(listChatgpt(token)).rejects.toThrow('CLOUDFLARE_CHALLENGE');
  });

  it('throws on HTTP error', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValueOnce(
      new Response('Too Many Requests', { status: 429, headers: { 'content-type': 'text/plain' } }),
    ));
    await expect(listChatgpt(token)).rejects.toThrow('HTTP 429');
  });
});

describe('fetchChatgptConv', () => {
  const token = sessionFixture.accessToken;

  it('fetches from /backend-api/conversation/<id> (singular) and returns JSON', async () => {
    const fetchMock = vi.fn().mockResolvedValueOnce(jsonResp(convFixture));
    vi.stubGlobal('fetch', fetchMock);

    const result = await fetchChatgptConv(convFixture.id, token);

    expect(fetchMock).toHaveBeenCalledTimes(1);
    const url = fetchMock.mock.calls[0]?.[0] as string;
    // Singular "conversation" — not "conversations"
    expect(url).toMatch(/\/backend-api\/conversation\/[^/]+$/);
    expect(url).toContain(convFixture.id);
    expect(result).toMatchObject({ id: convFixture.id });
  });

  it('sends the bearer token', async () => {
    const fetchMock = vi.fn().mockResolvedValueOnce(jsonResp(convFixture));
    vi.stubGlobal('fetch', fetchMock);
    await fetchChatgptConv(convFixture.id, token);
    const headers = fetchMock.mock.calls[0]?.[1] as RequestInit;
    expect((headers.headers as Record<string, string>)?.['Authorization']).toBe(`Bearer ${token}`);
  });

  it('throws CLOUDFLARE_CHALLENGE on cf-mitigated response', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValueOnce(cfResp(503)));
    await expect(fetchChatgptConv('some-id', token)).rejects.toThrow('CLOUDFLARE_CHALLENGE');
  });

  it('throws on HTTP error', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValueOnce(
      new Response(null, { status: 404, headers: { 'content-type': 'application/json' } }),
    ));
    await expect(fetchChatgptConv('some-id', token)).rejects.toThrow('HTTP 404');
  });
});
