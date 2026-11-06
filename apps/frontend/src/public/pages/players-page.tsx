import { useQuery } from '@tanstack/react-query';
import { FilterX, Search, Users } from 'lucide-react';
import { useState } from 'react';

import { EmptyState } from '@/public/components/common/empty-state';
import { PageHeader } from '@/public/components/common/page-header';
import { PlayerAvatar, PlayerNameLink } from '@/public/components/common/player-avatar';
import { Button } from '@/public/components/ui/button';
import { Card } from '@/public/components/ui/card';
import { DataTable, type DataTableColumn } from '@/public/components/ui/data-table';
import { Input } from '@/public/components/ui/input';
import { Pagination } from '@/public/components/ui/pagination';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/public/components/ui/select';
import { useListParams } from '@/public/hooks/use-list-params';
import { formatDateTime, formatNumber } from '@/public/lib/format';
import { publicQueries } from '@/public/services/public-service';
import { usePageMeta } from '@/public/seo/use-page-meta';
import type { PlayerSummaryPayload } from '@cngokz/shared-types';

export function PlayersPage() {
  usePageMeta({
    title: '玩家目录',
    description: '搜索 KZ 玩家，查看完成地图数、世界纪录、成绩数量与最近活动。',
    path: '/players',
  });

  const list = useListParams();
  const mode = list.read('mode', 'all');
  const sort = list.read('sort', 'completion');
  const search = list.read('search');
  const [draft, setDraft] = useState(search);

  const modes = useQuery(publicQueries.modes());
  const query = useQuery(
    publicQueries.players({
      search: search || undefined,
      mode: mode === 'all' ? undefined : mode,
      sort,
      page: list.page,
      page_size: list.pageSize,
    }),
  );

  const columns: DataTableColumn<PlayerSummaryPayload>[] = [
    {
      id: 'rank',
      header: '#',
      className: 'w-14',
      cell: (row) => (
        <span className="font-mono text-xs text-muted-foreground tabular-nums">
          {row.rank === null ? '—' : `#${row.rank}`}
        </span>
      ),
    },
    {
      id: 'player',
      header: '玩家',
      cell: (row) => (
        <div className="flex items-center gap-2.5">
          <PlayerAvatar steamId64={row.steam_id64} displayName={row.display_name} size="sm" />
          <div className="min-w-0">
            <PlayerNameLink steamId64={row.steam_id64} displayName={row.display_name} />
            <p className="truncate font-mono text-[10px] text-muted-foreground">{row.steam_id64}</p>
          </div>
        </div>
      ),
    },
    {
      id: 'country',
      header: '国家',
      cell: (row) => <span className="text-xs text-muted-foreground">{row.country ?? '—'}</span>,
    },
    {
      id: 'maps',
      header: '完成地图',
      className: 'text-right',
      cell: (row) => <span className="font-medium tabular-nums">{formatNumber(row.completed_maps)}</span>,
    },
    {
      id: 'wr',
      header: 'WR',
      className: 'text-right',
      cell: (row) => <span className="tabular-nums">{formatNumber(row.wr_count)}</span>,
    },
    {
      id: 'records',
      header: '成绩数',
      className: 'text-right',
      cell: (row) => <span className="text-muted-foreground tabular-nums">{formatNumber(row.total_records)}</span>,
    },
    {
      id: 'last',
      header: '最近活动',
      cell: (row) => <span className="text-xs text-muted-foreground">{formatDateTime(row.last_activity_at)}</span>,
    },
    {
      id: 'points',
      header: '积分',
      className: 'text-right',
      cell: () => <span className="text-muted-foreground">—</span>,
    },
  ];

  const submitSearch = (event: React.FormEvent) => {
    event.preventDefault();
    list.setFilter('search', draft.trim());
  };

  return (
    <div className="mx-auto w-full max-w-7xl space-y-5 px-4 py-8 sm:px-6 lg:px-8">
      <PageHeader
        title="玩家目录"
        description="检索 Steam 玩家档案与 KZ 统计。排名按完成主线地图数计算，积分尚未定义。"
        meta={query.data ? <span>共 {formatNumber(query.data.meta.total)} 位玩家</span> : null}
      />

      <Card className="overflow-hidden">
        <form
          onSubmit={submitSearch}
          className="flex flex-col gap-2 border-b border-border/70 px-4 py-3 lg:flex-row lg:items-center lg:justify-between"
        >
          <div className="flex flex-wrap items-center gap-2">
            <div className="relative w-full sm:w-64">
              <Search className="pointer-events-none absolute top-1/2 left-2.5 size-3.5 -translate-y-1/2 text-muted-foreground" />
              <Input
                value={draft}
                onChange={(event) => setDraft(event.target.value)}
                placeholder="搜索昵称或 SteamID64…"
                className="pl-8"
                aria-label="搜索玩家"
              />
            </div>
            <Button type="submit" size="sm" variant="secondary">
              搜索
            </Button>
            <Select value={mode} onValueChange={(value) => list.setFilter('mode', value, 'all')}>
              <SelectTrigger className="w-32" aria-label="模式">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">全部模式</SelectItem>
                {(modes.data?.items ?? []).map((item) => (
                  <SelectItem key={item.code} value={item.code}>
                    {item.display_name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Select value={sort} onValueChange={(value) => list.setFilter('sort', value, 'completion')}>
              <SelectTrigger className="w-32" aria-label="排序">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="completion">完成地图</SelectItem>
                <SelectItem value="wr">WR 数量</SelectItem>
                <SelectItem value="records">成绩数量</SelectItem>
                <SelectItem value="recent">最近活动</SelectItem>
                <SelectItem value="name">昵称</SelectItem>
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
              disabled={!search && mode === 'all' && sort === 'completion' && list.page === 1}
            >
              <FilterX />
              重置
            </Button>
          </div>
        </form>

        <DataTable
          columns={columns}
          rows={query.data?.items ?? []}
          getRowKey={(row) => row.account_id}
          isLoading={query.isLoading}
          error={
            query.isError
              ? { title: '无法加载玩家', description: query.error.message, onRetry: () => void query.refetch() }
              : null
          }
          minWidthClass="min-w-[980px]"
          empty={
            <EmptyState
              icon={Users}
              title={search ? '没有匹配的玩家' : '暂无玩家数据'}
              description={search ? '换一个昵称或 SteamID64 再试。' : '数据库中还没有公开玩家。'}
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
    </div>
  );
}
