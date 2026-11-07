import { useQuery } from '@tanstack/react-query';
import { FilterX, Map as MapIcon, Search } from 'lucide-react';
import { useState } from 'react';
import { Link } from 'react-router';

import { EmptyState } from '@/public/components/common/empty-state';
import { MapPreview } from '@/public/components/common/map-preview';
import { ModeTabs } from '@/public/components/common/mode-tabs';
import { PageHeader } from '@/public/components/common/page-header';
import { TierBadge } from '@/public/components/common/tier-badge';
import { Badge } from '@/public/components/ui/badge';
import { Button } from '@/public/components/ui/button';
import { Callout } from '@/public/components/ui/callout';
import { Card } from '@/public/components/ui/card';
import { DataTable, type DataTableColumn } from '@/public/components/ui/data-table';
import { Input } from '@/public/components/ui/input';
import { Pagination } from '@/public/components/ui/pagination';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/public/components/ui/select';
import { useListParams } from '@/public/hooks/use-list-params';
import { formatNumber, formatSeconds } from '@/public/lib/format';
import { publicQueries } from '@/public/services/public-service';
import { usePageMeta } from '@/public/seo/use-page-meta';
import type { MapSummaryPayload } from '@cngokz/shared-types';

export function MapsPage() {
  usePageMeta({
    title: '地图列表',
    description: '浏览 KZ 地图、难度等级、课程数量、完成玩家与地图世界纪录。',
    path: '/maps',
  });

  const list = useListParams();
  const mode = list.read('mode', 'vanilla');
  const timeType = list.read('time_type', 'nub');
  const tier = list.read('tier', 'all');
  const sort = list.read('sort', 'completions');
  const search = list.read('search');
  const [draft, setDraft] = useState(search);

  const modes = useQuery(publicQueries.modes());
  const query = useQuery(
    publicQueries.maps({
      mode,
      time_type: timeType,
      tier: tier === 'all' ? undefined : Number(tier),
      sort,
      search: search || undefined,
      page: list.page,
      page_size: list.pageSize,
    }),
  );

  const columns: DataTableColumn<MapSummaryPayload>[] = [
    {
      id: 'map',
      header: '地图',
      cell: (row) => (
        <div className="flex items-center gap-2.5">
          <MapPreview name={row.name} url={row.preview_image_url} className="size-10" />
          <div className="min-w-0">
            <Link
              to={`/maps/${row.slug}`}
              className="block truncate font-mono text-[12px] font-medium hover:text-primary hover:underline"
            >
              {row.name}
            </Link>
            <p className="truncate text-[10px] text-muted-foreground">
              {row.workshop_id ? `Workshop ${row.workshop_id}` : 'Workshop —'}
            </p>
          </div>
        </div>
      ),
    },
    { id: 'tier', header: '难度', cell: (row) => <TierBadge tier={row.tier} /> },
    {
      id: 'courses',
      header: '课程',
      className: 'text-right',
      cell: (row) => <span className="tabular-nums">{row.courses}</span>,
    },
    {
      id: 'players',
      header: '完成玩家',
      className: 'text-right',
      cell: (row) => <span className="font-medium tabular-nums">{formatNumber(row.completed_players)}</span>,
    },
    {
      id: 'records',
      header: '成绩数',
      className: 'text-right',
      cell: (row) => <span className="text-muted-foreground tabular-nums">{formatNumber(row.records_count)}</span>,
    },
    {
      id: 'wr',
      header: '世界纪录',
      cell: (row) =>
        row.wr_time_seconds ? (
          <div className="flex items-center gap-2">
            <span className="font-mono text-xs tabular-nums">{formatSeconds(row.wr_time_seconds)}s</span>
            <span className="truncate text-[11px] text-muted-foreground">{row.wr_player_name ?? '—'}</span>
          </div>
        ) : (
          <span className="text-muted-foreground">—</span>
        ),
    },
    {
      id: 'author',
      header: '作者',
      cell: () => <span className="text-xs text-muted-foreground">未提供</span>,
    },
  ];

  const submit = (event: React.FormEvent) => {
    event.preventDefault();
    list.setFilter('search', draft.trim());
  };

  return (
    <div className="mx-auto w-full max-w-7xl space-y-5 px-4 py-8 sm:px-6 lg:px-8">
      <PageHeader
        title="地图列表"
        description="按模式与难度浏览 KZ 地图。缩略图与地图作者字段尚未在数据库中提供。"
        meta={query.data ? <span>共 {formatNumber(query.data.meta.total)} 张地图</span> : null}
      />

      <Callout
        variant="neutral"
        title="字段说明"
        description="地图封面与作者（mapper）目前没有对应的数据库字段，因此显示为占位；难度来自 course_tiers 的主线课程。"
      />

      <ModeTabs
        modes={modes.data?.items}
        value={mode}
        onChange={(value) => list.setFilter('mode', value, 'vanilla')}
        isLoading={modes.isLoading}
      />

      <Card className="overflow-hidden">
        <form
          onSubmit={submit}
          className="flex flex-col gap-2 border-b border-border/70 px-4 py-3 lg:flex-row lg:items-center lg:justify-between"
        >
          <div className="flex flex-wrap items-center gap-2">
            <div className="relative w-full sm:w-56">
              <Search className="pointer-events-none absolute top-1/2 left-2.5 size-3.5 -translate-y-1/2 text-muted-foreground" />
              <Input
                value={draft}
                onChange={(event) => setDraft(event.target.value)}
                placeholder="搜索地图名称…"
                className="pl-8"
                aria-label="搜索地图"
              />
            </div>
            <Button type="submit" size="sm" variant="secondary">
              搜索
            </Button>
            <Select value={timeType} onValueChange={(value) => list.setFilter('time_type', value, 'nub')}>
              <SelectTrigger className="w-28" aria-label="难度类型">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="nub">NUB 难度</SelectItem>
                <SelectItem value="pro">PRO 难度</SelectItem>
                <SelectItem value="all">全部类型</SelectItem>
              </SelectContent>
            </Select>
            <Select value={tier} onValueChange={(value) => list.setFilter('tier', value, 'all')}>
              <SelectTrigger className="w-28" aria-label="难度筛选">
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
            <Select value={sort} onValueChange={(value) => list.setFilter('sort', value, 'completions')}>
              <SelectTrigger className="w-32" aria-label="排序">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="completions">完成玩家</SelectItem>
                <SelectItem value="recent">最近添加</SelectItem>
                <SelectItem value="records">成绩数量</SelectItem>
                <SelectItem value="name">地图名称</SelectItem>
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
              disabled={!search && timeType === 'nub' && tier === 'all' && sort === 'completions' && list.page === 1}
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
              ? { title: '无法加载地图', description: query.error.message, onRetry: () => void query.refetch() }
              : null
          }
          minWidthClass="min-w-[1000px]"
          empty={
            <EmptyState
              icon={MapIcon}
              title={search ? '没有匹配的地图' : '暂无地图数据'}
              description={search ? '换一个地图名称再试。' : '数据库中还没有已上线地图，或该模式尚未关联地图。'}
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

      <div className="flex items-center gap-2 text-[11px] text-muted-foreground">
        <Badge variant="outline">提示</Badge>
        地图完成率与 WR 均按当前模式与难度类型计算。
      </div>
    </div>
  );
}
