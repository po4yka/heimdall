import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

vi.hoisted(() => {
  Object.defineProperty(globalThis, 'window', {
    value: { location: { search: '' }, setTimeout },
    configurable: true,
  });
});

import { statusByPlacement } from '../state/store';
import { clearStatus, setStatus } from './status';

const EMPTY_STATUS = {
  global: null,
  'rate-windows': null,
  rescan: null,
  'header-refresh': null,
  'agent-status': null,
  'community-signal': null,
};

beforeEach(() => {
  vi.useFakeTimers();
  vi.stubGlobal('window', {
    setTimeout,
  });
  statusByPlacement.value = { ...EMPTY_STATUS };
});

afterEach(() => {
  vi.useRealTimers();
  vi.unstubAllGlobals();
  statusByPlacement.value = { ...EMPTY_STATUS };
});

describe('status helpers', () => {
  it('stores inline statuses by placement and clears them explicitly', () => {
    setStatus('global', 'success', 'Saved');
    expect(statusByPlacement.value.global).toEqual({ kind: 'success', message: 'Saved' });

    clearStatus('global');
    expect(statusByPlacement.value.global).toBeNull();
  });

  it('auto-dismisses statuses and cancels replaced timers', () => {
    setStatus('rescan', 'loading', 'Refreshing', 5_000);
    setStatus('rescan', 'success', 'Done', 1_000);

    vi.advanceTimersByTime(999);
    expect(statusByPlacement.value.rescan).toEqual({ kind: 'success', message: 'Done' });

    vi.advanceTimersByTime(1);
    expect(statusByPlacement.value.rescan).toBeNull();
  });
});
