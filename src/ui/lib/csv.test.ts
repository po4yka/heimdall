import { afterEach, describe, expect, it, vi } from 'vitest';
import { csvField, csvTimestamp, downloadCSV } from './csv';

afterEach(() => {
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

describe('csv helpers', () => {
  it('escapes cells and blocks spreadsheet formula injection', () => {
    expect(csvField('plain')).toBe('plain');
    expect(csvField('=SUM(A1:A2)')).toBe("'=SUM(A1:A2)");
    expect(csvField('+cmd')).toBe("'+cmd");
    expect(csvField('hello,world')).toBe('"hello,world"');
    expect(csvField('say "hi"')).toBe('"say ""hi"""');
    expect(csvField('line\nbreak')).toBe('"line\nbreak"');
  });

  it('formats export timestamps and triggers blob downloads', () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2026-04-20T10:07:00Z'));

    const click = vi.fn();
    const anchor = { click, href: '', download: '' } as unknown as HTMLAnchorElement;
    const createObjectURL = vi.fn(() => 'blob:csv-download');
    const revokeObjectURL = vi.fn();
    const createElement = vi.fn(() => anchor);

    vi.stubGlobal('document', { createElement });
    vi.stubGlobal('URL', { createObjectURL, revokeObjectURL });

    expect(csvTimestamp()).toBe('2026-04-20_1407');

    downloadCSV('sessions', ['Project', 'Value'], [['alpha', '=1+1']]);

    expect(createElement).toHaveBeenCalledWith('a');
    expect(createObjectURL).toHaveBeenCalledTimes(1);
    expect(anchor.href).toBe('blob:csv-download');
    expect(anchor.download).toBe('sessions_2026-04-20_1407.csv');
    expect(click).toHaveBeenCalledTimes(1);

    vi.advanceTimersByTime(1_000);
    expect(revokeObjectURL).toHaveBeenCalledWith('blob:csv-download');
  });
});
