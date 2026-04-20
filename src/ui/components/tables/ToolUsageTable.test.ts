import { describe, expect, it, vi } from 'vitest';

vi.hoisted(() => {
  Object.defineProperty(globalThis, 'window', {
    value: { location: { pathname: '/dashboard', search: '' } },
    configurable: true,
  });
  Object.defineProperty(globalThis, 'history', {
    value: { replaceState: vi.fn() },
    configurable: true,
  });
});

import type { ToolSummary } from '../../state/types';
import { ToolUsageTable } from './ToolUsageTable';

function collectText(node: unknown): string[] {
  if (typeof node === 'string' || typeof node === 'number') return [String(node)];
  if (Array.isArray(node)) return node.flatMap(collectText);
  if (!node || typeof node !== 'object') return [];
  const vnode = node as { props?: { children?: unknown } };
  return collectText(vnode.props?.['children']);
}

describe('ToolUsageTable', () => {
  it('builds tool and error cells from tool summary rows', () => {
    const rows: ToolSummary[] = [
      {
        provider: 'claude',
        tool_name: 'read_file',
        category: 'mcp',
        mcp_server: 'filesystem',
        invocations: 10,
        turns_used: 5,
        sessions_used: 2,
        errors: 2,
      },
    ];

    const vnode = ToolUsageTable({ data: rows }) as { props: Record<string, unknown> };
    const columns = vnode.props['columns'] as Array<{ cell: (ctx: unknown) => unknown }>;
    const toolCell = columns[1]?.cell({
      getValue: () => rows[0]!.tool_name,
      row: { original: rows[0]! },
    } as never);
    const errorCell = columns[6]?.cell({
      getValue: () => rows[0]!.errors,
      row: { original: rows[0]! },
    } as never);

    expect(vnode.props['title']).toBe('Tool Usage');
    expect(collectText(toolCell)).toContain('mcp');
    expect(collectText(toolCell)).toContain('read_file');
    expect(collectText(errorCell).join('')).toContain('2 (20.0%)');
  });
});
