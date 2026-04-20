import { afterEach, describe, expect, it, vi } from 'vitest';
import {
  $,
  anyHasCredits,
  esc,
  fmt,
  fmtCost,
  fmtCostBig,
  fmtCredits,
  fmtRelativeTime,
  fmtResetTime,
  progressColor,
  truncateMid,
} from './format';

afterEach(() => {
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

describe('format helpers', () => {
  it('reads DOM ids and formats numeric magnitudes', () => {
    const mount = { id: 'header-mount' } as HTMLElement;
    vi.stubGlobal('document', {
      getElementById: vi.fn((id: string) => (id === 'header-mount' ? mount : null)),
    });

    expect($('header-mount')).toBe(mount);
    expect(fmt(950)).toBe('950');
    expect(fmt(1_500)).toBe('1.5K');
    expect(fmt(2_500_000)).toBe('2.50M');
    expect(fmt(4_000_000_000)).toBe('4.00B');
    expect(fmtCost(1.23456)).toBe('$1.2346');
    expect(fmtCostBig(1.239)).toBe('$1.24');
  });

  it('formats reset windows and relative timestamps', () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2026-04-20T04:30:00Z'));

    expect(fmtResetTime(null)).toBe('now');
    expect(fmtResetTime(0)).toBe('now');
    expect(fmtResetTime(95)).toBe('1h 35m');
    expect(fmtResetTime(1_500)).toBe('1d 1h');

    expect(fmtRelativeTime(null)).toBe('never');
    expect(fmtRelativeTime('bad-timestamp')).toBe('bad-timestamp');
    expect(fmtRelativeTime('2026-04-20T04:30:00Z')).toBe('just now');
    expect(fmtRelativeTime('2026-04-20T04:10:00Z')).toBe('20m ago');
    expect(fmtRelativeTime('2026-04-20T01:30:00Z')).toBe('3h ago');
    expect(fmtRelativeTime('2026-04-17T04:30:00Z')).toBe('3d ago');
  });

  it('formats credits, progress colors, HTML escaping, and middle truncation safely', () => {
    expect(progressColor(95)).toBe('var(--accent)');
    expect(progressColor(75)).toBe('var(--warning)');
    expect(progressColor(40)).toBe('var(--success)');

    expect(anyHasCredits([{ credits: null }, { credits: 1.5 }])).toBe(true);
    expect(anyHasCredits([{ credits: null }, {}])).toBe(false);
    expect(fmtCredits(null)).toBe('—');
    expect(fmtCredits(2)).toBe('2.00');

    expect(esc(`<script>"hi"&'bye'</script>`)).toBe(
      '&lt;script&gt;&quot;hi&quot;&amp;&#39;bye&#39;&lt;/script&gt;'
    );
    expect(truncateMid('GitRep/bite-size-reader', 14, 6)).toBe('GitRep/…reader');
    expect(truncateMid('short', 10)).toBe('short');
  });
});
