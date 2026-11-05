import { useQuery } from '@tanstack/react-query';
import { useState } from 'react';
import { useParams } from 'react-router';

import { EmptyState } from '@/admin/components/common/empty-state';
import { ErrorState } from '@/admin/components/common/error-state';
import { PageHeader } from '@/admin/components/common/page-header';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/admin/components/ui/card';
import { Pagination } from '@/admin/components/ui/pagination';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/admin/components/ui/select';
import { Skeleton } from '@/admin/components/ui/skeleton';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/admin/components/ui/table';
import { formatDateTime, formatNumber } from '@/admin/lib/format';
import { pointsQueries } from '@/admin/services/points-service';

export function PlayerPointsPage() {
  const { playerId = '' } = useParams<{ playerId: string }>();
  const [mode, setMode] = useState('all');
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);
  const query = pointsQueries.playerFilters(playerId, {
    mode: mode === 'all' ? undefined : mode,
    page,
    page_size: pageSize,
  });
  const filters = useQuery(query);
  const totals = useQuery(pointsQueries.player(playerId));

  return (
    <div className="space-y-5">
      <PageHeader
        title="玩家积分明细"
        description="按 SteamID64 查看每个 Filter 的 PB、WR、排名与积分。"
        meta={<span className="font-mono">{playerId}</span>}
      />

      <Card>
        <CardHeader>
          <CardTitle>三模式总分</CardTitle>
          <CardDescription>总分 = PRO + NUB，三种模式互不混合</CardDescription>
        </CardHeader>
        <CardContent className="grid gap-3 sm:grid-cols-3">
          {totals.isError ? (
            <ErrorState title="无法读取总分" description={totals.error.message} onRetry={() => void totals.refetch()} />
          ) : totals.isLoading ? (
            <>
              <Skeleton className="h-16 rounded-md" />
              <Skeleton className="h-16 rounded-md" />
              <Skeleton className="h-16 rounded-md" />
            </>
          ) : totals.data ? (
            totals.data.modes.map((mode) => (
              <div key={mode.code} className="rounded-md border border-border p-3">
                <p className="text-xs font-medium">{mode.code}</p>
                <p className="text-xl font-semibold tabular-nums">{formatNumber(mode.total_points)}</p>
                <p className="text-[11px] text-muted-foreground">
                  PRO {formatNumber(mode.pro_points)} · NUB {formatNumber(mode.nub_points)}
                </p>
              </div>
            ))
          ) : null}
        </CardContent>
      </Card>

      <Card className="overflow-hidden">
        <div className="flex flex-wrap items-center gap-2 border-b border-border/70 px-4 py-3">
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
        </div>
        {filters.isError ? (
          <div className="p-4">
            <ErrorState
              title="无法读取积分明细"
              description={filters.error.message}
              onRetry={() => void filters.refetch()}
            />
          </div>
        ) : filters.isLoading ? (
          <div className="p-4">
            <Skeleton className="h-40" />
          </div>
        ) : filters.data?.items.length ? (
          <Table className="min-w-[900px]">
            <TableHeader>
              <TableRow className="hover:bg-transparent">
                <TableHead>地图</TableHead>
                <TableHead>Course</TableHead>
                <TableHead>模式</TableHead>
                <TableHead>类型</TableHead>
                <TableHead>Tier</TableHead>
                <TableHead className="text-right">PB</TableHead>
                <TableHead className="text-right">排名</TableHead>
                <TableHead className="text-right">积分</TableHead>
                <TableHead>计算时间</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {filters.data.items.map((item) => (
                <TableRow key={item.filter_id}>
                  <TableCell className="font-mono text-xs">{item.map_name}</TableCell>
                  <TableCell className="text-xs">{item.course_name}</TableCell>
                  <TableCell className="text-xs">{item.mode.code}</TableCell>
                  <TableCell className="text-xs">{item.record_type.toUpperCase()}</TableCell>
                  <TableCell className="font-mono text-xs">T{item.tier}</TableCell>
                  <TableCell className="text-right font-mono text-xs">
                    {formatNumber(Number(item.pb_time_seconds))}s
                  </TableCell>
                  <TableCell className="text-right font-mono text-xs">#{item.rank}</TableCell>
                  <TableCell className="text-right font-medium tabular-nums">{formatNumber(item.points)}</TableCell>
                  <TableCell className="text-xs text-muted-foreground">{formatDateTime(item.calculated_at)}</TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        ) : (
          <div className="p-4">
            <EmptyState compact title="暂无积分明细" description="该玩家暂无已重算的 Filter。" />
          </div>
        )}
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
