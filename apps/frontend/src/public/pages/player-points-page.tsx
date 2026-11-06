import { useQuery } from '@tanstack/react-query';
import { useState } from 'react';
import { useParams } from 'react-router';

import { EmptyState } from '@/public/components/common/empty-state';
import { ErrorState } from '@/public/components/common/error-state';
import { PageHeader } from '@/public/components/common/page-header';
import { FilterPointsTable } from '@/public/components/points/filter-points-table';
import { ModePointsCards } from '@/public/components/points/mode-points-cards';
import { Button } from '@/public/components/ui/button';
import { Card } from '@/public/components/ui/card';
import { Input } from '@/public/components/ui/input';
import { Pagination } from '@/public/components/ui/pagination';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/public/components/ui/select';
import { Skeleton } from '@/public/components/ui/skeleton';
import { pointsQueries } from '@/public/services/points-service';
import { usePageMeta } from '@/public/seo/use-page-meta';
export function PlayerPointsPage() {
  const { steamid = '' } = useParams<{ steamid: string }>();
  usePageMeta({
    title: '玩家积分明细',
    description: '查看玩家在 KZT、SKZ、VNL 三种模式下的独立总分、PRO/NUB 积分和每个 Filter 的 PB 积分。',
    path: `/players/${steamid}/points`,
  });

  const [mode, setMode] = useState('all');
  const [recordType, setRecordType] = useState('all');
  const [tier, setTier] = useState('all');
  const [sort, setSort] = useState('points');
  const [search, setSearch] = useState('');
  const [draft, setDraft] = useState('');
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);

  const query = {
    mode: mode === 'all' ? undefined : mode,
    record_type: recordType === 'all' ? undefined : recordType,
    tier: tier === 'all' ? undefined : Number(tier),
    sort,
    map: search || undefined,
    page,
    page_size: pageSize,
  };
  const points = useQuery(pointsQueries.player(steamid));
  const filters = useQuery(pointsQueries.playerFilters(steamid, query));

  return (
    <div className="mx-auto w-full max-w-7xl space-y-5 px-4 py-8 sm:px-6 lg:px-8">
      <PageHeader
        title="玩家积分明细"
        description="三种模式总分独立；每个 Filter 由玩家 PB、当前 WR 和当前排名动态计算积分。"
        meta={<span className="font-mono">{steamid}</span>}
      />

      {points.isError ? (
        <ErrorState title="无法加载玩家积分" description={points.error.message} onRetry={() => void points.refetch()} />
      ) : points.isLoading ? (
        <div className="grid gap-3 sm:grid-cols-3">
          {Array.from({ length: 3 }).map((_, index) => (
            <Skeleton key={index} className="h-32 rounded-lg" />
          ))}
        </div>
      ) : points.data ? (
        <ModePointsCards points={points.data} />
      ) : (
        <EmptyState compact title="暂无积分" description="未找到该玩家的积分记录。" />
      )}

      <Card className="overflow-hidden">
        <form
          className="flex flex-wrap items-center gap-2 border-b border-border/70 px-4 py-3"
          onSubmit={(event) => {
            event.preventDefault();
            setSearch(draft.trim());
            setPage(1);
          }}
        >
          <Input
            value={draft}
            onChange={(event) => setDraft(event.target.value)}
            placeholder="搜索地图…"
            className="w-full sm:w-48"
            aria-label="搜索地图"
          />
          <Button type="submit" size="sm" variant="secondary">
            搜索
          </Button>
          <Select
            value={mode}
            onValueChange={(value) => {
              setMode(value);
              setPage(1);
            }}
          >
            <SelectTrigger className="w-28" aria-label="模式">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">全部模式</SelectItem>
              <SelectItem value="KZT">KZT</SelectItem>
              <SelectItem value="SKZ">SKZ</SelectItem>
              <SelectItem value="VNL">VNL</SelectItem>
            </SelectContent>
          </Select>
          <Select
            value={recordType}
            onValueChange={(value) => {
              setRecordType(value);
              setPage(1);
            }}
          >
            <SelectTrigger className="w-24" aria-label="类型">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">全部类型</SelectItem>
              <SelectItem value="pro">PRO</SelectItem>
              <SelectItem value="nub">NUB</SelectItem>
            </SelectContent>
          </Select>
          <Select
            value={tier}
            onValueChange={(value) => {
              setTier(value);
              setPage(1);
            }}
          >
            <SelectTrigger className="w-24" aria-label="Tier">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">全部 Tier</SelectItem>
              {Array.from({ length: 8 }).map((_, index) => (
                <SelectItem key={index + 1} value={String(index + 1)}>
                  T{index + 1}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <Select
            value={sort}
            onValueChange={(value) => {
              setSort(value);
              setPage(1);
            }}
          >
            <SelectTrigger className="w-28" aria-label="排序">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="points">积分排序</SelectItem>
              <SelectItem value="rank">排名排序</SelectItem>
              <SelectItem value="tier">Tier 排序</SelectItem>
              <SelectItem value="time">成绩排序</SelectItem>
            </SelectContent>
          </Select>
        </form>

        <FilterPointsTable
          items={filters.data?.items}
          isLoading={filters.isLoading}
          errorMessage={filters.isError ? filters.error.message : undefined}
          onRetry={() => void filters.refetch()}
        />
        <div className="border-t border-border/70 px-4 py-3">
          <Pagination
            page={filters.data?.meta.page ?? page}
            pageSize={filters.data?.meta.page_size ?? pageSize}
            total={filters.data?.meta.total ?? 0}
            onPageChange={setPage}
            onPageSizeChange={(size) => {
              setPageSize(size);
              setPage(1);
            }}
          />
        </div>
      </Card>
    </div>
  );
}
