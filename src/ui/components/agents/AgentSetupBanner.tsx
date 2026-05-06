import { useEffect } from 'preact/hooks';
import { signal } from '@preact/signals';
import { registryModalOpen, setupBannerDismissed } from '../../state/store';
import { unclassifiedDetectedRolesGlobal } from '../../lib/agents';
import type { AgentTelemetry } from '../../state/types';

interface AgentSetupBannerProps {
  telemetry: AgentTelemetry;
}

interface UnclassifiedGlobalResponse {
  count: number;
  any_configured: boolean;
}

const unclassifiedGlobal = signal<UnclassifiedGlobalResponse | null>(null);
let inFlight = false;

async function fetchUnclassifiedGlobal(): Promise<void> {
  if (inFlight || unclassifiedGlobal.value !== null) return;
  inFlight = true;
  try {
    const res = await fetch('/api/agents/unclassified-global');
    if (!res.ok) return;
    unclassifiedGlobal.value = (await res.json()) as UnclassifiedGlobalResponse;
  } finally {
    inFlight = false;
  }
}

export function AgentSetupBanner({ telemetry }: AgentSetupBannerProps) {
  useEffect(() => {
    void fetchUnclassifiedGlobal();
  }, []);

  if (setupBannerDismissed.value) return null;

  const server = unclassifiedGlobal.value;
  if (!server || server.count <= 0) return null;
  const roleCount = server.count;

  // Project list is derived client-side from the embedded telemetry to
  // pick which project the [Open registry] button should target.
  const projects = [
    ...new Set(unclassifiedDetectedRolesGlobal(telemetry).map(d => d.project)),
  ];
  const projectCount = projects.length || 1;

  function openRegistry() {
    const firstProject = projects[0];
    if (firstProject) {
      registryModalOpen.value = { project: firstProject };
    }
  }

  function dismiss() {
    setupBannerDismissed.value = true;
  }

  return (
    <div class="agent-setup-banner">
      <div class="agent-setup-banner-body">
        <div class="agent-setup-banner-line1">
          {roleCount} unclassified agent role{roleCount !== 1 ? 's' : ''} detected
          {' '}across {projectCount} project{projectCount !== 1 ? 's' : ''}
        </div>
        <div class="agent-setup-banner-line2">
          Classify them in the registry to see them in distribution and timeline.{' '}
          <button
            type="button"
            class="agent-setup-banner-action"
            onClick={openRegistry}
          >
            [Open registry]
          </button>
        </div>
      </div>
      <button
        type="button"
        class="agent-setup-banner-dismiss"
        aria-label="Dismiss"
        onClick={dismiss}
      >
        [X]
      </button>
    </div>
  );
}
