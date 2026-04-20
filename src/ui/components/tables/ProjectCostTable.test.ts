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

vi.mock('preact/hooks', async () => {
  const actual = await vi.importActual<typeof import('preact/hooks')>('preact/hooks');
  return {
    ...actual,
    useMemo: <T>(factory: () => T) => factory(),
  };
});

import type { ProjectAgg } from '../../state/types';
import { ProjectCostTable } from './ProjectCostTable';

describe('ProjectCostTable', () => {
  it('passes table columns and export wiring through to DataTable', () => {
    const rows: ProjectAgg[] = [
      {
        project: 'alpha/heimdall',
        display_name: 'Alpha Heimdall',
        input: 10,
        output: 5,
        cache_read: 1,
        cache_creation: 0,
        reasoning_output: 0,
        turns: 3,
        sessions: 1,
        cost: 1.25,
        credits: 2,
      },
    ];

    const vnode = ProjectCostTable({
      byProject: rows,
      onExportCSV: () => undefined,
      onSelectProject: () => undefined,
    }) as { props: Record<string, unknown> };
    const columns = vnode.props['columns'] as Array<{ cell: (ctx: unknown) => unknown }>;
    const projectCell = columns[0]?.cell({
      getValue: () => rows[0]!.project,
      row: { original: rows[0]! },
    } as never) as { type: string };

    expect(vnode.props['title']).toBe('Cost by Project');
    expect(vnode.props['exportFn']).toBeTypeOf('function');
    expect(projectCell.type).toBe('button');
  });
});
