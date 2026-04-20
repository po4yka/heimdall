import { afterEach, describe, expect, it, vi } from 'vitest';
import {
  RANGE_LABELS,
  RANGE_TICKS,
  apexThemeMode,
  cssVar,
  industrialChartOptions,
  modelSeriesColors,
  tokenSeriesColors,
  withAlpha,
} from './charts';

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('chart helpers', () => {
  it('exposes stable range labels and CSS-driven theme helpers', () => {
    vi.stubGlobal('document', {
      documentElement: {
        getAttribute: () => 'light',
      },
    });
    vi.stubGlobal('getComputedStyle', () => ({
      getPropertyValue: (name: string) =>
        ({
          '--text-display': '#eeeeee',
          '--success': '#00aa00',
          '--warning': '#ffaa00',
          '--interactive': '#222222',
          '--text-secondary': '#999999',
          '--border': '#333333',
          '--border-visible': '#777777',
        })[name] ?? '',
    }));

    expect(RANGE_LABELS['30d']).toBe('Last 30 Days');
    expect(RANGE_TICKS['7d']).toBe(7);
    expect(apexThemeMode()).toBe('light');
    expect(cssVar('--text-display')).toBe('#eeeeee');
    expect(withAlpha('--text-display', 0.5)).toBe('rgba(238, 238, 238, 0.5)');
  });

  it('builds monochrome token palettes and industrial chart defaults', () => {
    vi.stubGlobal('document', {
      documentElement: {
        getAttribute: () => null,
      },
    });
    vi.stubGlobal('getComputedStyle', () => ({
      getPropertyValue: (name: string) =>
        ({
          '--text-display': '#eeeeee',
          '--success': '#00aa00',
          '--warning': '#ffaa00',
          '--interactive': '#222222',
          '--text-secondary': '#999999',
          '--border': '#333333',
          '--border-visible': '#777777',
        })[name] ?? '',
    }));

    expect(tokenSeriesColors()).toEqual([
      'rgba(238, 238, 238, 1)',
      'rgba(238, 238, 238, 0.6)',
      'rgba(238, 238, 238, 0.3)',
      'rgba(238, 238, 238, 0.15)',
    ]);
    expect(modelSeriesColors(6)).toEqual([
      '#eeeeee',
      '#00aa00',
      '#ffaa00',
      '#222222',
      'rgba(238, 238, 238, 0.75)',
      'rgba(0, 170, 0, 0.75)',
    ]);

    const donut = industrialChartOptions('donut');
    const line = industrialChartOptions('line');

    expect(donut.legend?.position).toBe('bottom');
    expect(donut.theme?.mode).toBe('dark');
    expect(line.legend?.show).toBe(false);
    expect(line.fill).toEqual({ type: 'solid', opacity: 0 });
    expect(line.stroke).toEqual({ width: 1.5, curve: 'straight' });
  });
});
