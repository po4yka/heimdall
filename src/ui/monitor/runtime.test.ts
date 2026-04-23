import { afterEach, describe, expect, it, vi } from 'vitest';

const renderSpy = vi.hoisted(() => vi.fn());
const viewSpy = vi.hoisted(() => vi.fn(() => ({ type: 'LiveMonitorView', props: {} })));

vi.mock('preact', () => ({
  render: renderSpy,
}));

vi.mock('./view', () => ({
  renderLiveMonitorView: viewSpy,
}));

function makeResponse(data: unknown, ok = true, status = 200): Response {
  return {
    ok,
    status,
    json: async () => data,
  } as Response;
}

async function flushAsyncWork(): Promise<void> {
  await Promise.resolve();
  await Promise.resolve();
}

describe('live monitor runtime', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
    vi.resetModules();
    renderSpy.mockReset();
    viewSpy.mockReset();
  });

  it('polls while visible and refetches on scan_completed', async () => {
    const listeners = new Map<string, EventListener>();
    const intervals: Array<() => void> = [];
    const eventListeners = new Map<string, EventListener>();
    const eventSource = {
      addEventListener: vi.fn((event: string, handler: EventListener) => {
        eventListeners.set(event, handler);
      }),
      close: vi.fn(),
    };
    class MockEventSource {
      addEventListener = eventSource.addEventListener;
      close = eventSource.close;
      constructor(_: string) {}
    }

    vi.stubGlobal('document', {
      hidden: false,
      title: '',
      getElementById: vi.fn((id: string) => id === 'main-content' ? { id } : { id, style: {} }),
      addEventListener: vi.fn((event: string, handler: EventListener) => {
        listeners.set(event, handler);
      }),
      removeEventListener: vi.fn(),
    });
    vi.stubGlobal('window', {
      setInterval: vi.fn((handler: () => void) => {
        intervals.push(handler);
        return intervals.length;
      }),
      clearInterval: vi.fn(),
    });
    vi.stubGlobal('EventSource', MockEventSource);
    const fetchSpy = vi.fn(async () => makeResponse({
      contract_version: 1,
      generated_at: '2026-04-22T10:00:00Z',
      default_focus: 'all',
      freshness: { stale_providers: [], has_stale_providers: false, refresh_state: 'current' },
      providers: [],
    }));
    vi.stubGlobal('fetch', fetchSpy);

    const { createLiveMonitorRuntime } = await import('./runtime');
    const runtime = createLiveMonitorRuntime();
    runtime.start();
    await flushAsyncWork();

    expect(fetchSpy).toHaveBeenCalledTimes(1);
    expect(intervals).toHaveLength(1);

    const firstInterval = intervals[0];
    expect(firstInterval).toBeDefined();
    firstInterval?.();
    await flushAsyncWork();
    expect(fetchSpy).toHaveBeenCalledTimes(2);

    eventListeners.get('scan_completed')?.(new Event('scan_completed'));
    await flushAsyncWork();
    expect(fetchSpy).toHaveBeenCalledTimes(3);

    runtime.stop();
    expect(eventSource.close).toHaveBeenCalledTimes(1);
  });

  it('skips interval refreshes while hidden and refetches on visibility restore', async () => {
    const listeners = new Map<string, EventListener>();
    const intervals: Array<() => void> = [];
    const doc = {
      hidden: true,
      title: '',
      getElementById: vi.fn((id: string) => id === 'main-content' ? { id } : { id, style: {} }),
      addEventListener: vi.fn((event: string, handler: EventListener) => {
        listeners.set(event, handler);
      }),
      removeEventListener: vi.fn(),
    };
    class MockEventSource {
      addEventListener = vi.fn();
      close = vi.fn();
      constructor(_: string) {}
    }

    vi.stubGlobal('document', doc);
    vi.stubGlobal('window', {
      setInterval: vi.fn((handler: () => void) => {
        intervals.push(handler);
        return intervals.length;
      }),
      clearInterval: vi.fn(),
    });
    vi.stubGlobal('EventSource', MockEventSource);
    const fetchSpy = vi.fn(async () => makeResponse({
      contract_version: 1,
      generated_at: '2026-04-22T10:00:00Z',
      default_focus: 'all',
      freshness: { stale_providers: [], has_stale_providers: false, refresh_state: 'current' },
      providers: [],
    }));
    vi.stubGlobal('fetch', fetchSpy);

    const { createLiveMonitorRuntime } = await import('./runtime');
    const runtime = createLiveMonitorRuntime();
    runtime.start();
    await flushAsyncWork();

    expect(fetchSpy).toHaveBeenCalledTimes(1);
    const firstInterval = intervals[0];
    expect(firstInterval).toBeDefined();
    firstInterval?.();
    await flushAsyncWork();
    expect(fetchSpy).toHaveBeenCalledTimes(1);

    doc.hidden = false;
    listeners.get('visibilitychange')?.(new Event('visibilitychange'));
    await flushAsyncWork();
    expect(fetchSpy).toHaveBeenCalledTimes(2);

    runtime.stop();
  });
});
