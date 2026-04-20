import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

vi.hoisted(() => {
  Object.defineProperty(globalThis, 'window', {
    value: { location: { search: '' } },
    configurable: true,
  });
});

import { statusByPlacement } from '../state/store';
import { InlineStatus } from './InlineStatus';

const EMPTY_STATUS = {
  global: null,
  'rate-windows': null,
  rescan: null,
  'header-refresh': null,
  'agent-status': null,
  'community-signal': null,
};

function collectText(node: unknown): string[] {
  if (typeof node === 'string' || typeof node === 'number') return [String(node)];
  if (Array.isArray(node)) return node.flatMap(collectText);
  if (!node || typeof node !== 'object') return [];
  const vnode = node as { props?: { children?: unknown } };
  return collectText(vnode.props?.children);
}

beforeEach(() => {
  statusByPlacement.value = { ...EMPTY_STATUS };
});

afterEach(() => {
  statusByPlacement.value = { ...EMPTY_STATUS };
});

describe('InlineStatus', () => {
  it('renders alert content and dismisses non-loading statuses', () => {
    statusByPlacement.value = {
      ...EMPTY_STATUS,
      global: { kind: 'error', message: 'Sync failed' },
    };

    const vnode = InlineStatus({ placement: 'global' }) as { props: Record<string, unknown> };
    const children = vnode.props['children'] as unknown[];
    const dismissButton = children[1] as { props: { onClick: () => void } };

    expect(vnode.props['role']).toBe('alert');
    expect(collectText(vnode)).toContain('[ERROR: Sync failed]');

    dismissButton.props.onClick();
    expect(statusByPlacement.value.global).toBeNull();
  });

  it('renders inline loading statuses without a dismiss control', () => {
    statusByPlacement.value = {
      ...EMPTY_STATUS,
      'header-refresh': { kind: 'loading', message: 'Refreshing' },
    };

    const vnode = InlineStatus({ placement: 'header-refresh', inline: true }) as {
      props: Record<string, unknown>;
    };
    const children = vnode.props['children'] as unknown[];

    expect(vnode.props['role']).toBe('status');
    expect(vnode.props['style']).toMatchObject({
      display: 'inline-flex',
      border: 'none',
      background: 'transparent',
    });
    expect(children.filter(Boolean)).toHaveLength(1);
    expect(collectText(vnode)).toContain('[LOADING: Refreshing]');
  });
});
