import type { JSX } from 'preact';
import { SegmentedProgressBar } from '../components/SegmentedProgressBar';
import { DepletionForecastCard } from '../components/DepletionForecastCard';
import { fmt, fmtCostCompact, fmtRelativeTime, fmtResetTime } from '../lib/format';
import type {
  LiveMonitorBlock,
  LiveMonitorContextWindow,
  LiveMonitorFocus,
  LiveMonitorProvider,
  LiveMonitorResponse,
} from '../state/types';
import { liveMonitorData, liveMonitorFocus } from './store';

function providersForFocus(data: LiveMonitorResponse, focus: LiveMonitorFocus): LiveMonitorProvider[] {
  return focus === 'all'
    ? data.providers
    : data.providers.filter(provider => provider.provider === focus);
}

function detailProviders(data: LiveMonitorResponse, focus: LiveMonitorFocus): LiveMonitorProvider[] {
  if (focus !== 'all') {
    return data.providers.filter(provider => provider.provider === focus);
  }
  return data.providers.filter(provider =>
    provider.active_block
    || provider.context_window
    || provider.recent_session
    || provider.depletion_forecast
    || provider.warnings.length > 0
  );
}

function stateTone(state: LiveMonitorProvider['visual_state']): string {
  switch (state) {
    case 'error':
      return 'var(--accent)';
    case 'incident':
      return 'var(--warning)';
    case 'degraded':
      return 'var(--warning)';
    case 'stale':
      return 'var(--text-secondary)';
    default:
      return 'var(--text-primary)';
  }
}

function stateLabel(state: LiveMonitorProvider['visual_state']): string {
  return state.toUpperCase();
}

function ProviderLaneCard({ provider }: { provider: LiveMonitorProvider }) {
  return (
    <div class="card" style={{ display: 'grid', gap: '14px' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', gap: '12px', alignItems: 'flex-start' }}>
        <div>
          <div class="stat-label" style={{ marginBottom: '6px' }}>{provider.title}</div>
          <div style={{ fontSize: '28px', lineHeight: 1.1 }}>{fmtCostCompact(provider.today_cost_usd)}</div>
          <div class="stat-sub">Today cost</div>
        </div>
        <div
          style={{
            border: '1px solid var(--border-visible)',
            borderRadius: '999px',
            padding: '4px 8px',
            fontFamily: 'var(--font-mono)',
            fontSize: '10px',
            letterSpacing: '0.08em',
            color: stateTone(provider.visual_state),
          }}
        >
          {stateLabel(provider.visual_state)}
        </div>
      </div>

      <div style={{ display: 'grid', gap: '12px' }}>
        {[provider.primary, provider.secondary].filter(Boolean).map((window, index) => (
          <div key={`${provider.provider}-${index}`} style={{ display: 'grid', gap: '6px' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', gap: '12px' }}>
              <span class="stat-label">{index === 0 ? 'Primary' : 'Secondary'}</span>
              <span class="stat-sub">{window?.used_percent.toFixed(1)}% used</span>
            </div>
            <SegmentedProgressBar
              value={window?.used_percent ?? 0}
              max={100}
              status={window && window.used_percent >= 80 ? 'accent' : window && window.used_percent >= 50 ? 'warning' : 'success'}
              aria-label={`${provider.title} ${index === 0 ? 'primary' : 'secondary'} quota`}
            />
            <div class="stat-sub">
              {window?.resets_in_minutes != null ? `Resets in ${fmtResetTime(window.resets_in_minutes)}` : 'No reset time available'}
            </div>
          </div>
        ))}
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit,minmax(140px,1fr))', gap: '12px' }}>
        <div>
          <div class="stat-label">Weekly Projection</div>
          <div class="stat-value" style={{ fontSize: '20px' }}>
            {provider.projected_weekly_spend_usd != null ? fmtCostCompact(provider.projected_weekly_spend_usd) : '—'}
          </div>
        </div>
        <div>
          <div class="stat-label">Freshness</div>
          <div class="stat-value" style={{ fontSize: '20px' }}>{fmtRelativeTime(provider.last_refresh)}</div>
          <div class="stat-sub">{provider.last_refresh_label}</div>
        </div>
      </div>

      <div style={{ display: 'grid', gap: '4px' }}>
        <div class="stat-sub">{provider.source_label}</div>
        {provider.identity_label && <div class="stat-sub">{provider.identity_label}</div>}
        {provider.warnings.length > 0 && (
          <div class="stat-sub" style={{ color: stateTone(provider.visual_state) }}>
            {provider.warnings[0]}
          </div>
        )}
      </div>
    </div>
  );
}

function BlockPanel({ block }: { block: LiveMonitorBlock }) {
  const totalTokens =
    block.tokens.input +
    block.tokens.output +
    block.tokens.cache_read +
    block.tokens.cache_creation +
    block.tokens.reasoning_output;

  return (
    <div class="card stat-card">
      <div class="stat-content">
        <div class="stat-label">Active Block</div>
        <div class="stat-value">{fmt(totalTokens)}</div>
        <div class="stat-sub">{block.entry_count} entries · ends {new Date(block.end).toLocaleTimeString()}</div>
        {block.burn_rate && (
          <div class="stat-sub">
            {fmt(totalTokens)} tokens · {fmtCostCompact(block.burn_rate.cost_per_hour_nanos / 1e9)}/hr
          </div>
        )}
        {block.quota && (
          <div style={{ marginTop: '12px' }}>
            <SegmentedProgressBar
              value={block.quota.projected_pct * 100}
              max={100}
              status={block.quota.projected_severity === 'danger' ? 'accent' : block.quota.projected_severity === 'warn' ? 'warning' : 'success'}
              aria-label="Projected billing block quota"
            />
            <div class="stat-sub" style={{ marginTop: '8px' }}>
              {Math.min(block.quota.projected_pct * 100, 999).toFixed(0)}% projected · {fmt(block.quota.remaining_tokens)} tokens left
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

function ContextPanel({ data }: { data: LiveMonitorContextWindow }) {
  return (
    <div class="card stat-card">
      <div class="stat-content">
        <div class="stat-label">Context Window</div>
        <div class="stat-value">{fmt(data.total_input_tokens)}</div>
        <div class="stat-sub">of {fmt(data.context_window_size)} · {(data.pct * 100).toFixed(1)}%</div>
        <div style={{ marginTop: '12px' }}>
          <SegmentedProgressBar
            value={data.total_input_tokens}
            max={data.context_window_size}
            status={data.severity === 'danger' ? 'accent' : data.severity === 'warn' ? 'warning' : 'success'}
            aria-label="Context window usage"
          />
        </div>
      </div>
    </div>
  );
}

function SessionPanel({ provider }: { provider: LiveMonitorProvider }) {
  if (!provider.recent_session) return null;
  const session = provider.recent_session;
  return (
    <div class="card stat-card">
      <div class="stat-content">
        <div class="stat-label">Recent Session</div>
        <div class="stat-value" style={{ fontSize: '22px' }}>{provider.title}</div>
        <div class="stat-sub">{session.display_name}</div>
        <div class="stat-sub">{session.turns} turns · {session.duration_minutes}m · {fmtCostCompact(session.cost_usd)}</div>
        {session.model && <div class="stat-sub">{session.model}</div>}
      </div>
    </div>
  );
}

function QuotaSuggestionsPanel({ provider }: { provider: LiveMonitorProvider }) {
  const suggestions = provider.quota_suggestions;
  if (!suggestions || suggestions.levels.length === 0) {
    return null;
  }

  return (
    <div class="card stat-card">
      <div class="stat-content">
        <div class="stat-label">Suggested Quotas</div>
        <div class="stat-sub">{suggestions.sample_count} completed blocks</div>
        <div style={{ display: 'grid', gap: '8px', marginTop: '12px' }}>
          {suggestions.levels.map(level => (
            <div key={level.key} style={{ display: 'flex', justifyContent: 'space-between', gap: '12px', alignItems: 'baseline' }}>
              <span class="stat-sub">
                {level.label}
                {level.key === suggestions.recommended_key && (
                  <span style={{ marginLeft: '6px', color: 'var(--success)' }}>[RECOMMENDED]</span>
                )}
              </span>
              <span class="stat-value" style={{ fontSize: '18px' }}>{fmt(level.limit_tokens)}</span>
            </div>
          ))}
        </div>
        {suggestions.note && (
          <div class="stat-sub" style={{ marginTop: '10px', fontStyle: 'italic' }}>
            {suggestions.note}
          </div>
        )}
      </div>
    </div>
  );
}

function ProviderDetails({ provider }: { provider: LiveMonitorProvider }) {
  return (
    <section style={{ display: 'grid', gap: '14px' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', gap: '12px', alignItems: 'baseline', flexWrap: 'wrap' }}>
        <h2 style={{ margin: 0 }}>{provider.title} Details</h2>
        <div class="stat-sub">{provider.last_refresh_label}</div>
      </div>
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit,minmax(240px,1fr))', gap: '16px' }}>
        {provider.active_block && <BlockPanel block={provider.active_block} />}
        {provider.depletion_forecast && <DepletionForecastCard forecast={provider.depletion_forecast} />}
        <QuotaSuggestionsPanel provider={provider} />
        {provider.context_window && <ContextPanel data={provider.context_window} />}
        <SessionPanel provider={provider} />
      </div>
      {provider.warnings.length > 0 && (
        <div class="card" style={{ padding: '16px 18px' }}>
          <div class="stat-label">Warnings</div>
          <ul style={{ margin: '10px 0 0', paddingLeft: '18px' }}>
            {provider.warnings.map(warning => <li key={warning}>{warning}</li>)}
          </ul>
        </div>
      )}
    </section>
  );
}

export function renderLiveMonitorView(): JSX.Element {
  const data = liveMonitorData.value;
  if (!data) {
    return (
      <div class="card" style={{ padding: '20px' }}>
        <div class="stat-label">Live Monitor</div>
        <div class="stat-sub">Waiting for provider data…</div>
      </div>
    );
  }

  const laneProviders = providersForFocus(data, liveMonitorFocus.value);
  const details = detailProviders(data, liveMonitorFocus.value);

  return (
    <div style={{ display: 'grid', gap: '24px' }}>
      <section style={{ display: 'grid', gap: '14px' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', gap: '12px', alignItems: 'baseline', flexWrap: 'wrap' }}>
          <h2 style={{ margin: 0 }}>Provider Lanes</h2>
          <div class="stat-sub">
            {data.freshness.has_stale_providers ? `${data.freshness.stale_providers.join(', ')} stale` : 'All providers current'}
          </div>
        </div>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit,minmax(320px,1fr))', gap: '16px' }}>
          {laneProviders.map(provider => <ProviderLaneCard key={provider.provider} provider={provider} />)}
        </div>
      </section>

      {details.map(provider => <ProviderDetails key={`details-${provider.provider}`} provider={provider} />)}
    </div>
  );
}
