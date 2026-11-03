import { Outlet, ScrollRestoration } from 'react-router';

import { Sidebar } from '@/components/layout/sidebar';
import { Topbar } from '@/components/layout/topbar';
import { TooltipProvider } from '@/components/ui/tooltip';
import { Sheet, SheetContent, SheetDescription, SheetTitle } from '@/components/ui/sheet';
import { cn } from '@/lib/utils';
import { useUiStore } from '@/stores/ui-store';

/**
 * Application shell: fixed collapsible sidebar, sticky top bar, content area.
 */
export function AppLayout() {
  const collapsed = useUiStore((state) => state.sidebarCollapsed);
  const mobileNavOpen = useUiStore((state) => state.mobileNavOpen);
  const setMobileNavOpen = useUiStore((state) => state.setMobileNavOpen);

  return (
    <TooltipProvider>
      <div className="flex min-h-screen">
        <a
          href="#main-content"
          className="sr-only rounded-md px-3 py-2 text-sm shadow focus:not-sr-only focus:absolute focus:top-2 focus:left-2 focus:z-50 focus:bg-background focus:text-foreground"
        >
          跳转到主内容
        </a>

        <aside
          className={cn(
            'sticky top-0 hidden h-screen shrink-0 border-r bg-sidebar transition-[width] duration-200 ease-out lg:flex',
            collapsed ? 'w-[68px]' : 'w-64',
          )}
        >
          <Sidebar collapsed={collapsed} />
        </aside>

        <Sheet open={mobileNavOpen} onOpenChange={setMobileNavOpen}>
          <SheetContent side="left" className="w-72 bg-sidebar p-0">
            <SheetTitle className="sr-only">导航菜单</SheetTitle>
            <SheetDescription className="sr-only">CNGOKZAPI 管理控制台主菜单</SheetDescription>
            <Sidebar collapsed={false} onNavigate={() => setMobileNavOpen(false)} />
          </SheetContent>
        </Sheet>

        <div className="flex min-w-0 flex-1 flex-col">
          <Topbar onOpenMobileNav={() => setMobileNavOpen(true)} />
          <main id="main-content" className="flex-1 px-4 py-6 sm:px-6 lg:px-8">
            <div className="mx-auto w-full max-w-7xl space-y-6">
              <Outlet />
            </div>
          </main>
          <footer className="border-t px-4 py-4 text-xs text-muted-foreground sm:px-6 lg:px-8">
            <div className="mx-auto flex w-full max-w-7xl flex-col gap-1 sm:flex-row sm:items-center sm:justify-between">
              <span>CNGOKZAPI · 面向 CS:GO / CS2 KZ 社区的 API 与管理平台</span>
              <span>阶段 1 · 基础架构</span>
            </div>
          </footer>
        </div>
      </div>
      <ScrollRestoration />
    </TooltipProvider>
  );
}
