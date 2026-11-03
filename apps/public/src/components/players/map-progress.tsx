import { useQuery } from '@tanstack/react-query';
import { FilterX, Map as MapIcon, Search } from 'lucide-react';
import { useState } from 'react';
import { Link } from 'react-router';

import { EmptyState } from '@/components/common/empty-state';
import { TierBadge } from '@/components/common/tier-badge';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card } from '@/components/ui/card';
import { DataTable, type DataTableColumn } from '@/components/ui/data-table';
import { Input } from '@/components/ui/input';
import { Pagination } from '@/components/ui/pagination';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { useListParams } from '@/hooks/use-list-params';
import { formatDateTime, formatNumber, formatSeconds, timeTypeLabel } from '@/lib/format';
import { publicQueries } from '@/services/public-service';
import type { PlayerMapProgressPayload } from '@cngokz/shared-types';

export function PlayerMapProgress({ steamid, mode, timeType }: { steamid: string; mode: string; timeType: string }) {
  const list = useListParams({ pageKey: 'm_page', pageSizeKey: 'm_page_size' });
  const status = list.read('m_status', 'all');
  const tier = list.read('m_tier', 'all');
  const sort = list.read('m_sort', 'tier');
  const search = list.read('m_search');
  const [draft, setDraft] = useState(search);

  const query = useQuery(
    publicQueries.playerMaps(steamid, {
      mode,
      time_type: timeType === 'all' ? 'all' : timeType,
      status,
      tier: tier === 'all' ? undefined : Number(tier),
      search: search || undefined,
      sort,
      page: list.page,
      page_size: list.pageSize,
    }),
  );

  const columns: DataTableColumn<PlayerMapProgressPayload>[] = [
    {
      id: 'map',
      header: '地图',
      cell: (row) => (
        <div className="flex items-center gap-2.5">
          <span className="flex size-7 items-center justify-center rounded border border-border bg-surface-elevated text-muted-foreground">
            <MapIcon className="size-3.5" aria-hidden="true" />
          </span>
          <div className="min-w-0">
            <Link
              to={`/maps/${row.slug}`}
              className="block truncate font-mono text-[12px] hover:text-primary hover:underline"
            >
              {row.name}
            </Link>
            <p className="text-[10px] text-muted-foreground">
              Bonus {formatNumber(row.bonus_completed)}/{formatNumber(row.bonus_total)}
            </p>
          </div>
        </div>
      ),
    },
    { id: 'tier', header: '难度', cell: (row) => <TierBadge tier={row.tier} /> },
    {
      id: 'status',
      header: '状态',
      cell: (row) => (
        <Badge variant={row.completed ? 'success' : 'outline'}>{row.completed ? '已完成' : '未完成'}</Badge>
      ),
    },
    {
      id: 'best',
      header: '最佳成绩',
      className: 'text-right',
      cell: (row) =>
        row.best_time_seconds ? (
          <span className="font-mono text-[13px] tabular-nums">
            {formatSeconds(row.best_time_seconds)}s
            <span className="ml-1 text-[10px] text-muted-foreground">
              {timeTypeLabel(row.best_time_type as 'pro' | 'nub' | null)}
            </span>
          </span>
        ) : (
          <span className="text-muted-foreground">—</span>
        ),
    },
    {
      id: 'rank',
      header: '排名',
      className: 'text-right',
      cell: (row) => (
        <span className="font-mono text-xs tabular-nums">{row.best_rank ? `#${row.best_rank}` : '—'}</span>
      ),
    },
    {
      id: 'points',
      header: '积分',
      className: 'text-right',
      cell: () => <span className="text-muted-foreground">—</span>,
    },
    {
      id: 'last',
      header: '最近成绩上传',
      cell: (row) => <span className="text-[11px] text-muted-foreground">{formatDateTime(row.last_record_at)}</span>,
    },
  ];

  const submit = (event: React.FormEvent) => {
    event.preventDefault();
    list.setFilter('m_search', draft.trim());
  };

  return (
    <Card className="overflow-hidden">
      <div className="space-y-3 border-b border-border/70 px-4 py-3">
        <div className="flex flex-col gap-2 lg:flex-row lg:items-center lg:justify-between">
          <Tabs value={status} onValueChange={(value) => list.setFilter('m_status', value, 'all')}>
            <TabsList>
              <TabsTrigger value="all">全部地图</TabsTrigger>
              <TabsTrigger value="completed">已完成</TabsTrigger>
              <TabsTrigger value="uncompleted">未完成</TabsTrigger>
            </TabsList>
          </Tabs>
          <form onSubmit={submit} className="flex flex-wrap items-center gap-2">
            <div className="relative w-full sm:w-52">
              <Search className="pointer-events-none absolute top-1/2 left-2.5 size-3.5 -translate-y-1/2 text-muted-foreground" />
              <Input
                value={draft}
                onChange={(event) => setDraft(event.target.value)}
                placeholder="搜索地图…"
                className="pl-8"
                aria-label="搜索地图"
              />
            </div>
            <Select value={tier} onValueChange={(value) => list.setFilter('m_tier', value, 'all')}>
              <SelectTrigger className="w-28" aria-label="难度">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">全部难度</SelectItem>
                <SelectItem value="0">未评级</SelectItem>
                {Array.from({ length: 10 }).map((_, index) => (
                  <SelectItem key={index + 1} value={String(index + 1)}>
                    T{index + 1}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Select value={sort} onValueChange={(value) => list.setFilter('m_sort', value, 'tier')}>
              <SelectTrigger className="w-28" aria-label="排序">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="tier">难度排序</SelectItem>
                <SelectItem value="name">名称排序</SelectItem>
                <SelectItem value="time">成绩排序</SelectItem>
                <SelectItem value="recent">最近排序</SelectItem>
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
              disabled={!search && status === 'all' && tier === 'all' && sort === 'tier' && list.page === 1}
            >
              <FilterX />
              重置
            </Button>
          </form>
        </div>
        <p className="text-[11px] text-muted-foreground">
          完成定义：该模式下主线课程（stage 0）存在已通过成绩；Bonus 完成数单独统计。
        </p>
      </div>

      <DataTable
        columns={columns}
        rows={query.data?.items ?? []}
        getRowKey={(row) => row.map_id}
        isLoading={query.isLoading}
        error={
          query.isError
            ? { title: '无法加载地图进度', description: query.error.message, onRetry: () => void query.refetch() }
            : null
        }
        minWidthClass="min-w-[980px]"
        empty={
          <EmptyState
            icon={MapIcon}
            title={
              status === 'uncompleted' ? '没有未完成地图' : status === 'completed' ? '还没有完成地图' : '暂无地图数据'
            }
            description={
              status === 'uncompleted'
                ? '该玩家已完成当前模式下所有已上线地图。'
                : status === 'completed'
                  ? '该玩家在该模式下还没有主线的已通过成绩。'
                  : '地图库为空，或该模式还没有关联地图。'
            }
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
  );
}
