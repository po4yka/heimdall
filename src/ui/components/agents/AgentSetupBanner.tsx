import { registryModalOpen, setupBannerDismissed } from '../../state/store';
import { unclassifiedDetectedRolesGlobal } from '../../lib/agents';
import type { AgentTelemetry } from '../../state/types';

interface AgentSetupBannerProps {
  telemetry: AgentTelemetry;
}

export function AgentSetupBanner({ telemetry }: AgentSetupBannerProps) {
  if (setupBannerDismissed.value) return null;

  const unclassified = unclassifiedDetectedRolesGlobal(telemetry);
  if (!unclassified.length) return null;

  const projects = [...new Set(unclassified.map(d => d.project))];
  const roleCount = unclassified.length;
  const projectCount = projects.length;

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
