interface EstimationMetaProps {
  confidenceBreakdown: Array<[string, { sessions: number; cost: number }]>;
  billingModeBreakdown: Array<[string, { sessions: number; cost: number }]>;
  pricingVersions: string[];
}

function humanizeKey(key: string): string {
  return key
    .split(/[_\s-]+/)
    .filter(Boolean)
    .map(w => w.charAt(0).toUpperCase() + w.slice(1).toLowerCase())
    .join(' ');
}

function formatPricingVersion(v: string): string {
  const m = v.match(/^\d{4}-\d{2}-\d{2}/);
  return m ? m[0] : v;
}

export function EstimationMeta({
  confidenceBreakdown,
  billingModeBreakdown,
  pricingVersions,
}: EstimationMetaProps) {
  const formatBreakdown = (rows: Array<[string, { sessions: number; cost: number }]>) =>
    rows.map(([key, value]) => `${humanizeKey(key)}: ${value.sessions.toLocaleString()}`).join(' · ');

  return (
    <>
      <div class="card stat-card">
        <div class="stat-content">
          <div class="stat-label">Cost Confidence</div>
          <div class="stat-value" style={{ fontSize: '18px' }}>
            {confidenceBreakdown.length ? formatBreakdown(confidenceBreakdown) : 'n/a'}
          </div>
          <div class="stat-sub">Session mix in current filter</div>
        </div>
      </div>
      <div class="card stat-card">
        <div class="stat-content">
          <div class="stat-label">Billing Mode</div>
          <div class="stat-value" style={{ fontSize: '18px' }}>
            {billingModeBreakdown.length ? formatBreakdown(billingModeBreakdown) : 'n/a'}
          </div>
          <div class="stat-sub">Local estimate vs subscriber-included sessions</div>
        </div>
      </div>
      <div class="card stat-card">
        <div class="stat-content">
          <div class="stat-label">Pricing Snapshot</div>
          <div class="stat-value" style={{ fontSize: '18px' }}>
            {pricingVersions.length === 0
              ? 'n/a'
              : pricingVersions.length === 1
                ? formatPricingVersion(pricingVersions[0])
                : `mixed (${pricingVersions.length})`}
          </div>
          <div class="stat-sub">Stored per-session pricing metadata</div>
        </div>
      </div>
    </>
  );
}
