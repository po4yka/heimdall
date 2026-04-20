import { ApexChart } from './ApexChart';
import { cssVar } from '../../lib/charts';
import type { DailyAgg } from '../../state/types';

export function Sparkline({ daily }: { daily: DailyAgg[] }) {
  const last7 = daily.slice(-7);
  if (last7.length < 2) return null;

  const options = {
    chart: { type: 'line', height: 30, width: 120, sparkline: { enabled: true },
             background: 'transparent', fontFamily: 'inherit' },
    series: [{ data: last7.map(d => d.input + d.output) }],
    stroke: { width: 1.5, curve: 'smooth' },
    colors: [cssVar('--accent')],
    tooltip: { enabled: false },
  };

  return (
    <div>
      <div class="sub" style={{ marginBottom: '4px' }}>7-day trend</div>
      <ApexChart options={options} />
    </div>
  );
}
