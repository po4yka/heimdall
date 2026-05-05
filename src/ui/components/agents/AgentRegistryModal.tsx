import { useState, useEffect, useCallback } from 'preact/hooks';
import { registryModalOpen } from '../../state/store';
import { InlineStatus } from '../InlineStatus';
import { setStatus, clearStatus } from '../../lib/status';
import {
  fetchRegistry,
  upsertRole,
  deleteRole,
  type RegistryUpsertBody,
} from '../../lib/agents';
import type { AgentRegistryRow, AgentTelemetry } from '../../state/types';
import { esc } from '../../lib/format';

interface AgentRegistryModalProps {
  project: string;
  telemetry: AgentTelemetry;
  onReload: () => Promise<void>;
}

interface RowState {
  display_name: string;
  description: string;
  enabled: boolean;
  merged_into: string;
}

function initialRowState(row: AgentRegistryRow | undefined): RowState {
  return {
    display_name: row?.display_name ?? '',
    description: row?.description ?? '',
    enabled: row?.enabled ?? true,
    merged_into: row?.merged_into ?? '',
  };
}

export function AgentRegistryModal({ project, telemetry, onReload }: AgentRegistryModalProps) {
  const [registryRows, setRegistryRows] = useState<AgentRegistryRow[]>([]);
  const [loading, setLoading] = useState(true);
  const [rowStates, setRowStates] = useState<Record<string, RowState>>({});

  // All detected roles for this project (classified or not)
  const detectedForProject = telemetry.detected.filter(d => d.project === project);

  // All raw roles to display: union of detected + registry rows
  const allRoles = [
    ...new Set([
      ...detectedForProject.map(d => d.raw_role),
      ...registryRows.map(r => r.raw_role),
    ]),
  ].filter(r => r !== 'unknown');

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const resp = await fetchRegistry(project);
      setRegistryRows(resp.registry);
      const states: Record<string, RowState> = {};
      for (const rawRole of allRoles) {
        const existing = resp.registry.find(r => r.raw_role === rawRole);
        states[rawRole] = initialRowState(existing);
      }
      setRowStates(states);
    } catch {
      // keep empty
    } finally {
      setLoading(false);
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [project]);

  useEffect(() => { void load(); }, [load]);

  // Close on ESC
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') registryModalOpen.value = null;
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, []);

  function updateRow(rawRole: string, patch: Partial<RowState>) {
    setRowStates(prev => ({
      ...prev,
      [rawRole]: { ...prev[rawRole]!, ...patch },
    }));
  }

  async function handleSave(rawRole: string) {
    const state = rowStates[rawRole];
    if (!state) return;
    const body: RegistryUpsertBody = {
      display_name: state.display_name || null,
      description: state.description || null,
      enabled: state.enabled,
      merged_into: state.merged_into || null,
    };
    // Guard: self-merge
    if (state.merged_into && state.merged_into === rawRole) {
      setStatus('agent-registry', 'error', 'cannot merge a role into itself', 2000);
      return;
    }
    clearStatus('agent-registry');
    try {
      await upsertRole(project, rawRole, body);
      setStatus('agent-registry', 'success', 'SAVED', 1500);
      await onReload();
      await load();
    } catch (err) {
      setStatus('agent-registry', 'error', err instanceof Error ? err.message : String(err), 3000);
    }
  }

  async function handleDelete(rawRole: string) {
    if (!window.confirm(`Delete registry entry for "${rawRole}"?`)) return;
    clearStatus('agent-registry');
    try {
      await deleteRole(project, rawRole);
      setStatus('agent-registry', 'success', 'DELETED', 1500);
      await onReload();
      await load();
    } catch (err) {
      setStatus('agent-registry', 'error', err instanceof Error ? err.message : String(err), 3000);
    }
  }

  // Other detected roles for merged_into dropdown
  const mergeOptions = allRoles.filter(r => r !== 'unknown');

  return (
    <div class="agent-registry-overlay" onClick={() => registryModalOpen.value = null}>
      <div
        class="agent-registry-modal"
        onClick={(e: Event) => e.stopPropagation()}
        role="dialog"
        aria-modal="true"
        aria-label={`Agent registry — ${project}`}
      >
        <div class="agent-registry-header">
          <h2 class="agent-registry-title">Agent registry — {esc(project)}</h2>
          <button
            type="button"
            class="agent-registry-close"
            aria-label="Close"
            onClick={() => registryModalOpen.value = null}
          >
            [X]
          </button>
        </div>

        <div style={{ padding: '0 20px 8px' }}>
          <InlineStatus placement="agent-registry" inline />
        </div>

        {loading ? (
          <div style={{ padding: '20px', color: 'var(--text-secondary)', fontFamily: 'var(--font-mono)', fontSize: '11px' }}>
            Loading…
          </div>
        ) : allRoles.length === 0 ? (
          <div class="empty-state" style={{ margin: '20px' }}>No agent roles detected for this project</div>
        ) : (
          <div class="agent-registry-table-wrap">
            <table class="agent-registry-table">
              <thead>
                <tr>
                  <th>ROLE</th>
                  <th>DISPLAY NAME</th>
                  <th>DESCRIPTION</th>
                  <th>ENABLED</th>
                  <th>MERGED INTO</th>
                  <th>CONFIDENCE</th>
                  <th>ACTIONS</th>
                </tr>
              </thead>
              <tbody>
                {allRoles.map(rawRole => {
                  const state = rowStates[rawRole] ?? initialRowState(undefined);
                  const registered = registryRows.find(r => r.raw_role === rawRole);
                  const detected = detectedForProject.find(d => d.raw_role === rawRole);
                  // confidence: show count as a simple "detected N times" badge
                  const countBadge = detected ? `${detected.count}×` : '—';

                  return (
                    <tr key={rawRole} class={!registered ? 'agent-row-unclassified' : ''}>
                      <td>
                        <span class="model-tag" title={rawRole}>{esc(rawRole)}</span>
                      </td>
                      <td>
                        <input
                          type="text"
                          class="agent-registry-input"
                          value={state.display_name}
                          placeholder="Display name"
                          onInput={(e) => updateRow(rawRole, { display_name: (e.target as HTMLInputElement).value })}
                        />
                      </td>
                      <td>
                        <input
                          type="text"
                          class="agent-registry-input"
                          value={state.description}
                          placeholder="Description"
                          onInput={(e) => updateRow(rawRole, { description: (e.target as HTMLInputElement).value })}
                        />
                      </td>
                      <td style={{ textAlign: 'center' }}>
                        <input
                          type="checkbox"
                          checked={state.enabled}
                          onChange={(e) => updateRow(rawRole, { enabled: (e.target as HTMLInputElement).checked })}
                        />
                      </td>
                      <td>
                        <select
                          class="agent-registry-select"
                          value={state.merged_into}
                          onChange={(e) => updateRow(rawRole, { merged_into: (e.target as HTMLSelectElement).value })}
                        >
                          <option value="">(none)</option>
                          {mergeOptions.filter(r => r !== rawRole).map(r => (
                            <option key={r} value={r}>{esc(r)}</option>
                          ))}
                        </select>
                      </td>
                      <td>
                        <span class="agent-confidence-badge">{countBadge}</span>
                      </td>
                      <td>
                        <div style={{ display: 'flex', gap: '6px' }}>
                          <button
                            type="button"
                            class="filter-btn"
                            style={{ fontSize: '10px', padding: '2px 8px' }}
                            onClick={() => void handleSave(rawRole)}
                          >
                            Save
                          </button>
                          {registered && (
                            <button
                              type="button"
                              class="filter-btn"
                              style={{ fontSize: '10px', padding: '2px 8px', color: 'var(--accent)', borderColor: 'var(--accent)' }}
                              onClick={() => void handleDelete(rawRole)}
                            >
                              Delete
                            </button>
                          )}
                        </div>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
}
