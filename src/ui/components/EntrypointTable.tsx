import { fmt } from '../lib/format';
import type { EntrypointSummary } from '../state/types';

export function EntrypointTable({ data }: { data: EntrypointSummary[] }) {
  if (!data.length) return null;

  return (
    <div class="table-card">
      <div class="section-title">Usage by Entrypoint</div>
      <table>
        <thead>
          <tr>
            <th>Entrypoint</th>
            <th>Sessions</th>
            <th>Turns</th>
            <th>Input</th>
            <th>Output</th>
          </tr>
        </thead>
        <tbody>
          {data.map(e => (
            <tr key={e.entrypoint}>
              <td><span class="model-tag">{e.entrypoint}</span></td>
              <td class="num">{e.sessions}</td>
              <td class="num">{fmt(e.turns)}</td>
              <td class="num">{fmt(e.input)}</td>
              <td class="num">{fmt(e.output)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
