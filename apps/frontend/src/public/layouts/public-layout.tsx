import { useEffect, useState } from 'react';
import { Outlet, ScrollRestoration, useLocation } from 'react-router';

import { AuthErrorBanner } from '@/public/components/layout/auth-error-banner';
import { SearchDialog } from '@/public/components/layout/search-dialog';
import { SiteFooter } from '@/public/components/layout/site-footer';
import { SiteHeader } from '@/public/components/layout/site-header';
import { TooltipProvider } from '@/public/components/ui/tooltip';

export function PublicLayout() {
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
      <div className="flex min-h-screen flex-col bg-background">
        <a
          href="#main-content"
          className="sr-only rounded-md px-3 py-2 text-sm shadow focus:not-sr-only focus:absolute focus:top-2 focus:left-2 focus:z-50 focus:bg-surface"
        >
          跳转到主内容
        </a>
        <SiteHeader onOpenSearch={() => setSearchOpen(true)} />
        <AuthErrorBanner />
        <main id="main-content" className="flex-1">
          <div key={pathname} className="page-enter">
            <Outlet />
          </div>
        </main>
        <SiteFooter />
        <SearchDialog open={searchOpen} onOpenChange={setSearchOpen} />
        <ScrollRestoration />
      </div>
    </TooltipProvider>
  );
}
