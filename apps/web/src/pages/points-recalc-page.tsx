import { useQuery } from '@tanstack/react-query';
import { useState } from 'react';

import { ErrorState } from '@/components/common/error-state';
import { PageHeader } from '@/components/common/page-header';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Callout } from '@/components/ui/callout';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Skeleton } from '@/components/ui/skeleton';
import { formatDateTime, formatNumber } from '@/lib/format';
import { pointsQueries } from '@/services/points-service';

const token = () => import.meta.env.VITE_MAINTENANCE_TOKEN ?? '';

export function PointsRecalcPage() {
  const [filterId, setFilterId] = useState('');
  const [mode, setMode] = useState('KZT');
  const [message, setMessage] = useState('');
  const status = useQuery(pointsQueries.status());
  const runs = useQuery(pointsQueries.runs(1, 20));

  const request = async (url: string) => {
    const headers: Record<string, string> = {};
    if (token()) headers['X-Maintenance-Token'] = token();
    const response = await fetch(url, { method: 'POST', headers });
    const body = (await response.json()) as { message?: string; data?: { queued_filters?: number } };
    setMessage(
      response.ok
        ? `已入队：${formatNumber(body.data?.queued_filters)} 个 Filter`
        : `请求失败：${body.message ?? response.status}`,
    );
    void status.refetch();
    void runs.refetch();
  };

  return (
    <div className="space-y-5">
      <PageHeader
        title="积分重算管理"
        description="查看待重算/失败任务与最近重算记录；单个 Filter 与单模式全量重算需要维护令牌。"
      />

      <Callout
        variant="warning"
        title="权限说明"
        description="管理员接口由 X-Maintenance-Token 保护；未配置 VITE_MAINTENANCE_TOKEN 时后端返回 403，页面不会伪装成功。"
      />

      <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
        {status.data ? (
          [
            { label: '待重算任务', value: status.data.pending },
            { label: '正在重算', value: status.data.running },
            { label: '失败任务', value: status.data.failed },
            { label: 'Filter 总数', value: status.data.filters },
          ].map((item) => (
            <Card key={item.label} className="gap-1 p-4">
              <p className="text-xs text-muted-foreground">{item.label}</p>
              <p className="text-2xl font-semibold tabular-nums">{formatNumber(item.value)}</p>
            </Card>
          ))
        ) : (
          <Skeleton className="col-span-full h-24 rounded-lg" />
        )}
        {status.isError ? (
          <ErrorState title="状态读取失败" description={status.error.message} onRetry={() => void status.refetch()} />
        ) : null}
      </div>

      <div className="grid gap-3 lg:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>指定 Filter 重算</CardTitle>
            <CardDescription>只标记该 Filter，不影响其他模式</CardDescription>
          </CardHeader>
          <CardContent className="flex flex-wrap items-center gap-2">
            <Input
              value={filterId}
              onChange={(event) => setFilterId(event.target.value)}
              placeholder="Filter ID"
              className="w-40"
              aria-label="Filter ID"
            />
            <Button
              size="sm"
              disabled={!filterId.trim()}
              onClick={() => void request(`/api/v1/admin/points/recalc/filter/${filterId.trim()}`)}
            >
              发起重算
            </Button>
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>单模式全量重算</CardTitle>
            <CardDescription>会标记该模式全部 Filter，不跨模式</CardDescription>
          </CardHeader>
          <CardContent className="flex flex-wrap items-center gap-2">
            <select
              value={mode}
              onChange={(event) => setMode(event.target.value)}
              className="h-9 rounded-md border border-border bg-surface-elevated/60 px-2.5 text-[13px] outline-none"
              aria-label="模式"
            >
              <option value="KZT">KZT</option>
              <option value="SKZ">SKZ</option>
              <option value="VNL">VNL</option>
            </select>
            <Button
              size="sm"
              variant="secondary"
              onClick={() => void request(`/api/v1/admin/points/recalc/mode/${mode}`)}
            >
              全量重算 {mode}
            </Button>
          </CardContent>
        </Card>
      </div>
      {message ? (
        <p className="text-xs text-muted-foreground" role="status">
          {message}
        </p>
      ) : null}

      <Card>
        <CardHeader>
          <CardTitle>最近重算记录</CardTitle>
          <CardDescription>事务结果、PB 数量与耗时</CardDescription>
        </CardHeader>
        <CardContent>
          {runs.isError ? (
            <ErrorState title="无法读取重算记录" description={runs.error.message} onRetry={() => void runs.refetch()} />
          ) : runs.isLoading ? (
            <Skeleton className="h-32" />
          ) : runs.data?.items.length ? (
            <ul className="divide-y divide-border/70 text-xs">
              {runs.data.items.map((run) => (
                <li key={run.id} className="flex flex-wrap items-center justify-between gap-2 py-2">
                  <span className="font-mono">Filter #{run.filter_id}</span>
                  <Badge variant={run.status === 'success' ? 'success' : 'danger'}>{run.status}</Badge>
                  <span className="tabular-nums">
                    {run.pb_count} PB · {run.duration_ms} ms
                  </span>
                  <span className="text-muted-foreground">{formatDateTime(run.created_at)}</span>
                  {run.error ? <span className="w-full text-danger">{run.error}</span> : null}
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
