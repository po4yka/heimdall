import { describe, expect, it } from 'vitest';
import type { ContextWindowResponse } from '../state/types';
import { ContextWindowCard } from './ContextWindowCard';

function collectText(node: unknown): string[] {
  if (typeof node === 'string' || typeof node === 'number') return [String(node)];
  if (Array.isArray(node)) return node.flatMap(collectText);
  if (!node || typeof node !== 'object') return [];
  const vnode = node as { props?: { children?: unknown } };
  return collectText(vnode.props?.children);
}

describe('ContextWindowCard', () => {
  it('returns null for disabled or incomplete payloads', () => {
    expect(ContextWindowCard({ data: null })).toBeNull();
    expect(
      ContextWindowCard({ data: { enabled: false } as unknown as ContextWindowResponse })
    ).toBeNull();
    expect(
      ContextWindowCard({
        data: {
          enabled: true,
          total_input_tokens: 10,
          context_window_size: 0,
        } as unknown as ContextWindowResponse,
      })
    ).toBeNull();
  });

  it('renders usage, severity, and progress data for active context windows', () => {
    const data = {
      enabled: true,
      total_input_tokens: 15_000,
      context_window_size: 20_000,
      pct: 0.75,
      severity: 'warn',
    } as unknown as ContextWindowResponse;

    const vnode = ContextWindowCard({ data }) as { props: Record<string, unknown> };
    const progress = (vnode.props['children'] as unknown[])[1] as { props: Record<string, unknown> };
    const text = collectText(vnode).join(' ');

    expect(text).toContain('15.0K');
    expect(text).toContain('20.0K');
    expect(text).toContain('[WARN]');
    expect(progress.props['aria-label']).toBe('Context window usage');
  });
});
