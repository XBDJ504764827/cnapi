import { Compass, Home, Search } from 'lucide-react';
import { Link } from 'react-router';

import { Button } from '@/components/ui/button';
import { Kbd } from '@/components/ui/kbd';

export function NotFoundPage() {
  return (
    <div className="flex min-h-[60vh] flex-col items-center justify-center text-center">
      <span className="mb-4 flex size-11 items-center justify-center rounded-lg border border-border bg-surface">
        <Compass className="size-5 text-muted-foreground" aria-hidden="true" />
      </span>
      <p className="font-mono text-xs tracking-[0.2em] text-muted-foreground uppercase">404 · Not Found</p>
      <h1 className="mt-2 text-2xl font-semibold tracking-tight">页面不存在</h1>
      <p className="mt-2 max-w-md text-[13px] leading-relaxed text-muted-foreground">
        请求的地址没有对应的页面，可能已被移动，或尚未实现。
      </p>
      <div className="mt-5 flex flex-wrap items-center justify-center gap-2">
        <Button asChild size="sm">
          <Link to="/dashboard">
            <Home />
            返回仪表盘
          </Link>
        </Button>
        <Button asChild variant="outline" size="sm">
          <Link to="/players">
            <Search />
            玩家列表
          </Link>
        </Button>
      </div>
      <p className="mt-6 flex items-center gap-1.5 text-[11px] text-muted-foreground">
        提示：按 <Kbd>⌘K</Kbd> 搜索页面
      </p>
    </div>
  );
}
