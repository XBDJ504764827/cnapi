import { RotateCcw } from 'lucide-react';
import { isRouteErrorResponse, Link, useRouteError } from 'react-router';

import { Button } from '@/components/ui/button';
import { Callout } from '@/components/ui/callout';

/** Rendered by the router when a route throws or a loader fails. */
export function RouteErrorPage() {
  const error = useRouteError();

  const message = isRouteErrorResponse(error)
    ? `${error.status} ${error.statusText}`
    : error instanceof Error
      ? error.message
      : '未知错误';

  return (
    <div className="flex min-h-screen items-center justify-center bg-background px-4">
      <div className="w-full max-w-xl space-y-4">
        <div className="flex size-10 items-center justify-center rounded-lg border border-border bg-surface text-[11px] font-bold text-primary">
          KZ
        </div>
        <div className="space-y-1">
          <h1 className="text-xl font-semibold tracking-tight">页面加载失败</h1>
          <p className="text-[13px] text-muted-foreground">发生了未预期的错误，可以重试或返回仪表盘。</p>
        </div>
        <Callout
          variant="danger"
          title="错误详情"
          description={<span className="font-mono text-[11px]">{message}</span>}
        />
        <div className="flex flex-wrap gap-2">
          <Button size="sm" onClick={() => window.location.reload()}>
            <RotateCcw />
            重新加载
          </Button>
          <Button size="sm" variant="outline" asChild>
            <Link to="/dashboard">返回仪表盘</Link>
          </Button>
        </div>
      </div>
    </div>
  );
}
