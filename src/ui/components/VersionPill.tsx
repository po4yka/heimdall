import { versionInfo, versionChecking } from '../state/store';
import { esc } from '../lib/format';

export function VersionPill() {
  const info = versionInfo.value;
  const checking = versionChecking.value;

  if (!info) return null;

  const current = `v${info.current}`;

  if (checking) {
    return <span class="version-pill version-pill--checking">[CHECKING]</span>;
  }

  if (info.update_available && info.latest && info.latest_url) {
    return (
      <a
        class="version-pill version-pill--update"
        href={info.latest_url}
        target="_blank"
        rel="noopener noreferrer"
        title={`Latest: v${esc(info.latest)} (current: ${current})`}
      >
        [v{esc(info.latest)} &rarr;]
      </a>
    );
  }

  return (
    <span class="version-pill version-pill--current" title={`Current: ${current}`}>
      [{current}]
    </span>
  );
}
