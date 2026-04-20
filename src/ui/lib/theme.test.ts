import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

vi.hoisted(() => {
  Object.defineProperty(globalThis, 'window', {
    value: { location: { search: '' }, matchMedia: () => ({ matches: false }) },
    configurable: true,
  });
});

import { themeMode } from '../state/store';
import { applyTheme, getTheme } from './theme';

beforeEach(() => {
  themeMode.value = 'dark';
});

afterEach(() => {
  vi.unstubAllGlobals();
  themeMode.value = 'dark';
});

describe('theme helpers', () => {
  it('prefers stored themes and falls back to matchMedia', () => {
    vi.stubGlobal('localStorage', {
      getItem: vi.fn(() => 'light'),
    });
    vi.stubGlobal('window', {
      matchMedia: vi.fn(() => ({ matches: true })),
    });

    expect(getTheme()).toBe('light');

    vi.stubGlobal('localStorage', {
      getItem: vi.fn(() => null),
    });
    vi.stubGlobal('window', {
      matchMedia: vi.fn(() => ({ matches: false })),
    });

    expect(getTheme()).toBe('light');
  });

  it('applies the selected theme to the DOM and shared state', () => {
    const setAttribute = vi.fn();
    const removeAttribute = vi.fn();

    vi.stubGlobal('document', {
      documentElement: {
        setAttribute,
        removeAttribute,
      },
    });

    applyTheme('light');
    expect(setAttribute).toHaveBeenCalledWith('data-theme', 'light');
    expect(themeMode.value).toBe('light');

    applyTheme('dark');
    expect(removeAttribute).toHaveBeenCalledWith('data-theme');
    expect(themeMode.value).toBe('dark');
  });
});
