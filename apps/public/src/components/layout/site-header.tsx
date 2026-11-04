import { LogIn, Menu, Search } from 'lucide-react';
import { useState } from 'react';
import { Link, NavLink } from 'react-router';

import { Button } from '@/components/ui/button';
import { Kbd } from '@/components/ui/kbd';
import { Sheet, SheetContent, SheetDescription, SheetTitle } from '@/components/ui/sheet';
import { cn } from '@/lib/utils';

import { SteamLogin } from './steam-login';
import { ThemeToggle } from './theme-toggle';

const publicNav = [
  { label: '首页', to: '/' },
  { label: '排行榜', to: '/leaderboards' },
  { label: '积分榜', to: '/points' },
  { label: '玩家', to: '/players' },
  { label: '地图', to: '/maps' },
  { label: '成绩', to: '/records' },
  { label: '服务器', to: '/servers' },
];

export function SiteHeader({ onOpenSearch }: { onOpenSearch: () => void }) {
  const [mobileOpen, setMobileOpen] = useState(false);

  return (
    <header className="sticky top-0 z-40 border-b border-border bg-background/85 backdrop-blur-md">
      <div className="mx-auto flex h-14 w-full max-w-7xl items-center gap-2 px-4 sm:px-6 lg:px-8">
        <Button
          variant="ghost"
          size="icon-sm"
          className="md:hidden"
          onClick={() => setMobileOpen(true)}
          aria-label="打开菜单"
        >
          <Menu />
        </Button>

        <Link
          to="/"
          className="flex items-center gap-2.5 rounded-md outline-none focus-visible:ring-2 focus-visible:ring-ring/40"
        >
          <span className="relative flex size-7 items-center justify-center rounded-md border border-primary/30 bg-primary/15">
            <span className="text-[11px] font-bold tracking-tight text-primary">KZ</span>
          </span>
          <span className="min-w-0">
            <span className="block text-[13px] leading-tight font-semibold tracking-tight">CNGOKZ</span>
            <span className="hidden text-[10px] leading-tight text-muted-foreground sm:block">KZ DATA PLATFORM</span>
          </span>
        </Link>

        <nav className="ml-4 hidden h-14 items-stretch md:flex" aria-label="主导航">
          {publicNav.map((item) => (
            <NavLink
              key={item.to}
              to={item.to}
              end={item.to === '/'}
              className={({ isActive }) =>
                cn(
                  'relative flex items-center px-3 text-[13px] transition-colors',
                  isActive ? 'text-foreground' : 'text-muted-foreground hover:text-foreground',
                )
              }
            >
              {({ isActive }) => (
                <>
                  {item.label}
                  {isActive ? <span className="absolute right-2 bottom-0 left-2 h-0.5 rounded-t bg-primary" /> : null}
                </>
              )}
            </NavLink>
          ))}
        </nav>

        <div className="ml-auto flex items-center gap-1">
          <button
            type="button"
            onClick={onOpenSearch}
            className="hidden h-8 w-52 items-center gap-2 rounded-md border border-border bg-surface-elevated/50 px-2.5 text-left text-xs text-muted-foreground transition-colors hover:bg-surface-hover lg:flex"
            aria-label="搜索玩家或地图"
          >
            <Search className="size-3.5" aria-hidden="true" />
            <span className="flex-1 truncate">搜索玩家、地图…</span>
            <Kbd>⌘K</Kbd>
          </button>
          <Button variant="ghost" size="icon-sm" className="lg:hidden" onClick={onOpenSearch} aria-label="搜索">
            <Search />
          </Button>
          <ThemeToggle />
          <SteamLogin />
        </div>
      </div>

      <Sheet open={mobileOpen} onOpenChange={setMobileOpen}>
        <SheetContent side="left" className="w-72 bg-background p-0">
          <SheetTitle className="sr-only">导航菜单</SheetTitle>
          <SheetDescription className="sr-only">CNGOKZ 公开数据平台导航</SheetDescription>
          <div className="flex h-14 items-center gap-2.5 border-b border-border px-4">
            <span className="flex size-7 items-center justify-center rounded-md border border-primary/30 bg-primary/15 text-[11px] font-bold text-primary">
              KZ
            </span>
            <span className="text-sm font-semibold">CNGOKZ</span>
          </div>
          <nav className="space-y-1 p-3" aria-label="移动端主导航">
            {publicNav.map((item) => (
              <NavLink
                key={item.to}
                to={item.to}
                end={item.to === '/'}
                onClick={() => setMobileOpen(false)}
                className={({ isActive }) =>
                  cn(
                    'flex h-10 items-center rounded-md px-3 text-sm transition-colors',
                    isActive
                      ? 'bg-surface-hover font-medium text-foreground'
                      : 'text-muted-foreground hover:bg-surface-hover',
                  )
                }
              >
                {item.label}
              </NavLink>
            ))}
          </nav>
          <div className="mt-auto border-t border-border p-3">
            <Button
              variant="outline"
              size="sm"
              className="w-full"
              onClick={() => {
                setMobileOpen(false);
                onOpenSearch();
              }}
            >
              <Search />
              搜索玩家、地图
            </Button>
            <p className="mt-3 flex items-center gap-1.5 text-[11px] text-muted-foreground">
              <LogIn className="size-3" />
              Steam 登录将在后续阶段提供
            </p>
          </div>
        </SheetContent>
      </Sheet>
    </header>
  );
}
