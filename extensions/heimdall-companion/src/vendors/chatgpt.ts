//! Runs in chatgpt.com tab context. Bearer auth obtained from /api/auth/session.

export interface ChatgptConvItem { id: string; title?: string; update_time?: number }

async function getAccessToken(): Promise<string> {
  const r = await fetch('/api/auth/session', { credentials: 'include' });
  if (!r.ok) throw new Error(`chatgpt auth/session: HTTP ${r.status}`);
  const body = await r.json() as { accessToken?: string };
  if (!body.accessToken) throw new Error('chatgpt: no accessToken in session response');
  return body.accessToken;
}

export async function listConversations(pageSize = 28): Promise<ChatgptConvItem[]> {
  const token = await getAccessToken();
  const all: ChatgptConvItem[] = [];
  let offset = 0;
  for (;;) {
    const r = await fetch(`/backend-api/conversations?offset=${offset}&limit=${pageSize}&order=updated`, {
      credentials: 'include',
      headers: { Authorization: `Bearer ${token}` },
    });
    if (!r.ok) throw new Error(`chatgpt conversations: HTTP ${r.status}`);
    const body = await r.json() as { items?: ChatgptConvItem[]; total?: number };
    const items = body.items ?? [];
    all.push(...items);
    offset += items.length;
    if (items.length < pageSize) break;
    if (typeof body.total === 'number' && offset >= body.total) break;
  }
  return all;
}

export async function fetchConversation(convId: string): Promise<unknown> {
  const token = await getAccessToken();
  const r = await fetch(`/backend-api/conversation/${convId}`, {
    credentials: 'include',
    headers: { Authorization: `Bearer ${token}` },
  });
  if (!r.ok) throw new Error(`chatgpt conv ${convId}: HTTP ${r.status}`);
  return r.json();
}
