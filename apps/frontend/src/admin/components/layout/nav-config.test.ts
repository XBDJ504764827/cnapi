import { describe, expect, it } from 'vitest';

import { findNavItem, findNavSection, navItems, searchNavItems } from './nav-config';

describe('nav-config', () => {
  it('resolves nested paths to the parent navigation item', () => {
    expect(findNavItem('/admin/players/76561198000000001')?.label).toBe('玩家列表');
    expect(findNavItem('/admin/system/status')?.label).toBe('系统状态');
    expect(findNavItem('/admin/does-not-exist')).toBeUndefined();
    expect(findNavSection('/admin/players/76561198000000001')?.label).toBe('玩家管理');
  });

  it('searches by label, path, description and keywords', () => {
    const chinese = searchNavItems('玩家');
    expect(chinese.map((item) => item.label)).toContain('玩家列表');

    const english = searchNavItems('swagger');
    expect(english.map((item) => item.label)).toContain('API 文档');

    const path = searchNavItems('/admin/system');
    expect(path.map((item) => item.label)).toEqual(expect.arrayContaining(['系统状态', '系统设置']));

    expect(searchNavItems('zzzz-not-found')).toHaveLength(0);
    expect(searchNavItems('')).toHaveLength(navItems.length);
  });

  it('exposes every navigation item under exactly one section', () => {
    const labels = navItems.map((item) => item.label);
    expect(new Set(labels).size).toBe(labels.length);
    expect(navItems.length).toBeGreaterThan(10);
  });
});
