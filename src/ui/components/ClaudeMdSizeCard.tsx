import { rawData } from '../state/store';
import type { ClaudeMdSizeSummary, ClaudeMdFileTrend, ClaudeMdSizePoint } from '../state/dashboard-types';

function fmtTokens(n: number): string {
  return n >= 1000 ? `${(n / 1000).toFixed(1)}k` : `${n}`;
}

function fmtDelta(n: number, pct: number): string {
  const sign = n >= 0 ? '+' : '';
  return `${sign}${fmtTokens(n)} (${sign}${(pct * 100).toFixed(1)}%)`;
}

function fmtDate(iso: string): string {
  return iso ? iso.slice(0, 10) : '';
}

function fmtCorrelation(corr: number | null | undefined, sampleSize: number): string {
  if (corr == null) return 'n/a';
  const confidence = sampleSize < 10 ? ' low-conf' : '';
  return `${corr >= 0 ? '+' : ''}${corr.toFixed(2)}${confidence}`;
}

function KpiTile({ label, value }: { label: string; value: string | number }) {
  return (
    <div>
      <div class="stat-label" style={{ fontSize: '10px' }}>{label}</div>
      <div style={{ fontFamily: 'var(--font-mono)', fontSize: '18px' }}>{value}</div>
    </div>
  );
}

function TokenTrendBars({ revisions }: { revisions: ClaudeMdSizePoint[] }) {
  if (revisions.length === 0) return null;
  const shown = revisions.slice(-20);
  const maxTokens = Math.max(...shown.map(r => r.token_count), 1);

  return (
    <div style={{ marginBottom: '12px' }}>
      <div class="stat-label" style={{ marginBottom: '6px', fontSize: '10px' }}>
        Token count over time (last {shown.length} revisions)
      </div>
      <div
        style={{
          display: 'flex',
          gap: '3px',
          alignItems: 'flex-end',
          padding: '10px',
          border: '1px solid var(--border)',
          borderRadius: '8px',
          height: '80px',
        }}
      >
        {shown.map((r, i) => {
          const heightPct = ((r.token_count / maxTokens) * 100).toFixed(1);
          return (
            <div
              key={`rev-${i}`}
              style={{
                flex: '1',
                height: `${heightPct}%`,
                background: 'rgba(var(--text-primary-rgb,232,232,232),0.7)',
                borderRadius: '1px 1px 0 0',
                minHeight: '2px',
              }}
              title={`${fmtDate(r.commit_iso)}: ${r.token_count} tokens`}
            />
          );
        })}
      </div>
    </div>
  );
}

function RevisionTable({ revisions }: { revisions: ClaudeMdSizePoint[] }) {
  const shown = revisions.slice(-5).reverse();
  if (shown.length === 0) return null;

  return (
    <div>
      <div class="stat-label" style={{ marginBottom: '6px', fontSize: '10px' }}>Recent revisions</div>
      <div style={{ padding: '10px', border: '1px solid var(--border)', borderRadius: '8px' }}>
        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'auto 1fr auto auto auto',
            gap: '3px 12px',
            alignItems: 'center',
          }}
        >
          {['SHA', 'Date', 'Tokens', 'Lines', 'Bytes'].map(h => (
            <div
              key={h}
              style={{
                fontFamily: 'var(--font-mono)',
                fontSize: '9px',
                color: 'var(--text-secondary)',
              }}
            >
              {h}
            </div>
          ))}
          {shown.map((r, i) => (
            <>
              <div key={`sha-${i}`} style={{ fontFamily: 'var(--font-mono)', fontSize: '10px', color: 'var(--text-secondary)' }}>
                {r.commit_sha.slice(0, 7)}
              </div>
              <div style={{ fontFamily: 'var(--font-mono)', fontSize: '10px', color: 'var(--text-primary)' }}>
                {fmtDate(r.commit_iso)}
              </div>
              <div style={{ fontFamily: 'var(--font-mono)', fontSize: '10px', textAlign: 'right', fontFeatureSettings: '"tnum"', color: 'var(--text-primary)' }}>
                {r.token_count}
              </div>
              <div style={{ fontFamily: 'var(--font-mono)', fontSize: '10px', textAlign: 'right', fontFeatureSettings: '"tnum"', color: 'var(--text-secondary)' }}>
                {r.line_count}
              </div>
              <div style={{ fontFamily: 'var(--font-mono)', fontSize: '10px', textAlign: 'right', fontFeatureSettings: '"tnum"', color: 'var(--text-secondary)' }}>
                {r.byte_size}
              </div>
            </>
          ))}
        </div>
      </div>
    </div>
  );
}

function FileTrendRow({ file }: { file: ClaudeMdFileTrend }) {
  const deltaColor = file.token_delta_30d > 0
    ? 'rgba(var(--text-primary-rgb,232,232,232),0.9)'
    : file.token_delta_30d < 0 ? 'var(--success,#3fb950)' : 'var(--text-secondary)';

  return (
    <div
      style={{
        padding: '14px',
        border: '1px solid var(--border)',
        borderRadius: '8px',
        marginBottom: '12px',
      }}
    >
      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'baseline',
          marginBottom: '10px',
          flexWrap: 'wrap',
          gap: '8px',
        }}
      >
        <div style={{ fontFamily: 'var(--font-mono)', fontSize: '12px', color: 'var(--text-primary)', fontWeight: 500 }}>
          {file.label}
        </div>
        <div style={{ display: 'flex', gap: '16px', alignItems: 'baseline' }}>
          <span style={{ fontFamily: 'var(--font-mono)', fontSize: '11px', color: 'var(--text-primary)' }}>
            {fmtTokens(file.current_token_count)} tok
          </span>
          {file.token_delta_30d !== 0 && (
            <span style={{ fontFamily: 'var(--font-mono)', fontSize: '10px', color: deltaColor }}>
              {fmtDelta(file.token_delta_30d, file.token_delta_pct_30d)}
            </span>
          )}
          <span style={{ fontFamily: 'var(--font-mono)', fontSize: '10px', color: 'var(--text-secondary)' }}>
            r={fmtCorrelation(file.cost_correlation, file.cost_correlation_sample_size)}
          </span>
        </div>
      </div>
      <TokenTrendBars revisions={file.revisions} />
      <RevisionTable revisions={file.revisions} />
    </div>
  );
}

function ClaudeMdSizeCardInner({ summary }: { summary: ClaudeMdSizeSummary }) {
  const filesWithGrowth = summary.files.filter(f => f.token_delta_pct_30d >= 0.20).length;
  const filesWithCorr = summary.files.filter(f => (f.cost_correlation ?? 0) >= 0.3).length;

  return (
    <div class="card" style={{ padding: '16px' }}>
      <div class="stat-label" style={{ marginBottom: '10px' }}>CLAUDE.md size over time</div>

      <div style={{ display: 'flex', gap: '20px', flexWrap: 'wrap', marginBottom: '16px' }}>
        <KpiTile label="Files tracked" value={summary.total_files_tracked} />
        <KpiTile label="Total revisions" value={summary.total_revisions} />
        <KpiTile label="Growth ≥20% (30d)" value={filesWithGrowth} />
        <KpiTile label="Correlation ≥0.3" value={filesWithCorr} />
      </div>

      {summary.files.map((file, i) => (
        <FileTrendRow key={`file-${i}`} file={file} />
      ))}

      <div
        style={{
          marginTop: '12px',
          fontFamily: 'var(--font-mono)',
          fontSize: '10px',
          color: 'var(--text-secondary)',
          lineHeight: '1.5',
        }}
      >
        Correlation between token growth and per-session cost is statistical only — not causal.
        Low confidence when sample size &lt;10 days.
      </div>
    </div>
  );
}

export function ClaudeMdSizeCard() {
  const data = rawData.value;

  if (!data) {
    return (
      <div class="card" style={{ padding: '16px' }}>
        <div class="stat-label">CLAUDE.md size over time</div>
        <div style={{ color: 'var(--text-secondary)', fontFamily: 'var(--font-mono)', fontSize: '12px', marginTop: '8px' }}>
          loading...
        </div>
      </div>
    );
  }

  const summary = data.claude_md_size;
  if (!summary || summary.total_files_tracked === 0) {
    return (
      <div class="card" style={{ padding: '16px' }}>
        <div class="stat-label">CLAUDE.md size over time</div>
        <div style={{ color: 'var(--text-secondary)', fontFamily: 'var(--font-mono)', fontSize: '12px', marginTop: '8px' }}>
          No git-tracked CLAUDE.md found. Run <code>heimdall scan</code> to populate history.
        </div>
      </div>
    );
  }

  return <ClaudeMdSizeCardInner summary={summary} />;
}
