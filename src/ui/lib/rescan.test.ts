import { describe, expect, it, vi } from 'vitest';

import { createTriggerRescan, type RescanButtonLike } from './rescan';

describe('createTriggerRescan', () => {
  it('shows scan counts and reloads data after a successful rescan', async () => {
    const button: RescanButtonLike = { disabled: false, textContent: 'Rescan' };
    const errors: string[] = [];
    const timers: Array<{ callback: () => void; delayMs: number }> = [];
    const loadData = vi.fn(async () => undefined);

    const triggerRescan = createTriggerRescan({
      button,
      fetchImpl: async () => ({
        ok: true,
        status: 200,
        statusText: 'OK',
        json: async () => ({ new: 4, updated: 9 }),
      }),
      loadData,
      showError: (message) => errors.push(message),
      setTimer: (callback, delayMs) => {
        timers.push({ callback, delayMs });
        return timers.length;
      },
    });

    await triggerRescan();

    expect(loadData).toHaveBeenCalledWith(true);
    expect(errors).toEqual([]);
    expect(button.disabled).toBe(true);
    expect(button.textContent).toBe('\u21bb Rescan (4 new, 9 updated)');
    expect(timers).toHaveLength(1);
    expect(timers[0]!.delayMs).toBe(3000);

    timers[0]!.callback();
    expect(button.disabled).toBe(false);
    expect(button.textContent).toBe('\u21bb Rescan');
  });

  it('re-enables the button after an HTTP failure', async () => {
    const button: RescanButtonLike = { disabled: false, textContent: 'Rescan' };
    const errors: string[] = [];
    const timers: Array<{ callback: () => void; delayMs: number }> = [];

    const triggerRescan = createTriggerRescan({
      button,
      fetchImpl: async () => ({
        ok: false,
        status: 500,
        statusText: 'Internal Server Error',
        json: async () => ({ new: 0, updated: 0 }),
      }),
      loadData: vi.fn(async () => undefined),
      showError: (message) => errors.push(message),
      setTimer: (callback, delayMs) => {
        timers.push({ callback, delayMs });
        return timers.length;
      },
    });

    await triggerRescan();

    expect(button.disabled).toBe(true);
    expect(button.textContent).toBe('\u21bb Rescan (failed)');
    expect(errors).toEqual(['Rescan failed: HTTP 500 Internal Server Error']);
    expect(timers).toHaveLength(1);
    expect(timers[0]!.delayMs).toBe(3000);

    timers[0]!.callback();
    expect(button.disabled).toBe(false);
    expect(button.textContent).toBe('\u21bb Rescan');
  });

  it('surfaces thrown fetch errors and logs the original error', async () => {
    const button: RescanButtonLike = { disabled: false, textContent: 'Rescan' };
    const errors: string[] = [];
    const timers: Array<() => void> = [];
    const logged: unknown[] = [];

    const triggerRescan = createTriggerRescan({
      button,
      fetchImpl: async () => {
        throw new Error('socket hung up');
      },
      loadData: vi.fn(async () => undefined),
      showError: (message) => errors.push(message),
      setTimer: (callback) => {
        timers.push(callback);
        return timers.length;
      },
      logError: (error) => logged.push(error),
    });

    await triggerRescan();

    expect(errors).toEqual(['Rescan failed: socket hung up']);
    expect(button.disabled).toBe(true);
    expect(button.textContent).toBe('\u21bb Rescan (error)');
    expect(logged).toHaveLength(1);
    expect(logged[0]).toBeInstanceOf(Error);
    expect((logged[0] as Error).message).toBe('socket hung up');

    timers[0]!();
    expect(button.disabled).toBe(false);
    expect(button.textContent).toBe('\u21bb Rescan');
  });
});
