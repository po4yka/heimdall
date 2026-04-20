import { describe, expect, it } from 'vitest';
import type { SubagentSummary as SubagentSummaryType } from '../state/types';
import { SubagentSummary } from './SubagentSummary';

function collectText(node: unknown): string[] {
  if (typeof node === 'string' || typeof node === 'number') return [String(node)];
  if (Array.isArray(node)) return node.flatMap(collectText);
  if (!node || typeof node !== 'object') return [];
  const vnode = node as { props?: { children?: unknown } };
  return collectText(vnode.props?.children);
}

describe('SubagentSummary', () => {
  it('returns null when no subagent turns were recorded', () => {
    const summary = {
      parent_turns: 2,
      parent_input: 100,
      parent_output: 50,
      subagent_turns: 0,
      subagent_input: 0,
      subagent_output: 0,
      unique_agents: 0,
    } as SubagentSummaryType;

    expect(SubagentSummary({ summary })).toBeNull();
  });

  it('renders parent/subagent token shares when activity is present', () => {
    const summary = {
      parent_turns: 2,
      parent_input: 100,
      parent_output: 50,
      subagent_turns: 3,
      subagent_input: 50,
      subagent_output: 150,
      unique_agents: 2,
    } as SubagentSummaryType;

    const text = collectText(SubagentSummary({ summary })).join(' ');

    expect(text).toContain('Subagent Breakdown');
    expect(text).toContain('Parent:');
    expect(text).toContain('Subagent:');
    expect(text).toContain('unique agents');
    expect(text).toContain('33.3');
    expect(text).toContain('75.0');
  });
});
