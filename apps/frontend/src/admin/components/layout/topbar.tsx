import { ChevronRight, Menu, Monitor, Moon, PanelLeftClose, PanelLeftOpen, Search, Sun } from 'lucide-react';
import { useLocation } from 'react-router';

import { Button } from '@/admin/components/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/admin/components/ui/dropdown-menu';
import { Kbd } from '@/admin/components/ui/kbd';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/admin/components/ui/tooltip';
import { useTheme } from '@/admin/hooks/use-theme';
import { useUiStore, type Theme } from '@/admin/stores/ui-store';
import { cn } from '@/admin/lib/utils';

import { findNavItem, findNavSection } from './nav-config';
import { NotificationsMenu } from './notifications-menu';
import { UserMenu } from './user-menu';

interface TopbarProps {
  onOpenMobileNav: () => void;
  onOpenSearch: () => void;
}

export function Topbar({ onOpenMobileNav, onOpenSearch }: TopbarProps) {
  const { pathname } = useLocation();
  const collapsed = useUiStore((state) => state.sidebarCollapsed);
  const toggleSidebar = useUiStore((state) => state.toggleSidebar);
  const current = findNavItem(pathname);
  const section = findNavSection(pathname);

  return (
    <header className="sticky top-0 z-30 flex h-14 shrink-0 items-center gap-1.5 border-b border-border bg-background/85 px-3 backdrop-blur-md sm:px-4">
      <Button variant="ghost" size="icon-sm" className="lg:hidden" onClick={onOpenMobileNav} aria-label="打开导航">
        <Menu />
      </Button>
      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            variant="ghost"
            size="icon-sm"
            className="hidden lg:inline-flex"
            onClick={toggleSidebar}
            aria-label={collapsed ? '展开侧边栏' : '折叠侧边栏'}
            aria-pressed={collapsed}
          >
            {collapsed ? <PanelLeftOpen /> : <PanelLeftClose />}
          </Button>
        </TooltipTrigger>
        <TooltipContent>{collapsed ? '展开侧边栏' : '折叠侧边栏'}</TooltipContent>
      </Tooltip>

      <nav aria-label="面包屑" className="ml-1 min-w-0 flex-1">
        <ol className="flex min-w-0 items-center gap-1 text-[13px]">
          {section ? (
            <>
              <li className="hidden shrink-0 text-muted-foreground sm:block">{section.label}</li>
              <li className="hidden shrink-0 text-muted-foreground/40 sm:block" aria-hidden="true">
                <ChevronRight className="size-3.5" />
              </li>
            </>
          ) : null}
          <li className="min-w-0 truncate font-medium text-foreground">{current?.label ?? 'CNGOKZAPI'}</li>
        </ol>
      </nav>

      <button
        type="button"
        onClick={onOpenSearch}
        className={cn(
          'hidden h-8 w-60 items-center gap-2 rounded-md border border-border bg-surface-elevated/50 px-2.5 text-left text-xs text-muted-foreground transition-colors hover:bg-surface-hover md:flex',
        )}
        aria-label="打开全局搜索"
      >
        <Search className="size-3.5 shrink-0" aria-hidden="true" />
        <span className="flex-1 truncate">搜索页面…</span>
        <Kbd>⌘K</Kbd>
      </button>
      <Button variant="ghost" size="icon-sm" className="md:hidden" onClick={onOpenSearch} aria-label="搜索">
        <Search />
      </Button>

      <NotificationsMenu />
      <ThemeMenu />

      <div className="mx-1 hidden h-5 w-px bg-border sm:block" aria-hidden="true" />
      <UserMenu />
    </header>
  );
}

function ThemeMenu() {
  const { theme, setTheme } = useTheme();

  return (
    <DropdownMenu>
      <Tooltip>
        <TooltipTrigger asChild>
          <DropdownMenuTrigger asChild>
            <Button variant="ghost" size="icon-sm" className="relative" aria-label="切换主题">
              <Sun className="size-4 scale-100 rotate-0 transition-transform dark:scale-0 dark:-rotate-90" />
              <Moon className="absolute size-4 scale-0 rotate-90 transition-transform dark:scale-100 dark:rotate-0" />
            </Button>
          </DropdownMenuTrigger>
        </TooltipTrigger>
        <TooltipContent>外观</TooltipContent>
      </Tooltip>
      <DropdownMenuContent align="end" className="w-44">
        <DropdownMenuLabel>外观</DropdownMenuLabel>
        <DropdownMenuSeparator />
        <DropdownMenuRadioGroup value={theme} onValueChange={(value) => setTheme(value as Theme)}>
          <DropdownMenuRadioItem value="dark">
            <Moon className="size-4" />
            深色
          </DropdownMenuRadioItem>
          <DropdownMenuRadioItem value="light">
            <Sun className="size-4" />
            浅色
          </DropdownMenuRadioItem>
          <DropdownMenuRadioItem value="system">
            <Monitor className="size-4" />
            跟随系统
          </DropdownMenuRadioItem>
        </DropdownMenuRadioGroup>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
