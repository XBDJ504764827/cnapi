import { isRouteErrorResponse, Link, useRouteError } from 'react-router';

import { EmptyState } from '@/components/common/empty-state';
import { PageHeader } from '@/components/common/page-header';
import { Button } from '@/components/ui/button';

/** Rendered by the router when a route throws or a loader fails. */
export function RouteErrorPage() {
  const error = useRouteError();

  const message = isRouteErrorResponse(error)
    ? `${error.status} ${error.statusText}`
    : error instanceof Error
      ? error.message
      : '未知错误';

  return (
    <div className="mx-auto max-w-3xl space-y-6 px-4 py-10">
      <PageHeader title="页面加载失败" description="发生了未预期的错误，请刷新页面重试。" />
      <EmptyState
        title="路由错误"
        description={message}
        action={
          <Button asChild variant="outline">
            <Link to="/dashboard">返回系统概览</Link>
          </Button>
        }
      />
    </div>
  );
}
