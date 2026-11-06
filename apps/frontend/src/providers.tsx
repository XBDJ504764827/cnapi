import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { useEffect, type ReactNode } from 'react';
import { Outlet, useLocation } from 'react-router';

import { useUiStore } from '@/admin/stores/ui-store';
import { usePublicUiStore } from '@/public/stores/ui-store';

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1,
      refetchOnWindowFocus: false,
      staleTime: 10_000,
    },
  },
});

/** Shared providers for both public and admin route trees. */
export function AppProviders() {
  return (
    <QueryClientProvider client={queryClient}>
      <ThemeManager>
        <Outlet />
      </ThemeManager>
    </QueryClientProvider>
  );
}

function ThemeManager({ children }: { children: ReactNode }) {
  const { pathname } = useLocation();
  const adminTheme = useUiStore((state) => state.theme);
  const publicTheme = usePublicUiStore((state) => state.theme);
  const isAdmin = pathname === '/admin' || pathname.startsWith('/admin/');
  const theme = isAdmin ? adminTheme : publicTheme;

  useEffect(() => {
    const root = document.documentElement;
    const media = window.matchMedia('(prefers-color-scheme: dark)');
    const apply = () => {
      const isDark = theme === 'dark' || (theme === 'system' && media.matches);
      root.classList.toggle('dark', isDark);
      root.style.colorScheme = isDark ? 'dark' : 'light';
    };
    apply();
    if (theme !== 'system') return;
    media.addEventListener('change', apply);
    return () => media.removeEventListener('change', apply);
  }, [theme]);

  return children;
}
