import type { VersionInfo } from '../state/dashboard-types';
import { versionInfo, versionChecking } from '../state/store';

const LOCK_NAME = 'heimdall-version-poll';
const CHANNEL_NAME = 'heimdall-version-poll';
const MIN_SPINNER_MS = 1200;
const MIN_DELAY_MS = 5_000;
const FALLBACK_DELAY_MS = 5 * 60 * 1000; // 5min if server doesn't tell us nextCheckAt
const POST_POLL_BUFFER_MS = 500;

interface PollState {
  started: boolean;
  isLeader: boolean;
  timer: ReturnType<typeof setTimeout> | null;
  bc: BroadcastChannel | null;
}

declare global {
  interface Window {
    __heimdallVersionPoll?: PollState;
  }
}

function ensureState(): PollState {
  if (!window.__heimdallVersionPoll) {
    window.__heimdallVersionPoll = { started: false, isLeader: false, timer: null, bc: null };
  }
  return window.__heimdallVersionPoll;
}

async function fetchOnce(): Promise<VersionInfo | null> {
  const t0 = Date.now();
  versionChecking.value = true;
  try {
    const res = await fetch('/api/version');
    if (!res.ok) return null;
    const info = (await res.json()) as VersionInfo;
    versionInfo.value = info;
    return info;
  } catch (_) {
    return null;
  } finally {
    const elapsed = Date.now() - t0;
    if (elapsed < MIN_SPINNER_MS) {
      await new Promise<void>(r => setTimeout(r, MIN_SPINNER_MS - elapsed));
    }
    versionChecking.value = false;
  }
}

function broadcastInfo(info: VersionInfo): void {
  const st = ensureState();
  st.bc?.postMessage({ type: 'version-data', payload: info });
}

function scheduleNext(info: VersionInfo | null): void {
  const st = ensureState();
  if (st.timer) clearTimeout(st.timer);
  let delay = FALLBACK_DELAY_MS;
  if (info?.next_check_at) {
    const t = Date.parse(info.next_check_at);
    if (!Number.isNaN(t)) {
      delay = Math.max(MIN_DELAY_MS, t + POST_POLL_BUFFER_MS - Date.now());
    }
  }
  st.timer = setTimeout(() => { void tick(); }, delay);
}

async function tick(): Promise<void> {
  const info = await fetchOnce();
  if (info) broadcastInfo(info);
  scheduleNext(info);
}

export function startVersionPoll(): void {
  const st = ensureState();
  if (st.started) return;
  st.started = true;
  st.bc = new BroadcastChannel(CHANNEL_NAME);
  st.bc.onmessage = (ev: MessageEvent) => {
    if (ev.data?.type === 'version-data') {
      versionInfo.value = ev.data.payload as VersionInfo;
    } else if (ev.data?.type === 'poke' && st.isLeader) {
      void tick();
    } else if (ev.data?.type === 'hello' && st.isLeader && versionInfo.value) {
      broadcastInfo(versionInfo.value);
    }
  };
  st.bc.postMessage({ type: 'hello' });

  // Web Locks: only one tab holds the exclusive lock; all others wait.
  // Optional chaining: `navigator.locks` is undefined in Node test envs and
  // older browsers; treat its absence as "this tab leads, no coordination."
  const locks = (globalThis as { navigator?: { locks?: LockManager } }).navigator?.locks;
  if (locks) {
    locks
      .request(LOCK_NAME, { mode: 'exclusive' }, async () => {
        st.isLeader = true;
        await tick();
        // Hold the lock for the lifetime of this tab.
        await new Promise<void>(() => {});
      })
      .catch(() => { /* ignore — browser may not support locks */ });
  } else {
    st.isLeader = true;
    void tick();
  }
}

export function pokeVersionPoll(): void {
  const st = ensureState();
  if (st.isLeader) {
    void tick();
  } else {
    st.bc?.postMessage({ type: 'poke' });
  }
}
