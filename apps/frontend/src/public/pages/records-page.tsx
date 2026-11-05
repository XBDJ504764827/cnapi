import { useQuery } from '@tanstack/react-query';
import { FilterX, Search, Trophy } from 'lucide-react';
import { useState } from 'react';

import { PageHeader } from '@/public/components/common/page-header';
import { RecordDetailDialog } from '@/public/components/records/record-detail-dialog';
import { RecordTable } from '@/public/components/records/record-table';
import { Button } from '@/public/components/ui/button';
import { Callout } from '@/public/components/ui/callout';
import { Card } from '@/public/components/ui/card';
import { Input } from '@/public/components/ui/input';
import { Pagination } from '@/public/components/ui/pagination';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/public/components/ui/select';
import { useListParams } from '@/public/hooks/use-list-params';
import { publicQueries } from '@/public/services/public-service';
import { usePageMeta } from '@/public/seo/use-page-meta';
import type { RecordSummaryPayload } from '@cngokz/shared-types';

function dateToIso(value: string): string | undefined {
  if (!value) return undefined;
  const date = new Date(`${value}T00:00:00Z`);
  return Number.isNaN(date.getTime()) ? undefined : date.toISOString();
}

export function RecordsPage() {
  usePageMeta({
    title: '成绩查询',
    description: '搜索公开 KZ 成绩：玩家、地图、模式、TP/PRO、难度、世界纪录与上传时间。',
    path: '/records',
  });

  const list = useListParams();
  const player = list.read('player');
  const map = list.read('map');
  const mode = list.read('mode', 'all');
  const timeType = list.read('time_type', 'all');
  const tier = list.read('tier', 'all');
  const wr = list.read('wr', 'all');
  const sort = list.read('sort', 'uploaded');
  const from = list.read('from');
  const to = list.read('to');
  const [playerDraft, setPlayerDraft] = useState(player);
  const [mapDraft, setMapDraft] = useState(map);
  const [selected, setSelected] = useState<RecordSummaryPayload | null>(null);
  const hasFilters =
    Boolean(player) ||
    Boolean(map) ||
    mode !== 'all' ||
    timeType !== 'all' ||
    tier !== 'all' ||
    wr !== 'all' ||
    sort !== 'uploaded' ||
    Boolean(from) ||
    Boolean(to);

  const modes = useQuery(publicQueries.modes());
  const query = useQuery(
    publicQueries.records({
      player: player || undefined,
      map: map || undefined,
      mode: mode === 'all' ? undefined : mode,
      time_type: timeType,
      tier: tier === 'all' ? undefined : Number(tier),
      world_record: wr === 'all' ? undefined : wr === 'true',
      sort,
      from: dateToIso(from),
      to: to ? new Date(`${to}T23:59:59Z`).toISOString() : undefined,
      page: list.page,
      page_size: list.pageSize,
    }),
  );

  const applyFilters = (event: React.FormEvent) => {
    event.preventDefault();
    // Both text inputs must be written in one navigation; separate calls would
    // each start from the same stale search params and overwrite each other.
    list.setFilters({ player: playerDraft.trim(), map: mapDraft.trim() });
  };

  return (
    <div className="mx-auto w-full max-w-7xl space-y-5 px-4 py-8 sm:px-6 lg:px-8">
      <PageHeader
        title="成绩查询"
        description="所有公开已通过成绩。完成时间 = 跑图时长；上传时间 = 记录进入数据库的时间，两者不混用。"
        meta={query.data ? <span>共 {query.data.meta.total.toLocaleString('zh-CN')} 条成绩</span> : null}
      />

      <Callout
        variant="neutral"
        title="排序与排名"
        description="默认按上传时间倒序。表格中的全球排名基于同一地图课程、模式、Style 与 TP/PRO 维度的成绩时间。"
      />

      <Card className="overflow-hidden">
        <form onSubmit={applyFilters} className="space-y-2 border-b border-border/70 px-4 py-3">
          <div className="flex flex-wrap items-center gap-2">
            <div className="relative w-full sm:w-52">
              <Search className="pointer-events-none absolute top-1/2 left-2.5 size-3.5 -translate-y-1/2 text-muted-foreground" />
              <Input
                value={playerDraft}
                onChange={(event) => setPlayerDraft(event.target.value)}
                placeholder="玩家昵称 / SteamID64"
                className="pl-8"
                aria-label="玩家筛选"
              />
            </div>
            <div className="relative w-full sm:w-52">
              <Search className="pointer-events-none absolute top-1/2 left-2.5 size-3.5 -translate-y-1/2 text-muted-foreground" />
              <Input
                value={mapDraft}
                onChange={(event) => setMapDraft(event.target.value)}
                placeholder="地图名称"
                className="pl-8"
                aria-label="地图筛选"
              />
            </div>
            <Button type="submit" size="sm" variant="secondary">
              应用
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
            <Select value={timeType} onValueChange={(value) => list.setFilter('time_type', value, 'all')}>
              <SelectTrigger className="w-28" aria-label="TP / PRO">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">TP / PRO</SelectItem>
                <SelectItem value="pro">PRO</SelectItem>
                <SelectItem value="nub">NUB</SelectItem>
              </SelectContent>
            </Select>
            <Select value={tier} onValueChange={(value) => list.setFilter('tier', value, 'all')}>
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
            <Select value={wr} onValueChange={(value) => list.setFilter('wr', value, 'all')}>
              <SelectTrigger className="w-28" aria-label="世界纪录">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">WR 不限</SelectItem>
                <SelectItem value="true">仅 WR</SelectItem>
                <SelectItem value="false">非 WR</SelectItem>
              </SelectContent>
            </Select>
            <Select value={sort} onValueChange={(value) => list.setFilter('sort', value, 'uploaded')}>
              <SelectTrigger className="w-32" aria-label="排序">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="uploaded">上传时间</SelectItem>
                <SelectItem value="time">成绩时间</SelectItem>
                <SelectItem value="rank">全球排名</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <label className="flex items-center gap-1.5 text-[11px] text-muted-foreground">
              从
              <Input
                type="date"
                value={from}
                onChange={(event) => list.setFilter('from', event.target.value)}
                className="h-8 w-36"
                aria-label="上传起始日期"
              />
            </label>
            <label className="flex items-center gap-1.5 text-[11px] text-muted-foreground">
              至
              <Input
                type="date"
                value={to}
                onChange={(event) => list.setFilter('to', event.target.value)}
                className="h-8 w-36"
                aria-label="上传结束日期"
              />
            </label>
            <Button
              type="button"
              variant="ghost"
              size="sm"
              onClick={() => {
                setPlayerDraft('');
                setMapDraft('');
                list.reset();
              }}
              disabled={
                !player &&
                !map &&
                mode === 'all' &&
                timeType === 'all' &&
                tier === 'all' &&
                wr === 'all' &&
                sort === 'uploaded' &&
                !from &&
                !to &&
                list.page === 1
              }
            >
              <FilterX />
              重置
            </Button>
          </div>
        </form>

        <RecordTable
          records={query.data?.items}
          isLoading={query.isLoading}
          errorMessage={query.isError ? query.error.message : undefined}
          onRetry={() => void query.refetch()}
          onSelect={setSelected}
          showRank
          showPlayer
          showMap
          emptyTitle={hasFilters ? '没有匹配的成绩' : '暂无成绩'}
          emptyDescription={hasFilters ? '调整筛选条件后再试。' : '数据库中还没有已通过的公开成绩。'}
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
        <Trophy className="size-3.5" />
        WR 标记表示该成绩当前持有该维度世界纪录；历史 WR 状态尚未记录。
      </div>

      <RecordDetailDialog
        record={selected}
        onOpenChange={(open) => {
          if (!open) setSelected(null);
        }}
      />
    </div>
  );
}
