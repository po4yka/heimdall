// Self-contained — NO imports. These functions are serialized and injected into a
// claude.ai tab via chrome.scripting.executeScript({ func }). Any reference to a
// symbol defined outside the function body will be a runtime ReferenceError in
// the target tab. All helpers must be inner function declarations.

export function listClaude(): Promise<Array<{ id: string; updated_at?: string }>> {
  function guardResp(r: Response, ctx: string): void {
    const cf = r.headers.get('cf-mitigated');
    const ct = r.headers.get('content-type') ?? '';
    if (cf || (!r.ok && ct.includes('text/html'))) {
      throw new Error(`CLOUDFLARE_CHALLENGE:${r.status}:${ctx}`);
    }
    if (!r.ok) throw new Error(`claude.ai ${ctx}: HTTP ${r.status}`);
  }
  return (async () => {
    const orgsResp = await fetch('/api/organizations', { credentials: 'include' });
    guardResp(orgsResp, 'orgs');
    const orgs = await orgsResp.json() as Array<{ uuid: string }>;

    const out: Array<{ id: string; updated_at?: string }> = [];
    for (const o of orgs) {
      const convsResp = await fetch(
        `/api/organizations/${o.uuid}/chat_conversations`,
        { credentials: 'include' },
      );
      guardResp(convsResp, `conversations/${o.uuid}`);
      const convs = await convsResp.json() as Array<{ uuid: string; updated_at?: string }>;
      for (const c of convs) {
        out.push({ id: `${o.uuid}/${c.uuid}`, updated_at: c.updated_at });
      }
    }
    return out;
  })();
}

export function fetchClaudeConv(combinedId: string): Promise<unknown> {
  function guardResp(r: Response, ctx: string): void {
    const cf = r.headers.get('cf-mitigated');
    const ct = r.headers.get('content-type') ?? '';
    if (cf || (!r.ok && ct.includes('text/html'))) {
      throw new Error(`CLOUDFLARE_CHALLENGE:${r.status}:${ctx}`);
    }
    if (!r.ok) throw new Error(`claude.ai ${ctx}: HTTP ${r.status}`);
  }
  return (async () => {
    const [orgId, convId] = combinedId.split('/', 2) as [string, string];
    const r = await fetch(
      `/api/organizations/${orgId}/chat_conversations/${convId}`,
      { credentials: 'include' },
    );
    guardResp(r, `conv/${convId}`);
    return r.json();
  })();
}
