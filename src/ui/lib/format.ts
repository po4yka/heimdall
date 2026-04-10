export function esc(s: unknown): string {
  const d = document.createElement('div');
  d.textContent = String(s);
  return d.innerHTML;
}

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

export function progressColor(percent: number): string {
  if (percent >= 90) return 'var(--red)';
  if (percent >= 70) return 'var(--yellow)';
  return 'var(--green)';
}
