import type { JSX } from 'preact';
import { SegmentedProgressBar } from '../components/SegmentedProgressBar';
import { DepletionForecastCard } from '../components/DepletionForecastCard';
import { PredictiveInsightsCard } from '../components/PredictiveInsightsCard';
import { fmt, fmtCostCompact, fmtRelativeTime, fmtResetTime } from '../lib/format';
import type {
  LiveMonitorBlock,
  LiveMonitorContextWindow,
  LiveMonitorFocus,
  LiveMonitorProvider,
  LiveMonitorResponse,
} from '../state/types';
import {
  type LiveMonitorDensity,
  type LiveMonitorPanelId,
  liveMonitorData,
  liveMonitorDensity,
  liveMonitorFocus,
  liveMonitorHiddenPanels,
} from './store';

interface DensityTokens {
  padding: string;
  fontSize: string | undefined;
  marginTop: string;
  sectionGap: string;
  headerGap: string;
  gridGap: string;
  listGap: string;
}

function densityTokens(density: LiveMonitorDensity): DensityTokens {
  return density === 'compact'
    ? { padding: '14px', fontSize: '11px', marginTop: '10px', sectionGap: '10px', headerGap: '10px', gridGap: '12px', listGap: '6px' }
    : { padding: '18px', fontSize: undefined, marginTop: '12px', sectionGap: '14px', headerGap: '12px', gridGap: '16px', listGap: '8px' };
}

function providersForFocus(data: LiveMonitorResponse, focus: LiveMonitorFocus): LiveMonitorProvider[] {
  return focus === 'all'
    ? data.providers
    : data.providers.filter(provider => provider.provider === focus);
}

function providerHasVisibleDetails(
  provider: LiveMonitorProvider,
  hiddenPanels: Set<LiveMonitorPanelId>
): boolean {
  return (
    (!hiddenPanels.has('active_block') && !!provider.active_block) ||
    (!!provider.claude_admin) ||
    (!hiddenPanels.has('predictive_insights') && !!provider.predictive_insights) ||
    (!hiddenPanels.has('depletion_forecast') && !!provider.depletion_forecast) ||
    (!hiddenPanels.has('quota_suggestions') && !!provider.quota_suggestions) ||
    (!hiddenPanels.has('context_window') && !!provider.context_window) ||
    (!hiddenPanels.has('recent_session') && !!provider.recent_session) ||
    (!hiddenPanels.has('warnings') && provider.warnings.length > 0)
  );
}

function detailProviders(
  data: LiveMonitorResponse,
  focus: LiveMonitorFocus,
  hiddenPanels: Set<LiveMonitorPanelId>
): LiveMonitorProvider[] {
  if (focus !== 'all') {
    return data.providers.filter(provider =>
      provider.provider === focus && providerHasVisibleDetails(provider, hiddenPanels)
    );
  }
  return data.providers.filter(provider => providerHasVisibleDetails(provider, hiddenPanels));
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
  const hasAdminFallback = !!provider.claude_admin;
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

      {hasAdminFallback ? (
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit,minmax(140px,1fr))', gap: '12px' }}>
          <div>
            <div class="stat-label">Active Users Today</div>
            <div class="stat-value" style={{ fontSize: '20px' }}>{fmt(provider.claude_admin?.today_active_users ?? 0)}</div>
          </div>
          <div>
            <div class="stat-label">Sessions Today</div>
            <div class="stat-value" style={{ fontSize: '20px' }}>{fmt(provider.claude_admin?.today_sessions ?? 0)}</div>
          </div>
          <div>
            <div class="stat-label">Accepted Lines</div>
            <div class="stat-value" style={{ fontSize: '20px' }}>{fmt(provider.claude_admin?.lookback_lines_accepted ?? 0)}</div>
            <div class="stat-sub">{provider.claude_admin?.lookback_days ?? 0}d window</div>
          </div>
          <div>
            <div class="stat-label">Estimated Spend</div>
            <div class="stat-value" style={{ fontSize: '20px' }}>{fmtCostCompact(provider.claude_admin?.lookback_estimated_cost_usd ?? 0)}</div>
            <div class="stat-sub">{provider.claude_admin?.data_latency_note}</div>
          </div>
        </div>
      ) : (
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
      )}

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

function BlockPanel({ block, density }: { block: LiveMonitorBlock; density: LiveMonitorDensity }) {
  const totalTokens =
    block.tokens.input +
    block.tokens.output +
    block.tokens.cache_read +
    block.tokens.cache_creation +
    block.tokens.reasoning_output;
  const d = densityTokens(density);

  return (
    <div class="card stat-card" style={{ padding: d.padding }}>
      <div class="stat-content">
        <div class="stat-label">Active Block</div>
        <div class="stat-value">{fmt(totalTokens)}</div>
        <div class="stat-sub" style={{ fontSize: d.fontSize }}>
          {block.entry_count} entries · ends {new Date(block.end).toLocaleTimeString()}
        </div>
        {block.burn_rate && (
          <div class="stat-sub" style={{ fontSize: d.fontSize }}>
            {fmt(totalTokens)} tokens · {fmtCostCompact(block.burn_rate.cost_per_hour_nanos / 1e9)}/hr
          </div>
        )}
        {block.projection && (
          <div class="stat-sub" style={{ fontSize: d.fontSize }}>
            Projects {fmt(block.projection.projected_tokens)} tokens · {fmtCostCompact(block.projection.projected_cost_nanos / 1e9)}
          </div>
        )}
        {block.quota && (
          <div style={{ marginTop: d.marginTop }}>
            <SegmentedProgressBar
              value={block.quota.projected_pct * 100}
              max={100}
              status={block.quota.projected_severity === 'danger' ? 'accent' : block.quota.projected_severity === 'warn' ? 'warning' : 'success'}
              aria-label="Projected billing block quota"
            />
            <div class="stat-sub" style={{ marginTop: '8px', fontSize: d.fontSize }}>
              {Math.min(block.quota.projected_pct * 100, 999).toFixed(0)}% projected · {fmt(block.quota.remaining_tokens)} tokens left
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

function ContextPanel({ data, density }: { data: LiveMonitorContextWindow; density: LiveMonitorDensity }) {
  const d = densityTokens(density);
  return (
    <div class="card stat-card" style={{ padding: d.padding }}>
      <div class="stat-content">
        <div class="stat-label">Context Window</div>
        <div class="stat-value">{fmt(data.total_input_tokens)}</div>
        <div class="stat-sub" style={{ fontSize: d.fontSize }}>
          of {fmt(data.context_window_size)} · {(data.pct * 100).toFixed(1)}%
        </div>
        <div style={{ marginTop: d.marginTop }}>
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

function SessionPanel({ provider, density }: { provider: LiveMonitorProvider; density: LiveMonitorDensity }) {
  if (!provider.recent_session) return null;
  const session = provider.recent_session;
  const d = densityTokens(density);
  return (
    <div class="card stat-card" style={{ padding: d.padding }}>
      <div class="stat-content">
        <div class="stat-label">Recent Session</div>
        <div class="stat-value" style={{ fontSize: '22px' }}>{provider.title}</div>
        <div class="stat-sub" style={{ fontSize: d.fontSize }}>{session.display_name}</div>
        <div class="stat-sub" style={{ fontSize: d.fontSize }}>
          {session.turns} turns · {session.duration_minutes}m · {fmtCostCompact(session.cost_usd)}
        </div>
        {session.model && <div class="stat-sub" style={{ fontSize: d.fontSize }}>{session.model}</div>}
      </div>
    </div>
  );
}

function QuotaSuggestionsPanel({
  provider,
  density,
}: {
  provider: LiveMonitorProvider;
  density: LiveMonitorDensity;
}) {
  const suggestions = provider.quota_suggestions;
  if (!suggestions || suggestions.levels.length === 0) {
    return null;
  }

  const d = densityTokens(density);
  return (
    <div class="card stat-card" style={{ padding: d.padding }}>
      <div class="stat-content">
        <div class="stat-label">Suggested Quotas</div>
        <div class="stat-sub" style={{ fontSize: d.fontSize }}>
          {suggestions.sample_label}
        </div>
        <div style={{ display: 'grid', gap: d.listGap, marginTop: d.marginTop }}>
          {suggestions.levels.map(level => (
            <div key={level.key} style={{ display: 'flex', justifyContent: 'space-between', gap: '12px', alignItems: 'baseline' }}>
              <span class="stat-sub" style={{ fontSize: d.fontSize }}>
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
          <div class="stat-sub" style={{ marginTop: '10px', fontStyle: 'italic', fontSize: d.fontSize }}>
            {suggestions.note}
          </div>
        )}
        {suggestions.sample_count !== suggestions.population_count && (
          <div class="stat-sub" style={{ fontSize: d.fontSize }}>
            Drawn from {suggestions.population_count} completed blocks, weighted toward near-limit history.
          </div>
        )}
      </div>
    </div>
  );
}

function ProviderDetails({
  provider,
  density,
  hiddenPanels,
}: {
  provider: LiveMonitorProvider;
  density: LiveMonitorDensity;
  hiddenPanels: Set<LiveMonitorPanelId>;
}) {
  const d = densityTokens(density);
  return (
    <section style={{ display: 'grid', gap: d.sectionGap }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', gap: d.headerGap, alignItems: 'baseline', flexWrap: 'wrap' }}>
        <h2 style={{ margin: 0 }}>{provider.title} Details</h2>
        <div class="stat-sub" style={{ fontSize: d.fontSize }}>{provider.last_refresh_label}</div>
      </div>
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit,minmax(240px,1fr))', gap: d.gridGap }}>
        {!hiddenPanels.has('active_block') && provider.active_block && <BlockPanel block={provider.active_block} density={density} />}
        {!hiddenPanels.has('predictive_insights') && provider.predictive_insights && (
          <PredictiveInsightsCard insights={provider.predictive_insights} />
        )}
        {!hiddenPanels.has('depletion_forecast') && provider.depletion_forecast && <DepletionForecastCard forecast={provider.depletion_forecast} />}
        {!hiddenPanels.has('quota_suggestions') && <QuotaSuggestionsPanel provider={provider} density={density} />}
        {!hiddenPanels.has('context_window') && provider.context_window && <ContextPanel data={provider.context_window} density={density} />}
        {!hiddenPanels.has('recent_session') && <SessionPanel provider={provider} density={density} />}
      </div>
      {!hiddenPanels.has('warnings') && provider.warnings.length > 0 && (
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
  const hiddenPanels = new Set(liveMonitorHiddenPanels.value);
  const density = liveMonitorDensity.value;
  const details = detailProviders(data, liveMonitorFocus.value, hiddenPanels);

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

      {details.map(provider => (
        <ProviderDetails
          key={`details-${provider.provider}`}
          provider={provider}
          density={density}
          hiddenPanels={hiddenPanels}
        />
      ))}
    </div>
  );
}
