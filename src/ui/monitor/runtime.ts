import { render } from 'preact';
import { $ } from '../lib/format';
import type { LiveMonitorResponse } from '../state/types';
import { liveMonitorError, liveMonitorRefreshing, setLiveMonitorData } from './store';
import { renderLiveMonitorView } from './view';

export interface LiveMonitorRuntime {
  loadData: () => Promise<void>;
  start: () => void;
  stop: () => void;
}

export function createLiveMonitorRuntime(): LiveMonitorRuntime {
  let intervalId: number | null = null;
  let eventSource: EventSource | null = null;
  let visibilityHandler: (() => void) | null = null;
  const mount = $('main-content');

  function renderView(): void {
    render(renderLiveMonitorView(), mount);
  }

  async function loadData(): Promise<void> {
    liveMonitorRefreshing.value = true;
    try {
      const tzOffset = new Date().getTimezoneOffset() * -1;
      const response = await fetch(`/api/live-monitor?tz_offset_min=${tzOffset}`);
      if (!response.ok) {
        throw new Error(`Monitor request failed (${response.status})`);
      }
      setLiveMonitorData(await response.json() as LiveMonitorResponse);
    } catch (error) {
      liveMonitorError.value = error instanceof Error ? error.message : 'Live monitor refresh failed';
    } finally {
      liveMonitorRefreshing.value = false;
      renderView();
    }
  }

  function toggleVisibility(hidden: boolean): void {
    const filterMount = document.getElementById('filter-bar-mount');
    const tabsMount = document.getElementById('dashboard-tabs-mount');
    if (filterMount) filterMount.style.display = hidden ? 'none' : '';
    if (tabsMount) tabsMount.style.display = hidden ? 'none' : '';
  }

  function subscribeToStream(): void {
    if (typeof EventSource === 'undefined') return;
    eventSource = new EventSource('/api/stream');
    eventSource.addEventListener('scan_completed', () => {
      void loadData();
    });
  }

  function start(): void {
    document.title = 'Live Monitor';
    toggleVisibility(true);
    renderView();
    void loadData();
    visibilityHandler = () => {
      if (!document.hidden) {
        void loadData();
      }
    };
    document.addEventListener('visibilitychange', visibilityHandler);
    intervalId = window.setInterval(() => {
      if (!document.hidden) {
        void loadData();
      }
    }, 10_000);
    subscribeToStream();
  }

  function stop(): void {
    if (intervalId != null) {
      window.clearInterval(intervalId);
    }
    intervalId = null;
    eventSource?.close();
    eventSource = null;
    if (visibilityHandler) {
      document.removeEventListener('visibilitychange', visibilityHandler);
    }
    visibilityHandler = null;
    toggleVisibility(false);
  }

  return { loadData, start, stop };
}
