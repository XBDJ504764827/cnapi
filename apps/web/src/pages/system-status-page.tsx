import type { ApiClientError } from '@cngokz/api-client';
import { Database, ExternalLink, HeartPulse, RefreshCw, Server } from 'lucide-react';

import { ErrorState } from '@/components/common/error-state';
import { PageHeader } from '@/components/common/page-header';
import { DependencyStatusCard } from '@/components/dashboard/dependency-status-card';
import { SystemInfoCard } from '@/components/dashboard/system-info-card';
import { Button } from '@/components/ui/button';
import { Callout } from '@/components/ui/callout';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Skeleton } from '@/components/ui/skeleton';
import { StatusIndicator } from '@/components/ui/status-indicator';
import { useSystemInfo } from '@/hooks/use-system-info';
import { useSystemStatus } from '@/hooks/use-system-status';
import { formatDateTime, formatUptime } from '@/lib/format';
import { toneFromStatus } from '@/lib/status';

const healthEndpoints = [
  { path: '/health', label: '基础健康检查', description: '进程可响应 HTTP 请求即返回 200，不探测依赖。' },
  { path: '/health/live', label: '存活检查', description: '用于容器 liveness probe，返回进程运行时长。' },
  { path: '/health/ready', label: '就绪检查', description: '探测 PostgreSQL 与 Redis，任一不可用返回 503。' },
];

/** Detailed dependency and endpoint status. */
export function SystemStatusPage() {
  const statusQuery = useSystemStatus();
  const infoQuery = useSystemInfo();

  const status = statusQuery.data;
  const checks = status?.checks ?? [];
  const findCheck = (name: string) => checks.find((check) => check.name === name);
  const statusError = statusQuery.error as ApiClientError | null;
  const tone = statusQuery.isError ? 'down' : toneFromStatus(status?.status);

  return (
    <div className="space-y-6">
      <PageHeader
        title="系统状态"
        description="API、PostgreSQL 与 Redis 的实时探测结果。每次请求都会重新检查依赖，不使用缓存或模拟状态。"
        meta={<span>最近检查 {formatDateTime(status?.checked_at)}</span>}
        actions={
          <Button
            variant="outline"
            size="sm"
            onClick={() => void statusQuery.refetch()}
            loading={statusQuery.isFetching}
          >
            <RefreshCw />
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

      <Card>
        <CardContent className="flex flex-col gap-4 pt-5 sm:flex-row sm:items-center sm:justify-between">
          <div className="flex items-center gap-3">
            <span className="flex size-10 items-center justify-center rounded-lg border border-border bg-surface-elevated text-muted-foreground">
              <HeartPulse className="size-5" aria-hidden="true" />
            </span>
            <div className="space-y-1">
              {statusQuery.isLoading ? (
                <Skeleton className="h-6 w-32" />
              ) : (
                <StatusIndicator
                  tone={tone}
                  label={
                    statusQuery.isError ? 'API 不可达' : status?.status === 'healthy' ? '所有服务正常' : '服务降级'
                  }
                  className="text-base"
                  pulse={statusQuery.isLoading}
                />
              )}
              <p className="text-xs text-muted-foreground">
                {status ? `环境 ${status.environment} · 版本 ${status.version}` : '等待后端响应'}
              </p>
            </div>
          </div>

          <dl className="grid grid-cols-2 gap-x-8 gap-y-2 text-xs sm:grid-cols-3">
            <div>
              <dt className="text-muted-foreground">运行时长</dt>
              <dd className="mt-0.5 font-mono">{status ? formatUptime(status.uptime_seconds) : '—'}</dd>
            </div>
            <div>
              <dt className="text-muted-foreground">启动时间</dt>
              <dd className="mt-0.5 font-mono">{formatDateTime(status?.started_at)}</dd>
            </div>
            <div className="hidden sm:block">
              <dt className="text-muted-foreground">检查时间</dt>
              <dd className="mt-0.5 font-mono">{formatDateTime(status?.checked_at)}</dd>
            </div>
          </dl>
        </CardContent>
      </Card>

      <section aria-label="依赖状态" className="grid gap-3 lg:grid-cols-3">
        <DependencyStatusCard
          title="API 服务"
          description="HTTP /api/v1"
          icon={HeartPulse}
          status={statusQuery.isError ? 'down' : status?.status}
          detail={statusQuery.isError ? '不可达' : '可达'}
          isLoading={statusQuery.isLoading}
          className={statusQuery.isError ? 'ring-1 ring-danger/20' : undefined}
        />
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

      {status?.status === 'degraded' ? (
        <Callout
          variant="warning"
          title="检测到依赖不可用"
          description="请查看上方各依赖的探测详情，探测详情字段只包含稳定错误分类，不会暴露连接串或 SQL。"
        />
      ) : null}

      <div className="grid gap-3 lg:grid-cols-2">
        <SystemInfoCard info={infoQuery.data} isLoading={infoQuery.isLoading} />

        <Card>
          <CardHeader>
            <CardTitle>健康检查端点</CardTitle>
            <CardDescription>可直接访问，用于负载均衡或监控系统探活</CardDescription>
          </CardHeader>
          <CardContent>
            <ul className="divide-y divide-border/70">
              {healthEndpoints.map((endpoint) => (
                <li key={endpoint.path} className="flex items-center justify-between gap-3 py-2.5 first:pt-0 last:pb-0">
                  <div className="min-w-0">
                    <p className="text-[13px] font-medium">{endpoint.label}</p>
                    <p className="font-mono text-[11px] text-muted-foreground">{endpoint.path}</p>
                    <p className="mt-0.5 hidden text-[11px] text-muted-foreground sm:block">{endpoint.description}</p>
                  </div>
                  <Button variant="outline" size="xs" asChild>
                    <a href={endpoint.path} target="_blank" rel="noreferrer">
                      打开
                      <ExternalLink className="size-3" />
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
