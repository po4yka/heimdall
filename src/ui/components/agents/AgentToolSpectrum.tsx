import { withAlpha } from '../../lib/charts';
import { esc } from '../../lib/format';
import type { ToolSpectrumCell } from '../../state/types';

const MAX_TOOLS = 12;

interface AgentToolSpectrumProps {
  data: ToolSpectrumCell[];
}

export function AgentToolSpectrum({ data }: AgentToolSpectrumProps) {
  if (!data.length) {
    return (
      <div class="table-card" style={{ padding: '20px' }}>
        <div class="section-title">Tool spectrum</div>
        <div class="empty-state">No tool data</div>
      </div>
    );
  }

  // Collect top-K tools by global count
  const toolTotals = new Map<string, number>();
  for (const c of data) {
    toolTotals.set(c.tool, (toolTotals.get(c.tool) ?? 0) + c.count);
  }
  const sortedTools = [...toolTotals.entries()]
    .sort((a, b) => b[1] - a[1])
    .map(([t]) => t);
  const topTools = sortedTools.slice(0, MAX_TOOLS);
  const hasOtherTool = sortedTools.length > MAX_TOOLS;
  const displayTools = hasOtherTool ? [...topTools, 'Other'] : topTools;

  // Collect rows: roles sorted by total cost across all data (use count as proxy)
  const roleTotals = new Map<string, number>();
  for (const c of data) roleTotals.set(c.role, (roleTotals.get(c.role) ?? 0) + c.count);
  const roles = [...roleTotals.entries()]
    .sort((a, b) => b[1] - a[1])
    .map(([r]) => r);

  // Build cell lookup: role -> tool -> count
  const cellMap = new Map<string, Map<string, number>>();
  for (const c of data) {
    if (!cellMap.has(c.role)) cellMap.set(c.role, new Map());
    const toolKey = topTools.includes(c.tool) ? c.tool : (hasOtherTool ? 'Other' : null);
    if (!toolKey) continue;
    const row = cellMap.get(c.role)!;
    row.set(toolKey, (row.get(toolKey) ?? 0) + c.count);
  }

  // Per-row max for independent normalization
  function rowMax(role: string): number {
    const row = cellMap.get(role);
    if (!row) return 0;
    return Math.max(...row.values(), 0);
  }

  const gridCols = displayTools.length;

  return (
    <div class="table-card" style={{ padding: '20px' }}>
      <div class="section-title" style={{ marginBottom: '16px' }}>Tool spectrum</div>
      <div
        class="agent-tool-spectrum"
        style={{
          gridTemplateColumns: `minmax(80px, 160px) repeat(${gridCols}, minmax(0, 1fr))`,
        }}
      >
        {/* Header row */}
        <div class="spectrum-cell spectrum-header-corner" />
        {displayTools.map(tool => (
          <div key={tool} class="spectrum-cell spectrum-col-header" title={tool}>
            {esc(tool.length > 14 ? tool.slice(0, 12) + '…' : tool)}
          </div>
        ))}

        {/* Data rows */}
        {roles.map(role => {
          const maxVal = rowMax(role);
          const row = cellMap.get(role);
          return [
            <div key={`label-${role}`} class="spectrum-cell spectrum-row-label" title={role}>
              {esc(role.length > 18 ? role.slice(0, 16) + '…' : role)}
            </div>,
            ...displayTools.map(tool => {
              const count = row?.get(tool) ?? 0;
              const opacity = maxVal > 0 && count > 0
                ? Math.min(0.08 + 0.82 * (count / maxVal), 0.90)
                : 0;
              const bg = opacity > 0 ? withAlpha('--text-primary', opacity) : 'transparent';
              const textColor = opacity > 0.5
                ? 'var(--canvas)'
                : opacity > 0
                ? 'var(--text-primary)'
                : 'var(--text-disabled)';
              return (
                <div
                  key={`${role}-${tool}`}
                  class="spectrum-cell spectrum-data-cell"
                  title={`${role} / ${tool}: ${count}`}
                  style={{ background: bg }}
                >
                  <span style={{ color: textColor }}>{count > 0 ? count : ''}</span>
                </div>
              );
            }),
          ];
        })}
      </div>
    </div>
  );
}
