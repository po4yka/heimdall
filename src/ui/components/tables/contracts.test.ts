import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

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

import {
  lastFilteredSessions,
  sessionsTableColumnVisibility,
  sessionsTablePagination,
} from '../../state/store';
import type {
  BranchSummary,
  EntrypointSummary,
  McpServerSummary,
  ModelAgg,
  ProjectAgg,
  ServiceTierSummary,
  SessionRow,
  ToolSummary,
  VersionSummary,
} from '../../state/types';
import { BranchTable } from './BranchTable';
import { DataTable } from './DataTable';
import { EntrypointTable } from './EntrypointTable';
import { McpSummaryTable } from './McpSummaryTable';
import { ModelCostTable } from './ModelCostTable';
import { ProjectCostTable } from './ProjectCostTable';
import { ServiceTiersTable } from './ServiceTiers';
import { SessionsTable } from './SessionsTable';
import { ToolUsageTable } from './ToolUsageTable';
import { VersionTable } from './VersionTable';
import { renderActionCell, renderCostCell, renderCreditsCell, renderNumberCell, renderTagCell } from './cells';

function collectText(node: unknown): string[] {
  if (typeof node === 'string' || typeof node === 'number') return [String(node)];
  if (Array.isArray(node)) return node.flatMap(collectText);
  if (!node || typeof node !== 'object') return [];
  const vnode = node as { props?: { children?: unknown } };
  return collectText(vnode.props?.['children']);
}

function sampleCellContext<T>(value: unknown, row: T) {
  return {
    getValue: () => value,
    row: { original: row },
  } as never;
}

const branchRows: BranchSummary[] = [
  { provider: 'claude', branch: 'main', sessions: 4, turns: 9, input: 10, output: 5, reasoning_output: 0, cost: 1.25 },
];
const entrypointRows: EntrypointSummary[] = [
  { provider: 'claude', entrypoint: 'cli', sessions: 2, turns: 5, input: 10, output: 5 },
];
const mcpRows: McpServerSummary[] = [
  { provider: 'codex', server: 'filesystem', tools_used: 3, invocations: 7, sessions_used: 2 },
];
const versionRows: VersionSummary[] = [
  { provider: 'claude', version: '1.2.3', turns: 5, sessions: 2, cost: 1.5, tokens: 100 },
];
const serviceTiers: ServiceTierSummary[] = [
  { provider: 'claude', service_tier: 'default', inference_geo: 'us', turns: 3 },
];
const toolRows: ToolSummary[] = [
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
const projectRows: ProjectAgg[] = [
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
const modelRows: ModelAgg[] = [
  {
    model: 'sonnet',
    input: 10,
    output: 5,
    cache_read: 1,
    cache_creation: 1,
    reasoning_output: 0,
    turns: 3,
    sessions: 1,
    cost: 1.25,
    is_billable: true,
    input_cost: 0.5,
    output_cost: 0.5,
    cache_read_cost: 0.1,
    cache_write_cost: 0.15,
    credits: 2,
  },
];
const sessionRows: SessionRow[] = [
  {
    session_id: 'session-1',
    provider: 'claude',
    project: 'alpha/heimdall',
    display_name: 'Alpha Heimdall',
    last: '2026-04-20 08:00',
    last_date: '2026-04-20',
    duration_min: 30,
    model: 'sonnet',
    turns: 3,
    input: 10,
    output: 5,
    cache_read: 1,
    cache_creation: 1,
    reasoning_output: 0,
    cost: 1.25,
    is_billable: true,
    pricing_version: 'v1',
    billing_mode: 'estimated_local',
    cost_confidence: 'medium',
    subagent_count: 1,
    subagent_turns: 0,
    title: 'Improve dashboard',
    cache_hit_ratio: 0.5,
    tokens_per_min: 20,
    credits: 2,
  },
];

beforeEach(() => {
  lastFilteredSessions.value = sessionRows;
  sessionsTablePagination.value = { pageIndex: 1, pageSize: 25 };
  sessionsTableColumnVisibility.value = { cost: false };
});

afterEach(() => {
  lastFilteredSessions.value = [];
  sessionsTablePagination.value = { pageIndex: 0, pageSize: 25 };
  sessionsTableColumnVisibility.value = {};
});

describe('table leaf contracts', () => {
  it('exports DataTable directly for reuse', () => {
    expect(typeof DataTable).toBe('function');
  });

  it('renders shared cell helpers with expected affordances', () => {
    expect(collectText(renderNumberCell(1200))).toContain('1.2K');
    expect(collectText(renderCreditsCell(2))).toContain('2.00');
    expect(collectText(renderCostCell(1.25, false))).toContain('n/a');
    expect((renderTagCell('sonnet') as { type: string }).type).toBe('span');
    expect((renderActionCell('Alpha', 'alpha/project', () => undefined) as { type: string }).type).toBe('button');
  });

  it('builds simple table wrappers with expected titles and columns', () => {
    const branchTable = BranchTable({ data: branchRows }) as { props: Record<string, unknown> };
    const entrypointTable = EntrypointTable({ data: entrypointRows }) as { props: Record<string, unknown> };
    const mcpTable = McpSummaryTable({ data: mcpRows }) as { props: Record<string, unknown> };
    const serviceTierTable = ServiceTiersTable({ data: serviceTiers }) as { props: Record<string, unknown> };
    const versionTable = VersionTable({ data: versionRows }) as { props: Record<string, unknown> };

    expect(branchTable.props['title']).toBe('Usage by Git Branch');
    expect(entrypointTable.props['title']).toBe('Usage by Entrypoint');
    expect(mcpTable.props['title']).toBe('MCP Server Usage');
    expect(serviceTierTable.props['title']).toBe('Service Tiers');
    expect(versionTable.props['title']).toBe('CLI Versions');

    const branchColumns = branchTable.props['columns'] as Array<{ cell: (ctx: unknown) => unknown }>;
    const branchCell = branchColumns[2]?.cell(sampleCellContext(4, branchRows[0]!)) as {
      props: Record<string, unknown>;
    };
    expect(branchCell.props['label']).toBe('4');
  });

  it('builds tool usage rows with mcp badges and error percentages', () => {
    const vnode = ToolUsageTable({ data: toolRows }) as { props: Record<string, unknown> };
    const columns = vnode.props['columns'] as Array<{ cell: (ctx: unknown) => unknown }>;
    const toolCell = columns[1]?.cell(sampleCellContext('read_file', toolRows[0]!));
    const errorCell = columns[6]?.cell(sampleCellContext(2, toolRows[0]!));

    expect(collectText(toolCell)).toContain('mcp');
    expect(collectText(toolCell)).toContain('read_file');
    expect(collectText(errorCell).join('')).toContain('2 (20.0%)');
  });

  it('builds project, model, and session tables with reusable action-cell contracts', () => {
    const projectTable = ProjectCostTable({
      byProject: projectRows,
      onExportCSV: () => undefined,
      onSelectProject: () => undefined,
    }) as { props: Record<string, unknown> };
    const modelTable = ModelCostTable({
      byModel: modelRows,
      onSelectModel: () => undefined,
    }) as { props: Record<string, unknown> };
    const sessionsTable = SessionsTable({
      onExportCSV: () => undefined,
      onSelectProject: () => undefined,
      onSelectModel: () => undefined,
    }) as { props: Record<string, unknown> };

    expect(projectTable.props['exportFn']).toBeTypeOf('function');
    expect(modelTable.props['costRows']).toBe(true);
    expect(sessionsTable.props['enableColumnVisibility']).toBe(true);
    expect(sessionsTable.props['paginationState']).toEqual({ pageIndex: 1, pageSize: 25 });
    expect(sessionsTable.props['columnVisibilityState']).toEqual({ cost: false });

    const projectColumns = projectTable.props['columns'] as Array<{ cell: (ctx: unknown) => unknown }>;
    const modelColumns = modelTable.props['columns'] as Array<{ cell: (ctx: unknown) => unknown }>;
    const sessionColumns = sessionsTable.props['columns'] as Array<{ id: string; cell: (ctx: unknown) => unknown }>;

    const projectCell = projectColumns[0]?.cell(sampleCellContext(projectRows[0]!.project, projectRows[0]!)) as { type: string };
    const modelCell = modelColumns[0]?.cell(sampleCellContext(modelRows[0]!.model, modelRows[0]!)) as { type: string };
    const sessionModelCell = sessionColumns.find(column => column.id === 'model')?.cell(
      sampleCellContext(sessionRows[0]!.model, sessionRows[0]!)
    ) as { type: string };

    expect(projectCell.type).toBe('button');
    expect(modelCell.type).toBe('button');
    expect(sessionModelCell.type).toBe('button');
    expect(
      collectText(
        sessionColumns.find(column => column.id === 'cost')?.cell(
          sampleCellContext(sessionRows[0]!.cost, sessionRows[0]!)
        )
      )
    ).toContain('$1.2500');
    expect(
      collectText(
        sessionColumns.find(column => column.id === 'credits')?.cell(
          sampleCellContext(sessionRows[0]!.credits, sessionRows[0]!)
        )
      )
    ).toContain('2.00');
  });

  it('returns null for empty leaf tables where the dashboard expects conditional rendering', () => {
    expect(BranchTable({ data: [] })).toBeNull();
    expect(EntrypointTable({ data: [] })).toBeNull();
    expect(McpSummaryTable({ data: [] })).toBeNull();
    expect(ServiceTiersTable({ data: [] })).toBeNull();
    expect(ToolUsageTable({ data: [] })).toBeNull();
    expect(VersionTable({ data: [] })).toBeNull();
  });
});
