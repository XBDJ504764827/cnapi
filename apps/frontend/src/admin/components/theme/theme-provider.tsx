import { useEffect, type ReactNode } from 'react';

import { useUiStore } from '@/admin/stores/ui-store';

/**
 * Applies the `dark` class to `<html>` based on the persisted preference and
 * the OS color scheme.
 */
export function ThemeProvider({ children }: { children: ReactNode }) {
  const theme = useUiStore((state) => state.theme);

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
