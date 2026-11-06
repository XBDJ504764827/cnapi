import { useQuery } from '@tanstack/react-query';

import { ErrorState } from '@/admin/components/common/error-state';
import { PageHeader } from '@/admin/components/common/page-header';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/admin/components/ui/card';
import { Skeleton } from '@/admin/components/ui/skeleton';
import { pointsQueries } from '@/admin/services/points-service';
import { formatNumber } from '@/admin/lib/format';

export function PointsPage() {
  const status = useQuery(pointsQueries.status());
  const runs = useQuery(pointsQueries.runs(1, 5));
  const kzt = useQuery(pointsQueries.modeLeaderboard('KZT', 1, 5));
  const skz = useQuery(pointsQueries.modeLeaderboard('SKZ', 1, 5));
  const vnl = useQuery(pointsQueries.modeLeaderboard('VNL', 1, 5));
  const boards = [
    { mode: 'KZT', query: kzt },
    { mode: 'SKZ', query: skz },
    { mode: 'VNL', query: vnl },
  ];

  return (
    <div className="space-y-5">
      <PageHeader
        title="积分概览"
        description="查看三种模式的真实积分排名与 Filter 重算任务状态。积分数据来自 PostgreSQL，Redis 仅作缓存。"
      />

      {status.isError ? (
        <ErrorState
          title="无法读取积分系统状态"
          description={status.error.message}
          onRetry={() => void status.refetch()}
        />
      ) : status.isLoading ? (
        <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
          {Array.from({ length: 4 }).map((_, index) => (
            <Skeleton key={index} className="h-24 rounded-lg" />
          ))}
        </div>
      ) : status.data ? (
        <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
          {[
            { label: 'Filter 总数', value: status.data.filters },
            { label: '待重算', value: status.data.pending },
            { label: '正在重算', value: status.data.running },
            { label: '失败任务', value: status.data.failed },
          ].map((item) => (
            <Card key={item.label} className="gap-1 p-4">
              <p className="text-xs text-muted-foreground">{item.label}</p>
              <p className="text-2xl font-semibold tabular-nums">{formatNumber(item.value)}</p>
            </Card>
          ))}
        </div>
      ) : null}

      <div className="grid gap-3 lg:grid-cols-3">
        {boards.map(({ mode, query }) => (
          <Card key={mode}>
            <CardHeader>
              <CardTitle>{mode} 积分榜</CardTitle>
              <CardDescription>总分 = PRO + NUB</CardDescription>
            </CardHeader>
            <CardContent>
              {query.isError ? (
                <ErrorState title="加载失败" description={query.error.message} onRetry={() => void query.refetch()} />
              ) : query.isLoading ? (
                <Skeleton className="h-32" />
              ) : query.data && query.data.items.length > 0 ? (
                <ol className="space-y-1.5 text-xs">
                  {query.data.items.map((entry) => (
                    <li key={entry.account_id} className="flex items-center justify-between gap-2">
                      <span className="font-mono text-muted-foreground tabular-nums">#{entry.rank}</span>
                      <span className="min-w-0 flex-1 truncate">{entry.display_name}</span>
                      <span className="font-medium tabular-nums">{formatNumber(entry.total_points)}</span>
                    </li>
                  ))}
                </ol>
              ) : (
                <p className="text-xs text-muted-foreground">该模式暂无已重算积分。</p>
              )}
            </CardContent>
          </Card>
        ))}
      </div>

      <Card>
        <CardHeader>
          <CardTitle>最近重算记录</CardTitle>
          <CardDescription>每次 Filter 重算的事务结果与耗时</CardDescription>
        </CardHeader>
        <CardContent>
          {runs.isError ? (
            <ErrorState title="无法读取重算记录" description={runs.error.message} onRetry={() => void runs.refetch()} />
          ) : runs.isLoading ? (
            <Skeleton className="h-24" />
          ) : runs.data && runs.data.items.length > 0 ? (
            <ul className="divide-y divide-border/70 text-xs">
              {runs.data.items.map((run) => (
                <li key={run.id} className="flex items-center justify-between gap-3 py-2">
                  <span className="font-mono">Filter #{run.filter_id}</span>
                  <span className={run.status === 'success' ? 'text-success' : 'text-danger'}>{run.status}</span>
                  <span className="tabular-nums">
                    {run.pb_count} PB · {run.duration_ms} ms
                  </span>
                </li>
              ))}
            </ul>
          ) : (
            <p className="text-xs text-muted-foreground">暂无重算记录。</p>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
