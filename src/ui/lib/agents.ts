import type {
  AgentRegistryRow,
  AgentTelemetry,
  DetectedRole,
} from '../state/dashboard-types';

export interface RegistryListResponse {
  project: string;
  registry: AgentRegistryRow[];
}

export interface RegistryUpsertBody {
  // omit field => unchanged; include field with value => set; include with null => clear
  display_name?: string | null;
  description?: string | null;
  enabled?: boolean;
  merged_into?: string | null;
}

export interface DeleteResponse {
  deleted: boolean;
}

export interface AcknowledgeResponse {
  project: string;
  acknowledged: number;
  already_existed: number;
}

const ENC = encodeURIComponent;

async function jsonOrThrow<T>(res: Response): Promise<T> {
  if (!res.ok) {
    const text = await res.text().catch(() => '');
    throw new Error(`${res.status} ${res.statusText}${text ? ': ' + text : ''}`);
  }
  return res.json() as Promise<T>;
}

export async function fetchRegistry(projectId: string): Promise<RegistryListResponse> {
  const res = await fetch(`/api/agents/${ENC(projectId)}/registry`);
  return jsonOrThrow<RegistryListResponse>(res);
}

export async function upsertRole(
  projectId: string,
  rawRole: string,
  body: RegistryUpsertBody,
): Promise<AgentRegistryRow> {
  const res = await fetch(`/api/agents/${ENC(projectId)}/registry/${ENC(rawRole)}`, {
    method: 'PUT',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(body),
  });
  return jsonOrThrow<AgentRegistryRow>(res);
}

export async function deleteRole(projectId: string, rawRole: string): Promise<DeleteResponse> {
  const res = await fetch(`/api/agents/${ENC(projectId)}/registry/${ENC(rawRole)}`, {
    method: 'DELETE',
  });
  return jsonOrThrow<DeleteResponse>(res);
}

export async function acknowledgeAll(projectId: string): Promise<AcknowledgeResponse> {
  const res = await fetch(`/api/agents/${ENC(projectId)}/registry/acknowledge-all`, {
    method: 'POST',
  });
  return jsonOrThrow<AcknowledgeResponse>(res);
}

/**
 * Filter detected roles for a given project to those that need user attention:
 * have at least one observed session, are not the synthetic "unknown" bucket,
 * and are not yet present in the registry.
 *
 * Used by the setup banner to decide whether to render itself.
 */
export function unclassifiedDetectedRoles(
  telemetry: AgentTelemetry,
  projectId: string,
): DetectedRole[] {
  return telemetry.detected.filter(
    d => d.project === projectId && d.raw_role !== 'unknown' && !d.registered,
  );
}

/**
 * Same as above but across all projects (used by global banner).
 */
export function unclassifiedDetectedRolesGlobal(telemetry: AgentTelemetry): DetectedRole[] {
  return telemetry.detected.filter(
    d => d.raw_role !== 'unknown' && !d.registered,
  );
}
