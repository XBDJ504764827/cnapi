import { describe, expect, it } from 'vitest';

import { useUiStore } from './ui-store';

describe('ui-store', () => {
  it('defaults to the dark theme', () => {
    expect(useUiStore.getState().theme).toBe('dark');
    expect(useUiStore.getState().sidebarCollapsed).toBe(false);
  });

  it('toggles the sidebar and switches theme', () => {
    const before = useUiStore.getState().sidebarCollapsed;
    useUiStore.getState().toggleSidebar();
    expect(useUiStore.getState().sidebarCollapsed).toBe(!before);

    useUiStore.getState().setTheme('light');
    expect(useUiStore.getState().theme).toBe('light');

    useUiStore.getState().setTheme('dark');
    useUiStore.getState().toggleSidebar();
  });
});
