import { fmt } from '../lib/format';
import type { ServiceTierSummary } from '../state/types';

export function ServiceTiersTable({ data }: { data: ServiceTierSummary[] }) {
  if (!data.length) return null;

  return (
    <div class="table-card">
      <div class="section-title">Service Tiers</div>
      <table>
        <thead>
          <tr>
            <th>Tier</th>
            <th>Region</th>
            <th>Turns</th>
          </tr>
        </thead>
        <tbody>
          {data.map(s => (
            <tr key={`${s.service_tier}-${s.inference_geo}`}>
              <td>{s.service_tier}</td>
              <td>{s.inference_geo}</td>
              <td class="num">{fmt(s.turns)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
