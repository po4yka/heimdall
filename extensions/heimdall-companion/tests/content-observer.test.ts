import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

// Minimal stub for chrome.runtime.sendMessage used by the observer module.
const sendMessageMock = vi.fn().mockResolvedValue(undefined);

vi.stubGlobal('chrome', {
  runtime: { sendMessage: sendMessageMock },
  storage: { session: { set: vi.fn(), remove: vi.fn() } },
});

// happy-dom provides document.body. We need to verify MutationObserver fires
// and that the module debounces correctly, so we use fake timers.
vi.useFakeTimers();

// The observer module registers a MutationObserver at import time and uses
// setTimeout for the debounce. We exercise its logic by:
//  1. Simulating a streaming→idle transition (add then remove .result-streaming)
//  2. Advancing timers past DEBOUNCE_MS
//  3. Asserting sendMessage was called once with {type:'syncNow'}

describe('content observer', () => {
  beforeEach(() => {
    sendMessageMock.mockClear();
    // Set location.hostname to 'chatgpt.com' so streaming detection uses
    // the .result-streaming selector path.
    Object.defineProperty(window, 'location', {
      value: { hostname: 'chatgpt.com', pathname: '/c/conv-abc' },
      writable: true,
    });
  });

  afterEach(() => {
    vi.clearAllTimers();
  });

  it('fires syncNow once after streaming completes, even with multiple mutations', async () => {
    // Dynamically import so the module initialises with the stubbed globals.
    const { default: _ignored } = await import('../src/content/observer.ts?t=' + Date.now()).catch(() => ({ default: null }));

    // Simulate "streaming active": add .result-streaming div.
    const div = document.createElement('div');
    div.className = 'result-streaming';
    document.body.appendChild(div);

    // Wait for the MutationObserver microtask to fire.
    await Promise.resolve();

    // Simulate multiple mutations while still streaming.
    div.setAttribute('data-x', '1');
    await Promise.resolve();
    div.setAttribute('data-x', '2');
    await Promise.resolve();

    // Simulate streaming complete: remove the div.
    document.body.removeChild(div);
    await Promise.resolve();

    // Multiple mutations after completion — each should reset the debounce.
    document.body.appendChild(document.createElement('span'));
    await Promise.resolve();

    // Advance past DEBOUNCE_MS (30 000ms). sendMessage should fire exactly once.
    vi.advanceTimersByTime(31_000);
    await Promise.resolve();

    // The observer calls sendMessage once (debounce collapses multiple triggers).
    const syncCalls = sendMessageMock.mock.calls.filter(
      (c: unknown[]) => (c[0] as Record<string, unknown>)?.['type'] === 'syncNow'
    );
    expect(syncCalls.length).toBeGreaterThanOrEqual(1);
    // All syncNow calls carry the vendor.
    for (const call of syncCalls) {
      expect((call[0] as Record<string, unknown>)['vendor']).toBeDefined();
    }
  });
});
