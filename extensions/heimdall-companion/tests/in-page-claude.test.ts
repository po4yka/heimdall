import { afterEach, describe, expect, it, vi } from 'vitest';
import { listClaude, fetchClaudeConv } from '../src/in-page/claude';
import orgsFixture from './fixtures/claude-organizations.json';
import convsFixture from './fixtures/claude-conversations.json';
import convFixture from './fixtures/claude-conversation.json';

function jsonResp(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'content-type': 'application/json' },
  });
}

function htmlResp(status: number): Response {
  return new Response('<html><body>Access denied</body></html>', {
    status,
    headers: { 'content-type': 'text/html' },
  });
}

function cfResp(status: number): Response {
  return new Response('<html>cf challenge</html>', {
    status,
    headers: { 'content-type': 'text/html', 'cf-mitigated': 'challenge' },
  });
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('listClaude', () => {
  it('fetches organizations then conversations and returns composite ids', async () => {
    const fetchMock = vi.fn()
      .mockResolvedValueOnce(jsonResp(orgsFixture))         // GET /api/organizations
      .mockResolvedValueOnce(jsonResp(convsFixture))         // GET /api/organizations/org-abc123/chat_conversations
      .mockResolvedValueOnce(jsonResp([]));                  // GET /api/organizations/org-def456/chat_conversations

    vi.stubGlobal('fetch', fetchMock);

    const result = await listClaude();

    expect(fetchMock).toHaveBeenCalledTimes(3);
    expect(fetchMock.mock.calls[0]?.[0]).toBe('/api/organizations');
    expect(fetchMock.mock.calls[1]?.[0]).toBe(
      `/api/organizations/${orgsFixture[0]?.uuid}/chat_conversations`,
    );
    expect(result).toHaveLength(2);
    expect(result[0]?.id).toBe(`${orgsFixture[0]?.uuid}/${convsFixture[0]?.uuid}`);
    expect(result[0]?.updated_at).toBe(convsFixture[0]?.updated_at);
    expect(result[1]?.id).toBe(`${orgsFixture[0]?.uuid}/${convsFixture[1]?.uuid}`);
  });

  it('uses credentials: include on every request', async () => {
    const fetchMock = vi.fn()
      .mockResolvedValueOnce(jsonResp(orgsFixture))
      .mockResolvedValueOnce(jsonResp([]))
      .mockResolvedValueOnce(jsonResp([]));

    vi.stubGlobal('fetch', fetchMock);
    await listClaude();

    for (const call of fetchMock.mock.calls) {
      expect((call[1] as RequestInit).credentials).toBe('include');
    }
  });

  it('throws CLOUDFLARE_CHALLENGE when orgs response is HTML with cf-mitigated header', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValueOnce(cfResp(403)));
    await expect(listClaude()).rejects.toThrow('CLOUDFLARE_CHALLENGE');
  });

  it('throws CLOUDFLARE_CHALLENGE when orgs response is HTML without cf-mitigated (firewall block)', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValueOnce(htmlResp(403)));
    await expect(listClaude()).rejects.toThrow('CLOUDFLARE_CHALLENGE');
  });

  it('throws on non-HTML HTTP error from orgs endpoint', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValueOnce(
      new Response('Unauthorized', { status: 401, headers: { 'content-type': 'text/plain' } }),
    ));
    await expect(listClaude()).rejects.toThrow('HTTP 401');
    await expect(listClaude()).rejects.not.toThrow('CLOUDFLARE_CHALLENGE');
  });

  it('throws on conversations endpoint HTTP error', async () => {
    const fetchMock = vi.fn()
      .mockResolvedValueOnce(jsonResp(orgsFixture))
      .mockResolvedValueOnce(new Response('Too Many Requests', {
        status: 429,
        headers: { 'content-type': 'text/plain' },
      }));
    vi.stubGlobal('fetch', fetchMock);
    await expect(listClaude()).rejects.toThrow('HTTP 429');
  });
});

describe('fetchClaudeConv', () => {
  it('fetches the correct conversation URL and returns parsed JSON', async () => {
    const fetchMock = vi.fn().mockResolvedValueOnce(jsonResp(convFixture));
    vi.stubGlobal('fetch', fetchMock);

    const combinedId = `${orgsFixture[0]?.uuid}/${convFixture.uuid}`;
    const result = await fetchClaudeConv(combinedId);

    expect(fetchMock).toHaveBeenCalledTimes(1);
    const url = fetchMock.mock.calls[0]?.[0] as string;
    expect(url).toContain(orgsFixture[0]?.uuid);
    expect(url).toContain(convFixture.uuid);
    expect(result).toMatchObject({ uuid: convFixture.uuid });
  });

  it('throws CLOUDFLARE_CHALLENGE on html cf-mitigated response', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValueOnce(cfResp(503)));
    await expect(fetchClaudeConv('org/conv')).rejects.toThrow('CLOUDFLARE_CHALLENGE');
  });

  it('throws on HTTP error', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValueOnce(
      new Response(null, { status: 404, headers: { 'content-type': 'application/json' } }),
    ));
    await expect(fetchClaudeConv('org/conv')).rejects.toThrow('HTTP 404');
  });
});
