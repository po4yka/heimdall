import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const renderSpy = vi.hoisted(() => vi.fn());
const runtimeSpies = vi.hoisted(() => ({
  loadData: vi.fn(),
  handleDashboardTabChange: vi.fn(),
  applyFilter: vi.fn(),
  start: vi.fn(),
}));
const liveMonitorRuntimeSpies = vi.hoisted(() => ({
  loadData: vi.fn(),
  start: vi.fn(),
  stop: vi.fn(),
}));
const themeSpies = vi.hoisted(() => ({
  getTheme: vi.fn(() => 'dark' as const),
  applyTheme: vi.fn(),
}));
const storeMock = vi.hoisted(() => ({
  rawData: { value: null as unknown },
  syncDashboardUrl: vi.fn(),
}));

vi.mock('preact', () => ({
  render: renderSpy,
}));

vi.mock('./components/DashboardTabs', () => ({
  DashboardTabs: (props: unknown) => ({ type: 'DashboardTabs', props }),
}));
vi.mock('./components/FilterBar', () => ({
  FilterBar: (props: unknown) => ({ type: 'FilterBar', props }),
}));
vi.mock('./components/Footer', () => ({
  Footer: () => ({ type: 'Footer', props: {} }),
}));
vi.mock('./components/Header', () => ({
  Header: (props: unknown) => ({ type: 'Header', props }),
}));
vi.mock('./components/InlineStatus', () => ({
  InlineStatus: (props: unknown) => ({ type: 'InlineStatus', props }),
}));
vi.mock('./dashboard/runtime', () => ({
  createDashboardRuntime: () => runtimeSpies,
}));
vi.mock('./monitor/runtime', () => ({
  createLiveMonitorRuntime: () => liveMonitorRuntimeSpies,
}));
vi.mock('./monitor/MonitorHeader', () => ({
  MonitorHeader: (props: unknown) => ({ type: 'MonitorHeader', props }),
}));
vi.mock('./lib/theme', () => themeSpies);
// Partial-mock so the test only overrides the two signals it asserts on
// (rawData + syncDashboardUrl) while every other signal app.tsx subscribes
// to (backupModalOpen, settingsModalOpen, commandPaletteOpen, projectsRegistry,
// archiveImports, activeDashboardTab, …) keeps its real exported shape.
vi.mock('./state/store', async (importOriginal) => {
  const actual = await importOriginal<typeof import('./state/store')>();
  return {
    ...actual,
    rawData: storeMock.rawData,
    syncDashboardUrl: storeMock.syncDashboardUrl,
  };
});

describe('app entrypoint', () => {
  beforeEach(() => {
    renderSpy.mockReset();
    Object.values(runtimeSpies).forEach(spy => spy.mockReset());
    Object.values(liveMonitorRuntimeSpies).forEach(spy => spy.mockReset());
    Object.values(themeSpies).forEach(spy => spy.mockReset());
    themeSpies.getTheme.mockReturnValue('dark');
    storeMock.rawData.value = null;
    storeMock.syncDashboardUrl.mockReset();
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    vi.resetModules();
  });

  it('applies the theme, mounts dashboard shells, and starts the runtime', async () => {
    const headerMount = { id: 'header-mount' };
    const filterBarMount = { id: 'filter-bar-mount' };
    const tabsMount = { id: 'dashboard-tabs-mount' };
    const statusMount = { id: 'inline-status-global' };
    const footerParent = { id: 'footer-parent' };
    const footerNode = { parentElement: footerParent };

    const getAttribute = vi.fn((_: string) => null as string | null);
    vi.stubGlobal('document', {
      documentElement: {
        getAttribute,
      },
      getElementById: vi.fn((id: string) => {
        if (id === 'header-mount') return headerMount;
        if (id === 'filter-bar-mount') return filterBarMount;
        if (id === 'dashboard-tabs-mount') return tabsMount;
        if (id === 'inline-status-global') return statusMount;
        return null;
      }),
      querySelector: vi.fn((selector: string) => (selector === 'footer' ? footerNode : null)),
    });
    vi.stubGlobal('localStorage', {
      setItem: vi.fn(),
      getItem: vi.fn(() => null),
    });
    vi.stubGlobal('window', {
      location: { pathname: '/', search: '' },
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      setTimeout: globalThis.setTimeout.bind(globalThis),
      clearTimeout: globalThis.clearTimeout.bind(globalThis),
    });

    // @ts-expect-error intentional source import so the test exercises app.tsx, not the bundled app.js artifact
    await import('./app.tsx');

    expect(themeSpies.getTheme).toHaveBeenCalledTimes(1);
    expect(themeSpies.applyTheme).toHaveBeenCalledWith('dark');
    expect(runtimeSpies.start).toHaveBeenCalledTimes(1);
    expect(renderSpy).toHaveBeenCalledTimes(5);
    const footerRender = renderSpy.mock.calls.find(([, mount]) => mount === footerParent);
    expect(footerRender).toBeDefined();
    expect(footerRender?.[2]).toBe(footerNode);

    const headerRender = renderSpy.mock.calls.find(([, mount]) => mount === headerMount);
    const headerVNode = headerRender?.[0] as { props: { onThemeToggle: () => void } };

    storeMock.rawData.value = { all_models: ['sonnet'] };
    getAttribute.mockReturnValue('light');
    headerVNode.props.onThemeToggle();

    expect(localStorage.setItem).toHaveBeenCalledWith('theme', 'dark');
    expect(themeSpies.applyTheme).toHaveBeenLastCalledWith('dark');
    expect(runtimeSpies.applyFilter).toHaveBeenCalledTimes(1);
  });

  it('boots the live monitor route without mounting dashboard shells', async () => {
    const headerMount = { id: 'header-mount' };
    const footerParent = { id: 'footer-parent' };
    const footerNode = { parentElement: footerParent };

    vi.stubGlobal('document', {
      documentElement: {
        getAttribute: vi.fn(() => null),
      },
      getElementById: vi.fn((id: string) => {
        if (id === 'header-mount') return headerMount;
        return null;
      }),
      querySelector: vi.fn((selector: string) => (selector === 'footer' ? footerNode : null)),
    });
    vi.stubGlobal('localStorage', {
      setItem: vi.fn(),
      getItem: vi.fn(() => null),
    });
    vi.stubGlobal('window', {
      location: { pathname: '/monitor', search: '' },
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      setTimeout: globalThis.setTimeout.bind(globalThis),
      clearTimeout: globalThis.clearTimeout.bind(globalThis),
    });

    // @ts-expect-error intentional source import so the test exercises app.tsx, not the bundled app.js artifact
    await import('./app.tsx');

    expect(runtimeSpies.start).not.toHaveBeenCalled();
    expect(liveMonitorRuntimeSpies.start).toHaveBeenCalledTimes(1);
    const headerRender = renderSpy.mock.calls.find(([, mount]) => mount === headerMount);
    expect((headerRender?.[0] as { type: { name?: string } }).type.name).toBe('MonitorHeader');
  });
});
