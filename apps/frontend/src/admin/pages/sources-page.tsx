import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Activity, Database, RefreshCw, ShieldAlert } from 'lucide-react';
import { useState } from 'react';

import { PageHeader } from '@/admin/components/common/page-header';
import { Badge } from '@/admin/components/ui/badge';
import { Button } from '@/admin/components/ui/button';
import { Callout } from '@/admin/components/ui/callout';
import { Card, CardContent, CardHeader, CardTitle } from '@/admin/components/ui/card';
import { Skeleton } from '@/admin/components/ui/skeleton';
import { formatDateTime, formatRelativeTime } from '@/admin/lib/format';
import { sourceAdmin, sourceKeys, type SourceSummary } from '@/admin/services/sources-service';

const statusMeta: Record<string, { label: string; variant: 'success' | 'warning' | 'danger' | 'outline' }> = {
  healthy: { label: '正常', variant: 'success' },
  degraded: { label: '降级', variant: 'warning' },
  offline: { label: '离线', variant: 'danger' },
  disabled: { label: '已禁用', variant: 'outline' },
  unknown: { label: '未知', variant: 'outline' },
};

function SourceCard({
  source,
  onToggle,
  onSync,
  busy,
}: {
  source: SourceSummary;
  onToggle: (enabled: boolean) => void;
  onSync: () => void;
  busy: boolean;
}) {
  const meta = statusMeta[source.status] ?? statusMeta.unknown!;
  return (
    <Card>
      <CardHeader className="flex-row items-start justify-between gap-3">
        <div>
          <CardTitle className="flex items-center gap-2 text-sm">
            <Database className="size-4" />
            {source.display_name}
          </CardTitle>
          <p className="mt-1 font-mono text-[10px] text-muted-foreground">
            {source.code} · {source.source_type}
          </p>
        </div>
        <div className="flex flex-wrap justify-end gap-1.5">
          <Badge variant={meta.variant}>{meta.label}</Badge>
          <Badge variant={source.is_enabled ? 'success' : 'danger'}>{source.is_enabled ? '启用' : '禁用'}</Badge>
        </div>
      </CardHeader>
      <CardContent className="space-y-3">
        <div className="grid grid-cols-2 gap-x-4 gap-y-1 text-xs">
          <span className="text-muted-foreground">最近成功</span>
          <span className="text-right">{formatRelativeTime(source.last_success_at)}</span>
          <span className="text-muted-foreground">最近尝试</span>
          <span className="text-right">{formatDateTime(source.last_attempt_at)}</span>
          <span className="text-muted-foreground">连续失败</span>
          <span className="text-right tabular-nums">{source.consecutive_failures}</span>
          <span className="text-muted-foreground">原始记录</span>
          <span className="text-right tabular-nums">{source.raw_records.toLocaleString('zh-CN')}</span>
          <span className="text-muted-foreground">标准化关联</span>
          <span className="text-right tabular-nums">{source.canonical_links.toLocaleString('zh-CN')}</span>
          <span className="text-muted-foreground">最近任务</span>
          <span className="text-right">{source.latest_run_status ?? '—'}</span>
        </div>
        {source.last_error ? (
          <p className="rounded-md border border-danger/30 bg-danger/8 px-2.5 py-2 text-[11px] text-danger">
            {source.last_error}
          </p>
        ) : null}
        {Object.keys(source.cursor).length > 0 ? (
          <pre className="max-h-28 overflow-auto rounded-md border border-border bg-surface-elevated p-2 text-[10px] text-muted-foreground">
            {JSON.stringify(source.cursor, null, 2)}
          </pre>
        ) : null}
        <div className="flex flex-wrap gap-2">
          <Button
            size="xs"
            variant="secondary"
            disabled={busy || !source.is_enabled || source.code === 'cngokz_plugin'}
            onClick={onSync}
          >
            <RefreshCw className={busy ? 'animate-spin' : undefined} />
            立即同步
          </Button>
          {source.code !== 'cngokz_plugin' ? (
            <Button size="xs" variant="outline" disabled={busy} onClick={() => onToggle(!source.is_enabled)}>
              {source.is_enabled ? '禁用' : '启用'}
            </Button>
          ) : null}
        </div>
      </CardContent>
    </Card>
  );
}

export function SourcesPage() {
  const queryClient = useQueryClient();
  const [feedback, setFeedback] = useState<string | null>(null);
  const [selected, setSelected] = useState('kzgo');
  const query = useQuery({
    queryKey: sourceKeys.list(),
    queryFn: sourceAdmin.list,
    refetchInterval: 15_000,
  });
  const runs = useQuery({
    queryKey: sourceKeys.runs(selected),
    queryFn: () => sourceAdmin.runs(selected),
    enabled: Boolean(selected),
    refetchInterval: 15_000,
  });
  const invalidate = () => {
    void queryClient.invalidateQueries({ queryKey: sourceKeys.all });
  };
  const toggle = useMutation({
    mutationFn: ({ code, enabled }: { code: string; enabled: boolean }) => sourceAdmin.setEnabled(code, enabled),
    onSuccess: (source) => {
      setFeedback(`${source.display_name} 已${source.is_enabled ? '启用' : '禁用'}`);
      invalidate();
    },
    onError: (error: Error) => setFeedback(error.message),
  });
  const sync = useMutation({
    mutationFn: (code: string) => sourceAdmin.sync(code),
    onSuccess: (result) => {
      setSelected(result.source);
      setFeedback(`${result.source}：${result.message}`);
      invalidate();
    },
    onError: (error: Error) => setFeedback(error.message),
  });

  return (
    <div className="space-y-5">
      <PageHeader
        title="数据源管理"
        description="GOKZ-Global、kzgo.eu 与自有插件的统一接入、健康状态、同步游标和原始成绩溯源。"
      />
      <Callout
        variant="warning"
        title="外部积分不参与 CNGOKZ 积分"
        description="kzgo.eu / GOKZ-Global 的 points、rank 只保留在原始 payload 中用于审计；公开排行榜只使用 CNGOKZ 第六阶段公式计算。"
      />
      {feedback ? (
        <p className="text-xs text-muted-foreground" role="status">
          {feedback}
        </p>
      ) : null}

      {query.isLoading ? (
        <div className="grid gap-3 lg:grid-cols-3">
          <Skeleton className="h-64" />
          <Skeleton className="h-64" />
          <Skeleton className="h-64" />
        </div>
      ) : query.isError ? (
        <Callout variant="danger" icon={ShieldAlert} title="无法加载数据源" description={query.error.message} />
      ) : (
        <div className="grid gap-3 lg:grid-cols-3">
          {query.data?.items.map((source) => (
            <div key={source.code} onClick={() => setSelected(source.code)}>
              <SourceCard
                source={source}
                busy={toggle.isPending || sync.isPending}
                onToggle={(enabled) => toggle.mutate({ code: source.code, enabled })}
                onSync={() => sync.mutate(source.code)}
              />
            </div>
          ))}
        </div>
      )}

      <Card>
        <CardHeader className="flex-row items-center justify-between gap-3">
          <CardTitle className="flex items-center gap-2 text-sm">
            <Activity className="size-4" />
            同步运行记录
          </CardTitle>
          <select
            value={selected}
            onChange={(event) => setSelected(event.target.value)}
            className="h-8 rounded-md border border-border bg-surface-elevated/60 px-2 text-xs"
            aria-label="选择数据源"
          >
            {(query.data?.items ?? []).map((source) => (
              <option key={source.code} value={source.code}>
                {source.display_name}
              </option>
            ))}
          </select>
        </CardHeader>
        <CardContent>
          {runs.isLoading ? (
            <Skeleton className="h-32" />
          ) : runs.isError ? (
            <p className="text-xs text-danger">{runs.error.message}</p>
          ) : runs.data?.items.length ? (
            <div className="max-h-96 space-y-1 overflow-auto text-xs">
              {runs.data.items.map((run) => (
                <div
                  key={run.id}
                  className="flex flex-wrap items-center justify-between gap-2 border-b border-border/60 py-1.5 last:border-0"
                >
                  <div className="flex items-center gap-2">
                    <Badge
                      variant={run.status === 'success' ? 'success' : run.status === 'failed' ? 'danger' : 'warning'}
                    >
                      {run.status}
                    </Badge>
                    <span className="font-mono text-[11px]">{run.run_type}</span>
                    <span className="text-muted-foreground">#{run.id}</span>
                  </div>
                  <span className="text-[11px] text-muted-foreground">{formatDateTime(run.started_at)}</span>
                  {run.error ? <span className="w-full text-danger">{run.error}</span> : null}
                  {Object.keys(run.stats).length > 0 ? (
                    <code className="w-full truncate text-[10px] text-muted-foreground">
                      {JSON.stringify(run.stats)}
                    </code>
                  ) : null}
                </div>
              ))}
            </div>
          ) : (
            <p className="text-xs text-muted-foreground">暂无同步运行记录。</p>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
