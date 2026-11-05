import { Compass, Home, Search } from 'lucide-react';
import { Link } from 'react-router';

import { Button } from '@/public/components/ui/button';
import { Kbd } from '@/public/components/ui/kbd';
import { usePageMeta } from '@/public/seo/use-page-meta';

export function NotFoundPage() {
  usePageMeta({
    title: '页面不存在',
    description: '请求的页面不存在。',
    path: '/404',
    robots: 'noindex',
  });

  return (
    <div className="mx-auto flex min-h-[60vh] w-full max-w-3xl flex-col items-center justify-center px-4 py-16 text-center">
      <span className="mb-4 flex size-11 items-center justify-center rounded-lg border border-border bg-surface">
        <Compass className="size-5 text-muted-foreground" />
      </span>
      <p className="font-mono text-xs tracking-[0.2em] text-muted-foreground uppercase">404 · Not Found</p>
      <h1 className="mt-2 text-2xl font-semibold tracking-tight">页面不存在</h1>
      <p className="mt-2 max-w-md text-[13px] leading-relaxed text-muted-foreground">
        请求的地址没有对应的页面。可以返回首页，或从玩家/地图列表继续浏览。
      </p>
      <div className="mt-5 flex flex-wrap justify-center gap-2">
        <Button size="sm" asChild>
          <Link to="/">
            <Home />
            返回首页
          </Link>
        </Button>
        <Button size="sm" variant="outline" asChild>
          <Link to="/players">
            <Search />
            玩家目录
          </Link>
        </Button>
      </div>
      <p className="mt-6 flex items-center gap-1.5 text-[11px] text-muted-foreground">
        提示：按 <Kbd>⌘K</Kbd> 搜索玩家或地图
      </p>
    </div>
  );
}
