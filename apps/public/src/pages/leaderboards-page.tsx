import { useQuery } from '@tanstack/react-query';
import { Trophy } from 'lucide-react';

import { EmptyState } from '@/components/common/empty-state';
import { ModeTabs } from '@/components/common/mode-tabs';
import { PageHeader } from '@/components/common/page-header';
import { PlayerAvatar, PlayerNameLink } from '@/components/common/player-avatar';
import { Callout } from '@/components/ui/callout';
import { Card } from '@/components/ui/card';
import { DataTable, type DataTableColumn } from '@/components/ui/data-table';
import { Pagination } from '@/components/ui/pagination';
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { useListParams } from '@/hooks/use-list-params';
import { formatDateTime, formatNumber } from '@/lib/format';
import { publicQueries } from '@/services/public-service';
import { usePageMeta } from '@/seo/use-page-meta';
import type { LeaderboardEntryPayload } from '@cngokz/shared-types';

export function LeaderboardsPage() {
  usePageMeta({
    title: '全球排行榜',
    description: '按游戏模式与 NUB/PRO 查看全球 KZ 排行榜：完成地图数、世界纪录与成绩数量。',
    path: '/leaderboards',
  });

  const list = useListParams();
  const mode = list.read('mode', 'vanilla');
  const timeType = list.read('time_type', 'all');
  const modes = useQuery(publicQueries.modes());
  const query = useQuery(
    publicQueries.leaderboard({ mode, time_type: timeType, page: list.page, page_size: list.pageSize }),
  );

  const columns: DataTableColumn<LeaderboardEntryPayload>[] = [
    {
      id: 'rank',
      header: '#',
      className: 'w-14',
      cell: (row) => <span className="font-mono text-xs text-muted-foreground tabular-nums">#{row.rank}</span>,
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

  return (
    <div className="mx-auto w-full max-w-7xl space-y-5 px-4 py-8 sm:px-6 lg:px-8">
      <PageHeader
        title="全球排行榜"
        description="按模式查看全球玩家排名。当前按完成主线地图数排序，积分算法尚未定义。"
        meta={query.data ? <span>共 {formatNumber(query.data.meta.total)} 位有成绩的玩家</span> : null}
      />

      <Callout
        variant="neutral"
        title="积分与排名口径"
        description="CNGOKZAPI 尚未实现积分算法，因此排行榜不使用 Points/Rating。排名按完成主线地图数（相同则依次比较 WR 数、成绩数）计算。"
      />

      <ModeTabs
        modes={modes.data?.items}
        value={mode}
        onChange={(value) => list.setFilter('mode', value, 'vanilla')}
        isLoading={modes.isLoading}
      />

      <Tabs value={timeType} onValueChange={(value) => list.setFilter('time_type', value, 'all')}>
        <TabsList>
          <TabsTrigger value="all">全部类型</TabsTrigger>
          <TabsTrigger value="pro">PRO</TabsTrigger>
          <TabsTrigger value="nub">NUB</TabsTrigger>
        </TabsList>
      </Tabs>

      <Card className="overflow-hidden">
        <DataTable
          columns={columns}
          rows={query.data?.items ?? []}
          getRowKey={(row) => row.account_id}
          isLoading={query.isLoading}
          error={
            query.isError
              ? { title: '无法加载排行榜', description: query.error.message, onRetry: () => void query.refetch() }
              : null
          }
          minWidthClass="min-w-[960px]"
          empty={
            <EmptyState icon={Trophy} title="暂无排行榜数据" description="当前模式与成绩类型还没有已通过的成绩。" />
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
