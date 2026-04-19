export function $(id: string): HTMLElement {
  return document.getElementById(id)!;
}

export function fmt(n: number): string {
  if (n >= 1e9) return (n / 1e9).toFixed(2) + 'B';
  if (n >= 1e6) return (n / 1e6).toFixed(2) + 'M';
  if (n >= 1e3) return (n / 1e3).toFixed(1) + 'K';
  return n.toLocaleString();
}

export function fmtCost(c: number): string {
  return '$' + c.toFixed(4);
}

export function fmtCostBig(c: number): string {
  return '$' + c.toFixed(2);
}

export function fmtResetTime(minutes: number | null | undefined): string {
  if (minutes == null || minutes <= 0) return 'now';
  if (minutes >= 1440) return Math.floor(minutes / 1440) + 'd ' + Math.floor((minutes % 1440) / 60) + 'h';
  if (minutes >= 60) return Math.floor(minutes / 60) + 'h ' + (minutes % 60) + 'm';
  return minutes + 'm';
}

export function fmtRelativeTime(iso: string | null | undefined): string {
  if (!iso) return 'never';
  const ts = Date.parse(iso);
  if (Number.isNaN(ts)) return iso;
  const diffMs = Date.now() - ts;
  if (diffMs <= 0) return 'just now';
  const minutes = Math.floor(diffMs / 60_000);
  if (minutes < 1) return 'just now';
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  return `${days}d ago`;
}

export function progressColor(percent: number): string {
  if (percent >= 90) return 'var(--accent)';
  if (percent >= 70) return 'var(--warning)';
  return 'var(--success)';
}

/** Phase 12: returns true when at least one row has a non-null credits value. */
export function anyHasCredits(rows: Array<{ credits?: number | null }>): boolean {
  return rows.some(r => r.credits != null);
}

/** Phase 12: formats an Amp credits value; returns em-dash for null/undefined. */
export function fmtCredits(n: number | null | undefined): string {
  if (n == null) return '\u2014';
  return n.toFixed(2);
}

/** HTML-escape a dynamic string before inserting into an innerHTML-style
 *  string. Use for every server-supplied value that lands inside a
 *  string-concatenated HTML fragment (e.g. ApexCharts `custom` tooltip
 *  builders). Covers the five XML predefined entities. */
export function esc(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

/** Middle-ellipsis truncation for paths or identifiers that carry meaning
 *  at both ends (e.g., `GitRep/bite-size-reader` → `GitRep/…e-reader`).
 *  Unlike suffix ellipsis this preserves the owner scope plus the terminal
 *  segment, which is how most users visually match project names.
 *
 *  Iterates by Unicode code points (Array.from) so emoji / surrogate pairs
 *  are not split mid-character, and clamps `tailChars` so the result is
 *  never wider than `max`. */
export function truncateMid(s: string, max: number, tailChars: number = 8): string {
  const codepoints = Array.from(s);
  if (codepoints.length <= max) return s;
  // Clamp tailChars so we never produce a string wider than `max`: reserve
  // at least 1 char for the head and 1 for the ellipsis.
  const safeTail = Math.min(tailChars, Math.max(0, max - 2));
  const head = Math.max(0, max - safeTail - 1);
  return codepoints.slice(0, head).join('') + '\u2026' + codepoints.slice(-safeTail).join('');
}
