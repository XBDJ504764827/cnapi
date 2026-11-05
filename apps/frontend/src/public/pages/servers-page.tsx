import { useQuery } from '@tanstack/react-query';
import { FilterX, Search, Server as ServerIcon, Users } from 'lucide-react';
import { useState } from 'react';

import { EmptyState } from '@/public/components/common/empty-state';
import { PageHeader } from '@/public/components/common/page-header';
import { Badge } from '@/public/components/ui/badge';
import { Button } from '@/public/components/ui/button';
import { Callout } from '@/public/components/ui/callout';
import { Card } from '@/public/components/ui/card';
import { DataTable, type DataTableColumn } from '@/public/components/ui/data-table';
import { Input } from '@/public/components/ui/input';
import { Pagination } from '@/public/components/ui/pagination';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/public/components/ui/select';
import { StatusIndicator } from '@/public/components/ui/status-indicator';
import { useListParams } from '@/public/hooks/use-list-params';
import { formatDateTime } from '@/public/lib/format';
import { publicQueries } from '@/public/services/public-service';
import { usePageMeta } from '@/public/seo/use-page-meta';
import type { OnlinePlayerSummaryPayload, ServerSummaryPayload } from '@cngokz/shared-types';

export function ServersPage() {
  usePageMeta({
    title: '服务器列表',
    description: '查看已接入 CNGOKZAPI 的 KZ 服务器：在线状态、当前地图、插件版本与最近心跳。',
    path: '/servers',
  });

  const list = useListParams();
  const online = list.read('online', 'all');
  const search = list.read('search');
  const [draft, setDraft] = useState(search);

  const query = useQuery(
    publicQueries.servers({
      search: search || undefined,
      online: online === 'all' ? undefined : online === 'true',
      page: list.page,
      page_size: list.pageSize,
    }),
  );
  const onlinePlayers = useQuery(publicQueries.onlinePlayers({ limit: 100 }));

  const columns: DataTableColumn<ServerSummaryPayload>[] = [
    {
      id: 'server',
      header: '服务器',
      cell: (row) => (
        <div className="flex items-center gap-2.5">
          <span className="flex size-8 items-center justify-center rounded-md border border-border bg-surface-elevated text-muted-foreground">
            <ServerIcon className="size-4" aria-hidden="true" />
          </span>
          <div className="min-w-0">
            <p className="truncate text-[13px] font-medium">{row.name}</p>
            <p className="truncate font-mono text-[10px] text-muted-foreground">
              {row.ip}:{row.port}
            </p>
          </div>
        </div>
      ),
    },
    { id: 'community', header: '社区', cell: (row) => <span className="text-xs">{row.community ?? '—'}</span> },
    {
      id: 'status',
      header: '状态',
      cell: (row) => {
        const meta: Record<string, { tone: 'up' | 'down' | 'unknown'; label: string }> = {
          online: { tone: 'up', label: '在线' },
          offline: { tone: 'unknown', label: '离线' },
          expired: { tone: 'down', label: '心跳过期' },
          unknown: { tone: 'unknown', label: '数据未知' },
        };
        const state = meta[row.status ?? ''] ?? meta.unknown!;
        return <StatusIndicator tone={state.tone} label={state.label} />;
      },
    },
    {
      id: 'map',
      header: '当前地图',
      cell: (row) => <span className="truncate font-mono text-xs">{row.current_map_name ?? '—'}</span>,
    },
    {
      id: 'players',
      header: '玩家',
      className: 'text-right',
      cell: (row) => <span className="text-xs tabular-nums">{row.players ?? '—'}</span>,
    },
    { id: 'mode', header: '模式', cell: (row) => <Badge variant="outline">{row.mode ?? '—'}</Badge> },
    {
      id: 'plugin',
      header: '插件版本',
      cell: (row) => (
        <Badge variant="outline" className="font-mono">
          {row.plugin_version ?? '—'}
        </Badge>
      ),
    },
    {
      id: 'heartbeat',
      header: '最近心跳',
      cell: (row) => (
        <div>
          <span className="text-[11px] text-muted-foreground">{formatDateTime(row.last_heartbeat_at)}</span>
          {row.heartbeat_age_seconds !== null && row.heartbeat_age_seconds !== undefined ? (
            <p className="text-[10px] text-muted-foreground">{row.heartbeat_age_seconds} 秒前</p>
          ) : null}
        </div>
      ),
    },
  ];

  const submit = (event: React.FormEvent) => {
    event.preventDefault();
    list.setFilter('search', draft.trim());
  };

  return (
    <div className="mx-auto w-full max-w-7xl space-y-5 px-4 py-8 sm:px-6 lg:px-8">
      <PageHeader
        title="服务器列表"
        description="服务器主动上报或后端采集的公开信息。在线按 2 分钟心跳判定，过期与未知分别标注。"
        meta={query.data ? <span>共 {query.data.meta.total.toLocaleString('zh-CN')} 台服务器</span> : null}
      />

      <Callout
        variant="neutral"
        title="数据来源"
        description="online = 2 分钟内有心跳；offline = 明确离线；expired = 心跳过期；unknown = 从未上报。玩家数、最大人数与模式来自真实心跳；在线玩家列表只展示心跳未过期且未被禁用的服务器。"
      />

      <Card className="overflow-hidden">
        <form
          onSubmit={submit}
          className="flex flex-col gap-2 border-b border-border/70 px-4 py-3 lg:flex-row lg:items-center lg:justify-between"
        >
          <div className="flex flex-wrap items-center gap-2">
            <div className="relative w-full sm:w-60">
              <Search className="pointer-events-none absolute top-1/2 left-2.5 size-3.5 -translate-y-1/2 text-muted-foreground" />
              <Input
                value={draft}
                onChange={(event) => setDraft(event.target.value)}
                placeholder="搜索服务器名称或地址…"
                className="pl-8"
                aria-label="搜索服务器"
              />
            </div>
            <Button type="submit" size="sm" variant="secondary">
              搜索
            </Button>
            <Select value={online} onValueChange={(value) => list.setFilter('online', value, 'all')}>
              <SelectTrigger className="w-28" aria-label="在线状态">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">全部状态</SelectItem>
                <SelectItem value="true">在线</SelectItem>
                <SelectItem value="false">离线</SelectItem>
              </SelectContent>
            </Select>
            <Button
              type="button"
              variant="ghost"
              size="sm"
              onClick={() => {
                setDraft('');
                list.reset();
              }}
              disabled={!search && online === 'all' && list.page === 1}
            >
              <FilterX />
              重置
            </Button>
          </div>
        </form>

        <DataTable
          columns={columns}
          rows={query.data?.items ?? []}
          getRowKey={(row) => row.id}
          isLoading={query.isLoading}
          error={
            query.isError
              ? { title: '无法加载服务器', description: query.error.message, onRetry: () => void query.refetch() }
              : null
          }
          minWidthClass="min-w-[980px]"
          empty={
            <EmptyState
              icon={ServerIcon}
              title={search ? '没有匹配的服务器' : '暂无服务器数据'}
              description={search ? '换一个名称或地址再试。' : '还没有服务器注册或上报心跳。'}
            />
          }
        />
        <div className="border-t border-border/70 px-4 py-3">
          <Pagination
            page={query.data?.meta.page ?? list.page}
            pageSize={query.data?.meta.page_size ?? list.pageSize}
            total={query.data?.meta.total ?? 0}
            onPageChange={list.setPage}
            onPageSizeChange={list.setPageSize}
          />
        </div>
      </Card>

      <Card className="overflow-hidden">
        <div className="flex items-center justify-between border-b border-border/70 px-4 py-3">
          <div className="flex items-center gap-2">
            <Users className="size-4 text-muted-foreground" />
            <h2 className="text-sm font-semibold">当前在线玩家</h2>
            <Badge variant="outline">{onlinePlayers.data?.count ?? 0}</Badge>
          </div>
          <span className="text-[11px] text-muted-foreground">仅来自心跳正常的服务器</span>
        </div>
        {onlinePlayers.isError ? (
          <div className="p-4 text-xs text-danger">{onlinePlayers.error.message}</div>
        ) : onlinePlayers.isLoading ? (
          <div className="p-4 text-xs text-muted-foreground">正在加载在线玩家…</div>
        ) : (onlinePlayers.data?.items.length ?? 0) === 0 ? (
          <div className="p-4">
            <EmptyState icon={Users} title="暂无在线玩家" description="没有心跳正常的服务器上报在线玩家。" compact />
          </div>
        ) : (
          <div className="grid gap-2 p-4 sm:grid-cols-2 xl:grid-cols-3">
            {onlinePlayers.data?.items.map((player: OnlinePlayerSummaryPayload) => (
              <div
                key={`${player.server_id}-${player.player.steam_id64}`}
                className="flex items-center justify-between gap-3 rounded-md border border-border/70 px-3 py-2"
              >
                <div className="min-w-0">
                  <p className="truncate text-[13px] font-medium">{player.player.display_name}</p>
                  <p className="truncate font-mono text-[10px] text-muted-foreground">
                    {player.server_name} · {player.current_map_name ?? '—'}
                  </p>
                </div>
                <div className="text-right text-[10px] text-muted-foreground">
                  <p>
                    {player.mode ?? '—'} · {player.status}
                  </p>
                  <p>{Math.floor(player.online_seconds / 60)} 分钟</p>
                </div>
              </div>
            ))}
          </div>
        )}
      </Card>
    </div>
  );
}
