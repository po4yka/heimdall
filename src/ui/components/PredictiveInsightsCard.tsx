import { fmt, fmtCostCompact } from '../lib/format';
import type { HistoricalEnvelope, LimitHitAnalysis, PredictiveBurnRate, PredictiveInsights } from '../state/types';

function fmtTokensPerMin(value: number): string {
  return `${fmt(Math.round(value))}/min`;
}

function burnTone(tier: string): string | undefined {
  if (tier === 'high') return 'var(--accent)';
  if (tier === 'moderate') return 'var(--warning)';
  return undefined;
}

function riskTone(level: string): string | undefined {
  if (level === 'high') return 'var(--accent)';
  if (level === 'elevated') return 'var(--warning)';
  return undefined;
}

function EnvelopeRow({
  label,
  average,
  p50,
  p75,
  p90,
  p95,
  formatter,
}: {
  label: string;
  average: number;
  p50: number;
  p75: number;
  p90: number;
  p95: number;
  formatter: (value: number) => string;
}) {
  return (
    <div style={{ display: 'grid', gap: '4px' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', gap: '12px', alignItems: 'baseline' }}>
        <span class="stat-sub">{label}</span>
        <span class="stat-sub" style={{ fontFamily: 'var(--font-mono)', fontSize: '11px' }}>
          avg {formatter(average)}
        </span>
      </div>
      <div class="stat-sub" style={{ fontFamily: 'var(--font-mono)', fontSize: '11px' }}>
        P50 {formatter(p50)} · P75 {formatter(p75)} · P90 {formatter(p90)} · P95 {formatter(p95)}
      </div>
    </div>
  );
}

function HistoricalEnvelopeSection({ envelope }: { envelope: HistoricalEnvelope }) {
  return (
    <div style={{ display: 'grid', gap: '8px' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', gap: '12px', alignItems: 'baseline' }}>
        <span class="stat-sub" style={{ fontSize: '10px', letterSpacing: '0.08em' }}>HISTORICAL ENVELOPES</span>
        <span class="stat-sub" style={{ fontFamily: 'var(--font-mono)', fontSize: '11px' }}>
          {envelope.sample_count} completed blocks
        </span>
      </div>
      <EnvelopeRow label="Tokens" formatter={value => fmt(value)} {...envelope.tokens} />
      <EnvelopeRow label="Cost" formatter={value => fmtCostCompact(value)} {...envelope.cost_usd} />
      <EnvelopeRow label="Turns" formatter={value => fmt(value)} {...envelope.turns} />
    </div>
  );
}

function RollingBurnSection({ burn }: { burn: PredictiveBurnRate }) {
  return (
    <div style={{ display: 'grid', gap: '6px' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', gap: '12px', alignItems: 'baseline' }}>
        <span class="stat-sub" style={{ fontSize: '10px', letterSpacing: '0.08em' }}>ROLLING 1H BURN</span>
        <span class="stat-sub" style={{ color: burnTone(String(burn.tier)) }}>
          {String(burn.tier).toUpperCase()}
        </span>
      </div>
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit,minmax(120px,1fr))', gap: '8px' }}>
        <div>
          <div class="stat-sub">Tokens</div>
          <div class="stat-value" style={{ fontSize: '20px' }}>{fmtTokensPerMin(burn.tokens_per_min)}</div>
        </div>
        <div>
          <div class="stat-sub">Cost</div>
          <div class="stat-value" style={{ fontSize: '20px' }}>{fmtCostCompact(burn.cost_per_hour_nanos / 1e9)}/hr</div>
        </div>
        <div>
          <div class="stat-sub">Coverage</div>
          <div class="stat-value" style={{ fontSize: '20px' }}>{burn.coverage_minutes}m</div>
        </div>
      </div>
    </div>
  );
}

function LimitHitSection({ analysis }: { analysis: LimitHitAnalysis }) {
  return (
    <div style={{ display: 'grid', gap: '6px' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', gap: '12px', alignItems: 'baseline' }}>
        <span class="stat-sub" style={{ fontSize: '10px', letterSpacing: '0.08em' }}>LIMIT-HIT RISK</span>
        <span class="stat-sub" style={{ color: riskTone(analysis.risk_level) }}>
          {analysis.risk_level.toUpperCase()}
        </span>
      </div>
      <div class="stat-sub">
        {analysis.summary_label}
      </div>
      <div class="stat-sub" style={{ fontFamily: 'var(--font-mono)', fontSize: '11px' }}>
        {analysis.hit_count}/{analysis.sample_count} hits · {(analysis.hit_rate * 100).toFixed(0)}% rate · threshold {fmt(analysis.threshold_tokens)}
      </div>
    </div>
  );
}

export function PredictiveInsightsCard({
  insights,
  title = 'Predictive Signals',
}: {
  insights: PredictiveInsights;
  title?: string;
}) {
  const hasAny =
    !!insights.rolling_hour_burn ||
    !!insights.historical_envelope ||
    !!insights.limit_hit_analysis;
  if (!hasAny) {
    return null;
  }

  return (
    <div class="card stat-card">
      <div class="stat-content" style={{ display: 'grid', gap: '12px' }}>
        <div>
          <div class="stat-label">{title}</div>
          <div class="stat-sub">Heuristic forecast from rolling burn and completed billing blocks.</div>
        </div>

        {insights.rolling_hour_burn && <RollingBurnSection burn={insights.rolling_hour_burn} />}
        {insights.limit_hit_analysis && <LimitHitSection analysis={insights.limit_hit_analysis} />}
        {insights.historical_envelope && <HistoricalEnvelopeSection envelope={insights.historical_envelope} />}
      </div>
    </div>
  );
}
