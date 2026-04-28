//! Runs in claude.ai tab context. Same-origin fetch against /api/.

export interface ClaudeOrg { uuid: string; name?: string }
export interface ClaudeConvSummary { uuid: string; name?: string; updated_at?: string }

export async function listOrgs(): Promise<ClaudeOrg[]> {
  const r = await fetch('/api/organizations', { credentials: 'include' });
  if (!r.ok) throw new Error(`claude orgs: HTTP ${r.status}`);
  return r.json() as Promise<ClaudeOrg[]>;
}

export async function listConversations(orgId: string): Promise<ClaudeConvSummary[]> {
  const r = await fetch(`/api/organizations/${orgId}/chat_conversations`, { credentials: 'include' });
  if (!r.ok) throw new Error(`claude conversations: HTTP ${r.status}`);
  return r.json() as Promise<ClaudeConvSummary[]>;
}

export async function fetchConversation(orgId: string, convId: string): Promise<unknown> {
  const r = await fetch(`/api/organizations/${orgId}/chat_conversations/${convId}`, { credentials: 'include' });
  if (!r.ok) throw new Error(`claude conv ${convId}: HTTP ${r.status}`);
  return r.json();
}
