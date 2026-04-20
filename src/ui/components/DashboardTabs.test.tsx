import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

vi.hoisted(() => {
  Object.defineProperty(globalThis, 'window', {
    value: { location: { search: '' } },
    configurable: true,
  });
});

import { activeDashboardTab } from '../state/store';
import { DashboardTabs } from './DashboardTabs';

beforeEach(() => {
  activeDashboardTab.value = 'overview';
});

afterEach(() => {
  activeDashboardTab.value = 'overview';
});

describe('DashboardTabs', () => {
  it('marks the active tab and delegates clicks to the controller', () => {
    activeDashboardTab.value = 'tables';
    const clicked: string[] = [];

    const vnode = DashboardTabs({ onTabChange: tab => clicked.push(tab) }) as {
      props: { children: Array<{ props: Record<string, unknown> }> };
    };
    const buttons = vnode.props.children;
    const tablesButton = buttons[3]!;
    const overviewButton = buttons[0]!;

    expect(tablesButton.props['class']).toContain('active');
    expect(tablesButton.props['aria-pressed']).toBe(true);
    expect(overviewButton.props['aria-pressed']).toBe(false);

    (overviewButton.props['onClick'] as () => void)();
    expect(clicked).toEqual(['overview']);
  });
});
