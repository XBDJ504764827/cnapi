import type { ApiClientError } from '@cngokz/api-client';
import { API_VERSION } from '@cngokz/shared-types';
import {
  Activity,
  ArrowRight,
  ChartNoAxesColumn,
  Database,
  RefreshCw,
  Server,
  Trophy,
  Users,
  Waypoints,
} from 'lucide-react';
import { Link } from 'react-router';

import { ErrorState } from '@/admin/components/common/error-state';
import { PageHeader } from '@/admin/components/common/page-header';
import { RecentEventsCard } from '@/admin/components/dashboard/recent-events-card';
import { ServiceStatusRow } from '@/admin/components/dashboard/service-status-row';
import { StatCard } from '@/admin/components/dashboard/stat-card';
import { Button } from '@/admin/components/ui/button';
import { Callout } from '@/admin/components/ui/callout';
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/admin/components/ui/card';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/admin/components/ui/select';
import { EmptyState } from '@/admin/components/common/empty-state';
import { useSystemEvents } from '@/admin/hooks/use-system-events';
import { useDashboardStats } from '@/admin/hooks/use-dashboard-stats';
import { useSystemStatus } from '@/admin/hooks/use-system-status';
import { formatDateTime, formatNumber, formatUptime } from '@/admin/lib/format';

/**
 * Dashboard. Every number is either real API data or an explicit placeholder;
 * no metric is invented while the statistics endpoints are missing.
 */
export function DashboardPage() {
  const statusQuery = useSystemStatus();
  const eventsQuery = useSystemEvents(10);

  const status = statusQuery.data;
  const checks = status?.checks ?? [];
  const findCheck = (name: string) => checks.find((check) => check.name === name);
  const statusError = statusQuery.error as ApiClientError | null;
  const statsQuery = useDashboardStats();
  const stats = statsQuery.data;

  const refreshAll = () => {
    void statusQuery.refetch();
    void eventsQuery.refetch();
    void statsQuery.refetch();
  };

  return (
    <div className="space-y-6">
      <PageHeader
        title="仪表盘"
        description="CNGOKZAPI 平台运行概览：系统健康状态来自实时探测，业务统计来自 /api/v1/public/statistics。"
        meta={
          <>
            <span>最近检查 {formatDateTime(status?.checked_at)}</span>
            <span className="text-border-strong">·</span>
            <span>每 15 秒自动刷新</span>
            {stats ? (
              <>
                <span className="text-border-strong">·</span>
                <span>统计更新 {formatDateTime(stats.generated_at)}</span>
              </>
            ) : null}
          </>
        }
        actions={
          <>
            <div className="flex items-center gap-1.5">
              <Select value="24h" disabled>
                <SelectTrigger className="w-28" aria-label="时间范围（统计接口未接入）">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="24h">最近 24 小时</SelectItem>
                </SelectContent>
              </Select>
              <span className="hidden text-[11px] text-muted-foreground sm:inline">需统计接口</span>
            </div>
            <Button
              variant="outline"
              size="sm"
              onClick={refreshAll}
              loading={statusQuery.isFetching || eventsQuery.isFetching}
            >
              <RefreshCw />
              刷新
            </Button>
          </>
        }
      />

      {statusQuery.isError ? (
        <ErrorState
          title="无法获取系统状态"
          description={statusError?.message ?? '请确认后端服务已启动，且 API 地址配置正确。'}
          onRetry={() => void statusQuery.refetch()}
        />
      ) : null}

      {status?.status === 'degraded' ? (
        <Callout
          variant="warning"
          title="部分基础服务不可用"
          description="系统处于降级状态，请查看系统状态页中的依赖探测结果。"
          action={
            <Button variant="outline" size="sm" asChild>
              <Link to="/admin/system/status">查看详情</Link>
            </Button>
          }
        />
      ) : null}

      <section aria-label="业务统计" className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
        <StatCard
          label="玩家总数"
          value={stats ? formatNumber(stats.players) : '—'}
          icon={Users}
          hint={stats ? '已登记 Steam 账号' : statsQuery.isError ? '统计接口不可用' : '加载中'}
        />
        <StatCard
          label="地图总数"
          value={stats ? formatNumber(stats.maps) : '—'}
          icon={Waypoints}
          hint={stats ? `${formatNumber(stats.courses)} 条课程` : statsQuery.isError ? '统计接口不可用' : '加载中'}
        />
        <StatCard
          label="成绩总数"
          value={stats ? formatNumber(stats.records) : '—'}
          icon={Trophy}
          hint={
            stats ? `${formatNumber(stats.world_records)} 条世界纪录` : statsQuery.isError ? '统计接口不可用' : '加载中'
          }
        />
        <StatCard
          label="在线服务器"
          value={stats ? formatNumber(stats.servers_online) : '—'}
          icon={Server}
          hint={stats ? '心跳 120 秒内的服务器' : statsQuery.isError ? '统计接口不可用' : '加载中'}
        />
      </section>

      <div className="grid gap-4 lg:grid-cols-3">
        <Card className="lg:col-span-2">
          <CardHeader className="flex-row items-center justify-between gap-3">
            <div className="space-y-1">
              <CardTitle>API 请求趋势</CardTitle>
              <CardDescription>最近 24 小时的接口调用量与延迟</CardDescription>
            </div>
            <span className="rounded border border-border px-1.5 py-0.5 text-[10px] text-muted-foreground">
              接口未接入
            </span>
          </CardHeader>
          <CardContent>
            <EmptyState
              icon={ChartNoAxesColumn}
              title="暂无请求指标"
              description="请求统计接口尚未实现。接入后此处展示调用量、错误率与延迟趋势，当前不展示任何模拟数据。"
              className="min-h-56 [&_p:last-child]:max-w-lg"
            />
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>系统服务</CardTitle>
            <CardDescription>API / PostgreSQL / Redis 实时状态</CardDescription>
          </CardHeader>
          <CardContent className="pt-1">
            <div className="divide-y divide-border/70">
              <ServiceStatusRow
                name="API 服务"
                description={`/api/${API_VERSION}`}
                icon={Activity}
                status={status?.status}
                fallbackDown={statusQuery.isError}
                detail={statusQuery.isError ? '不可达' : '可达'}
                isLoading={statusQuery.isLoading}
              />
              <ServiceStatusRow
                name="PostgreSQL"
                description="主数据库"
                icon={Database}
                status={findCheck('postgresql')?.status}
                latencyMs={findCheck('postgresql')?.latency_ms}
                detail={findCheck('postgresql')?.detail}
                isLoading={statusQuery.isLoading}
              />
              <ServiceStatusRow
                name="Redis"
                description="缓存与临时数据"
                icon={Server}
                status={findCheck('redis')?.status}
                latencyMs={findCheck('redis')?.latency_ms}
                detail={findCheck('redis')?.detail}
                isLoading={statusQuery.isLoading}
              />
            </div>
          </CardContent>
          <CardFooter className="justify-between text-[11px] text-muted-foreground">
            <span>
              版本 {status?.version ?? '—'}
              {status ? ` · 运行 ${formatUptime(status.uptime_seconds)}` : ''}
            </span>
            <Link to="/admin/system/status" className="inline-flex items-center gap-1 text-primary hover:underline">
              系统状态
              <ArrowRight className="size-3" />
            </Link>
          </CardFooter>
        </Card>
      </div>

      <div className="grid gap-4 lg:grid-cols-3">
        <Card>
          <CardHeader>
            <CardTitle>服务器在线状态</CardTitle>
            <CardDescription>在线 / 总服务器数</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="grid grid-cols-2 gap-2">
              <div className="rounded-md border border-border/70 bg-surface-elevated/30 px-3 py-2">
                <p className="text-[11px] text-muted-foreground">在线</p>
                <p className="mt-1 text-lg font-semibold tabular-nums">—</p>
              </div>
              <div className="rounded-md border border-border/70 bg-surface-elevated/30 px-3 py-2">
                <p className="text-[11px] text-muted-foreground">总数</p>
                <p className="mt-1 text-lg font-semibold tabular-nums">—</p>
              </div>
            </div>
            <EmptyState
              compact
              icon={Server}
              title="服务器监控未接入"
              description="心跳与在线玩家接口尚未实现，无法展示真实状态。"
              action={
                <Button variant="outline" size="xs" asChild>
                  <Link to="/admin/servers">服务器列表</Link>
                </Button>
              }
            />
          </CardContent>
        </Card>

        <RecentEventsCard
          className="lg:col-span-2"
          events={eventsQuery.data}
          isLoading={eventsQuery.isLoading}
          errorMessage={(eventsQuery.error as ApiClientError | null)?.message}
          onRetry={() => void eventsQuery.refetch()}
        />
      </div>
    </div>
  );
}
