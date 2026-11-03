import type { ApiClientError } from '@cngokz/api-client';
import { Database, RefreshCw, Server, Timer } from 'lucide-react';

import { ErrorState } from '@/components/common/error-state';
import { PageHeader } from '@/components/common/page-header';
import { StatusBadge } from '@/components/common/status-badge';
import { DependencyStatusCard } from '@/components/dashboard/dependency-status-card';
import { SystemInfoCard } from '@/components/dashboard/system-info-card';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Skeleton } from '@/components/ui/skeleton';
import { useSystemInfo } from '@/hooks/use-system-info';
import { useSystemStatus } from '@/hooks/use-system-status';
import { formatDateTime } from '@/lib/format';
import { toneFromStatus } from '@/lib/status';

const healthEndpoints = [
  { path: '/health', label: '基础健康检查' },
  { path: '/health/live', label: '存活检查' },
  { path: '/health/ready', label: '就绪检查' },
];

/** Detailed dependency and endpoint status. */
export function SystemStatusPage() {
  const statusQuery = useSystemStatus();
  const infoQuery = useSystemInfo();

  const status = statusQuery.data;
  const checks = status?.checks ?? [];
  const findCheck = (name: string) => checks.find((check) => check.name === name);
  const statusError = statusQuery.error as ApiClientError | null;

  return (
    <div className="space-y-6">
      <PageHeader
        title="系统状态"
        description="查看 API、PostgreSQL 与 Redis 的实时探测结果。所有状态均由后端在每次请求时重新检查。"
        meta={<span>最近检查：{formatDateTime(status?.checked_at)}</span>}
        actions={
          <Button
            variant="outline"
            size="sm"
            onClick={() => void statusQuery.refetch()}
            disabled={statusQuery.isFetching}
          >
            <RefreshCw className={statusQuery.isFetching ? 'animate-spin' : undefined} />
            重新检查
          </Button>
        }
      />

      {statusQuery.isError ? (
        <ErrorState
          title="无法获取系统状态"
          description={statusError?.message ?? '后端服务不可达，请检查服务与网络配置。'}
          onRetry={() => void statusQuery.refetch()}
        />
      ) : null}

      <section aria-label="依赖状态" className="grid gap-4 lg:grid-cols-3">
        <Card className="gap-3 py-4">
          <CardHeader className="flex-row items-start justify-between gap-2 space-y-0 px-4">
            <div className="flex min-w-0 items-center gap-2.5">
              <span className="flex size-8 shrink-0 items-center justify-center rounded-md bg-muted text-muted-foreground">
                <Timer className="size-4" aria-hidden="true" />
              </span>
              <div>
                <p className="text-sm font-medium">API 服务</p>
                <p className="text-[11px] text-muted-foreground">整体状态</p>
              </div>
            </div>
            {statusQuery.isLoading ? (
              <Skeleton className="h-5 w-14" />
            ) : (
              <StatusBadge tone={toneFromStatus(status?.status)} />
            )}
          </CardHeader>
          <CardContent className="px-4 text-xs text-muted-foreground">
            {status ? `环境：${status.environment} · 版本：${status.version}` : '—'}
          </CardContent>
        </Card>

        <DependencyStatusCard
          title="PostgreSQL"
          description="主数据库连接池"
          icon={Database}
          check={findCheck('postgresql')}
          isLoading={statusQuery.isLoading}
        />
        <DependencyStatusCard
          title="Redis"
          description="缓存连接与 PING"
          icon={Server}
          check={findCheck('redis')}
          isLoading={statusQuery.isLoading}
        />
      </section>

      <div className="grid gap-4 lg:grid-cols-2">
        <SystemInfoCard info={infoQuery.data} isLoading={infoQuery.isLoading} />

        <Card>
          <CardHeader>
            <CardTitle className="text-sm">健康检查端点</CardTitle>
            <CardDescription>可直接访问，用于负载均衡或监控系统探活。</CardDescription>
          </CardHeader>
          <CardContent>
            <ul className="divide-y">
              {healthEndpoints.map((endpoint) => (
                <li key={endpoint.path} className="flex items-center justify-between gap-3 py-4 first:pt-0 last:pb-0">
                  <div className="min-w-0">
                    <p className="text-sm font-medium">{endpoint.label}</p>
                    <p className="font-mono text-xs text-muted-foreground">{endpoint.path}</p>
                  </div>
                  <Button variant="outline" size="sm" asChild>
                    <a href={endpoint.path} target="_blank" rel="noreferrer">
                      查看
                    </a>
                  </Button>
                </li>
              ))}
            </ul>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
