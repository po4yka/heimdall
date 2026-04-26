import type {
  DailyAgg,
  DailyModelRow,
  ModelAgg,
  ProjectAgg,
  RangeKey,
  SessionRow,
  Totals,
  WeeklyAgg,
  WeeklyModelRow,
} from '../state/types';

export interface SessionCostBreakdown {
  sessions: number;
  cost: number;
}

export interface DashboardAggregations {
  daily: DailyAgg[];
  byModel: ModelAgg[];
  byProject: ProjectAgg[];
  totals: Totals;
  confidenceBreakdown: Array<[string, SessionCostBreakdown]>;
  billingModeBreakdown: Array<[string, SessionCostBreakdown]>;
  pricingVersions: string[];
}

export function formatLocalDate(date: Date): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
}

export function getRangeCutoff(range: RangeKey): string | null {
  if (range === 'all') return null;
  const days = range === '7d' ? 7 : range === '30d' ? 30 : 90;
  const date = new Date();
  date.setDate(date.getDate() - days);
  return formatLocalDate(date);
}

export function weekLabelToWeekStart(label: string): Date {
  const [yearStr, weekStr] = label.split('-');
  const year = parseInt(yearStr ?? '', 10);
  const week = parseInt(weekStr ?? '', 10);
  if (!Number.isFinite(year) || !Number.isFinite(week)) return new Date(NaN);

  const jan1 = new Date(Date.UTC(year, 0, 1));
  if (week === 0) return jan1;

  const jan1Dow = jan1.getUTCDay();
  const daysToFirstMon = (8 - jan1Dow) % 7;
  const firstMondayUtc = new Date(Date.UTC(year, 0, 1 + daysToFirstMon));
  return new Date(firstMondayUtc.getTime() + (week - 1) * 7 * 86400 * 1000);
}

export function buildWeeklyAgg(
  rows: WeeklyModelRow[],
  selectedModels: Set<string>,
  range: RangeKey
): WeeklyAgg[] {
  if (!rows.length) return [];

  const cutoff = getRangeCutoff(range);
  const weekMap: Record<string, WeeklyAgg> = {};

  for (const row of rows) {
    if (!selectedModels.has(row.model)) continue;
    if (cutoff) {
      const weekStart = weekLabelToWeekStart(row.week);
      if (Number.isNaN(weekStart.getTime())) continue;
      if (weekStart.toISOString().slice(0, 10) < cutoff) continue;
    }

    const weekly = weekMap[row.week] ?? (weekMap[row.week] = {
      week: row.week,
      input: 0,
      output: 0,
      cache_read: 0,
      cache_creation: 0,
      reasoning_output: 0,
      cost_nanos: 0,
    });
    weekly.input += row.input_tokens;
    weekly.output += row.output_tokens;
    weekly.cache_read += row.cache_read_tokens;
    weekly.cache_creation += row.cache_creation_tokens;
    weekly.reasoning_output += row.reasoning_output_tokens;
    weekly.cost_nanos += row.cost_nanos;
  }

  return Object.values(weekMap).sort((left, right) => left.week.localeCompare(right.week));
}

export function confidenceRank(confidence: string): number {
  switch (confidence) {
    case 'low':
      return 0;
    case 'medium':
      return 1;
    case 'high':
      return 2;
    default:
      return 3;
  }
}

export function buildAggregations(
  filteredDaily: DailyModelRow[],
  filteredSessions: SessionRow[]
): DashboardAggregations {
  const dailyMap: Record<string, DailyAgg> = {};
  for (const row of filteredDaily) {
    const daily = dailyMap[row.day] ?? (dailyMap[row.day] = {
      day: row.day,
      input: 0,
      output: 0,
      cache_read: 0,
      cache_creation: 0,
      reasoning_output: 0,
      cost: 0,
    });
    daily.input += row.input;
    daily.output += row.output;
    daily.cache_read += row.cache_read;
    daily.cache_creation += row.cache_creation;
    daily.reasoning_output += row.reasoning_output;
    daily.cost += row.cost;
  }
  const daily = Object.values(dailyMap).sort((left, right) => left.day.localeCompare(right.day));

  const modelMap: Record<string, ModelAgg> = {};
  for (const row of filteredDaily) {
    const model = modelMap[row.model] ?? (modelMap[row.model] = {
      model: row.model,
      input: 0,
      output: 0,
      cache_read: 0,
      cache_creation: 0,
      reasoning_output: 0,
      turns: 0,
      sessions: 0,
      cost: 0,
      is_billable: row.cost > 0,
      input_cost: 0,
      output_cost: 0,
      cache_read_cost: 0,
      cache_write_cost: 0,
      credits: null,
    });

    model.input += row.input;
    model.output += row.output;
    model.cache_read += row.cache_read;
    model.cache_creation += row.cache_creation;
    model.reasoning_output += row.reasoning_output;
    model.turns += row.turns;
    model.cost += row.cost;
    if (row.cost > 0) model.is_billable = true;
    model.input_cost = (model.input_cost ?? 0) + (row.input_cost ?? 0);
    model.output_cost = (model.output_cost ?? 0) + (row.output_cost ?? 0);
    model.cache_read_cost = (model.cache_read_cost ?? 0) + (row.cache_read_cost ?? 0);
    model.cache_write_cost = (model.cache_write_cost ?? 0) + (row.cache_write_cost ?? 0);
    if (row.credits != null) {
      model.credits = (model.credits ?? 0) + row.credits;
    }
  }

  for (const session of filteredSessions) {
    const model = modelMap[session.model];
    if (model) model.sessions += 1;
  }
  const byModel = Object.values(modelMap).sort(
    (left, right) => (right.input + right.output) - (left.input + left.output)
  );

  const projectMap: Record<string, ProjectAgg> = {};
  for (const session of filteredSessions) {
    const project = projectMap[session.project] ?? (projectMap[session.project] = {
      project: session.project,
      display_name: session.display_name || session.project,
      input: 0,
      output: 0,
      cache_read: 0,
      cache_creation: 0,
      reasoning_output: 0,
      turns: 0,
      sessions: 0,
      cost: 0,
      credits: null,
    });

    project.input += session.input;
    project.output += session.output;
    project.cache_read += session.cache_read;
    project.cache_creation += session.cache_creation;
    project.reasoning_output += session.reasoning_output;
    project.turns += session.turns;
    project.sessions += 1;
    project.cost += session.cost;
    if (session.credits != null) {
      project.credits = (project.credits ?? 0) + session.credits;
    }
  }
  const byProject = Object.values(projectMap).sort(
    (left, right) => (right.input + right.output) - (left.input + left.output)
  );

  const totals: Totals = {
    sessions: filteredSessions.length,
    turns: filteredSessions.reduce((sum, session) => sum + session.turns, 0),
    input: filteredSessions.reduce((sum, session) => sum + session.input, 0),
    output: filteredSessions.reduce((sum, session) => sum + session.output, 0),
    cache_read: filteredSessions.reduce((sum, session) => sum + session.cache_read, 0),
    cache_creation: filteredSessions.reduce((sum, session) => sum + session.cache_creation, 0),
    reasoning_output: filteredSessions.reduce((sum, session) => sum + session.reasoning_output, 0),
    cost: filteredSessions.reduce((sum, session) => sum + session.cost, 0),
    credits: filteredSessions.reduce<number | null>((sum, session) => {
      if (session.credits == null) return sum;
      return (sum ?? 0) + session.credits;
    }, null),
  };

  const confidenceBreakdown = Object.entries(
    filteredSessions.reduce<Record<string, SessionCostBreakdown>>((acc, session) => {
      const key = session.cost_confidence || 'low';
      if (!acc[key]) acc[key] = { sessions: 0, cost: 0 };
      acc[key].sessions += 1;
      acc[key].cost += session.cost;
      return acc;
    }, {})
  ).sort(([left], [right]) => confidenceRank(left) - confidenceRank(right));

  const billingModeBreakdown = Object.entries(
    filteredSessions.reduce<Record<string, SessionCostBreakdown>>((acc, session) => {
      const key = session.billing_mode || 'estimated_local';
      if (!acc[key]) acc[key] = { sessions: 0, cost: 0 };
      acc[key].sessions += 1;
      acc[key].cost += session.cost;
      return acc;
    }, {})
  ).sort((left, right) => right[1].sessions - left[1].sessions);

  const pricingVersions = Array.from(
    new Set(filteredSessions.map(session => session.pricing_version).filter(Boolean))
  );

  return {
    daily,
    byModel,
    byProject,
    totals,
    confidenceBreakdown,
    billingModeBreakdown,
    pricingVersions,
  };
}
