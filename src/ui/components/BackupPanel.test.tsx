import { describe, expect, it, vi } from 'vitest';

vi.hoisted(() => {
  Object.defineProperty(globalThis, 'window', {
    value: { location: { search: '' } },
    configurable: true,
  });
});

import { BackupPanel } from './BackupPanel';
import { backupSnapshots, backupLoadState } from '../state/store';

// Helper: recursively collect all text content from a vnode tree.
function collectText(node: unknown): string[] {
  if (typeof node === 'string' || typeof node === 'number') return [String(node)];
  if (Array.isArray(node)) return node.flatMap(collectText);
  if (!node || typeof node !== 'object') return [];
  const vnode = node as { props?: { children?: unknown } };
  return collectText(vnode.props?.children);
}

describe('BackupPanel', () => {
  it('shows the empty state when there are no snapshots', () => {
    backupSnapshots.value = [];
    backupLoadState.value = 'idle';
    const vnode = BackupPanel({ onSnapshot: async () => {}, onReload: async () => {} });
    const texts = collectText(vnode);
    expect(texts.some(t => t.includes('No snapshots yet'))).toBe(true);
  });

  it('renders a row per snapshot', () => {
    backupSnapshots.value = [
      { snapshot_id: '2026-04-28T080000.000000Z', created_at: '2026-04-28T08:00:00Z', total_files: 12, total_bytes: 9876 },
      { snapshot_id: '2026-04-27T080000.000000Z', created_at: '2026-04-27T08:00:00Z', total_files: 10, total_bytes: 8000 },
    ];
    backupLoadState.value = 'idle';
    const vnode = BackupPanel({ onSnapshot: async () => {}, onReload: async () => {} });
    const texts = collectText(vnode);
    expect(texts).toContain('2026-04-28T080000.000000Z');
    expect(texts).toContain('2026-04-27T080000.000000Z');
  });

  it('calls onSnapshot then onReload when the button is clicked', async () => {
    backupSnapshots.value = [];
    backupLoadState.value = 'idle';
    const snap = vi.fn(async () => {});
    const reload = vi.fn(async () => {});
    const vnode = BackupPanel({ onSnapshot: snap, onReload: reload });

    // Find the "Snapshot now" button in the header children.
    const section = vnode as { props: { children: unknown[] } };
    const header = section.props.children[0] as { props: { children: unknown[] } };
    const btn = header.props.children[1] as { props: { onClick: () => void } };

    btn.props.onClick();
    await new Promise(r => setTimeout(r, 0));
    expect(snap).toHaveBeenCalledOnce();
    expect(reload).toHaveBeenCalled();
  });

  it('shows an error message when backupLoadState is error', () => {
    backupSnapshots.value = [];
    backupLoadState.value = 'error';
    const vnode = BackupPanel({ onSnapshot: async () => {}, onReload: async () => {} });
    const texts = collectText(vnode);
    expect(texts.some(t => t.includes('Failed to load snapshots'))).toBe(true);
  });

  it('renders a TableSkeleton (>=1 .skeleton element) while loading', () => {
    // Recursively expand function-component vnodes and collect
    // anything whose className contains 'skeleton'.
    function findSkeletons(node: unknown, depth = 0): unknown[] {
      if (depth > 30) return [];
      if (Array.isArray(node)) return node.flatMap(n => findSkeletons(n, depth));
      if (!node || typeof node !== 'object') return [];
      const vnode = node as {
        type?: unknown;
        props?: Record<string, unknown> & { class?: string; className?: string };
      };
      const props = vnode.props ?? {};
      const cls = (props['class'] ?? props['className'] ?? '') as string;
      const hits = typeof cls === 'string' && cls.includes('skeleton') ? [vnode] : [];
      const childrenHits = findSkeletons(props['children'], depth + 1);
      // Walk into function-component vnodes by invoking them with their props.
      const expanded: unknown[] =
        typeof vnode.type === 'function'
          ? findSkeletons(
              (vnode.type as (p: Record<string, unknown>) => unknown)(props),
              depth + 1,
            )
          : [];
      return [...hits, ...childrenHits, ...expanded];
    }

    backupSnapshots.value = [];
    backupLoadState.value = 'loading';
    const vnode = BackupPanel({ onSnapshot: async () => {}, onReload: async () => {} });
    const skeletons = findSkeletons(vnode);
    expect(skeletons.length).toBeGreaterThan(0);
  });
});
