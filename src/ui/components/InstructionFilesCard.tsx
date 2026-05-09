import { useState, useEffect } from 'preact/hooks';
import type { InstructionFilesReport, InstructionScope, InstructionFile, SkillsBudgetRow } from '../state/dashboard-types';
import { instructionFilesReport, instructionFilesLoadState } from '../state/store';

function fmtBytes(b: number): string {
  if (b >= 1_048_576) return (b / 1_048_576).toFixed(1) + ' MB';
  if (b >= 1_024) return (b / 1_024).toFixed(1) + ' KB';
  return b + ' B';
}

function relativeTime(iso: string): string {
  const diff = Date.now() - new Date(iso).getTime();
  const days = Math.floor(diff / 86400000);
  if (days === 0) return 'today';
  if (days === 1) return '1d ago';
  return `${days}d ago`;
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
            ? `[OVER: ${Math.abs(row.headroom_tokens)} tok over, ~${row.simulated_drop_count} files dropped]`
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
        aria-label={`Instruction file tokens: ${row.used_tokens} of ${row.budget_tokens}`}
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

function FileStatusCell({ status }: { status: InstructionFile['frontmatter_status'] }) {
  if (status === 'invalid') {
    return (
      <span style={{ color: 'var(--accent, #D71921)', fontFamily: 'var(--font-mono)' }}>
        [INVALID]
      </span>
    );
  }
  if (status === 'not_applicable') {
    return <span style={{ color: 'var(--text-secondary)', fontFamily: 'var(--font-mono)' }}>—</span>;
  }
  return <span style={{ color: 'var(--text-secondary)', fontFamily: 'var(--font-mono)' }}>ok</span>;
}

function ScopeSection({ scope }: { scope: InstructionScope }) {
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
          {scope.files.length} files · {fmtBytes(scope.bytes)} · {scope.tokens} tok {open ? '▲' : '▼'}
        </span>
      </button>

      {open && scope.files.length > 0 && (
        <table style={{ width: '100%', borderCollapse: 'collapse', marginTop: '8px', fontSize: '11px' }}>
          <thead>
            <tr>
              <th style={{ textAlign: 'left', padding: '2px 4px', color: 'var(--text-secondary)', fontFamily: 'var(--font-mono)', fontWeight: 'normal', letterSpacing: '0.05em' }}>Path</th>
              <th style={{ textAlign: 'right', padding: '2px 4px', color: 'var(--text-secondary)', fontFamily: 'var(--font-mono)', fontWeight: 'normal', letterSpacing: '0.05em' }}>Bytes</th>
              <th style={{ textAlign: 'right', padding: '2px 4px', color: 'var(--text-secondary)', fontFamily: 'var(--font-mono)', fontWeight: 'normal', letterSpacing: '0.05em' }}>Tok</th>
              <th style={{ textAlign: 'right', padding: '2px 4px', color: 'var(--text-secondary)', fontFamily: 'var(--font-mono)', fontWeight: 'normal', letterSpacing: '0.05em' }}>Lines</th>
              <th style={{ textAlign: 'right', padding: '2px 4px', color: 'var(--text-secondary)', fontFamily: 'var(--font-mono)', fontWeight: 'normal', letterSpacing: '0.05em' }}>Modified</th>
              <th style={{ textAlign: 'right', padding: '2px 4px', color: 'var(--text-secondary)', fontFamily: 'var(--font-mono)', fontWeight: 'normal', letterSpacing: '0.05em' }}>Status</th>
            </tr>
          </thead>
          <tbody>
            {scope.files.map((f) => (
              <tr key={f.path}>
                <td style={{ padding: '2px 4px', fontFamily: 'var(--font-mono)', wordBreak: 'break-all' }}>
                  {f.path}
                  {f.is_symlink && <span style={{ color: 'var(--text-secondary)', marginLeft: '4px' }}>[link]</span>}
                </td>
                <td style={{ textAlign: 'right', padding: '2px 4px', fontFamily: 'var(--font-mono)', color: 'var(--text-secondary)' }}>
                  {fmtBytes(f.bytes)}
                </td>
                <td style={{ textAlign: 'right', padding: '2px 4px', fontFamily: 'var(--font-mono)', color: 'var(--text-secondary)' }}>
                  {f.tokens}
                </td>
                <td style={{ textAlign: 'right', padding: '2px 4px', fontFamily: 'var(--font-mono)', color: 'var(--text-secondary)' }}>
                  {f.line_count}
                </td>
                <td style={{ textAlign: 'right', padding: '2px 4px', fontFamily: 'var(--font-mono)', color: 'var(--text-secondary)' }}>
                  {relativeTime(f.modified)}
                </td>
                <td style={{ textAlign: 'right', padding: '2px 4px' }}>
                  <FileStatusCell status={f.frontmatter_status} />
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}

function InstructionFilesCardInner({ report }: { report: InstructionFilesReport }) {
  const anyOver = report.budget.some((r) => r.headroom_tokens < 0);

  return (
    <div class="card" style={{ padding: '16px' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'baseline', marginBottom: '12px' }}>
        <div class="stat-label">Instruction files inventory</div>
        {anyOver && (
          <span style={{ fontFamily: 'var(--font-mono)', fontSize: '11px', color: 'var(--accent, #D71921)' }}>
            [WARN: skills will be dropped...]
          </span>
        )}
      </div>

      {/* Summary row */}
      <div style={{ display: 'flex', gap: '16px', marginBottom: '14px' }}>
        <div>
          <div class="stat-label" style={{ fontSize: '10px' }}>Files</div>
          <div style={{ fontFamily: 'var(--font-mono)', fontSize: '18px' }}>{report.totals.file_count}</div>
        </div>
        <div>
          <div class="stat-label" style={{ fontSize: '10px' }}>Total disk</div>
          <div style={{ fontFamily: 'var(--font-mono)', fontSize: '18px' }}>{fmtBytes(report.totals.total_bytes)}</div>
        </div>
        <div>
          <div class="stat-label" style={{ fontSize: '10px' }}>Total tokens</div>
          <div style={{ fontFamily: 'var(--font-mono)', fontSize: '18px' }}>{report.totals.total_tokens}</div>
        </div>
        <div>
          <div class="stat-label" style={{ fontSize: '10px' }}>Projects</div>
          <div style={{ fontFamily: 'var(--font-mono)', fontSize: '18px' }}>{report.totals.project_count}</div>
        </div>
      </div>

      {/* Budget bars */}
      {report.budget.map((row) => (
        <BudgetBar key={row.model_label} row={row} />
      ))}

      {/* Per-scope expandable sections */}
      {report.scopes.map((scope) => (
        <ScopeSection key={`${scope.kind}:${scope.root}`} scope={scope} />
      ))}

      <div style={{ marginTop: '8px', fontSize: '10px', color: 'var(--text-secondary)', fontFamily: 'var(--font-mono)' }}>
        tokenizer: {report.tokenizer} · budget fraction: {(report.budget_fraction * 100).toFixed(1)}%
      </div>
    </div>
  );
}

export function InstructionFilesCard() {
  const report = instructionFilesReport.value;
  const loadState = instructionFilesLoadState.value;

  useEffect(() => {
    if (report !== null || loadState === 'loading') return;
    instructionFilesLoadState.value = 'loading';
    fetch('/api/instruction-files')
      .then((r) => r.json())
      .then((data: InstructionFilesReport) => {
        instructionFilesReport.value = data;
        instructionFilesLoadState.value = 'idle';
      })
      .catch(() => {
        instructionFilesLoadState.value = 'error';
      });
  }, []);

  if (loadState === 'loading') {
    return (
      <div class="card" style={{ padding: '16px' }}>
        <div class="stat-label">Instruction files inventory</div>
        <div style={{ color: 'var(--text-secondary)', fontFamily: 'var(--font-mono)', fontSize: '12px', marginTop: '8px' }}>
          scanning...
        </div>
      </div>
    );
  }

  if (loadState === 'error' || !report) {
    return (
      <div class="card" style={{ padding: '16px' }}>
        <div class="stat-label">Instruction files inventory</div>
        <div style={{ color: 'var(--accent, #D71921)', fontFamily: 'var(--font-mono)', fontSize: '12px', marginTop: '8px' }}>
          [ERROR: failed to load instruction files data]
        </div>
      </div>
    );
  }

  return <InstructionFilesCardInner report={report} />;
}
