import type { ApiClientError } from '@cngokz/api-client';
import { Database, RefreshCw, Server, Timer, Trophy, Users, Waypoints } from 'lucide-react';

import { ErrorState } from '@/components/common/error-state';
import { PageHeader } from '@/components/common/page-header';
import { StatusBadge } from '@/components/common/status-badge';
import { DependencyStatusCard } from '@/components/dashboard/dependency-status-card';
import { RecentEventsCard } from '@/components/dashboard/recent-events-card';
import { StatCard } from '@/components/dashboard/stat-card';
import { SystemInfoCard } from '@/components/dashboard/system-info-card';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader } from '@/components/ui/card';
import { Separator } from '@/components/ui/separator';
import { Skeleton } from '@/components/ui/skeleton';
import { useSystemEvents } from '@/hooks/use-system-events';
import { useSystemInfo } from '@/hooks/use-system-info';
import { useSystemStatus } from '@/hooks/use-system-status';
import { formatDateTime } from '@/lib/format';
import { toneFromStatus } from '@/lib/status';

/**
 * System overview. Every value on this page comes from the API; cards for
 * endpoints that do not exist yet show `—` instead of invented numbers.
 */
export function DashboardPage() {
  const statusQuery = useSystemStatus();
  const infoQuery = useSystemInfo();
  const eventsQuery = useSystemEvents(10);

  const refreshAll = () => {
    void statusQuery.refetch();
    void infoQuery.refetch();
    void eventsQuery.refetch();
  };

  const status = statusQuery.data;
  const checks = status?.checks ?? [];
  const findCheck = (name: string) => checks.find((check) => check.name === name);
  const statusError = statusQuery.error as ApiClientError | null;

  return (
    <div className="space-y-6">
      <PageHeader
        title="系统概览"
        description="CNGOKZAPI 平台运行状态与基础服务健康检查。数据来自后端实时探测接口。"
        meta={
          <>
            <span>最近检查：{formatDateTime(status?.checked_at)}</span>
            <span className="text-muted-foreground/60">·</span>
            <span>每 15 秒自动刷新</span>
          </>
        }
        actions={
          <Button variant="outline" size="sm" onClick={refreshAll} disabled={statusQuery.isFetching}>
            <RefreshCw className={statusQuery.isFetching ? 'animate-spin' : undefined} />
            刷新
          </Button>
        }
      />

      {statusQuery.isError ? (
        <ErrorState
          title="无法获取系统状态"
          description={statusError?.message ?? '请检查后端服务是否已启动，以及 API 地址配置是否正确。'}
          onRetry={() => void statusQuery.refetch()}
        />
      ) : null}

      {status?.status === 'degraded' ? (
        <div role="status" className="rounded-lg border border-warning/30 bg-warning/10 px-4 py-3 text-sm">
          检测到部分基础服务不可用，请查看下方依赖状态与系统日志。
        </div>
      ) : null}

      <section aria-label="业务统计" className="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
        <StatCard label="玩家数量" value="—" icon={Users} hint="统计接口将在后续阶段接入" />
        <StatCard label="服务器数量" value="—" icon={Server} hint="统计接口将在后续阶段接入" />
        <StatCard label="地图数量" value="—" icon={Waypoints} hint="统计接口将在后续阶段接入" />
        <StatCard label="成绩数量" value="—" icon={Trophy} hint="统计接口将在后续阶段接入" />
      </section>

      <section aria-label="基础服务状态" className="grid gap-4 lg:grid-cols-3">
        <Card className="gap-3 py-4">
          <CardHeader className="flex-row items-start justify-between gap-2 space-y-0 px-4">
            <div className="flex min-w-0 items-center gap-2.5">
              <span className="flex size-8 shrink-0 items-center justify-center rounded-md bg-muted text-muted-foreground">
                <Timer className="size-4" aria-hidden="true" />
              </span>
              <div className="min-w-0">
                <p className="truncate text-sm font-medium">API 服务</p>
                <p className="truncate text-[11px] text-muted-foreground">HTTP /api/v1</p>
              </div>
            </div>
            {statusQuery.isLoading ? (
              <Skeleton className="h-5 w-14 rounded-md" />
            ) : (
              <StatusBadge tone={statusQuery.isError ? 'down' : toneFromStatus(status?.status)} />
            )}
          </CardHeader>
          <CardContent className="px-4">
            <dl className="grid grid-cols-2 gap-2 text-xs">
              <div className="space-y-0.5">
                <dt className="text-muted-foreground">服务版本</dt>
                <dd className="font-mono font-medium">
                  {statusQuery.isLoading ? (
                    <Skeleton className="h-4 w-12" />
                  ) : (
                    (status?.version ?? infoQuery.data?.version ?? '—')
                  )}
                </dd>
              </div>
              <div className="space-y-0.5">
                <dt className="text-muted-foreground">运行环境</dt>
                <dd className="font-mono font-medium">{status?.environment ?? infoQuery.data?.environment ?? '—'}</dd>
              </div>
            </dl>
          </CardContent>
        </Card>

        <DependencyStatusCard
          title="PostgreSQL"
          description="主数据库"
          icon={Database}
          check={findCheck('postgresql')}
          isLoading={statusQuery.isLoading}
        />
        <DependencyStatusCard
          title="Redis"
          description="缓存与临时数据"
          icon={Server}
          check={findCheck('redis')}
          isLoading={statusQuery.isLoading}
        />
      </section>

      <div className="grid gap-4 lg:grid-cols-2">
        <SystemInfoCard info={infoQuery.data} isLoading={infoQuery.isLoading} />
        <RecentEventsCard
          events={eventsQuery.data}
          isLoading={eventsQuery.isLoading}
          errorMessage={(eventsQuery.error as ApiClientError | null)?.message}
          onRetry={() => void eventsQuery.refetch()}
        />
      </div>

      <Separator />
      <p className="text-xs text-muted-foreground">
        提示：本页不包含任何模拟数据。统计卡片保留为占位，接入对应查询接口后会自动展示真实数值。
      </p>
    </div>
  );
}
