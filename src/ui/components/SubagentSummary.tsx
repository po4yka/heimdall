import { fmt } from '../lib/format';
import type { SubagentSummary as SubagentSummaryType } from '../state/types';

export function SubagentSummary({ summary }: { summary: SubagentSummaryType }) {
  if (summary.subagent_turns === 0) return null;

  const totalInput = summary.parent_input + summary.subagent_input;
  const totalOutput = summary.parent_output + summary.subagent_output;
  const subPctInput = totalInput > 0 ? (summary.subagent_input / totalInput * 100) : 0;
  const subPctOutput = totalOutput > 0 ? (summary.subagent_output / totalOutput * 100) : 0;

  return (
    <div class="table-card">
      <div class="section-header" style={{ padding: '20px 20px 0' }}>
        <div class="section-title" style={{ padding: '0' }}>Subagent Breakdown</div>
      </div>
      <div style="display:grid;grid-template-columns:1fr 1fr 1fr;gap:16px;padding:12px 20px 20px">
        <div>
          <div class="stat-label">Turns</div>
          <div style="font-size:15px">Parent: <span class="num">{fmt(summary.parent_turns)}</span></div>
          <div style="font-size:15px">Subagent: <span class="num">{fmt(summary.subagent_turns)}</span></div>
          <div class="sub">{summary.unique_agents} unique agents</div>
        </div>
        <div>
          <div class="stat-label">Input Tokens</div>
          <div style="font-size:15px">Parent: <span class="num">{fmt(summary.parent_input)}</span></div>
          <div style="font-size:15px">
            Subagent: <span class="num">{fmt(summary.subagent_input)}</span> ({subPctInput.toFixed(1)}%)
          </div>
        </div>
        <div>
          <div class="stat-label">Output Tokens</div>
          <div style="font-size:15px">Parent: <span class="num">{fmt(summary.parent_output)}</span></div>
          <div style="font-size:15px">
            Subagent: <span class="num">{fmt(summary.subagent_output)}</span> ({subPctOutput.toFixed(1)}%)
          </div>
        </div>
      </div>
    </div>
  );
}
