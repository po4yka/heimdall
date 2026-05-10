/**
 * Client-side fallback extractor for Claude conversation payloads.
 *
 * Used only when `payload.heimdall_extracted.artifacts` is absent
 * (i.e. the conversation was captured before server-side extraction was deployed).
 * Mirrors the logic in src/archive/payload/anthropic.rs.
 */

export interface ArtifactBlock {
  message_id: string;
  identifier: string;
  type: string;
  language: string;
  title: string;
  body: string;
  byte_range: [number, number];
}

const ARTIFACT_RE = /(?:<antartifact([^>]*)>([\s\S]*?)<\/antartifact>)/g;
const ATTR_RE = /([\w-]+)="([^"]*)"/g;

function parseAttrs(attrs: string): Record<string, string> {
  const out: Record<string, string> = {};
  let m: RegExpExecArray | null;
  ATTR_RE.lastIndex = 0;
  while ((m = ATTR_RE.exec(attrs)) !== null) {
    out[m[1]!] = m[2]!;
  }
  return out;
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function extractArtifacts(payload: Record<string, any>): ArtifactBlock[] {
  // Prefer pre-computed server-side extraction.
  const precomputed = payload?.['heimdall_extracted']?.artifacts;
  if (Array.isArray(precomputed)) return precomputed as ArtifactBlock[];

  const artifacts: ArtifactBlock[] = [];
  const messages: unknown[] = Array.isArray(payload?.['chat_messages']) ? payload['chat_messages'] : [];

  for (const msg of messages) {
    if (typeof msg !== 'object' || msg === null) continue;
    const m = msg as Record<string, unknown>;
    const msgId = typeof m['uuid'] === 'string' ? m['uuid'] : '';
    const text = typeof m['text'] === 'string' ? m['text'] : '';
    if (!text) continue;

    ARTIFACT_RE.lastIndex = 0;
    let cap: RegExpExecArray | null;
    while ((cap = ARTIFACT_RE.exec(text)) !== null) {
      const attrs = parseAttrs(cap[1] ?? '');
      const start = cap.index;
      const end = start + cap[0].length;
      artifacts.push({
        message_id: msgId,
        identifier: attrs['identifier'] ?? '',
        type: attrs['type'] ?? '',
        language: attrs['language'] ?? '',
        title: attrs['title'] ?? '',
        body: cap[2] ?? '',
        byte_range: [start, end],
      });
    }
  }
  return artifacts;
}
