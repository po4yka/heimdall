/**
 * Client-side fallback extractor for ChatGPT conversation payloads.
 *
 * Used only when `payload.heimdall_extracted` is absent or incomplete.
 * Mirrors the logic in src/archive/payload/openai.rs.
 */

export interface BrowsingStep {
  node_id: string;
  query: string;
  result: unknown;
  results: unknown[];
}

export interface Citation {
  marker: string;
  index: string;
  anchor_text: string;
  message_id: string;
  url?: string;
}

const CITATION_RE = /【(\d+)†(L\d+(?:-L\d+)?)】/g;

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function extractBrowsingSteps(payload: Record<string, any>): BrowsingStep[] {
  const precomputed = payload?.heimdall_extracted?.browsing_steps;
  if (Array.isArray(precomputed)) return precomputed as BrowsingStep[];

  const steps: BrowsingStep[] = [];
  const mapping = payload?.mapping;
  if (!mapping || typeof mapping !== 'object') return steps;

  for (const [nodeId, node] of Object.entries(mapping)) {
    const n = node as Record<string, unknown>;
    const msg = n['message'];
    if (!msg || typeof msg !== 'object') continue;
    const m = msg as Record<string, unknown>;
    const content = m['content'];
    if (!content || typeof content !== 'object') continue;
    const c = content as Record<string, unknown>;
    if (c['content_type'] !== 'tether_browsing_display') continue;

    steps.push({
      node_id: nodeId,
      query: typeof c['tether_id'] === 'string' ? c['tether_id'] : '',
      result: c['result'] ?? null,
      results: Array.isArray(c['results']) ? c['results'] : [],
    });
  }
  return steps;
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function extractCitations(payload: Record<string, any>): Citation[] {
  const precomputed = payload?.heimdall_extracted?.citations;
  if (Array.isArray(precomputed)) return precomputed as Citation[];

  const citations: Citation[] = [];
  const mapping = payload?.mapping;
  if (!mapping || typeof mapping !== 'object') return citations;

  for (const node of Object.values(mapping)) {
    const n = node as Record<string, unknown>;
    const msg = n['message'];
    if (!msg || typeof msg !== 'object') continue;
    const m = msg as Record<string, unknown>;
    const author = m['author'] as Record<string, unknown> | undefined;
    if (author?.['role'] !== 'assistant') continue;

    const msgId = typeof m['id'] === 'string' ? m['id'] : '';
    const content = m['content'] as Record<string, unknown> | undefined;
    const parts = content?.['parts'];
    if (!Array.isArray(parts)) continue;

    for (const part of parts) {
      if (typeof part !== 'string') continue;
      CITATION_RE.lastIndex = 0;
      let cap: RegExpExecArray | null;
      while ((cap = CITATION_RE.exec(part)) !== null) {
        citations.push({
          marker: cap[0],
          index: cap[1] ?? '',
          anchor_text: cap[2] ?? '',
          message_id: msgId,
        });
      }
    }
  }
  return citations;
}
