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
      <div class="section-title">Subagent Breakdown</div>
      <div style="display:grid;grid-template-columns:1fr 1fr 1fr;gap:16px">
        <div>
          <div class="label" style="color:var(--muted);font-size:11px;text-transform:uppercase;margin-bottom:4px">
            Turns
          </div>
          <div style="font-size:15px">Parent: <strong>{fmt(summary.parent_turns)}</strong></div>
          <div style="font-size:15px">Subagent: <strong>{fmt(summary.subagent_turns)}</strong></div>
          <div class="sub">{summary.unique_agents} unique agents</div>
        </div>
        <div>
          <div class="label" style="color:var(--muted);font-size:11px;text-transform:uppercase;margin-bottom:4px">
            Input Tokens
          </div>
          <div style="font-size:15px">Parent: <strong>{fmt(summary.parent_input)}</strong></div>
          <div style="font-size:15px">
            Subagent: <strong>{fmt(summary.subagent_input)}</strong> ({subPctInput.toFixed(1)}%)
          </div>
        </div>
        <div>
          <div class="label" style="color:var(--muted);font-size:11px;text-transform:uppercase;margin-bottom:4px">
            Output Tokens
          </div>
          <div style="font-size:15px">Parent: <strong>{fmt(summary.parent_output)}</strong></div>
          <div style="font-size:15px">
            Subagent: <strong>{fmt(summary.subagent_output)}</strong> ({subPctOutput.toFixed(1)}%)
          </div>
        </div>
      </div>
    </div>
  );
}
