import { webConversations, companionHeartbeat, webConversationDetail, webConversationDetailLoading, loadWebConversation, type WebConversationSummary } from '../state/store';
import { esc } from '../lib/format';
import { WebConversationDetail } from './WebConversationDetail';

export interface WebCapturesPanelProps {
  onReload: () => Promise<void>;
}

function vendorCounts(rows: WebConversationSummary[]): Record<string, number> {
  const out: Record<string, number> = {};
  for (const r of rows) out[r.vendor] = (out[r.vendor] ?? 0) + 1;
  return out;
}

function relativeMinutes(iso: string): string {
  const ts = Date.parse(iso);
  if (Number.isNaN(ts)) return iso;
  const mins = Math.max(0, Math.round((Date.now() - ts) / 60000));
  if (mins < 1) return 'just now';
  if (mins < 60) return `${mins}m ago`;
  const hrs = Math.round(mins / 60);
  if (hrs < 48) return `${hrs}h ago`;
  return `${Math.round(hrs / 24)}d ago`;
}

function rowKey(r: WebConversationSummary): string {
  return `${r.vendor}/${r.conversation_id}`;
}

export function WebCapturesPanel({ onReload }: WebCapturesPanelProps) {
  const rows = webConversations.value;
  const heartbeat = companionHeartbeat.value;
  const counts = vendorCounts(rows);
  const detail = webConversationDetail.value;
  const detailLoading = webConversationDetailLoading.value;
  const selectedKey = detail ? `${detail.vendor}/${detail.conversation_id}` : null;

  function handleRowClick(r: WebConversationSummary) {
    const key = rowKey(r);
    if (selectedKey === key) {
      // Toggle: clicking the selected row clears the detail panel.
      webConversationDetail.value = null;
      return;
    }
    void loadWebConversation(r.vendor, r.conversation_id);
  }

  return (
    <section class="web-captures-panel">
      <header class="web-captures-panel-header">
        <h2>Web captures</h2>
        <button type="button" onClick={() => void onReload()}>Refresh</button>
      </header>
      {heartbeat && (
        <p class="web-captures-panel-heartbeat">
          Companion: connected{heartbeat.vendors_seen.length > 0 && (
            <> ({esc(heartbeat.vendors_seen.join(' + '))})</>
          )} · last seen {esc(relativeMinutes(heartbeat.last_seen_at))}
        </p>
      )}
      {!heartbeat && rows.length === 0 && (
        <p class="web-captures-panel-empty">
          No web captures yet. Install the Heimdall companion browser
          extension at <code>extensions/heimdall-companion/</code>, pair
          it with the token from <code>heimdall companion-token show</code>,
          and your claude.ai + chatgpt.com chats will appear here on the
          next sync.
        </p>
      )}
      {rows.length > 0 && (
        <>
          <p class="web-captures-panel-counts">
            {Object.entries(counts)
              .map(([vendor, n]) => `${vendor}: ${n}`)
              .join(' · ')}
          </p>
          <table class="data-table">
            <thead>
              <tr>
                <th>VENDOR</th>
                <th>CONVERSATION</th>
                <th>CAPTURED</th>
                <th>HISTORY</th>
              </tr>
            </thead>
            <tbody>
              {rows.map(r => {
                const key = rowKey(r);
                const isSelected = key === selectedKey;
                return (
                  <tr
                    key={key}
                    class={isSelected ? 'web-captures-panel-row--selected' : undefined}
                    style="cursor:pointer"
                    onClick={() => handleRowClick(r)}
                    role="button"
                    aria-expanded={isSelected}
                  >
                    <td>{esc(r.vendor)}</td>
                    <td><code>{esc(r.conversation_id)}</code></td>
                    <td>{esc(relativeMinutes(r.captured_at))}</td>
                    <td>{r.history_count}</td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </>
      )}
      {(detailLoading || detail) && <WebConversationDetail />}
    </section>
  );
}
