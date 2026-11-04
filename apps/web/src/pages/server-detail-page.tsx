import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import { Activity, ArrowLeft, Copy, Film, KeyRound, RefreshCw, Server as ServerIcon, Timer, Users } from 'lucide-react';
import type * as React from 'react';
import { Link, useParams } from 'react-router';

import { EmptyState } from '@/components/common/empty-state';
import { PageHeader } from '@/components/common/page-header';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Callout } from '@/components/ui/callout';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { DataTable, type DataTableColumn } from '@/components/ui/data-table';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { formatDateTime, formatRelativeTime } from '@/lib/format';
import {
  gameServerAdmin,
  gameServerAdminKeys,
  type AdminHeartbeatRow,
  type AdminPlayerRow,
  type AdminRecordRow,
  type AdminReplayRow,
  type AdminSyncLogRow,
  type AdminSyncTaskRow,
} from '@/services/game-servers-service';

function InfoRow({ label, value }: { label: string; value: React.ReactNode }) {
  return (
    <div className="flex items-start justify-between gap-4 border-b border-border/60 py-2 text-xs last:border-0">
      <span className="shrink-0 text-muted-foreground">{label}</span>
      <span className="min-w-0 text-right">{value}</span>
    </div>
  );
}

const playerColumns: DataTableColumn<AdminPlayerRow>[] = [
  { id: 'name', header: '玩家', cell: (row) => <span className="text-[13px] font-medium">{row.display_name}</span> },
  { id: 'steam', header: 'SteamID64', cell: (row) => <span className="font-mono text-[11px]">{row.steam_id64}</span> },
  { id: 'status', header: '状态', cell: (row) => <Badge variant="outline">{row.status}</Badge> },
  {
    id: 'connected',
    header: '连接时间',
    cell: (row) => <span className="text-[11px]">{formatDateTime(row.connected_at)}</span>,
  },
  {
    id: 'seen',
    header: '最近心跳',
    cell: (row) => <span className="text-[11px] text-muted-foreground">{formatRelativeTime(row.last_seen_at)}</span>,
  },
];

const recordColumns: DataTableColumn<AdminRecordRow>[] = [
  {
    id: 'player',
    header: '玩家',
    cell: (row) => (
      <div>
        <p className="text-[13px] font-medium">{row.display_name ?? 'Unknown player'}</p>
        <p className="font-mono text-[10px] text-muted-foreground">{row.steam_id64}</p>
      </div>
    ),
  },
  {
    id: 'map',
    header: '地图 / Course',
    cell: (row) => (
      <span className="font-mono text-xs">
        {row.map_name} · {row.course_stage === 0 ? 'Main' : `Bonus ${row.course_stage}`}
      </span>
    ),
  },
  {
    id: 'mode',
    header: '模式',
    cell: (row) => (
      <Badge variant="outline">
        {row.mode_code} · {row.time_type.toUpperCase()}
      </Badge>
    ),
  },
  {
    id: 'time',
    header: '成绩',
    cell: (row) => <span className="font-mono text-xs tabular-nums">{row.time_seconds}s</span>,
  },
  {
    id: 'wr',
    header: 'WR',
    cell: (row) =>
      row.is_world_record ? <Badge variant="primary">WR</Badge> : <span className="text-muted-foreground">—</span>,
  },
  {
    id: 'replay',
    header: 'Replay',
    cell: (row) => (
      <Badge variant={row.replay_status === 'available' ? 'success' : 'outline'}>{row.replay_status ?? '未上报'}</Badge>
    ),
  },
  {
    id: 'time_created',
    header: '提交时间',
    cell: (row) => <span className="text-[11px] text-muted-foreground">{formatRelativeTime(row.created_at)}</span>,
  },
];

const replayColumns: DataTableColumn<AdminReplayRow>[] = [
  { id: 'record', header: 'Record', cell: (row) => <span className="font-mono text-xs">#{row.record_id}</span> },
  {
    id: 'object',
    header: '对象',
    cell: (row) => (
      <span className="font-mono text-[11px]">
        {row.bucket}/{row.object_key}
      </span>
    ),
  },
  {
    id: 'size',
    header: '大小',
    cell: (row) => <span className="text-xs tabular-nums">{(row.size_bytes / 1024).toFixed(1)} KiB</span>,
  },
  {
    id: 'status',
    header: '状态',
    cell: (row) => (
      <Badge variant={row.status === 'available' ? 'success' : row.status === 'failed' ? 'danger' : 'warning'}>
        {row.status}
      </Badge>
    ),
  },
  {
    id: 'uploaded',
    header: '上传时间',
    cell: (row) => <span className="text-[11px] text-muted-foreground">{formatDateTime(row.uploaded_at)}</span>,
  },
];

const heartbeatColumns: DataTableColumn<AdminHeartbeatRow>[] = [
  {
    id: 'received',
    header: '接收时间',
    cell: (row) => <span className="text-[11px]">{formatDateTime(row.received_at)}</span>,
  },
  { id: 'map', header: '地图', cell: (row) => <span className="font-mono text-xs">{row.map_name ?? '—'}</span> },
  { id: 'mode', header: '模式', cell: (row) => <Badge variant="outline">{row.mode_code ?? '—'}</Badge> },
  {
    id: 'players',
    header: '玩家',
    cell: (row) => (
      <span className="text-xs tabular-nums">
        {row.online_players}/{row.max_players}
      </span>
    ),
  },
  {
    id: 'plugin',
    header: '插件',
    cell: (row) => <span className="font-mono text-[11px]">{row.plugin_version ?? '—'}</span>,
  },
  {
    id: 'game',
    header: '游戏版本',
    cell: (row) => <span className="font-mono text-[11px] text-muted-foreground">{row.game_version ?? '—'}</span>,
  },
];

export function ServerDetailPage() {
  const { serverId } = useParams<{ serverId: string }>();
  const id = Number(serverId);
  const queryClient = useQueryClient();
  const [token, setToken] = useState<string | null>(null);
  const [feedback, setFeedback] = useState<string | null>(null);

  const query = useQuery({
    queryKey: gameServerAdminKeys.detail(id),
    queryFn: () => gameServerAdmin.detail(id),
    enabled: Number.isFinite(id),
    refetchInterval: 15_000,
  });

  const invalidate = () => queryClient.invalidateQueries({ queryKey: gameServerAdminKeys.detail(id) });
  const action = useMutation({
    mutationFn: async (kind: 'enable' | 'disable' | 'rotate' | 'revoke') => {
      if (kind === 'enable') return gameServerAdmin.setEnabled(id, true);
      if (kind === 'disable') return gameServerAdmin.setEnabled(id, false);
      if (kind === 'rotate') return gameServerAdmin.rotateToken(id);
      return gameServerAdmin.revokeToken(id);
    },
    onSuccess: (result) => {
      if ('token' in result) {
        setToken(result.token);
        setFeedback('Token 已轮换，旧 Token 立即失效。');
      } else {
        setFeedback(result.message);
      }
      void invalidate();
    },
    onError: (error: Error) => setFeedback(error.message),
  });
  const retry = useMutation({
    mutationFn: (taskId: number) => gameServerAdmin.retryTask(taskId),
    onSuccess: (result) => {
      setFeedback(result.message);
      void invalidate();
    },
    onError: (error: Error) => setFeedback(error.message),
  });

  if (query.isError) {
    return (
      <div className="space-y-5">
        <Button variant="ghost" size="sm" asChild className="-ml-2 w-fit">
          <Link to="/servers">
            <ArrowLeft />
            返回服务器列表
          </Link>
        </Button>
        <Callout variant="danger" title="无法加载服务器详情" description={query.error.message} />
      </div>
    );
  }

  const data = query.data;
  const server = data?.server;
  const syncLogs: AdminSyncLogRow[] = data?.sync_logs ?? [];
  const syncTasks: AdminSyncTaskRow[] = data?.sync_tasks ?? [];

  return (
    <div className="space-y-5">
      <Button variant="ghost" size="sm" asChild className="-ml-2 w-fit">
        <Link to="/servers">
          <ArrowLeft />
          返回服务器列表
        </Link>
      </Button>

      <PageHeader
        title={server?.name ?? '服务器详情'}
        description="真实心跳、在线玩家、成绩提交、Replay 与同步错误。"
        meta={
          <>
            <span className="font-mono">#{serverId ?? '—'}</span>
            {server ? (
              <Badge variant={server.status === 'online' ? 'success' : 'outline'}>{server.status}</Badge>
            ) : null}
            {server ? (
              <Badge variant={server.is_enabled ? 'success' : 'danger'}>{server.is_enabled ? '启用' : '禁用'}</Badge>
            ) : null}
          </>
        }
      />

      {token ? (
        <Callout
          variant="success"
          title="新 Token（只显示一次）"
          description={
            <code className="mt-1 block overflow-x-auto rounded-md border border-border bg-surface-elevated p-2 font-mono text-xs">
              {token}
            </code>
          }
          action={
            <Button variant="outline" size="xs" onClick={() => void navigator.clipboard?.writeText(token)}>
              <Copy />
              复制
            </Button>
          }
        />
      ) : null}
      {feedback ? (
        <p className="text-xs text-muted-foreground" role="status">
          {feedback}
        </p>
      ) : null}

      <Card>
        <CardContent className="space-y-3 pt-5">
          <div className="flex flex-wrap items-center justify-between gap-3">
            <div className="min-w-0">
              <p className="text-base font-semibold">{server?.name ?? '—'}</p>
              <p className="font-mono text-xs text-muted-foreground">
                {server ? `${server.ip}:${server.port}` : '—'} · {server?.game.toUpperCase() ?? '—'} ·{' '}
                {server?.region ?? '—'}
              </p>
            </div>
            <div className="flex flex-wrap gap-2">
              <Button
                size="sm"
                variant="secondary"
                disabled={action.isPending || !server}
                onClick={() => action.mutate(server?.is_enabled ? 'disable' : 'enable')}
              >
                {server?.is_enabled ? '禁用服务器' : '启用服务器'}
              </Button>
              <Button size="sm" variant="outline" disabled={action.isPending} onClick={() => action.mutate('rotate')}>
                <RefreshCw />
                轮换 Token
              </Button>
              <Button
                size="sm"
                variant="outline"
                disabled={action.isPending || Boolean(server?.api_key_revoked_at)}
                onClick={() => action.mutate('revoke')}
              >
                <KeyRound />
                撤销 Token
              </Button>
            </div>
          </div>
          <div className="grid gap-x-8 md:grid-cols-2">
            <InfoRow
              label="状态"
              value={
                <Badge variant={server?.status === 'online' ? 'success' : 'outline'}>{server?.status ?? '—'}</Badge>
              }
            />
            <InfoRow
              label="当前地图 / 模式"
              value={
                <span className="font-mono">
                  {server?.current_map_name ?? '—'} · {server?.current_mode ?? '—'}
                </span>
              }
            />
            <InfoRow
              label="在线玩家"
              value={
                <span className="tabular-nums">
                  {server?.online_players ?? 0}/{server?.max_players ?? 0}
                </span>
              }
            />
            <InfoRow
              label="最近心跳"
              value={
                server
                  ? `${formatRelativeTime(server.last_heartbeat_at)}（${server.heartbeat_age_seconds ?? '—'} 秒）`
                  : '—'
              }
            />
            <InfoRow label="插件版本" value={<span className="font-mono">{server?.plugin_version ?? '—'}</span>} />
            <InfoRow label="Token 前缀" value={<span className="font-mono">{server?.api_key_prefix ?? '—'}</span>} />
            <InfoRow label="Token 创建" value={formatDateTime(server?.api_key_created_at)} />
            <InfoRow label="Token 撤销" value={formatDateTime(server?.api_key_revoked_at)} />
            <InfoRow label="查询端口" value={server?.query_port ?? '—'} />
            <InfoRow label="服务器类型" value={server?.server_type ?? '—'} />
          </div>
        </CardContent>
      </Card>

      <Tabs defaultValue="overview">
        <TabsList>
          <TabsTrigger value="overview">
            <ServerIcon className="size-3.5" />
            概览
          </TabsTrigger>
          <TabsTrigger value="players">
            <Users className="size-3.5" />
            玩家
          </TabsTrigger>
          <TabsTrigger value="records">
            <Timer className="size-3.5" />
            成绩
          </TabsTrigger>
          <TabsTrigger value="replays">
            <Film className="size-3.5" />
            Replay
          </TabsTrigger>
          <TabsTrigger value="heartbeat">
            <Activity className="size-3.5" />
            心跳
          </TabsTrigger>
          <TabsTrigger value="sync">
            <RefreshCw className="size-3.5" />
            同步
          </TabsTrigger>
        </TabsList>

        <TabsContent value="overview">
          <Card>
            <CardHeader>
              <CardTitle className="text-sm">同步概览</CardTitle>
            </CardHeader>
            <CardContent className="grid gap-x-8 md:grid-cols-2">
              <InfoRow label="最近成绩" value={data?.recent_records.length ?? 0} />
              <InfoRow label="最近 Replay" value={data?.recent_replays.length ?? 0} />
              <InfoRow label="在线玩家" value={data?.online_players.length ?? 0} />
              <InfoRow label="待处理/失败任务" value={syncTasks.filter((task) => task.status !== 'done').length} />
              <InfoRow
                label="同步错误"
                value={syncLogs.filter((log) => log.status === 'failed' || log.status === 'rejected').length}
              />
              <InfoRow label="创建时间" value={formatDateTime(server?.created_at)} />
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="players">
          <Card className="overflow-hidden py-0">
            <DataTable
              columns={playerColumns}
              rows={data?.online_players ?? []}
              getRowKey={(row) => row.account_id}
              isLoading={query.isLoading}
              minWidthClass="min-w-[760px]"
              empty={<EmptyState icon={Users} title="暂无在线玩家" description="服务器心跳正常但没有玩家会话。" />}
            />
          </Card>
        </TabsContent>

        <TabsContent value="records">
          <Card className="overflow-hidden py-0">
            <DataTable
              columns={recordColumns}
              rows={data?.recent_records ?? []}
              getRowKey={(row) => row.id}
              isLoading={query.isLoading}
              minWidthClass="min-w-[1000px]"
              empty={<EmptyState icon={Timer} title="暂无成绩" description="该服务器尚未提交成绩。" />}
            />
          </Card>
        </TabsContent>

        <TabsContent value="replays">
          <Card className="overflow-hidden py-0">
            <DataTable
              columns={replayColumns}
              rows={data?.recent_replays ?? []}
              getRowKey={(row) => row.id}
              isLoading={query.isLoading}
              minWidthClass="min-w-[900px]"
              empty={<EmptyState icon={Film} title="暂无 Replay" description="成绩已接收但还没有 Replay 元数据。" />}
            />
          </Card>
        </TabsContent>

        <TabsContent value="heartbeat">
          <Card className="overflow-hidden py-0">
            <DataTable
              columns={heartbeatColumns}
              rows={data?.recent_heartbeats ?? []}
              getRowKey={(row) => row.id}
              isLoading={query.isLoading}
              minWidthClass="min-w-[900px]"
              empty={<EmptyState icon={Activity} title="暂无心跳" description="插件尚未连接或 Token 不正确。" />}
            />
          </Card>
        </TabsContent>

        <TabsContent value="sync">
          <div className="space-y-3">
            <Card>
              <CardHeader>
                <CardTitle className="text-sm">待处理 / 失败任务</CardTitle>
              </CardHeader>
              <CardContent className="space-y-2">
                {syncTasks.length === 0 ? (
                  <p className="text-xs text-muted-foreground">暂无同步任务。</p>
                ) : (
                  syncTasks.map((task) => (
                    <div
                      key={task.id}
                      className="flex flex-wrap items-center justify-between gap-2 border-b border-border/60 py-2 text-xs last:border-0"
                    >
                      <div className="flex items-center gap-2">
                        <span className="font-mono">#{task.id}</span>
                        <Badge variant="outline">{task.task_type}</Badge>
                        <Badge
                          variant={task.status === 'done' ? 'success' : task.status === 'failed' ? 'danger' : 'warning'}
                        >
                          {task.status}
                        </Badge>
                        <span className="text-muted-foreground">
                          record {task.record_id ?? '—'} · attempts {task.attempts}
                        </span>
                      </div>
                      <div className="flex items-center gap-2">
                        {task.last_error ? (
                          <span className="max-w-64 truncate text-danger">{task.last_error}</span>
                        ) : null}
                        {task.status !== 'done' ? (
                          <Button
                            size="xs"
                            variant="outline"
                            disabled={retry.isPending}
                            onClick={() => retry.mutate(task.id)}
                          >
                            重试
                          </Button>
                        ) : null}
                      </div>
                    </div>
                  ))
                )}
              </CardContent>
            </Card>
            <Card>
              <CardHeader>
                <CardTitle className="text-sm">最近同步日志</CardTitle>
              </CardHeader>
              <CardContent className="max-h-96 space-y-1 overflow-auto text-xs">
                {syncLogs.length === 0 ? (
                  <p className="text-muted-foreground">暂无日志。</p>
                ) : (
                  syncLogs.map((log) => (
                    <div
                      key={log.id}
                      className="flex flex-wrap items-center justify-between gap-2 border-b border-border/60 py-1.5 last:border-0"
                    >
                      <div className="flex items-center gap-2">
                        <Badge
                          variant={
                            log.status === 'success' ? 'success' : log.status === 'duplicate' ? 'warning' : 'danger'
                          }
                        >
                          {log.status}
                        </Badge>
                        <span className="font-mono text-[11px]">{log.kind}</span>
                        <span className="text-muted-foreground">{log.message ?? '—'}</span>
                        {log.record_id ? <span className="font-mono text-[10px]">record #{log.record_id}</span> : null}
                      </div>
                      <span className="text-[10px] text-muted-foreground">{formatDateTime(log.created_at)}</span>
                    </div>
                  ))
                )}
              </CardContent>
            </Card>
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
}
