import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { FilterX, Plus, Server as ServerIcon, ShieldCheck } from 'lucide-react';
import { useState } from 'react';
import { Link } from 'react-router';

import { EmptyState } from '@/admin/components/common/empty-state';
import { FilterBar, ListPanel } from '@/admin/components/common/list-panel';
import { PageHeader } from '@/admin/components/common/page-header';
import { Badge } from '@/admin/components/ui/badge';
import { Button } from '@/admin/components/ui/button';
import { Callout } from '@/admin/components/ui/callout';
import { Card, CardContent, CardHeader, CardTitle } from '@/admin/components/ui/card';
import { DataTable, type DataTableColumn } from '@/admin/components/ui/data-table';
import { Input } from '@/admin/components/ui/input';
import { Label } from '@/admin/components/ui/label';
import { Pagination } from '@/admin/components/ui/pagination';
import { SearchInput } from '@/admin/components/ui/search-input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/admin/components/ui/select';
import { useTableState } from '@/admin/hooks/use-table-state';
import { pageSizeOptions } from '@/admin/lib/kz-constants';
import {
  gameServerAdmin,
  gameServerAdminKeys,
  type AdminServerSummary,
  type CreateServerInput,
} from '@/admin/services/game-servers-service';

const statusMeta: Record<string, { label: string; variant: 'success' | 'warning' | 'danger' | 'outline' }> = {
  online: { label: '在线', variant: 'success' },
  offline: { label: '离线', variant: 'outline' },
  expired: { label: '心跳过期', variant: 'warning' },
  unknown: { label: '未知', variant: 'outline' },
};

const emptyForm: CreateServerInput = {
  name: '',
  ip: '',
  port: 27015,
  query_port: undefined,
  region: '',
  server_type: 'community',
  max_players: 24,
  owner_steam_id64: '',
  owner_display_name: '',
};

export function ServersPage() {
  const table = useTableState();
  const queryClient = useQueryClient();
  const [enabled, setEnabled] = useState('all');
  const [showCreate, setShowCreate] = useState(false);
  const [form, setForm] = useState<CreateServerInput>(emptyForm);
  const [issuedToken, setIssuedToken] = useState<string | null>(null);
  const [feedback, setFeedback] = useState<string | null>(null);

  const query = useQuery({
    queryKey: gameServerAdminKeys.list({ search: table.search, enabled, page: table.page, pageSize: table.pageSize }),
    queryFn: () =>
      gameServerAdmin.list({
        search: table.search || undefined,
        enabled: enabled === 'all' ? undefined : enabled === 'true',
        page: table.page,
        page_size: table.pageSize,
      }),
    refetchInterval: 15_000,
  });

  const create = useMutation({
    mutationFn: () => gameServerAdmin.create(form),
    onSuccess: (result) => {
      setIssuedToken(result.token);
      setFeedback(`服务器 ${result.server.name} 已创建。Token 只显示这一次。`);
      setForm(emptyForm);
      setShowCreate(false);
      void queryClient.invalidateQueries({ queryKey: gameServerAdminKeys.all });
    },
    onError: (error: Error) => setFeedback(error.message),
  });

  const setEnabledMutation = useMutation({
    mutationFn: ({ id, next }: { id: number; next: boolean }) => gameServerAdmin.setEnabled(id, next),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: gameServerAdminKeys.all });
    },
    onError: (error: Error) => setFeedback(error.message),
  });

  const columns: DataTableColumn<AdminServerSummary>[] = [
    {
      id: 'server',
      header: '服务器',
      cell: (row) => (
        <div className="flex items-center gap-2.5">
          <span className="flex size-8 shrink-0 items-center justify-center rounded-md border border-border bg-surface-elevated text-muted-foreground">
            <ServerIcon className="size-4" aria-hidden="true" />
          </span>
          <div className="min-w-0">
            <Link
              to={`/servers/${row.id}`}
              className="truncate text-[13px] font-medium hover:text-primary hover:underline"
            >
              {row.name}
            </Link>
            <p className="truncate font-mono text-[10px] text-muted-foreground">
              {row.ip}:{row.port}
            </p>
          </div>
        </div>
      ),
    },
    {
      id: 'game',
      header: '游戏',
      cell: (row) => <Badge variant="outline">{row.game.toUpperCase()}</Badge>,
    },
    {
      id: 'status',
      header: '状态',
      cell: (row) => {
        const meta = statusMeta[row.status] ?? statusMeta.unknown!;
        return <Badge variant={meta.variant}>{meta.label}</Badge>;
      },
    },
    {
      id: 'enabled',
      header: '启用',
      cell: (row) => <Badge variant={row.is_enabled ? 'success' : 'danger'}>{row.is_enabled ? '启用' : '禁用'}</Badge>,
    },
    {
      id: 'map',
      header: '当前地图',
      cell: (row) => <span className="font-mono text-xs">{row.current_map_name ?? '—'}</span>,
    },
    { id: 'mode', header: '模式', cell: (row) => <Badge variant="outline">{row.current_mode ?? '—'}</Badge> },
    {
      id: 'players',
      header: '玩家',
      className: 'text-right',
      cell: (row) => (
        <span className="text-xs tabular-nums">
          {row.online_players}/{row.max_players}
        </span>
      ),
    },
    {
      id: 'plugin',
      header: '插件',
      cell: (row) => <span className="font-mono text-[11px] text-muted-foreground">{row.plugin_version ?? '—'}</span>,
    },
    {
      id: 'heartbeat',
      header: '最近心跳',
      cell: (row) => (
        <span className="text-[11px] text-muted-foreground">
          {row.heartbeat_age_seconds === null ? '—' : `${row.heartbeat_age_seconds} 秒前`}
        </span>
      ),
    },
    {
      id: 'actions',
      header: '',
      headerClassName: 'text-right',
      className: 'text-right',
      cell: (row) => (
        <div className="flex justify-end gap-1">
          <Button variant="ghost" size="xs" asChild>
            <Link to={`/servers/${row.id}`}>详情</Link>
          </Button>
          <Button
            variant="ghost"
            size="xs"
            disabled={setEnabledMutation.isPending}
            onClick={() => setEnabledMutation.mutate({ id: row.id, next: !row.is_enabled })}
          >
            {row.is_enabled ? '禁用' : '启用'}
          </Button>
        </div>
      ),
    },
  ];

  const update = <K extends keyof CreateServerInput>(key: K, value: CreateServerInput[K]) =>
    setForm((current) => ({ ...current, [key]: value }));

  const hasFilters = table.search.trim().length > 0 || enabled !== 'all';

  return (
    <div className="space-y-5">
      <PageHeader
        title="服务器列表"
        description="真实 CS:GO / GOKZ 服务器注册、Token、心跳、在线玩家与成绩接入状态。"
        meta={query.data ? <span>共 {query.data.meta.total.toLocaleString('zh-CN')} 台服务器</span> : null}
      />

      <Callout
        variant="warning"
        title="管理员权限与服务端 Token"
        description="管理操作需要以管理员身份登录。服务器 Token 只在创建或轮换时返回一次，数据库只保存 SHA-256 哈希。"
      />

      {issuedToken ? (
        <Callout
          variant="success"
          title="新服务器 Token（只显示一次）"
          description={
            <>
              复制到游戏服务器的 <code>cngokz_sync.cfg</code>，然后重启插件。
              <code className="mt-2 block overflow-x-auto rounded-md border border-border bg-surface-elevated p-2 font-mono text-xs">
                {issuedToken}
              </code>
            </>
          }
        />
      ) : null}
      {feedback ? <p className="text-xs text-muted-foreground">{feedback}</p> : null}

      <div className="flex justify-end">
        <Button size="sm" onClick={() => setShowCreate((value) => !value)}>
          <Plus />
          {showCreate ? '收起创建表单' : '创建服务器'}
        </Button>
      </div>

      {showCreate ? (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-sm">
              <ShieldCheck className="size-4" />
              创建 CS:GO 服务器
            </CardTitle>
          </CardHeader>
          <CardContent>
            <form
              className="grid gap-3 md:grid-cols-2 xl:grid-cols-3"
              onSubmit={(event) => {
                event.preventDefault();
                create.mutate();
              }}
            >
              <div className="space-y-1.5">
                <Label htmlFor="server-name">名称</Label>
                <Input
                  id="server-name"
                  value={form.name}
                  onChange={(event) => update('name', event.target.value)}
                  required
                />
              </div>
              <div className="space-y-1.5">
                <Label htmlFor="server-ip">IP</Label>
                <Input
                  id="server-ip"
                  value={form.ip}
                  onChange={(event) => update('ip', event.target.value)}
                  placeholder="203.0.113.10"
                  required
                />
              </div>
              <div className="space-y-1.5">
                <Label htmlFor="server-port">游戏端口</Label>
                <Input
                  id="server-port"
                  type="number"
                  value={form.port}
                  onChange={(event) => update('port', Number(event.target.value))}
                  required
                />
              </div>
              <div className="space-y-1.5">
                <Label htmlFor="server-query-port">查询端口（可选）</Label>
                <Input
                  id="server-query-port"
                  type="number"
                  value={form.query_port ?? ''}
                  onChange={(event) =>
                    update('query_port', event.target.value ? Number(event.target.value) : undefined)
                  }
                />
              </div>
              <div className="space-y-1.5">
                <Label htmlFor="server-region">区域</Label>
                <Input
                  id="server-region"
                  value={form.region ?? ''}
                  onChange={(event) => update('region', event.target.value)}
                  placeholder="eu / cn"
                />
              </div>
              <div className="space-y-1.5">
                <Label htmlFor="server-type">类型</Label>
                <Select value={form.server_type} onValueChange={(value) => update('server_type', value)}>
                  <SelectTrigger id="server-type">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="community">社区</SelectItem>
                    <SelectItem value="official">官方</SelectItem>
                    <SelectItem value="event">活动</SelectItem>
                    <SelectItem value="testing">测试</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-1.5">
                <Label htmlFor="server-max">最大玩家</Label>
                <Input
                  id="server-max"
                  type="number"
                  value={form.max_players ?? 24}
                  onChange={(event) => update('max_players', Number(event.target.value))}
                />
              </div>
              <div className="space-y-1.5">
                <Label htmlFor="server-owner">Owner SteamID64</Label>
                <Input
                  id="server-owner"
                  value={form.owner_steam_id64}
                  onChange={(event) => update('owner_steam_id64', event.target.value)}
                  placeholder="7656119..."
                  required
                />
              </div>
              <div className="space-y-1.5">
                <Label htmlFor="server-owner-name">Owner 显示名（可选）</Label>
                <Input
                  id="server-owner-name"
                  value={form.owner_display_name ?? ''}
                  onChange={(event) => update('owner_display_name', event.target.value)}
                />
              </div>
              <div className="flex items-end">
                <Button type="submit" size="sm" disabled={create.isPending}>
                  {create.isPending ? '创建中…' : '创建并生成 Token'}
                </Button>
              </div>
            </form>
          </CardContent>
        </Card>
      ) : null}

      <ListPanel
        toolbar={
          <FilterBar
            actions={
              <Button
                variant="ghost"
                size="sm"
                onClick={() => {
                  table.resetFilters();
                  setEnabled('all');
                }}
                disabled={!hasFilters}
              >
                <FilterX />
                重置
              </Button>
            }
          >
            <SearchInput
              value={table.search}
              onChange={table.setSearch}
              placeholder="搜索服务器名称或地址…"
              className="w-full sm:w-60"
              aria-label="搜索服务器"
            />
            <Select value={enabled} onValueChange={setEnabled}>
              <SelectTrigger className="w-28" aria-label="启用状态">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">全部</SelectItem>
                <SelectItem value="true">已启用</SelectItem>
                <SelectItem value="false">已禁用</SelectItem>
              </SelectContent>
            </Select>
          </FilterBar>
        }
        footer={
          <Pagination
            page={query.data?.meta.page ?? table.page}
            pageSize={query.data?.meta.page_size ?? table.pageSize}
            total={query.data?.meta.total ?? 0}
            onPageChange={table.setPage}
            onPageSizeChange={table.setPageSize}
            pageSizeOptions={pageSizeOptions}
          />
        }
      >
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
          minWidthClass="min-w-[1180px]"
          empty={
            <EmptyState
              icon={ServerIcon}
              title={hasFilters ? '没有匹配的服务器' : '暂无服务器'}
              description={hasFilters ? '调整筛选条件后再试。' : '创建第一台 CS:GO 服务器并获取一次性 Token。'}
            />
          }
        />
      </ListPanel>
    </div>
  );
}
