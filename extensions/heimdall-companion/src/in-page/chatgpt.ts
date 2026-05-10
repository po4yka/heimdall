// Self-contained — NO imports. These functions are serialized and injected into a
// chatgpt.com tab via chrome.scripting.executeScript({ func, args }). Any reference
// to a symbol defined outside the function body will be a runtime ReferenceError in
// the target tab. All helpers must be inner function declarations.

/** Fetch a fresh access token from the session endpoint. */
export function getChatgptToken(): Promise<string> {
  return (async () => {
    const r = await fetch('/api/auth/session', { credentials: 'include' });
    const cf = r.headers.get('cf-mitigated');
    const ct = r.headers.get('content-type') ?? '';
    if (cf || (!r.ok && ct.includes('text/html'))) {
      throw new Error(`CLOUDFLARE_CHALLENGE:${r.status}:auth/session`);
    }
    if (!r.ok) throw new Error(`chatgpt auth/session: HTTP ${r.status}`);
    const body = await r.json() as { accessToken?: string };
    if (!body.accessToken) throw new Error('chatgpt: no accessToken in session response');
    return body.accessToken;
  })();
}

/**
 * List all conversations. `token` is pre-fetched by the background worker via
 * getChatgptToken() to avoid one /api/auth/session call per conversation.
 */
export function listChatgpt(token: string): Promise<Array<{ id: string; updated_at?: string }>> {
  return (async () => {
    const out: Array<{ id: string; updated_at?: string }> = [];
    let offset = 0;
    const limit = 28;
    for (;;) {
      const r = await fetch(
        `/backend-api/conversations?offset=${offset}&limit=${limit}&order=updated`,
        { credentials: 'include', headers: { Authorization: `Bearer ${token}` } },
      );
      const cf = r.headers.get('cf-mitigated');
      const ct = r.headers.get('content-type') ?? '';
      if (cf || (!r.ok && ct.includes('text/html'))) {
        throw new Error(`CLOUDFLARE_CHALLENGE:${r.status}:conversations`);
      }
      if (!r.ok) throw new Error(`chatgpt conversations: HTTP ${r.status}`);
      const body = await r.json() as {
        items?: Array<{ id: string; update_time?: number }>;
        total?: number;
      };
      const items = body.items ?? [];
      for (const it of items) {
        out.push({
          id: it.id,
          updated_at: it.update_time != null ? String(it.update_time) : undefined,
        });
      }
      offset += items.length;
      if (items.length < limit) break;
      if (typeof body.total === 'number' && offset >= body.total) break;
    }
    return out;
  })();
}

/** Fetch a single conversation by ID. Accepts the pre-fetched bearer token. */
export function fetchChatgptConv(convId: string, token: string): Promise<unknown> {
  return (async () => {
    const r = await fetch(
      `/backend-api/conversation/${convId}`,
      { credentials: 'include', headers: { Authorization: `Bearer ${token}` } },
    );
    const cf = r.headers.get('cf-mitigated');
    const ct = r.headers.get('content-type') ?? '';
    if (cf || (!r.ok && ct.includes('text/html'))) {
      throw new Error(`CLOUDFLARE_CHALLENGE:${r.status}:conv/${convId}`);
    }
    if (!r.ok) throw new Error(`chatgpt conv ${convId}: HTTP ${r.status}`);
    return r.json();
  })();
}
