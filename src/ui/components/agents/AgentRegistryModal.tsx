import { useState, useEffect, useCallback, useMemo } from 'preact/hooks';
import { registryModalOpen } from '../../state/store';
import { InlineStatus } from '../InlineStatus';
import { TableSkeleton } from '../_primitives/Skeleton';
import { setStatus, clearStatus } from '../../lib/status';
import {
  fetchRegistry,
  upsertRole,
  deleteRole,
  type RegistryUpsertBody,
} from '../../lib/agents';
import type { AgentRegistryRow, AgentTelemetry, RoleConfidenceLevel } from '../../state/types';
import { esc, fmtLabel } from '../../lib/format';

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

function collectDisplayRoles(
  detectedRows: Array<{ raw_role: string }>,
  registryRows: Array<{ raw_role: string }>,
): string[] {
  const seen = new Set<string>();
  const roles: string[] = [];
  const add = (rawRole: string) => {
    if (rawRole === 'unknown' || seen.has(rawRole)) return;
    seen.add(rawRole);
    roles.push(rawRole);
  };
  for (const row of detectedRows) add(row.raw_role);
  for (const row of registryRows) add(row.raw_role);
  return roles;
}

function firstByRawRole<T extends { raw_role: string }>(rows: T[]): Map<string, T> {
  const byRole = new Map<string, T>();
  for (const row of rows) {
    if (!byRole.has(row.raw_role)) byRole.set(row.raw_role, row);
  }
  return byRole;
}

export function AgentRegistryModal({ project, telemetry, onReload }: AgentRegistryModalProps) {
  const [registryRows, setRegistryRows] = useState<AgentRegistryRow[]>([]);
  const [loading, setLoading] = useState(true);
  const [rowStates, setRowStates] = useState<Record<string, RowState>>({});

  // All detected roles for this project (classified or not)
  const detectedForProject = useMemo(
    () => telemetry.detected.filter(d => d.project === project),
    [project, telemetry.detected],
  );

  // All raw roles to display: union of detected + registry rows
  const allRoles = useMemo(
    () => collectDisplayRoles(detectedForProject, registryRows),
    [detectedForProject, registryRows],
  );

  const registryByRole = useMemo(() => firstByRawRole(registryRows), [registryRows]);
  const detectedByRole = useMemo(() => firstByRawRole(detectedForProject), [detectedForProject]);
  const mergeOptionsByRole = useMemo(() => {
    const optionsByRole = new Map<string, string[]>();
    for (const rawRole of allRoles) {
      const options: string[] = [];
      for (const option of allRoles) {
        if (option !== rawRole) options.push(option);
      }
      optionsByRole.set(rawRole, options);
    }
    return optionsByRole;
  }, [allRoles]);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const resp = await fetchRegistry(project);
      setRegistryRows(resp.registry);
      const fetchedRegistryByRole = firstByRawRole(resp.registry);
      const states: Record<string, RowState> = {};
      for (const rawRole of collectDisplayRoles(detectedForProject, resp.registry)) {
        states[rawRole] = initialRowState(fetchedRegistryByRole.get(rawRole));
      }
      setRowStates(states);
    } catch {
      // keep empty
    } finally {
      setLoading(false);
    }
  }, [project, detectedForProject]);

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
          <div style={{ padding: 'var(--space-4)' }}>
            <TableSkeleton rows={4} columns={3} />
          </div>
        ) : allRoles.length === 0 ? (
          <div class="empty-state" style={{ margin: '20px' }}>No agent roles detected for this project</div>
        ) : (
          <div class="agent-registry-table-wrap">
            <table class="agent-registry-table">
              <thead>
                <tr>
                  <th>Role</th>
                  <th>Display name</th>
                  <th>Description</th>
                  <th>Enabled</th>
                  <th>Merged into</th>
                  <th>Confidence</th>
                  <th>Actions</th>
                </tr>
              </thead>
              <tbody>
                {allRoles.map(rawRole => {
                  const state = rowStates[rawRole] ?? initialRowState(undefined);
                  const registered = registryByRole.get(rawRole);
                  const detected = detectedByRole.get(rawRole);
                  const confidence: RoleConfidenceLevel = detected?.confidence ?? 'unknown';
                  const sessionCount = detected?.count ?? 0;
                  const mergeOptions = mergeOptionsByRole.get(rawRole) ?? [];

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
                          {mergeOptions.map(r => (
                            <option key={r} value={r}>{esc(r)}</option>
                          ))}
                        </select>
                      </td>
                      <td>
                        {/* Confidence badge: highest tier observed across this role's sessions.
                            Session count shown dimmed alongside for reference. */}
                        <span class={`confidence-badge ${confidence}`}>[{fmtLabel(confidence)}]</span>
                        {sessionCount > 0 && (
                          <span style={{ color: 'var(--text-secondary)', fontFamily: 'var(--font-mono)', fontSize: '10px', marginLeft: '6px' }}>
                            ({sessionCount} {sessionCount === 1 ? 'session' : 'sessions'})
                          </span>
                        )}
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
