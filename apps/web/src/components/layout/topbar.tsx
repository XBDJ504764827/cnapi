import { BookOpen, ExternalLink, Menu, Monitor, Moon, PanelLeftClose, PanelLeftOpen, Sun } from 'lucide-react';
import { useLocation } from 'react-router';

import { Button } from '@/components/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Separator } from '@/components/ui/separator';
import { useTheme } from '@/hooks/use-theme';
import { apiDocsUrl } from '@/lib/env';
import { useUiStore, type Theme } from '@/stores/ui-store';

import { findNavItem } from './nav-config';

interface TopbarProps {
  onOpenMobileNav: () => void;
}

export function Topbar({ onOpenMobileNav }: TopbarProps) {
  const { pathname } = useLocation();
  const collapsed = useUiStore((state) => state.sidebarCollapsed);
  const toggleSidebar = useUiStore((state) => state.toggleSidebar);
  const current = findNavItem(pathname);

  return (
    <header className="sticky top-0 z-30 flex h-14 items-center gap-1.5 border-b bg-background/85 px-3 backdrop-blur sm:px-5">
      <Button variant="ghost" size="icon-sm" className="lg:hidden" onClick={onOpenMobileNav} aria-label="打开导航">
        <Menu />
      </Button>
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
      <Separator orientation="vertical" className="mx-1.5 hidden h-5 sm:block" />
      <div className="min-w-0 flex-1">
        <p className="truncate text-sm font-medium">{current?.label ?? 'CNGOKZAPI'}</p>
      </div>

      <Button variant="ghost" size="sm" asChild className="hidden sm:inline-flex">
        <a href={apiDocsUrl} target="_blank" rel="noreferrer">
          <BookOpen />
          API 文档
          <ExternalLink className="size-3 opacity-60" />
        </a>
      </Button>

      <ThemeMenu />
    </header>
  );
}

function ThemeMenu() {
  const { theme, setTheme } = useTheme();

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" size="icon-sm" className="relative" aria-label="切换主题">
          <Sun className="size-4 scale-100 rotate-0 transition-transform dark:scale-0 dark:-rotate-90" />
          <Moon className="absolute size-4 scale-0 rotate-90 transition-transform dark:scale-100 dark:rotate-0" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-40">
        <DropdownMenuLabel>外观</DropdownMenuLabel>
        <DropdownMenuSeparator />
        <DropdownMenuRadioGroup value={theme} onValueChange={(value) => setTheme(value as Theme)}>
          <DropdownMenuRadioItem value="light">
            <Sun className="size-4" />
            浅色
          </DropdownMenuRadioItem>
          <DropdownMenuRadioItem value="dark">
            <Moon className="size-4" />
            深色
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
