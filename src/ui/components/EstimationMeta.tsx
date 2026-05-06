import { fmtLabel } from '../lib/format';
import { KpiCard } from './_primitives/KpiCard';

interface EstimationMetaProps {
  confidenceBreakdown: Array<[string, { sessions: number; cost: number }]>;
  billingModeBreakdown: Array<[string, { sessions: number; cost: number }]>;
  pricingVersions: string[];
}

function formatPricingVersion(v: string): string {
  const m = v.match(/^\d{4}-\d{2}-\d{2}/);
  return m ? m[0] : v;
}

function formatBreakdown(
  rows: Array<[string, { sessions: number; cost: number }]>
): string {
  return rows
    .map(([key, value]) => `${fmtLabel(key)}: ${value.sessions.toLocaleString()}`)
    .join(' · ');
}

export function EstimationMeta({
  confidenceBreakdown,
  billingModeBreakdown,
  pricingVersions,
}: EstimationMetaProps) {
  const pricingValue =
    pricingVersions.length === 0
      ? 'n/a'
      : pricingVersions.length === 1
        ? formatPricingVersion(pricingVersions[0] ?? '')
        : `mixed (${pricingVersions.length})`;

  return (
    <>
      <KpiCard
        size="compact"
        label="Cost confidence"
        value={confidenceBreakdown.length ? formatBreakdown(confidenceBreakdown) : 'n/a'}
        sub="Session mix in current filter"
      />
      <KpiCard
        size="compact"
        label="Billing mode"
        value={billingModeBreakdown.length ? formatBreakdown(billingModeBreakdown) : 'n/a'}
        sub="Local estimate vs subscriber-included sessions"
      />
      <KpiCard
        size="compact"
        label="Pricing snapshot"
        value={pricingValue}
        sub="Stored per-session pricing metadata"
      />
    </>
  );
}
