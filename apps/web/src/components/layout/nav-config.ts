import {
  Activity,
  Ban,
  FileCode2,
  Film,
  Gauge,
  HeartPulse,
  KeyRound,
  LayoutDashboard,
  Map,
  Server,
  Settings,
  Timer,
  Trophy,
  Users,
} from 'lucide-react';

import type { NavItem, NavSection } from '@/types';

/**
 * Single source of truth for the sidebar and the top bar title.
 * Routes are registered in `src/router/index.tsx`.
 */
export const navSections: NavSection[] = [
  {
    label: '仪表盘',
    items: [{ label: '系统概览', to: '/dashboard', icon: LayoutDashboard }],
  },
  {
    label: 'KZ 管理',
    items: [
      { label: '玩家管理', to: '/kz/players', icon: Users },
      { label: '地图管理', to: '/kz/maps', icon: Map },
      { label: '成绩管理', to: '/kz/records', icon: Timer },
      { label: '排行榜', to: '/kz/leaderboard', icon: Trophy },
    ],
  },
  {
    label: '服务器管理',
    items: [
      { label: '服务器列表', to: '/servers', icon: Server },
      { label: '服务器状态', to: '/servers/status', icon: Activity },
    ],
  },
  {
    label: '数据管理',
    items: [
      { label: 'Jumpstats', to: '/data/jumpstats', icon: Gauge },
      { label: 'Replay', to: '/data/replays', icon: Film },
      { label: '封禁记录', to: '/data/bans', icon: Ban },
    ],
  },
  {
    label: '开发者',
    items: [
      { label: 'API 文档', to: '/developer/api-docs', icon: FileCode2 },
      { label: 'API 密钥', to: '/developer/api-keys', icon: KeyRound, badge: '预留' },
    ],
  },
  {
    label: '系统',
    items: [
      { label: '系统设置', to: '/system/settings', icon: Settings },
      { label: '系统状态', to: '/system/status', icon: HeartPulse },
    ],
  },
];

export const navItems: NavItem[] = navSections.flatMap((section) => section.items);

/** Finds the navigation entry matching a pathname (longest prefix wins). */
export function findNavItem(pathname: string): NavItem | undefined {
  return navItems
    .filter((item) => pathname === item.to || pathname.startsWith(`${item.to}/`))
    .sort((a, b) => b.to.length - a.to.length)
    .at(0);
}
