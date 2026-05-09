import { useState, useEffect } from 'preact/hooks';
import type { SkillsReport, SkillScope, SkillsBudgetRow, SkillsDuplicateGroup, SkillsDuplicateOccurrence } from '../state/dashboard-types';
import { skillsReport, skillsLoadState } from '../state/store';

function fmtBytes(b: number): string {
  if (b >= 1_048_576) return (b / 1_048_576).toFixed(1) + ' MB';
  if (b >= 1_024) return (b / 1_024).toFixed(1) + ' KB';
  return b + ' B';
}

function BudgetBar({ row }: { row: SkillsBudgetRow }) {
  const fill = Math.min(1, row.budget_tokens > 0 ? row.used_tokens / row.budget_tokens : 0);
  const isOver = row.headroom_tokens < 0;
  const barColor = isOver
    ? 'var(--accent, #D71921)'
    : fill > 0.8
      ? 'rgba(var(--text-primary-rgb, 232,232,232), 0.80)'
      : 'rgba(var(--text-primary-rgb, 232,232,232), 0.55)';

  return (
    <div style={{ marginBottom: '10px' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '4px' }}>
        <span class="stat-label" style={{ fontSize: '11px' }}>{row.model_label}</span>
        <span
          style={{
            fontFamily: 'var(--font-mono)',
            fontSize: '11px',
            color: isOver ? 'var(--accent, #D71921)' : 'var(--text-secondary)',
          }}
        >
          {isOver
            ? `[OVER: ${Math.abs(row.headroom_tokens)} tok over, ~${row.simulated_drop_count} skills dropped]`
            : `${row.used_tokens} / ${row.budget_tokens} tok`}
        </span>
      </div>
      <div
        style={{
          height: '4px',
          borderRadius: '2px',
          background: 'rgba(var(--text-primary-rgb, 232,232,232), 0.10)',
          overflow: 'hidden',
        }}
        role="img"
        aria-label={`Skills listing tokens: ${row.used_tokens} of ${row.budget_tokens}`}
      >
        <div
          style={{
            height: '100%',
            width: `${(fill * 100).toFixed(2)}%`,
            background: barColor,
            borderRadius: '2px',
            transition: 'width 300ms cubic-bezier(0.25,0.1,0.25,1)',
          }}
        />
      </div>
    </div>
  );
}

function ScopeSection({ scope }: { scope: SkillScope }) {
  const [open, setOpen] = useState(false);
  const kindLabel = scope.kind.replace(/_/g, ' ');
  const projectSuffix = scope.project_label ? ` [${scope.project_label}]` : '';

  return (
    <div style={{ marginBottom: '8px', borderTop: '1px solid rgba(var(--text-primary-rgb, 232,232,232), 0.08)', paddingTop: '8px' }}>
      <button
        type="button"
        onClick={() => setOpen(!open)}
        style={{
          background: 'none',
          border: 'none',
          cursor: 'pointer',
          width: '100%',
          textAlign: 'left',
          padding: '0',
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          color: 'var(--text-primary)',
        }}
      >
        <span style={{ fontFamily: 'var(--font-mono)', fontSize: '11px', color: 'var(--text-secondary)' }}>
          {scope.provider} · {kindLabel}{projectSuffix}
        </span>
        <span style={{ fontFamily: 'var(--font-mono)', fontSize: '11px', color: 'var(--text-secondary)' }}>
          {scope.skills.length} skills · {fmtBytes(scope.bytes)} · {scope.listing_tokens} tok {open ? '▲' : '▼'}
        </span>
      </button>

      {open && scope.skills.length > 0 && (
        <table style={{ width: '100%', borderCollapse: 'collapse', marginTop: '8px', fontSize: '11px' }}>
          <thead>
            <tr>
              <th style={{ textAlign: 'left', padding: '2px 4px', color: 'var(--text-secondary)', fontFamily: 'var(--font-mono)', fontWeight: 'normal', letterSpacing: '0.05em' }}>Name</th>
              <th style={{ textAlign: 'right', padding: '2px 4px', color: 'var(--text-secondary)', fontFamily: 'var(--font-mono)', fontWeight: 'normal', letterSpacing: '0.05em' }}>Disk</th>
              <th style={{ textAlign: 'right', padding: '2px 4px', color: 'var(--text-secondary)', fontFamily: 'var(--font-mono)', fontWeight: 'normal', letterSpacing: '0.05em' }}>Tok</th>
              <th style={{ textAlign: 'right', padding: '2px 4px', color: 'var(--text-secondary)', fontFamily: 'var(--font-mono)', fontWeight: 'normal', letterSpacing: '0.05em' }}>Last used</th>
              <th style={{ textAlign: 'right', padding: '2px 4px', color: 'var(--text-secondary)', fontFamily: 'var(--font-mono)', fontWeight: 'normal', letterSpacing: '0.05em' }}>Status</th>
            </tr>
          </thead>
          <tbody>
            {scope.skills.map((s) => (
              <tr key={s.path} title={s.description ?? undefined}>
                <td style={{ padding: '2px 4px', fontFamily: 'var(--font-mono)' }}>
                  {s.name}
                  {s.is_symlink && <span style={{ color: 'var(--text-secondary)', marginLeft: '4px' }}>[link]</span>}
                  {s.description_truncated && <span style={{ color: 'var(--text-secondary)', marginLeft: '2px' }}>+</span>}
                  {s.is_dormant && (
                    <span style={{ fontFamily: 'var(--font-mono)', fontSize: '9px', color: 'var(--text-secondary)', marginLeft: '6px', opacity: 0.6 }}>
                      {s.usage?.last_used
                        ? `[DORMANT ${Math.floor((Date.now() - new Date(s.usage.last_used).getTime()) / 86400000)}d]`
                        : '[NEVER]'}
                    </span>
                  )}
                </td>
                <td style={{ textAlign: 'right', padding: '2px 4px', fontFamily: 'var(--font-mono)', color: 'var(--text-secondary)' }}>
                  {fmtBytes(s.bytes)}
                </td>
                <td style={{ textAlign: 'right', padding: '2px 4px', fontFamily: 'var(--font-mono)', color: 'var(--text-secondary)' }}>
                  {s.listing_tokens}
                </td>
                <td style={{ textAlign: 'right', padding: '2px 4px', fontFamily: 'var(--font-mono)', color: 'var(--text-secondary)', opacity: 0.7 }}>
                  {s.usage ? (s.usage.last_used ? new Date(s.usage.last_used).toLocaleDateString() : '—') : '—'}
                </td>
                <td style={{ textAlign: 'right', padding: '2px 4px', fontFamily: 'var(--font-mono)', color: s.frontmatter_status !== 'ok' ? 'var(--accent, #D71921)' : 'var(--text-secondary)' }}>
                  {s.frontmatter_status}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}

function DuplicateOccurrenceRow({ occ }: { occ: SkillsDuplicateOccurrence }) {
  const kindLabel = occ.scope_kind.replace(/_/g, ' ');
  const projectSuffix = occ.project_label ? ` [${occ.project_label}]` : '';
  return (
    <div style={{ marginLeft: '12px', marginTop: '4px' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <span style={{ fontFamily: 'var(--font-mono)', fontSize: '10px', color: 'var(--text-secondary)' }}>
          {occ.provider} · {kindLabel}{projectSuffix}
          {occ.is_symlink && <span style={{ marginLeft: '4px', color: 'var(--text-secondary)' }}>[link]</span>}
        </span>
        <span style={{ fontFamily: 'var(--font-mono)', fontSize: '10px', color: 'var(--text-secondary)' }}>
          {fmtBytes(occ.bytes)} · {occ.listing_tokens} tok
          {occ.frontmatter_status !== 'ok' && (
            <span style={{ marginLeft: '4px', color: 'var(--accent, #D71921)' }}>[{occ.frontmatter_status}]</span>
          )}
        </span>
      </div>
      {occ.description_excerpt && (
        <div style={{ fontFamily: 'var(--font-mono)', fontSize: '10px', color: 'var(--text-secondary)', opacity: 0.7, marginTop: '2px', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
          {occ.description_excerpt}{occ.description_excerpt.length >= 120 ? '…' : ''}
        </div>
      )}
    </div>
  );
}

function DuplicateGroupRow({ group }: { group: SkillsDuplicateGroup }) {
  const [open, setOpen] = useState(false);
  const allSameDesc = group.occurrences.every(
    (o) => o.description_excerpt === group.occurrences[0]?.description_excerpt
  );
  return (
    <div style={{ marginBottom: '6px', borderTop: '1px solid rgba(var(--text-primary-rgb, 232,232,232), 0.08)', paddingTop: '6px' }}>
      <button
        type="button"
        onClick={() => setOpen(!open)}
        style={{
          background: 'none', border: 'none', cursor: 'pointer',
          width: '100%', textAlign: 'left', padding: '0',
          display: 'flex', justifyContent: 'space-between', alignItems: 'center',
        }}
      >
        <span style={{ fontFamily: 'var(--font-mono)', fontSize: '11px' }}>
          {group.name}
          {!allSameDesc && (
            <span style={{ marginLeft: '6px', color: 'var(--accent, #D71921)', fontSize: '10px' }}>[differs]</span>
          )}
        </span>
        <span style={{ fontFamily: 'var(--font-mono)', fontSize: '10px', color: 'var(--text-secondary)' }}>
          {group.count}× · {fmtBytes(group.wasted_bytes)} wasted · {group.wasted_tokens} tok {open ? '▲' : '▼'}
        </span>
      </button>
      {open && (
        <div style={{ marginTop: '4px' }}>
          {group.occurrences.map((occ, i) => (
            <DuplicateOccurrenceRow key={`${occ.scope_kind}:${occ.root}:${i}`} occ={occ} />
          ))}
        </div>
      )}
    </div>
  );
}

function DuplicatesSection({ groups }: { groups: SkillsDuplicateGroup[] }) {
  const [open, setOpen] = useState(false);
  if (groups.length === 0) return null;
  const totalWasted = groups.reduce((s, g) => s + g.wasted_bytes, 0);
  const totalWastedTok = groups.reduce((s, g) => s + g.wasted_tokens, 0);
  const hasConflicts = groups.some(
    (g) => !g.occurrences.every((o) => o.description_excerpt === g.occurrences[0]?.description_excerpt)
  );
  return (
    <div style={{ marginTop: '14px', borderTop: '1px solid rgba(var(--text-primary-rgb, 232,232,232), 0.08)', paddingTop: '10px' }}>
      <button
        type="button"
        onClick={() => setOpen(!open)}
        style={{
          background: 'none', border: 'none', cursor: 'pointer',
          width: '100%', textAlign: 'left', padding: '0',
          display: 'flex', justifyContent: 'space-between', alignItems: 'center',
          marginBottom: '8px',
        }}
      >
        <div class="stat-label" style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          Duplicates
          {hasConflicts && (
            <span style={{ fontFamily: 'var(--font-mono)', fontSize: '10px', color: 'var(--accent, #D71921)' }}>
              [WARN: conflicting descriptions]
            </span>
          )}
        </div>
        <span style={{ fontFamily: 'var(--font-mono)', fontSize: '11px', color: 'var(--text-secondary)' }}>
          {groups.length} names · {fmtBytes(totalWasted)} wasted · {totalWastedTok} tok {open ? '▲' : '▼'}
        </span>
      </button>
      {open && groups.map((g) => (
        <DuplicateGroupRow key={g.name} group={g} />
      ))}
    </div>
  );
}

function SkillsCardInner({ report }: { report: SkillsReport }) {
  const anyOver = report.budget.some((r) => r.headroom_tokens < 0);

  return (
    <div class="card" style={{ padding: '16px' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'baseline', marginBottom: '12px' }}>
        <div class="stat-label">Skills inventory</div>
        {anyOver && (
          <span style={{ fontFamily: 'var(--font-mono)', fontSize: '11px', color: 'var(--accent, #D71921)' }}>
            [WARN: skills will be dropped from listing]
          </span>
        )}
      </div>

      {/* Summary row */}
      <div style={{ display: 'flex', gap: '16px', marginBottom: '14px' }}>
        <div>
          <div class="stat-label" style={{ fontSize: '10px' }}>Skills</div>
          <div style={{ fontFamily: 'var(--font-mono)', fontSize: '18px' }}>{report.totals.skills_count}</div>
        </div>
        <div>
          <div class="stat-label" style={{ fontSize: '10px' }}>Disk</div>
          <div style={{ fontFamily: 'var(--font-mono)', fontSize: '18px' }}>{fmtBytes(report.totals.total_bytes)}</div>
        </div>
        <div>
          <div class="stat-label" style={{ fontSize: '10px' }}>Listing tokens</div>
          <div style={{ fontFamily: 'var(--font-mono)', fontSize: '18px' }}>{report.totals.total_listing_tokens}</div>
        </div>
        {report.totals.duplicate_count > 0 && (
          <div>
            <div class="stat-label" style={{ fontSize: '10px' }}>Duplicates</div>
            <div style={{ fontFamily: 'var(--font-mono)', fontSize: '18px', color: 'var(--accent, #D71921)' }}>
              {report.totals.duplicate_count}
            </div>
          </div>
        )}
        {(() => {
          const dormantCount = report.scopes.reduce(
            (acc, sc) => acc + sc.skills.filter((s) => s.is_dormant).length,
            0,
          );
          return dormantCount > 0 ? (
            <div>
              <div class="stat-label" style={{ fontSize: '10px' }}>Dormant</div>
              <div style={{ fontFamily: 'var(--font-mono)', fontSize: '18px', color: 'var(--text-secondary)', opacity: 0.7 }}>
                {dormantCount}
              </div>
            </div>
          ) : null;
        })()}
      </div>

      {/* Budget bars */}
      {report.budget.map((row) => (
        <BudgetBar key={row.model_label} row={row} />
      ))}

      {/* Per-scope expandable sections */}
      {report.scopes.map((scope) => (
        <ScopeSection key={`${scope.kind}:${scope.root}`} scope={scope} />
      ))}

      {/* Duplicates */}
      <DuplicatesSection groups={report.duplicates} />

      <div style={{ marginTop: '8px', fontSize: '10px', color: 'var(--text-secondary)', fontFamily: 'var(--font-mono)' }}>
        tokenizer: {report.tokenizer} · budget fraction: {(report.budget_fraction * 100).toFixed(1)}%
      </div>
    </div>
  );
}

export function SkillsCard() {
  const report = skillsReport.value;
  const loadState = skillsLoadState.value;

  useEffect(() => {
    if (report !== null || loadState === 'loading') return;
    skillsLoadState.value = 'loading';
    fetch('/api/skills')
      .then((r) => r.json())
      .then((data: SkillsReport) => {
        skillsReport.value = data;
        skillsLoadState.value = 'idle';
      })
      .catch(() => {
        skillsLoadState.value = 'error';
      });
  }, []);

  if (loadState === 'loading') {
    return (
      <div class="card" style={{ padding: '16px' }}>
        <div class="stat-label">Skills inventory</div>
        <div style={{ color: 'var(--text-secondary)', fontFamily: 'var(--font-mono)', fontSize: '12px', marginTop: '8px' }}>
          scanning...
        </div>
      </div>
    );
  }

  if (loadState === 'error' || !report) {
    return (
      <div class="card" style={{ padding: '16px' }}>
        <div class="stat-label">Skills inventory</div>
        <div style={{ color: 'var(--accent, #D71921)', fontFamily: 'var(--font-mono)', fontSize: '12px', marginTop: '8px' }}>
          [ERROR: failed to load skills data]
        </div>
      </div>
    );
  }

  return <SkillsCardInner report={report} />;
}
