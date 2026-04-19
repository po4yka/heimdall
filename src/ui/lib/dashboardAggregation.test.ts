import { describe, expect, it, vi } from 'vitest';
import {
  buildAggregations,
  buildWeeklyAgg,
  formatLocalDate,
  getRangeCutoff,
  weekLabelToWeekStart,
} from './dashboardAggregation';
import type {
  DailyModelRow,
  SessionRow,
  WeeklyModelRow,
} from '../state/types';

function makeDailyRow(
  overrides: Partial<DailyModelRow> & Pick<DailyModelRow, 'day' | 'model' | 'provider'>
): DailyModelRow {
  return {
    input: 0,
    output: 0,
    cache_read: 0,
    cache_creation: 0,
    reasoning_output: 0,
    turns: 0,
    cost: 0,
    input_cost: 0,
    output_cost: 0,
    cache_read_cost: 0,
    cache_write_cost: 0,
    credits: null,
    ...overrides,
    day: overrides.day,
    provider: overrides.provider,
    model: overrides.model,
  };
}

function makeSessionRow(
  overrides: Partial<SessionRow> & Pick<SessionRow, 'session_id' | 'provider' | 'project' | 'model'>
): SessionRow {
  return {
    session_id: overrides.session_id,
    provider: overrides.provider,
    project: overrides.project,
    display_name: overrides.display_name ?? overrides.project,
    last: overrides.last ?? '2026-04-19 12:00',
    last_date: overrides.last_date ?? '2026-04-19',
    duration_min: overrides.duration_min ?? 30,
    model: overrides.model,
    turns: overrides.turns ?? 0,
    input: overrides.input ?? 0,
    output: overrides.output ?? 0,
    cache_read: overrides.cache_read ?? 0,
    cache_creation: overrides.cache_creation ?? 0,
    reasoning_output: overrides.reasoning_output ?? 0,
    cost: overrides.cost ?? 0,
    is_billable: overrides.is_billable ?? true,
    pricing_version: overrides.pricing_version ?? 'v1',
    billing_mode: overrides.billing_mode ?? 'estimated_local',
    cost_confidence: overrides.cost_confidence ?? 'medium',
    subagent_count: overrides.subagent_count ?? 0,
    subagent_turns: overrides.subagent_turns ?? 0,
    title: overrides.title ?? null,
    cache_hit_ratio: overrides.cache_hit_ratio ?? 0,
    tokens_per_min: overrides.tokens_per_min ?? 0,
    credits: overrides.credits ?? null,
  };
}

describe('dashboardAggregation', () => {
  it('formats local dates and derives range cutoffs', () => {
    const date = new Date(2026, 3, 19);
    expect(formatLocalDate(date)).toBe('2026-04-19');

    vi.useFakeTimers();
    vi.setSystemTime(new Date('2026-04-19T10:00:00Z'));
    expect(getRangeCutoff('all')).toBeNull();
    expect(getRangeCutoff('7d')).toBe('2026-04-12');
    expect(getRangeCutoff('30d')).toBe('2026-03-20');
    vi.useRealTimers();
  });

  it('maps sqlite week labels to UTC week starts', () => {
    expect(weekLabelToWeekStart('2026-00').toISOString().slice(0, 10)).toBe('2026-01-01');
    expect(weekLabelToWeekStart('2026-01').toISOString().slice(0, 10)).toBe('2026-01-05');
  });

  it('aggregates daily, model, and project totals from filtered rows', () => {
    const dailyRows: DailyModelRow[] = [
      makeDailyRow({
        day: '2026-04-18',
        provider: 'claude',
        model: 'sonnet',
        input: 10,
        output: 5,
        cache_read: 2,
        cache_creation: 1,
        reasoning_output: 3,
        turns: 2,
        cost: 1.25,
        input_cost: 0.5,
        output_cost: 0.4,
        cache_read_cost: 0.2,
        cache_write_cost: 0.15,
      }),
      makeDailyRow({
        day: '2026-04-18',
        provider: 'claude',
        model: 'sonnet',
        input: 4,
        output: 6,
        cache_read: 1,
        cache_creation: 0,
        reasoning_output: 0,
        turns: 1,
        cost: 0.75,
        input_cost: 0.3,
        output_cost: 0.2,
        cache_read_cost: 0.1,
        cache_write_cost: 0.05,
        credits: 2,
      }),
    ];
    const sessions: SessionRow[] = [
      makeSessionRow({
        session_id: 's-1',
        provider: 'claude',
        project: 'heimdall',
        display_name: 'Heimdall UI',
        model: 'sonnet',
        turns: 3,
        input: 14,
        output: 11,
        cache_read: 3,
        cache_creation: 1,
        reasoning_output: 3,
        cost: 2,
        credits: 2,
      }),
    ];

    const result = buildAggregations(dailyRows, sessions);

    expect(result.daily).toEqual([
      {
        day: '2026-04-18',
        input: 14,
        output: 11,
        cache_read: 3,
        cache_creation: 1,
        reasoning_output: 3,
        cost: 2,
      },
    ]);
    expect(result.byModel[0]).toMatchObject({
      model: 'sonnet',
      sessions: 1,
      turns: 3,
      input: 14,
      output: 11,
      cost: 2,
      input_cost: 0.8,
      cache_write_cost: 0.2,
      credits: 2,
    });
    expect(result.byProject[0]).toMatchObject({
      project: 'heimdall',
      display_name: 'Heimdall UI',
      sessions: 1,
      turns: 3,
      cost: 2,
      credits: 2,
    });
    expect(result.totals).toMatchObject({
      sessions: 1,
      turns: 3,
      input: 14,
      output: 11,
      cache_read: 3,
      cache_creation: 1,
      reasoning_output: 3,
      cost: 2,
    });
    expect(result.confidenceBreakdown).toEqual([
      ['medium', { sessions: 1, cost: 2 }],
    ]);
    expect(result.billingModeBreakdown).toEqual([
      ['estimated_local', { sessions: 1, cost: 2 }],
    ]);
    expect(result.pricingVersions).toEqual(['v1']);
  });

  it('filters weekly aggregates to selected models and the requested range', () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2026-04-19T10:00:00Z'));

    const rows: WeeklyModelRow[] = [
      {
        week: '2026-15',
        model: 'sonnet',
        input_tokens: 10,
        output_tokens: 5,
        cache_read_tokens: 2,
        cache_creation_tokens: 1,
        reasoning_output_tokens: 0,
        cost_nanos: 100,
      },
      {
        week: '2026-10',
        model: 'sonnet',
        input_tokens: 99,
        output_tokens: 99,
        cache_read_tokens: 99,
        cache_creation_tokens: 99,
        reasoning_output_tokens: 99,
        cost_nanos: 999,
      },
      {
        week: '2026-15',
        model: 'haiku',
        input_tokens: 20,
        output_tokens: 10,
        cache_read_tokens: 0,
        cache_creation_tokens: 0,
        reasoning_output_tokens: 0,
        cost_nanos: 200,
      },
    ];

    const result = buildWeeklyAgg(rows, new Set(['sonnet']), '7d');

    expect(result).toEqual([
      {
        week: '2026-15',
        input: 10,
        output: 5,
        cache_read: 2,
        cache_creation: 1,
        reasoning_output: 0,
        cost_nanos: 100,
      },
    ]);

    vi.useRealTimers();
  });
});
