import { PanelLeftClose, PanelLeftOpen } from 'lucide-react';
import { Link, NavLink } from 'react-router';

import { StatusIndicator } from '@/components/ui/status-indicator';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { Button } from '@/components/ui/button';
import { useSystemInfo } from '@/hooks/use-system-info';
import { useSystemStatus } from '@/hooks/use-system-status';
import { toneFromStatus } from '@/lib/status';
import { cn } from '@/lib/utils';
import { useUiStore } from '@/stores/ui-store';
import type { NavItem } from '@/types';

import { navSections } from './nav-config';

interface SidebarProps {
  collapsed: boolean;
  /** Called after a navigation happened (used to close the mobile drawer). */
  onNavigate?: () => void;
}

export function Sidebar({ collapsed, onNavigate }: SidebarProps) {
  const toggleSidebar = useUiStore((state) => state.toggleSidebar);

  return (
    <div className="flex h-full w-full flex-col">
      <div
        className={cn(
          'flex h-14 shrink-0 items-center gap-2 border-b border-sidebar-border px-3',
          collapsed && 'justify-center px-0',
        )}
      >
        <Link
          to="/dashboard"
          onClick={onNavigate}
          className="flex min-w-0 items-center gap-2.5 rounded-md outline-none focus-visible:ring-2 focus-visible:ring-ring/40"
          aria-label="CNGOKZAPI 仪表盘"
        >
          <span className="relative flex size-7 shrink-0 items-center justify-center overflow-hidden rounded-md border border-primary/30 bg-primary/15">
            <span className="text-[11px] font-bold tracking-tight text-primary">KZ</span>
          </span>
          {collapsed ? null : (
            <span className="min-w-0">
              <span className="block truncate text-[13px] leading-tight font-semibold text-sidebar-foreground">
                CNGOKZAPI
              </span>
              <span className="block truncate text-[10px] leading-tight text-muted-foreground">KZ Data Platform</span>
            </span>
          )}
        </Link>
        {collapsed ? null : (
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="ghost"
                size="icon-xs"
                className="ml-auto"
                onClick={toggleSidebar}
                aria-label="折叠侧边栏"
              >
                <PanelLeftClose />
              </Button>
            </TooltipTrigger>
            <TooltipContent side="right">折叠侧边栏</TooltipContent>
          </Tooltip>
        )}
      </div>

      <nav className="flex-1 overflow-y-auto px-2 py-3" aria-label="主导航">
        <div className={cn('space-y-4', collapsed && 'space-y-2')}>
          {navSections.map((section, sectionIndex) => (
            <div key={section.id} className="space-y-0.5">
              {collapsed ? (
                sectionIndex > 0 ? (
                  <div className="mx-1 mb-1 h-px bg-sidebar-border" aria-hidden="true" />
                ) : null
              ) : (
                <p className="px-2 pb-1 text-[10px] font-medium tracking-[0.08em] text-sidebar-foreground/45 uppercase">
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
        </div>
      </nav>

      <SidebarFooter collapsed={collapsed} onNavigate={onNavigate} />
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
          'group relative flex h-8 items-center gap-2.5 rounded-md px-2 text-[13px] transition-colors',
          collapsed && 'mx-auto w-9 justify-center px-0',
          isActive
            ? 'bg-sidebar-accent font-medium text-sidebar-accent-foreground'
            : 'text-sidebar-foreground/75 hover:bg-sidebar-accent/60 hover:text-sidebar-accent-foreground',
        )
      }
    >
      {({ isActive }) => (
        <>
          {isActive ? (
            <span className="absolute top-1.5 bottom-1.5 left-0 w-0.5 rounded-r-full bg-primary" aria-hidden="true" />
          ) : null}
          <Icon
            className={cn('size-4 shrink-0', isActive ? 'text-primary' : 'text-sidebar-foreground/55')}
            aria-hidden="true"
          />
          {collapsed ? null : <span className="truncate">{item.label}</span>}
          {!collapsed && item.badge ? (
            <span className="ml-auto rounded border border-border px-1 py-px text-[10px] text-muted-foreground">
              {item.badge}
            </span>
          ) : null}
        </>
      )}
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

function SidebarFooter({ collapsed, onNavigate }: { collapsed: boolean; onNavigate?: () => void }) {
  const statusQuery = useSystemStatus();
  const infoQuery = useSystemInfo();

  const tone = statusQuery.isError ? 'down' : toneFromStatus(statusQuery.data?.status);
  const label = statusQuery.isLoading
    ? '检查中'
    : statusQuery.isError
      ? 'API 未连接'
      : statusQuery.data?.status === 'healthy'
        ? '系统正常'
        : '服务降级';
  const meta = infoQuery.data ? `${infoQuery.data.environment} · v${infoQuery.data.version}` : '—';

  const content = (
    <NavLink
      to="/system/status"
      onClick={onNavigate}
      className={cn(
        'flex h-9 items-center gap-2 rounded-md border border-sidebar-border bg-sidebar-accent/40 px-2 transition-colors hover:bg-sidebar-accent',
        collapsed && 'mx-auto w-9 justify-center px-0',
      )}
    >
      <StatusIndicator tone={tone} showLabel={!collapsed} label={label} pulse={statusQuery.isLoading} />
      {collapsed ? null : <span className="ml-auto truncate font-mono text-[10px] text-muted-foreground">{meta}</span>}
    </NavLink>
  );

  return (
    <div className={cn('border-t border-sidebar-border p-2', collapsed && 'px-1.5')}>
      {collapsed ? (
        <Tooltip>
          <TooltipTrigger asChild>{content}</TooltipTrigger>
          <TooltipContent side="right">
            {label} · {meta}
          </TooltipContent>
        </Tooltip>
      ) : (
        content
      )}
    </div>
  );
}

export function SidebarCollapseButton({ className }: { className?: string }) {
  const collapsed = useUiStore((state) => state.sidebarCollapsed);
  const toggleSidebar = useUiStore((state) => state.toggleSidebar);

  return (
    <Button
      variant="ghost"
      size="icon-sm"
      className={className}
      onClick={toggleSidebar}
      aria-label={collapsed ? '展开侧边栏' : '折叠侧边栏'}
      aria-pressed={collapsed}
    >
      {collapsed ? <PanelLeftOpen /> : <PanelLeftClose />}
    </Button>
  );
}
