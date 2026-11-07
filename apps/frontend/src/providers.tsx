import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { useEffect, useRef, type ReactNode } from 'react';
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

/**
 * 主题应用器。
 *
 * 抖动有三个来源，这里一次性处理：
 *  1. 几百个 `transition-colors` 元素各自做颜色动画 → 用 View Transition
 *     让浏览器整页截屏后原子替换（不支持时退回「切换瞬间禁用过渡」）。
 *  2. 首屏挂载 / public↔admin 路由切换时，两个独立的 theme store 会重复
 *     触发 apply → 只在 `isDark` 真正变化时才做过渡，否则静默同步。
 *  3. color-scheme 改变会让滚动条样式变化，可能造成 1px 布局跳动 →
 *     CSS 里 `scrollbar-gutter: stable` 预留位置。
 */
function ThemeManager({ children }: { children: ReactNode }) {
  const { pathname } = useLocation();
  const adminTheme = useUiStore((state) => state.theme);
  const publicTheme = usePublicUiStore((state) => state.theme);
  const isAdmin = pathname === '/admin' || pathname.startsWith('/admin/');
  const theme = isAdmin ? adminTheme : publicTheme;
  const appliedDark = useRef<boolean | null>(null);

  useEffect(() => {
    const root = document.documentElement;
    const media = window.matchMedia('(prefers-color-scheme: dark)');
    const isDark = theme === 'dark' || (theme === 'system' && media.matches);

    const commit = () => {
      root.classList.toggle('dark', isDark);
      root.style.colorScheme = isDark ? 'dark' : 'light';
    };

    // 首次挂载，或深浅没变（例如 public ↔ admin 路由切换）：静默同步，不产生重绘动画。
    if (appliedDark.current === null || appliedDark.current === isDark) {
      appliedDark.current = isDark;
      commit();
    } else {
      appliedDark.current = isDark;
      const reduceMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
      const startViewTransition = (
        document as Document & {
          startViewTransition?: (callback: () => void) => { finished: Promise<void> };
        }
      ).startViewTransition;

      if (startViewTransition && !reduceMotion) {
        // 浏览器整页截屏 + 原子替换；CSS 里把默认的 cross-fade 关掉，做到零闪烁。
        startViewTransition.call(document, commit);
      } else {
        root.classList.add('theme-switching');
        commit();
        void root.offsetHeight;
        requestAnimationFrame(() => root.classList.remove('theme-switching'));
      }
    }

    if (theme !== 'system') return;
    const onSystemChange = () => {
      const next = media.matches;
      if (appliedDark.current === next) return;
      appliedDark.current = next;
      root.classList.toggle('dark', next);
      root.style.colorScheme = next ? 'dark' : 'light';
    };
    media.addEventListener('change', onSystemChange);
    return () => media.removeEventListener('change', onSystemChange);
  }, [theme]);

  return children;
}
