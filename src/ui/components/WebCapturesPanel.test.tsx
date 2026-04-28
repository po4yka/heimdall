import { describe, expect, it, vi } from 'vitest';

vi.hoisted(() => {
  Object.defineProperty(globalThis, 'window', {
    value: { location: { search: '' } },
    configurable: true,
  });
});

import { WebCapturesPanel } from './WebCapturesPanel';
import { webConversations, companionHeartbeat } from '../state/store';

// Helper: recursively collect all text content from a vnode tree.
function collectText(node: unknown): string[] {
  if (typeof node === 'string' || typeof node === 'number') return [String(node)];
  if (Array.isArray(node)) return node.flatMap(collectText);
  if (!node || typeof node !== 'object') return [];
  const vnode = node as { props?: { children?: unknown } };
  return collectText(vnode.props?.children);
}

describe('WebCapturesPanel', () => {
  it('shows the empty state when there are no captures and no heartbeat', () => {
    webConversations.value = [];
    companionHeartbeat.value = null;
    const vnode = WebCapturesPanel({ onReload: async () => {} });
    const texts = collectText(vnode);
    expect(texts.some(t => t.includes('No web captures yet'))).toBe(true);
  });

  it('renders a heartbeat strip when present', () => {
    webConversations.value = [];
    companionHeartbeat.value = {
      last_seen_at: new Date().toISOString(),
      extension_version: '0.1.0',
      user_agent: 'Mozilla/5.0',
      vendors_seen: ['claude.ai', 'chatgpt.com'],
    };
    const vnode = WebCapturesPanel({ onReload: async () => {} });
    const texts = collectText(vnode);
    expect(texts.some(t => t.includes('Companion: connected'))).toBe(true);
    expect(texts.some(t => t.includes('claude.ai + chatgpt.com'))).toBe(true);
  });

  it('renders one row per capture', () => {
    webConversations.value = [
      { vendor: 'claude.ai', conversation_id: 'abc', captured_at: '2026-04-28T12:00:00Z', history_count: 2 },
      { vendor: 'chatgpt.com', conversation_id: 'xyz', captured_at: '2026-04-28T11:00:00Z', history_count: 0 },
    ];
    companionHeartbeat.value = null;
    const vnode = WebCapturesPanel({ onReload: async () => {} });
    const texts = collectText(vnode);
    expect(texts.some(t => t.includes('abc'))).toBe(true);
    expect(texts.some(t => t.includes('xyz'))).toBe(true);
  });

  it('calls onReload when the refresh button is clicked', async () => {
    webConversations.value = [];
    companionHeartbeat.value = null;
    const reload = vi.fn(async () => {});
    const vnode = WebCapturesPanel({ onReload: reload });

    // Find the "Refresh" button in the header children.
    const section = vnode as { props: { children: unknown[] } };
    const header = section.props.children[0] as { props: { children: unknown[] } };
    const btn = header.props.children[1] as { props: { onClick: () => void } };

    btn.props.onClick();
    await new Promise(r => setTimeout(r, 0));
    expect(reload).toHaveBeenCalled();
  });
});
