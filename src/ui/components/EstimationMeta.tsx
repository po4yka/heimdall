interface EstimationMetaProps {
  confidenceBreakdown: Array<[string, { sessions: number; cost: number }]>;
  billingModeBreakdown: Array<[string, { sessions: number; cost: number }]>;
  pricingVersions: string[];
}

export function EstimationMeta({
  confidenceBreakdown,
  billingModeBreakdown,
  pricingVersions,
}: EstimationMetaProps) {
  return (
    <>
      <div class="card stat-card">
        <div class="stat-content">
          <div class="stat-label">Cost Confidence</div>
          <div class="stat-value" style={{ fontSize: '18px' }}>
            {confidenceBreakdown.length
              ? confidenceBreakdown.map(([key, value]) => `${key} ${value.sessions}`).join(' / ')
              : 'n/a'}
          </div>
          <div class="stat-sub">Session mix in current filter</div>
        </div>
      </div>
      <div class="card stat-card">
        <div class="stat-content">
          <div class="stat-label">Billing Mode</div>
          <div class="stat-value" style={{ fontSize: '18px' }}>
            {billingModeBreakdown.length
              ? billingModeBreakdown.map(([key, value]) => `${key} ${value.sessions}`).join(' / ')
              : 'n/a'}
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
                ? pricingVersions[0]
                : `mixed (${pricingVersions.length})`}
          </div>
          <div class="stat-sub">Stored per-session pricing metadata</div>
        </div>
      </div>
    </>
  );
}
