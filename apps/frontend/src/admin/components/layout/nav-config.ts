import {
  Activity,
  Database,
  Ban,
  FileCode2,
  Film,
  Gauge,
  HeartPulse,
  KeyRound,
  LayoutDashboard,
  Map,
  ScrollText,
  Server,
  Settings,
  Timer,
  Trophy,
  Users,
  ListFilter,
} from 'lucide-react';

import type { NavItem, NavSection } from '@/admin/types';

/**
 * Single source of truth for the sidebar, breadcrumbs and the command menu.
 * Routes are registered in `src/router/index.tsx`.
 */
export const navSections: NavSection[] = [
  {
    id: 'overview',
    label: '总览',
    items: [
      {
        label: '仪表盘',
        to: '/admin/dashboard',
        icon: LayoutDashboard,
        description: '平台运行概览与关键指标',
        keywords: ['dashboard', 'overview', 'home'],
      },
      {
        label: '积分概览',
        to: '/admin/points',
        icon: Trophy,
        description: '三模式积分排名与重算任务状态',
        keywords: ['points', 'scoring', 'rank'],
      },
      {
        label: '地图等级',
        to: '/admin/points/tiers',
        icon: Map,
        description: '按 map × course × mode 配置等级与首通状态',
        keywords: ['tier', 'first completion'],
      },
      {
        label: '重算管理',
        to: '/admin/points/recalc',
        icon: Activity,
        description: 'Filter 重算任务与失败记录',
        keywords: ['recalc', 'queue'],
      },
      {
        label: '系统状态',
        to: '/admin/system/status',
        icon: HeartPulse,
        description: 'API、PostgreSQL、Redis 实时健康状态',
        keywords: ['health', 'status', 'healthz', 'postgres', 'redis'],
      },
    ],
  },
  {
    id: 'players',
    label: '玩家管理',
    items: [
      {
        label: '玩家列表',
        to: '/admin/players',
        icon: Users,
        description: '搜索玩家、查看积分与最近活动',
        keywords: ['player', 'steam', 'steamid', 'users'],
      },
    ],
  },
  {
    id: 'kz',
    label: 'KZ 数据',
    items: [
      {
        label: '地图管理',
        to: '/admin/maps',
        icon: Map,
        description: '地图、Workshop、难度与审核状态',
        keywords: ['map', 'workshop', 'course', 'tier'],
      },
      {
        label: '成绩管理',
        to: '/admin/records',
        icon: Timer,
        description: '成绩审核、TP/PRO 与提交来源',
        keywords: ['record', 'run', 'time', 'tp', 'pro'],
      },
      {
        label: '全球排行榜',
        to: '/admin/leaderboard',
        icon: Trophy,
        description: '全球与地图榜单',
        keywords: ['leaderboard', 'rank', 'ranking', 'top'],
      },
      {
        label: 'Jumpstats',
        to: '/admin/jumpstats',
        icon: Gauge,
        description: '跳跃数据与距离榜单',
        keywords: ['jump', 'jumpstats', 'lj', 'bhop'],
      },
      {
        label: 'Replay',
        to: '/admin/replays',
        icon: Film,
        description: 'Replay 元数据与对象存储',
        keywords: ['replay', 'r2', 'record'],
      },
    ],
  },
  {
    id: 'servers',
    label: '服务器管理',
    items: [
      {
        label: '服务器列表',
        to: '/admin/servers',
        icon: Server,
        description: '服务器注册、在线状态与插件版本',
        keywords: ['server', 'servers', 'heartbeat', 'plugin'],
      },
      {
        label: '数据源管理',
        to: '/admin/sources',
        icon: Database,
        description: 'GOKZ-Global / kzgo.eu 同步、健康状态与原始成绩溯源',
        keywords: ['source', 'kzgo', 'gokz global', 'sync', 'provenance'],
      },
    ],
  },
  {
    id: 'security',
    label: '安全管理',
    items: [
      {
        label: '封禁记录',
        to: '/admin/bans',
        icon: Ban,
        description: '封禁、解封与审计',
        keywords: ['ban', 'bans', 'moderation'],
      },
      {
        label: 'API 密钥',
        to: '/admin/developer/api-keys',
        icon: KeyRound,
        description: '服务器与插件访问凭据',
        keywords: ['api key', 'token', 'credential'],
      },
      {
        label: '操作日志',
        to: '/admin/audit-logs',
        icon: ScrollText,
        description: '管理员与系统操作审计',
        keywords: ['audit', 'log', 'history'],
      },
    ],
  },
  {
    id: 'developer',
    label: '开发者',
    items: [
      {
        label: 'API 文档',
        to: '/admin/developer/api-docs',
        icon: FileCode2,
        description: 'Swagger UI 与 OpenAPI 契约',
        keywords: ['api', 'docs', 'swagger', 'openapi'],
      },
      {
        label: 'API 请求日志',
        to: '/admin/developer/request-logs',
        icon: ListFilter,
        description: '接口调用、状态码与延迟',
        keywords: ['request', 'logs', 'http', 'latency'],
      },
    ],
  },
  {
    id: 'system',
    label: '系统',
    items: [
      {
        label: '系统设置',
        to: '/admin/system/settings',
        icon: Settings,
        description: '外观、界面与平台信息',
        keywords: ['settings', 'theme', 'appearance', 'preferences'],
      },
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

/** Section that owns a pathname, used by breadcrumbs. */
export function findNavSection(pathname: string): NavSection | undefined {
  const item = findNavItem(pathname);
  if (!item) return undefined;
  return navSections.find((section) => section.items.some((candidate) => candidate.to === item.to));
}

/** Case-insensitive navigation search for the command menu. */
export function searchNavItems(query: string): Array<NavItem & { sectionLabel: string }> {
  const normalized = query.trim().toLowerCase();
  return navSections.flatMap((section) =>
    section.items
      .filter((item) => {
        if (!normalized) return true;
        const haystack = [item.label, item.to, item.description ?? '', ...(item.keywords ?? [])]
          .join(' ')
          .toLowerCase();
        return haystack.includes(normalized);
      })
      .map((item) => ({ ...item, sectionLabel: section.label })),
  );
}
