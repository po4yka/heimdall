import type { OfficialSyncSummary } from '../state/types';
import { official_sync_expanded, syncDashboardUrl } from '../state/store';
import { fmtLabel } from '../lib/format';

type ProviderFilter = 'claude' | 'codex' | 'both';

interface OfficialSyncPanelProps {
  summary: OfficialSyncSummary;
  providerFilter: ProviderFilter;
}

function sourceVisible(provider: string, providerFilter: ProviderFilter): boolean {
  if (providerFilter === 'both') return true;
  if (provider === 'frankfurter') return true;
  if (providerFilter === 'claude') return provider === 'anthropic';
  return provider === 'openai';
}

function statusLabel(status: string): string {
  if (status === 'success') return 'OK';
  if (status === 'skipped') return 'SKIP';
  if (status === 'parse_error') return 'PARSE';
  if (status === 'fetch_error') return 'FETCH';
  return fmtLabel(status);
}

function statusColor(status: string): string {
  if (status === 'success') return 'var(--text-primary)';
  if (status === 'skipped') return 'var(--text-secondary)';
  return 'var(--accent)';
}

function formatTs(ts: string | null | undefined): string {
  if (!ts) return 'n/a';
  return ts.slice(0, 19).replace('T', ' ');
}

export function OfficialSyncPanel({ summary, providerFilter }: OfficialSyncPanelProps) {
  const expanded = official_sync_expanded.value;
  const sources = summary.sources.filter(source => sourceVisible(source.provider, providerFilter));
  const recordCounts = summary.record_counts.filter(record => {
    if (providerFilter === 'both') return true;
    if (providerFilter === 'claude') {
      return !record.record_type.startsWith('usage_');
    }
    return true;
  });
  const successCount = sources.filter(source => source.status === 'success').length;
  const errorCount = sources.filter(
    source => source.status === 'fetch_error' || source.status === 'parse_error'
  ).length;
  const skippedCount = sources.filter(source => source.status === 'skipped').length;

  return (
    <div class="card stat-card">
      <div class="stat-content">
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            marginBottom: '8px',
          }}
        >
          <div class="stat-label">Official history</div>
          {summary.available && (
            <button
              onClick={() => {
                official_sync_expanded.value = !expanded;
                syncDashboardUrl();
              }}
              style={{
                background: 'none',
                border: 'none',
                cursor: 'pointer',
                color: 'var(--text-secondary)',
                fontSize: '11px',
                fontFamily: 'var(--font-mono)',
                padding: '2px 4px',
              }}
              aria-expanded={expanded}
              aria-label="Toggle official history details"
            >
              {expanded ? '▲ collapse' : '▼ expand'}
            </button>
          )}
        </div>

        {!summary.available ? (
          <div class="muted">No persisted official sync history yet</div>
        ) : (
          <>
            <div
              style={{
                display: 'grid',
                gridTemplateColumns: 'repeat(auto-fit,minmax(160px,1fr))',
                gap: '12px',
                marginBottom: expanded ? '12px' : '0',
              }}
            >
              <div>
                <div class="stat-value" style={{ fontSize: '18px' }}>{formatTs(summary.last_sync_at)}</div>
                <div class="stat-sub">Latest sync</div>
              </div>
              <div>
                <div class="stat-value" style={{ fontSize: '18px' }}>{summary.total_runs} / {summary.total_records}</div>
                <div class="stat-sub">Runs / extracted records</div>
              </div>
              <div>
                <div class="stat-value" style={{ fontSize: '18px' }}>{successCount} / {errorCount} / {skippedCount}</div>
                <div class="stat-sub">OK / error / skipped sources</div>
              </div>
            </div>

            {expanded && (
              <div style={{ display: 'grid', gap: '12px' }}>
                <div>
                  <div
                    style={{
                      fontSize: '11px',
                      fontFamily: 'var(--font-mono)',
                      color: 'var(--text-secondary)',
                      marginBottom: '6px',
                      letterSpacing: '0.06em',
                    }}
                  >
                    Latest sources
                  </div>
                  <table style={{ width: '100%', fontSize: '12px', borderCollapse: 'collapse' }}>
                    <thead>
                      <tr style={{ color: 'var(--text-secondary)' }}>
                        <th style={{ textAlign: 'left', padding: '2px 8px 2px 0', fontWeight: 500 }}>Source</th>
                        <th style={{ textAlign: 'left', padding: '2px 8px 2px 0', fontWeight: 500 }}>Kind</th>
                        <th style={{ textAlign: 'left', padding: '2px 8px 2px 0', fontWeight: 500 }}>Status</th>
                        <th style={{ textAlign: 'right', padding: '2px 0', fontWeight: 500 }}>Rows</th>
                      </tr>
                    </thead>
                    <tbody>
                      {sources.map(source => (
                        <tr key={source.source_slug}>
                          <td style={{ padding: '2px 8px 2px 0', fontFamily: 'var(--font-mono)' }}>
                            {fmtLabel(source.source_slug)}
                          </td>
                          <td style={{ padding: '2px 8px 2px 0', color: 'var(--text-secondary)' }}>
                            {fmtLabel(source.source_kind)}
                          </td>
                          <td style={{ padding: '2px 8px 2px 0', color: statusColor(source.status), fontFamily: 'var(--font-mono)' }}>
                            {statusLabel(source.status)}
                          </td>
                          <td style={{ padding: '2px 0', textAlign: 'right', fontFamily: 'var(--font-mono)' }}>
                            {(source.record_count ?? 0).toLocaleString()}
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>

                <div>
                  <div
                    style={{
                      fontSize: '11px',
                      fontFamily: 'var(--font-mono)',
                      color: 'var(--text-secondary)',
                      marginBottom: '6px',
                      letterSpacing: '0.06em',
                    }}
                  >
                    Record types
                  </div>
                  <div style={{ display: 'flex', flexWrap: 'wrap', gap: '8px' }}>
                    {recordCounts.map(record => (
                      <div
                        key={record.record_type}
                        style={{
                          border: '1px solid var(--border)',
                          padding: '6px 8px',
                          minWidth: '140px',
                        }}
                      >
                        <div style={{ fontFamily: 'var(--font-mono)', fontSize: '12px' }}>
                          {fmtLabel(record.record_type)}
                        </div>
                        <div style={{ fontSize: '11px', color: 'var(--text-secondary)' }}>
                          {(record.count ?? 0).toLocaleString()} rows
                        </div>
                      </div>
                    ))}
                  </div>
                </div>

                {summary.latest_success_at && (
                  <div
                    style={{
                      fontSize: '10px',
                      color: 'var(--text-secondary)',
                      fontFamily: 'var(--font-mono)',
                    }}
                  >
                    Latest successful extraction {formatTs(summary.latest_success_at)} UTC
                  </div>
                )}
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
}
