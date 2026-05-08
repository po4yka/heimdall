import { useState, useEffect } from 'preact/hooks';
import type {
  McpServersReport,
  McpServerEntry,
  McpTransport,
  McpRuntimeState,
  McpRedactedValue,
} from '../state/dashboard-types';
import { mcpServersReport, mcpServersLoadState } from '../state/store';

function fmtBytes(b: number): string {
  if (b >= 1_048_576) return (b / 1_048_576).toFixed(1) + ' MB';
  if (b >= 1_024) return (b / 1_024).toFixed(1) + ' KB';
  return b + ' B';
}

function relativeTime(iso: string): string {
  const diff = Date.now() - new Date(iso).getTime();
  const m = Math.floor(diff / 60000);
  if (m < 60) return `${m}m ago`;
  const h = Math.floor(m / 60);
  if (h < 24) return `${h}h ago`;
  return `${Math.floor(h / 24)}d ago`;
}

function TransportPill({ transport }: { transport: McpTransport }) {
  return (
    <span
      style={{
        fontFamily: 'var(--font-mono)',
        fontSize: '10px',
        padding: '1px 5px',
        border: '1px solid rgba(var(--text-primary-rgb, 232,232,232), 0.20)',
        borderRadius: '3px',
        color: 'var(--text-secondary)',
        marginLeft: '6px',
      }}
    >
      {transport.kind}
    </span>
  );
}

function RuntimeBadge({ runtime }: { runtime: McpRuntimeState }) {
  if (runtime.kind === 'running') {
    return (
      <span
        style={{
          fontFamily: 'var(--font-mono)',
          fontSize: '10px',
          color: 'var(--success, #4caf50)',
          marginLeft: '8px',
        }}
      >
        [RUNNING pid:{runtime.pid}]
      </span>
    );
  }
  if (runtime.kind === 'not_running') {
    return (
      <span
        style={{
          fontFamily: 'var(--font-mono)',
          fontSize: '10px',
          color: 'var(--text-secondary)',
          marginLeft: '8px',
          opacity: 0.6,
        }}
      >
        [STOPPED]
      </span>
    );
  }
  return (
    <span
      style={{
        fontFamily: 'var(--font-mono)',
        fontSize: '10px',
        color: 'var(--text-secondary)',
        marginLeft: '8px',
        opacity: 0.5,
      }}
    >
      [N/A]
    </span>
  );
}

function RedactedValueCell({ val }: { val: McpRedactedValue }) {
  if (val.kind === 'plain') {
    return (
      <span style={{ fontFamily: 'var(--font-mono)', fontSize: '10px', wordBreak: 'break-all' }}>
        {val.value}
      </span>
    );
  }
  if (val.kind === 'secret') {
    return (
      <span style={{ fontFamily: 'var(--font-mono)', fontSize: '10px' }}>
        <span style={{ opacity: 0.5 }}>{val.masked}</span>
        <span style={{ marginLeft: '4px', color: 'var(--accent)', fontSize: '9px' }}>[SECRET]</span>
      </span>
    );
  }
  // env_from_file
  return (
    <span style={{ fontFamily: 'var(--font-mono)', fontSize: '10px', color: 'var(--text-secondary)' }}>
      {val.path}
      {!val.exists && <span style={{ marginLeft: '4px', color: 'var(--accent)', fontSize: '9px' }}>[MISSING]</span>}
      {val.exists && <span style={{ marginLeft: '4px', opacity: 0.5 }}>{fmtBytes(val.bytes)}</span>}
    </span>
  );
}

function ServerRow({ server }: { server: McpServerEntry }) {
  const [open, setOpen] = useState(false);
  const envKeys = Object.keys(server.env);

  return (
    <div
      style={{
        borderTop: '1px solid rgba(var(--text-primary-rgb, 232,232,232), 0.08)',
        paddingTop: '8px',
        marginBottom: '4px',
      }}
    >
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
          alignItems: 'center',
          color: 'var(--text-primary)',
        }}
      >
        <span style={{ fontFamily: 'var(--font-mono)', fontSize: '12px', flexShrink: 0 }}>
          {server.name}
        </span>
        <TransportPill transport={server.transport} />
        <RuntimeBadge runtime={server.runtime} />
        {server.is_dormant && (
          <span
            style={{
              fontFamily: 'var(--font-mono)',
              fontSize: '10px',
              color: 'var(--text-secondary)',
              marginLeft: '6px',
              opacity: 0.6,
            }}
          >
            {server.usage?.last_used
              ? `[DORMANT ${Math.floor((Date.now() - new Date(server.usage.last_used).getTime()) / 86400000)}d]`
              : '[NEVER]'}
          </span>
        )}
        <span
          style={{
            marginLeft: 'auto',
            fontFamily: 'var(--font-mono)',
            fontSize: '10px',
            color: 'var(--text-secondary)',
            flexShrink: 0,
            paddingLeft: '8px',
          }}
        >
          {open ? '▲' : '▼'}
        </span>
      </button>

      {open && (
        <div style={{ marginTop: '8px', paddingLeft: '4px' }}>
          {/* Source path */}
          <div style={{ marginBottom: '6px' }}>
            <span class="stat-label" style={{ fontSize: '10px', display: 'block', marginBottom: '2px' }}>
              Source
            </span>
            <span style={{ fontFamily: 'var(--font-mono)', fontSize: '10px', color: 'var(--text-secondary)', wordBreak: 'break-all' }}>
              {server.source_path}
            </span>
          </div>

          {/* Managed by */}
          {server.managed_by && (
            <div style={{ marginBottom: '6px' }}>
              <span class="stat-label" style={{ fontSize: '10px', display: 'block', marginBottom: '2px' }}>
                Managed by
              </span>
              <span style={{ fontFamily: 'var(--font-mono)', fontSize: '10px', color: 'var(--text-secondary)' }}>
                {server.managed_by}
              </span>
            </div>
          )}

          {/* Transport details */}
          {server.transport.kind === 'stdio' && (
            <div style={{ marginBottom: '6px' }}>
              <span class="stat-label" style={{ fontSize: '10px', display: 'block', marginBottom: '2px' }}>
                Command
              </span>
              <span style={{ fontFamily: 'var(--font-mono)', fontSize: '10px', color: 'var(--text-secondary)', wordBreak: 'break-all' }}>
                {server.transport.command}
                {server.transport.args.length > 0 && (
                  <span style={{ opacity: 0.7 }}> {server.transport.args.join(' ')}</span>
                )}
              </span>
            </div>
          )}
          {(server.transport.kind === 'http' || server.transport.kind === 'sse') && (
            <div style={{ marginBottom: '6px' }}>
              <span class="stat-label" style={{ fontSize: '10px', display: 'block', marginBottom: '2px' }}>
                URL
              </span>
              <span style={{ fontFamily: 'var(--font-mono)', fontSize: '10px', color: 'var(--text-secondary)', wordBreak: 'break-all' }}>
                {server.transport.url}
              </span>
            </div>
          )}

          {/* Env table */}
          {envKeys.length > 0 && (
            <div style={{ marginBottom: '6px' }}>
              <span class="stat-label" style={{ fontSize: '10px', display: 'block', marginBottom: '4px' }}>
                Environment
              </span>
              <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '10px' }}>
                <thead>
                  <tr>
                    <th style={{ textAlign: 'left', padding: '2px 4px', color: 'var(--text-secondary)', fontFamily: 'var(--font-mono)', fontWeight: 'normal', textTransform: 'uppercase', letterSpacing: '0.05em', width: '40%' }}>KEY</th>
                    <th style={{ textAlign: 'left', padding: '2px 4px', color: 'var(--text-secondary)', fontFamily: 'var(--font-mono)', fontWeight: 'normal', textTransform: 'uppercase', letterSpacing: '0.05em' }}>VALUE</th>
                  </tr>
                </thead>
                <tbody>
                  {envKeys.map((key) => {
                    const val = server.env[key];
                    if (!val) return null;
                    return (
                      <tr key={key}>
                        <td style={{ padding: '2px 4px', fontFamily: 'var(--font-mono)', color: 'var(--text-secondary)', verticalAlign: 'top' }}>
                          {key}
                        </td>
                        <td style={{ padding: '2px 4px', verticalAlign: 'top' }}>
                          <RedactedValueCell val={val} />
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          )}

          {/* Log probe */}
          {server.log_probe && (
            <div style={{ marginBottom: '6px' }}>
              <span class="stat-label" style={{ fontSize: '10px', display: 'block', marginBottom: '2px' }}>
                Log
              </span>
              <span style={{ fontFamily: 'var(--font-mono)', fontSize: '10px', color: 'var(--text-secondary)' }}>
                {server.log_probe.path}
                <span style={{ marginLeft: '6px', opacity: 0.6 }}>
                  {fmtBytes(server.log_probe.bytes)} · {server.log_probe.recent_line_count} recent lines · {relativeTime(server.log_probe.modified)}
                </span>
              </span>
            </div>
          )}

          {/* Usage stats */}
          {server.usage && (
            <div style={{ marginBottom: '2px' }}>
              <span class="stat-label" style={{ fontSize: '10px', display: 'block', marginBottom: '2px' }}>
                Usage
              </span>
              <span style={{ fontFamily: 'var(--font-mono)', fontSize: '10px', color: 'var(--text-secondary)' }}>
                {server.usage.total_calls} calls
                {server.usage.last_used && <span> · last {relativeTime(server.usage.last_used)}</span>}
                <span> · {server.usage.distinct_sessions} sessions</span>
                <span> · {server.usage.distinct_tools} tools</span>
              </span>
            </div>
          )}
          {!server.usage && (
            <div>
              <span style={{ fontFamily: 'var(--font-mono)', fontSize: '10px', color: 'var(--text-secondary)', opacity: 0.5 }}>
                [never invoked]
              </span>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

function ProviderColumn({ title, servers }: { title: string; servers: McpServerEntry[] }) {
  if (servers.length === 0) return null;
  return (
    <div style={{ flex: '1 1 0', minWidth: '0' }}>
      <div class="stat-label" style={{ fontSize: '10px', marginBottom: '8px' }}>
        {title} ({servers.length})
      </div>
      {servers.map((s) => (
        <ServerRow key={`${s.provider}:${s.name}:${s.scope}`} server={s} />
      ))}
    </div>
  );
}

function McpServersCardInner({ report }: { report: McpServersReport }) {
  const t = report.totals;
  const hasNeverInvoked = t.never_invoked_count > 0;
  const showTwoColumns = report.claude.length > 0 && report.codex.length > 0;

  return (
    <div class="card" style={{ padding: '16px' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'baseline', marginBottom: '12px' }}>
        <div class="stat-label">MCP servers</div>
        {hasNeverInvoked && (
          <span style={{ fontFamily: 'var(--font-mono)', fontSize: '11px', color: 'var(--text-secondary)', opacity: 0.7 }}>
            [{t.never_invoked_count} never invoked]
          </span>
        )}
      </div>

      {/* KPI row */}
      <div style={{ display: 'flex', gap: '16px', flexWrap: 'wrap', marginBottom: '14px' }}>
        <div>
          <div class="stat-label" style={{ fontSize: '10px' }}>Configured</div>
          <div style={{ fontFamily: 'var(--font-mono)', fontSize: '18px' }}>{t.configured_count}</div>
        </div>
        <div>
          <div class="stat-label" style={{ fontSize: '10px' }}>Running</div>
          <div
            style={{
              fontFamily: 'var(--font-mono)',
              fontSize: '18px',
              color: t.running_count > 0 ? 'var(--success, #4caf50)' : undefined,
            }}
          >
            {t.running_count}
          </div>
        </div>
        <div>
          <div class="stat-label" style={{ fontSize: '10px' }}>Never invoked</div>
          <div
            style={{
              fontFamily: 'var(--font-mono)',
              fontSize: '18px',
              color: t.never_invoked_count > 0 ? 'var(--text-secondary)' : undefined,
              opacity: t.never_invoked_count > 0 ? 0.7 : 1,
            }}
          >
            {t.never_invoked_count}
          </div>
        </div>
        <div>
          <div class="stat-label" style={{ fontSize: '10px' }}>Claude</div>
          <div style={{ fontFamily: 'var(--font-mono)', fontSize: '18px' }}>{t.claude_count}</div>
        </div>
        <div>
          <div class="stat-label" style={{ fontSize: '10px' }}>Codex</div>
          <div style={{ fontFamily: 'var(--font-mono)', fontSize: '18px' }}>{t.codex_count}</div>
        </div>
        <div>
          <div class="stat-label" style={{ fontSize: '10px' }}>Projects</div>
          <div style={{ fontFamily: 'var(--font-mono)', fontSize: '18px' }}>{t.project_count}</div>
        </div>
        {t.dormant_count > 0 && (
          <div>
            <div class="stat-label" style={{ fontSize: '10px' }}>Dormant</div>
            <div style={{ fontFamily: 'var(--font-mono)', fontSize: '18px', color: 'var(--text-secondary)', opacity: 0.7 }}>
              {t.dormant_count}
            </div>
          </div>
        )}
      </div>

      {/* Server lists */}
      {t.configured_count === 0 ? (
        <div style={{ fontFamily: 'var(--font-mono)', fontSize: '11px', color: 'var(--text-secondary)', opacity: 0.6 }}>
          No MCP servers configured.
        </div>
      ) : (
        <div style={{ display: 'flex', gap: '24px', alignItems: 'flex-start' }}>
          {showTwoColumns ? (
            <>
              <ProviderColumn title="Claude Code" servers={report.claude} />
              <div style={{ width: '1px', background: 'rgba(var(--text-primary-rgb, 232,232,232), 0.08)', alignSelf: 'stretch', flexShrink: 0 }} />
              <ProviderColumn title="Codex" servers={report.codex} />
            </>
          ) : (
            <>
              {report.claude.length > 0 && <ProviderColumn title="Claude Code" servers={report.claude} />}
              {report.codex.length > 0 && <ProviderColumn title="Codex" servers={report.codex} />}
            </>
          )}
        </div>
      )}

      <div style={{ marginTop: '8px', fontSize: '10px', color: 'var(--text-secondary)', fontFamily: 'var(--font-mono)' }}>
        generated {relativeTime(report.generated_at)}
      </div>
    </div>
  );
}

export function McpServersCard() {
  const report = mcpServersReport.value;
  const loadState = mcpServersLoadState.value;

  useEffect(() => {
    if (report !== null || loadState === 'loading') return;
    mcpServersLoadState.value = 'loading';
    fetch('/api/mcp-servers')
      .then((r) => r.json())
      .then((data: McpServersReport) => {
        mcpServersReport.value = data;
        mcpServersLoadState.value = 'idle';
      })
      .catch(() => {
        mcpServersLoadState.value = 'error';
      });
  }, []);

  if (loadState === 'loading') {
    return (
      <div class="card" style={{ padding: '16px' }}>
        <div class="stat-label">MCP servers</div>
        <div style={{ color: 'var(--text-secondary)', fontFamily: 'var(--font-mono)', fontSize: '12px', marginTop: '8px' }}>
          scanning...
        </div>
      </div>
    );
  }

  if (loadState === 'error' || !report) {
    return (
      <div class="card" style={{ padding: '16px' }}>
        <div class="stat-label">MCP servers</div>
        <div style={{ color: 'var(--accent)', fontFamily: 'var(--font-mono)', fontSize: '12px', marginTop: '8px' }}>
          [ERROR: failed to load MCP servers data]
        </div>
      </div>
    );
  }

  return <McpServersCardInner report={report} />;
}
