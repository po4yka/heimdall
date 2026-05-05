import { esc } from '../lib/format';
import type { CodexPlanSnapshot } from '../state/dashboard-types';

interface Props {
  today: CodexPlanSnapshot;
}

function planLabel(planType: string | null): string {
  if (!planType) return 'Unknown';
  const s = planType.trim();
  if (!s) return 'Unknown';
  return s.charAt(0).toUpperCase() + s.slice(1).toLowerCase();
}

function creditState(snapshot: CodexPlanSnapshot): string {
  const c = snapshot.credits;
  if (!c) return '—';
  if (c.unlimited) return 'Unlimited';
  if (c.has_credits && c.balance != null) return `$${c.balance.toFixed(2)} balance`;
  if (c.has_credits) return 'Has credits';
  return '—';
}

function progressClass(pct: number): string {
  if (pct >= 85) return 'codex-plan-progress codex-plan-progress--high';
  if (pct >= 60) return 'codex-plan-progress codex-plan-progress--mid';
  return 'codex-plan-progress codex-plan-progress--low';
}

export function CodexPlanKpi({ today }: Props) {
  const pct = Math.min(100, Math.max(0, today.primary?.used_percent ?? 0));
  const pctText = pct.toFixed(1) + '%';
  const plan = planLabel(today.plan_type);
  const credits = creditState(today);

  return (
    <div class="card stat-card codex-plan-kpi">
      <div class="stat-content">
        <div class="stat-label">Codex plan</div>
        <div class="stat-value" style={{ fontFamily: 'var(--font-mono)' }}>
          {esc(plan)}
        </div>
        <div class="stat-sub">
          <span
            class={progressClass(pct)}
            style={{ width: `${pct}%` }}
            aria-valuenow={pct}
            aria-valuemin={0}
            aria-valuemax={100}
            role="progressbar"
            aria-label="5h window usage"
          />
        </div>
        <div class="stat-sub" style={{ fontFamily: 'var(--font-mono)', marginTop: '4px' }}>
          {pctText} &middot; 5h window
        </div>
        <div class="stat-sub">{esc(credits)}</div>
      </div>
    </div>
  );
}
