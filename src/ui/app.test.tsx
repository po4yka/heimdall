import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const renderSpy = vi.hoisted(() => vi.fn());
const runtimeSpies = vi.hoisted(() => ({
  loadData: vi.fn(),
  handleDashboardTabChange: vi.fn(),
  applyFilter: vi.fn(),
  start: vi.fn(),
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
vi.mock('./lib/theme', () => themeSpies);
vi.mock('./state/store', () => storeMock);

describe('app entrypoint', () => {
  beforeEach(() => {
    renderSpy.mockReset();
    Object.values(runtimeSpies).forEach(spy => spy.mockReset());
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
      location: { search: '' },
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
});
