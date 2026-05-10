import { webConversationDetail, webConversationDetailLoading } from '../state/store';
import { esc } from '../lib/format';
import { extractArtifacts } from '../lib/payload/anthropic';
import { extractBrowsingSteps, extractCitations } from '../lib/payload/openai';
import { highlight } from '../lib/highlight';

export function WebConversationDetail() {
  const loading = webConversationDetailLoading.value;
  const conv = webConversationDetail.value;

  if (loading) {
    return <p class="web-conv-detail-loading">Loading…</p>;
  }
  if (!conv) return null;

  const vendor = conv.vendor;
  const payload = conv.payload ?? {};

  const artifacts = vendor === 'claude.ai' ? extractArtifacts(payload) : [];
  const browsingSteps = vendor === 'chatgpt.com' ? extractBrowsingSteps(payload) : [];
  const citations = vendor === 'chatgpt.com' ? extractCitations(payload) : [];

  const hasContent = artifacts.length > 0 || browsingSteps.length > 0 || citations.length > 0;

  return (
    <section class="web-conv-detail">
      <header class="web-conv-detail-header">
        <span class="web-conv-detail-vendor">{esc(vendor)}</span>
        <code class="web-conv-detail-id">{esc(conv.conversation_id)}</code>
        <span class="web-conv-detail-ts">{esc(conv.captured_at)}</span>
      </header>

      {!hasContent && (
        <p class="web-conv-detail-empty">
          No artifacts, citations, or browsing steps found in this conversation.
        </p>
      )}

      {artifacts.length > 0 && (
        <div class="web-conv-detail-section">
          <h3>Artifacts ({artifacts.length})</h3>
          {artifacts.map((a, i) => {
            const lang = a.language || (a.type === 'text/html' ? 'html' : '');
            const highlighted = highlight(a.body, lang);
            return (
              <details key={`art-${i}`} class="web-conv-detail-artifact">
                <summary>
                  <span class="web-conv-detail-artifact-type">{esc(a.type || 'unknown')}</span>
                  {lang && <span class="web-conv-detail-artifact-lang">{esc(lang)}</span>}
                  {' '}
                  <strong>{esc(a.title || a.identifier || '(untitled)')}</strong>
                </summary>
                <pre class="web-conv-detail-artifact-body"><code
                  dangerouslySetInnerHTML={{ __html: highlighted }}
                /></pre>
              </details>
            );
          })}
        </div>
      )}

      {citations.length > 0 && (
        <div class="web-conv-detail-section">
          <h3>Citations ({citations.length})</h3>
          <table class="data-table">
            <thead>
              <tr>
                <th>MARKER</th>
                <th>URL</th>
                <th>TITLE</th>
              </tr>
            </thead>
            <tbody>
              {citations.map((c, i) => (
                <tr key={`cit-${i}`}>
                  <td><code>{esc(c.marker)}</code></td>
                  <td>
                    {c.url
                      ? <a href={esc(c.url)} target="_blank" rel="noopener noreferrer">{esc(c.url)}</a>
                      : <span class="web-conv-detail-unresolved">(URL unresolved — re-capture from a logged-in tab to fill)</span>
                    }
                  </td>
                  <td>{esc((c as { title?: string }).title ?? c.anchor_text)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {browsingSteps.length > 0 && (
        <div class="web-conv-detail-section">
          <h3>Browsing steps ({browsingSteps.length})</h3>
          {browsingSteps.map((s, i) => {
            const results = Array.isArray(s.results) ? s.results : [];
            return (
              <details key={`step-${i}`} class="web-conv-detail-browsing-step">
                <summary>
                  {esc(s.query || s.node_id || `Step ${i + 1}`)}
                  {results.length > 0 && <span class="web-conv-detail-count"> ({results.length} result{results.length !== 1 ? 's' : ''})</span>}
                </summary>
                {results.length > 0 && (
                  <ul class="web-conv-detail-results">
                    {results.map((r, j) => {
                      const res = r as Record<string, unknown>;
                      const title = typeof res['title'] === 'string' ? res['title'] : '';
                      const url = typeof res['url'] === 'string' ? res['url'] : '';
                      const snippet = typeof res['snippet'] === 'string' ? res['snippet'] : '';
                      return (
                        <li key={`res-${j}`}>
                          {url
                            ? <a href={esc(url)} target="_blank" rel="noopener noreferrer">{esc(title || url)}</a>
                            : <strong>{esc(title)}</strong>
                          }
                          {snippet && <p class="web-conv-detail-snippet">{esc(snippet)}</p>}
                        </li>
                      );
                    })}
                  </ul>
                )}
              </details>
            );
          })}
        </div>
      )}
    </section>
  );
}
