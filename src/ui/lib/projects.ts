import type {
  ProjectPatchBody,
  ProjectRegistryRow,
  ProjectSettingsRow,
  ProjectsListResponse,
} from '../state/dashboard-types';

/** GET /api/projects — full project registry. */
export async function fetchProjectsRegistry(): Promise<ProjectRegistryRow[]> {
  const r = await fetch('/api/projects');
  if (!r.ok) throw new Error(`HTTP ${r.status}`);
  const body = (await r.json()) as ProjectsListResponse;
  return body.projects;
}

/** PATCH /api/projects/{uuid} — update label and/or pinned. Returns the updated row. */
export async function patchProject(
  uuid: string,
  body: ProjectPatchBody,
): Promise<ProjectSettingsRow> {
  const r = await fetch(`/api/projects/${encodeURIComponent(uuid)}`, {
    method: 'PATCH',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(body),
  });
  if (!r.ok) throw new Error(`HTTP ${r.status}`);
  return (await r.json()) as ProjectSettingsRow;
}
