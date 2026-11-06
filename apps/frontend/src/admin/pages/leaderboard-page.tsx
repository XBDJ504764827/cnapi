import { useQuery } from '@tanstack/react-query';
import { Trophy } from 'lucide-react';
import { useState } from 'react';

import { EmptyState } from '@/admin/components/common/empty-state';
import { ListPanel } from '@/admin/components/common/list-panel';
import { PageHeader } from '@/admin/components/common/page-header';
import { PlayerCell } from '@/admin/components/common/player-cell';
import { Badge } from '@/admin/components/ui/badge';
import { DataTable, type DataTableColumn } from '@/admin/components/ui/data-table';
import { Pagination } from '@/admin/components/ui/pagination';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/admin/components/ui/select';
import { Tabs, TabsList, TabsTrigger } from '@/admin/components/ui/tabs';
import { useTableState } from '@/admin/hooks/use-table-state';
import { formatDateTime, formatNumber } from '@/admin/lib/format';
import { timeTypes } from '@/admin/lib/kz-constants';
import { api } from '@/admin/lib/api';
import { directoryQueries } from '@/admin/services/directory-service';
import type { components } from '@cngokz/shared-types';

type Entry = components['schemas']['LeaderboardEntry'];

const columns: DataTableColumn<Entry>[] = [
  {
    id: 'rank',
    header: '#',
    cell: (row) => (
      <span className="font-mono text-xs font-semibold tabular-nums">
        {row.rank <= 3 ? <Badge variant={row.rank === 1 ? 'success' : 'outline'}>{row.rank}</Badge> : row.rank}
      </span>
    ),
  },
  {
    id: 'player',
    header: '玩家',
    cell: (row) => <PlayerCell steamId64={row.steam_id64} displayName={row.display_name} avatarUrl={row.avatar_url} />,
  },
  {
    id: 'completed',
    header: '完成地图',
    cell: (row) => <span className="tabular-nums">{formatNumber(row.completed_maps)}</span>,
  },
  {
    id: 'wr',
    header: '世界纪录',
    cell: (row) => <span className="tabular-nums">{formatNumber(row.wr_count)}</span>,
  },
  {
    id: 'records',
    header: '成绩',
    cell: (row) => <span className="tabular-nums">{formatNumber(row.total_records)}</span>,
  },
  {
    id: 'country',
    header: '地区',
    cell: (row) => <span className="font-mono text-xs text-muted-foreground">{row.country ?? '—'}</span>,
  },
  {
    id: 'last',
    header: '最后活跃',
    cell: (row) => (
      <span className="text-xs text-muted-foreground">
        {row.last_activity_at ? formatDateTime(row.last_activity_at) : '—'}
      </span>
    ),
  },
];

/** 全局排行榜：读取 `/api/v1/public/leaderboards`。 */
export function LeaderboardPage() {
  const table = useTableState({ initialPageSize: 25 });
  const [mode, setMode] = useState('vanilla');
  const [timeType, setTimeType] = useState('all');

  const modes = useQuery({
    queryKey: ['admin', 'modes'],
    queryFn: () => api.public.getModes(),
    staleTime: 5 * 60_000,
  });

  const query = useQuery(
    directoryQueries.leaderboard({
      mode,
      time_type: timeType === 'all' ? undefined : timeType,
      page: table.page,
      page_size: table.pageSize,
    }),
  );

  const items = query.data?.items ?? [];
  const total = query.data?.meta.total ?? 0;
  const modeOptions = modes.data?.items ?? [];

  return (
    <div className="space-y-5">
      <PageHeader
        title="排行榜"
        description="按完成地图数排序的全局榜单，并列时依次比较世界纪录数与成绩总数。"
        meta={
          query.data ? (
            <span>
              统计口径 {query.data.ranking_metric === 'completed_maps' ? '完成地图数' : query.data.ranking_metric} · 共{' '}
              {formatNumber(total)} 名玩家
            </span>
          ) : null
        }
      />

      <div className="flex flex-wrap items-center gap-2">
        {modeOptions.length > 0 ? (
          <Tabs
            value={mode}
            onValueChange={(value) => {
              setMode(value);
              table.setPage(1);
            }}
          >
            <TabsList>
              {modeOptions.map((item) => (
                <TabsTrigger key={item.code} value={item.code}>
                  {item.display_name}
                </TabsTrigger>
              ))}
            </TabsList>
          </Tabs>
        ) : null}

        <Select
          value={timeType}
          onValueChange={(value) => {
            setTimeType(value);
            table.setPage(1);
          }}
        >
          <SelectTrigger className="w-32" aria-label="时间类型">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">全部类型</SelectItem>
            {timeTypes.map((option) => (
              <SelectItem key={option.value} value={option.value}>
                {option.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>

      <ListPanel
        footer={
          <Pagination
            page={table.page}
            pageSize={table.pageSize}
            total={total}
            onPageChange={table.setPage}
            onPageSizeChange={table.setPageSize}
            pageSizeOptions={[25, 50, 100]}
          />
        }
      >
        <DataTable
          columns={columns}
          rows={items}
          getRowKey={(row) => row.account_id}
          isLoading={query.isLoading}
          error={
            query.isError
              ? {
                  title: '排行榜读取失败',
                  description: (query.error as Error).message,
                  onRetry: () => void query.refetch(),
                }
              : null
          }
          empty={<EmptyState icon={Trophy} title="暂无排名数据" description="该模式还没有已通过的成绩。" />}
        />
      </ListPanel>
    </div>
  );
}
