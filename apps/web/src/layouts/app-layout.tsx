import { useEffect, useState } from 'react';
import { Outlet, ScrollRestoration, useLocation } from 'react-router';

import { CommandMenu } from '@/components/layout/command-menu';
import { Sidebar } from '@/components/layout/sidebar';
import { Topbar } from '@/components/layout/topbar';
import { Sheet, SheetContent, SheetDescription, SheetTitle } from '@/components/ui/sheet';
import { TooltipProvider } from '@/components/ui/tooltip';
import { cn } from '@/lib/utils';
import { useUiStore } from '@/stores/ui-store';

/**
 * Application shell: fixed collapsible sidebar, sticky top bar, content area.
 */
export function AppLayout() {
  const collapsed = useUiStore((state) => state.sidebarCollapsed);
  const mobileNavOpen = useUiStore((state) => state.mobileNavOpen);
  const setMobileNavOpen = useUiStore((state) => state.setMobileNavOpen);
  const [searchOpen, setSearchOpen] = useState(false);
  const { pathname } = useLocation();

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
        event.preventDefault();
        setSearchOpen((open) => !open);
      }
    };

    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, []);

  return (
    <TooltipProvider>
      <div className="flex min-h-screen bg-background">
        <a
          href="#main-content"
          className="sr-only rounded-md px-3 py-2 text-sm shadow focus:not-sr-only focus:absolute focus:top-2 focus:left-2 focus:z-50 focus:bg-surface focus:text-foreground"
        >
          跳转到主内容
        </a>

        <aside
          className={cn(
            'sticky top-0 hidden h-screen shrink-0 border-r border-sidebar-border bg-sidebar transition-[width] duration-200 ease-out lg:flex',
            collapsed ? 'w-16' : 'w-60',
          )}
        >
          <Sidebar collapsed={collapsed} />
        </aside>

        <Sheet open={mobileNavOpen} onOpenChange={setMobileNavOpen}>
          <SheetContent side="left" className="w-[280px] p-0">
            <SheetTitle className="sr-only">导航菜单</SheetTitle>
            <SheetDescription className="sr-only">CNGOKZAPI 管理控制台主菜单</SheetDescription>
            <Sidebar collapsed={false} onNavigate={() => setMobileNavOpen(false)} />
          </SheetContent>
        </Sheet>

        <div className="flex min-w-0 flex-1 flex-col">
          <Topbar onOpenMobileNav={() => setMobileNavOpen(true)} onOpenSearch={() => setSearchOpen(true)} />

          <main id="main-content" className="flex-1 px-4 py-5 sm:px-6 lg:px-8 lg:py-7">
            <div key={pathname} className="page-enter mx-auto w-full max-w-[1500px] space-y-6">
              <Outlet />
            </div>
          </main>

          <footer className="border-t border-border/70 px-4 py-3 text-[11px] text-muted-foreground sm:px-6 lg:px-8">
            <div className="mx-auto flex w-full max-w-[1500px] flex-col gap-1 sm:flex-row sm:items-center sm:justify-between">
              <span>CNGOKZAPI · 面向 CS:GO / CS2 KZ 社区的数据管理与 API 平台</span>
              <span className="font-mono">阶段 3 · UI/UX 重构</span>
            </div>
          </footer>
        </div>
      </div>

      <CommandMenu open={searchOpen} onOpenChange={setSearchOpen} />
      <ScrollRestoration />
    </TooltipProvider>
  );
}
