import { NavLink } from 'react-router';

import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { Badge } from '@/components/ui/badge';
import { env } from '@/lib/env';
import { cn } from '@/lib/utils';
import type { NavItem } from '@/types';

import { navSections } from './nav-config';

interface SidebarProps {
  collapsed: boolean;
  /** Called after a navigation happened (used to close the mobile drawer). */
  onNavigate?: () => void;
}

export function Sidebar({ collapsed, onNavigate }: SidebarProps) {
  return (
    <div className="flex h-full w-full flex-col">
      <div className={cn('flex h-14 shrink-0 items-center gap-2.5 border-b px-4', collapsed && 'justify-center px-0')}>
        <div
          className="flex size-8 shrink-0 items-center justify-center rounded-lg bg-primary text-sm font-bold text-primary-foreground"
          aria-hidden="true"
        >
          KZ
        </div>
        {collapsed ? null : (
          <div className="min-w-0">
            <p className="truncate text-sm font-semibold">{env.VITE_APP_NAME}</p>
            <p className="truncate text-[11px] text-muted-foreground">KZ 社区管理控制台</p>
          </div>
        )}
      </div>

      <nav className="flex-1 space-y-5 overflow-y-auto px-3 py-4" aria-label="主导航">
        {navSections.map((section, sectionIndex) => (
          <div key={section.label} className="space-y-1">
            {collapsed ? (
              sectionIndex > 0 ? (
                <div className="mx-2 mb-2 h-px bg-sidebar-border" aria-hidden="true" />
              ) : null
            ) : (
              <p className="px-2 pb-1 text-[11px] font-medium tracking-wider text-muted-foreground uppercase">
                {section.label}
              </p>
            )}
            <ul className="space-y-0.5">
              {section.items.map((item) => (
                <li key={item.to}>
                  <SidebarLink item={item} collapsed={collapsed} onNavigate={onNavigate} />
                </li>
              ))}
            </ul>
          </div>
        ))}
      </nav>

      <div className={cn('border-t px-3 py-3', collapsed && 'px-2')}>
        <p className={cn('text-[11px] text-muted-foreground', collapsed && 'text-center')}>
          {collapsed ? 'v1' : '阶段 1 · 基础架构'}
        </p>
      </div>
    </div>
  );
}

function SidebarLink({ item, collapsed, onNavigate }: { item: NavItem; collapsed: boolean; onNavigate?: () => void }) {
  const Icon = item.icon;

  const link = (
    <NavLink
      to={item.to}
      onClick={onNavigate}
      className={({ isActive }) =>
        cn(
          'group flex items-center gap-2.5 rounded-md px-2.5 py-2 text-sm transition-colors',
          collapsed && 'justify-center px-0',
          isActive
            ? 'bg-sidebar-accent font-medium text-sidebar-accent-foreground'
            : 'text-sidebar-foreground/80 hover:bg-sidebar-accent/60 hover:text-sidebar-accent-foreground',
        )
      }
    >
      <Icon className="size-4 shrink-0" aria-hidden="true" />
      {collapsed ? null : <span className="truncate">{item.label}</span>}
      {!collapsed && item.badge ? (
        <Badge variant="outline" className="ml-auto px-1.5 py-0 text-[10px]">
          {item.badge}
        </Badge>
      ) : null}
    </NavLink>
  );

  if (!collapsed) return link;

  return (
    <Tooltip>
      <TooltipTrigger asChild>{link}</TooltipTrigger>
      <TooltipContent side="right">{item.label}</TooltipContent>
    </Tooltip>
  );
}
