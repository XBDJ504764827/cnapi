import { useQuery } from '@tanstack/react-query';
import { ExternalLink, Map as MapIcon } from 'lucide-react';
import { Link, useParams } from 'react-router';

import { EmptyState } from '@/public/components/common/empty-state';
import { ErrorState } from '@/public/components/common/error-state';
import { ModeTabs } from '@/public/components/common/mode-tabs';
import { MapPreview } from '@/public/components/common/map-preview';
import { PageHeader } from '@/public/components/common/page-header';
import { SectionHeader } from '@/public/components/common/section-header';
import { TierBadge } from '@/public/components/common/tier-badge';
import { RecordTable } from '@/public/components/records/record-table';
import { Badge } from '@/public/components/ui/badge';
import { Button } from '@/public/components/ui/button';
import { Callout } from '@/public/components/ui/callout';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/public/components/ui/card';
import { Pagination } from '@/public/components/ui/pagination';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/public/components/ui/select';
import { Skeleton } from '@/public/components/ui/skeleton';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/public/components/ui/table';
import { Tabs, TabsList, TabsTrigger } from '@/public/components/ui/tabs';
import { useListParams } from '@/public/hooks/use-list-params';
import { formatNumber, formatPercent, modeLabel } from '@/public/lib/format';
import { publicQueries } from '@/public/services/public-service';
import { usePageMeta } from '@/public/seo/use-page-meta';
import type { ApiClientError } from '@cngokz/api-client';

export function MapDetailPage() {
  const { slug = '' } = useParams<{ slug: string }>();
  const list = useListParams();
  const mode = list.read('mode', 'vanilla');
  const timeType = list.read('time_type', 'pro');
  const stage = list.read('stage', '0');

  const modes = useQuery(publicQueries.modes());
  const map = useQuery(publicQueries.map(slug, { mode, time_type: timeType }));
  const leaderboard = useQuery(
    publicQueries.mapLeaderboard(slug, {
      mode,
      time_type: timeType,
      stage: Number(stage),
      page: list.page,
      page_size: list.pageSize,
    }),
  );
  // 后端只提供前 100 名，超出部分不展示也不提供翻页入口
  const visibleTotal = Math.min(leaderboard.data?.meta.total ?? 0, 100);

  usePageMeta({
    title: map.data ? `${map.data.map.name} · 地图` : '地图详情',
    description: map.data
      ? `${map.data.map.name} 的 KZ 数据：难度 ${map.data.map.tier ?? '未评级'}、${map.data.map.courses} 个课程、${map.data.statistics.completed_players} 位完成玩家。`
      : '查看 KZ 地图课程、完成统计与地图排行榜。',
    path: `/maps/${slug}`,
  });

  const error = map.error as ApiClientError | null;
  if (error?.status === 404) {
    return (
      <div className="mx-auto w-full max-w-3xl px-4 py-16 sm:px-6">
        <EmptyState
          icon={MapIcon}
          title="未找到该地图"
          description="该地图不在公开数据库中，或尚未审核通过。"
          action={
            <Link to="/maps" className="text-sm text-primary hover:underline">
              返回地图列表
            </Link>
          }
        />
      </div>
    );
  }

  const detail = map.data;
  const selectedCourse = detail?.courses.find((course) => String(course.stage) === stage) ?? detail?.courses[0];

  return (
    <div className="mx-auto w-full max-w-7xl space-y-5 px-4 py-8 sm:px-6 lg:px-8">
      {map.isError && error ? (
        <ErrorState title="无法加载地图" description={error.message} onRetry={() => void map.refetch()} />
      ) : null}

      {map.isLoading ? (
        <div className="space-y-3">
          <Skeleton className="h-8 w-64" />
          <Skeleton className="h-4 w-96" />
          <Skeleton className="h-28 rounded-lg" />
          <Skeleton className="h-64 rounded-lg" />
        </div>
      ) : detail ? (
        <>
          <MapPreview
            name={detail.map.name}
            url={detail.map.preview_image_url}
            className="mb-4 h-44 w-full max-w-2xl sm:h-56"
            rounded="lg"
          />
          <PageHeader
            title={detail.map.name}
            description={detail.map.description ?? '该地图没有填写简介。'}
            meta={
              <>
                <TierBadge tier={detail.map.tier} />
                <span>{formatNumber(detail.map.courses)} 个课程</span>
                <span className="text-border-strong">·</span>
                <span>完成玩家 {formatNumber(detail.statistics.completed_players)}</span>
                {detail.map.workshop_id ? (
                  <>
                    <span className="text-border-strong">·</span>
                    <a
                      href={`https://steamcommunity.com/sharedfiles/filedetails/?id=${detail.map.workshop_id}`}
                      target="_blank"
                      rel="noreferrer"
                      className="inline-flex items-center gap-1 text-primary hover:underline"
                    >
                      Workshop
                      <ExternalLink className="size-3" />
                    </a>
                  </>
                ) : null}
              </>
            }
            actions={
              <Button size="sm" variant="outline" asChild>
                <Link to="/maps">返回地图列表</Link>
              </Button>
            }
          />

          <div className="grid grid-cols-2 gap-3 lg:grid-cols-4">
            <Card className="gap-1 p-4">
              <p className="text-[11px] text-muted-foreground">完成玩家</p>
              <p className="text-2xl font-semibold tabular-nums">{formatNumber(detail.statistics.completed_players)}</p>
              <p className="text-[10px] text-muted-foreground">
                PRO {detail.statistics.pro_completed_players} / NUB {detail.statistics.nub_completed_players}
              </p>
            </Card>
            <Card className="gap-1 p-4">
              <p className="text-[11px] text-muted-foreground">完成率</p>
              <p className="text-2xl font-semibold tabular-nums">{formatPercent(detail.statistics.completion_rate)}</p>
              <p className="text-[10px] text-muted-foreground">完成玩家 / 全部注册玩家</p>
            </Card>
            <Card className="gap-1 p-4">
              <p className="text-[11px] text-muted-foreground">成绩数量</p>
              <p className="text-2xl font-semibold tabular-nums">{formatNumber(detail.statistics.record_count)}</p>
              <p className="text-[10px] text-muted-foreground">当前模式</p>
            </Card>
            <Card className="gap-1 p-4">
              <p className="text-[11px] text-muted-foreground">支持模式</p>
              <p className="text-2xl font-semibold tabular-nums">{detail.supported_modes.length}</p>
              <p className="truncate text-[10px] text-muted-foreground">
                {detail.supported_modes.map((item) => item.display_name).join(' · ') || '—'}
              </p>
            </Card>
          </div>

          <Callout
            variant="neutral"
            title="成绩分布与字段说明"
            description="“成绩分布”按模式与 TP/PRO 汇总；作者、封面与逐秒时间分布尚未建模。完成统计以主线课程（stage 0）为准。"
          />

          <div className="grid gap-4 lg:grid-cols-3">
            <Card className="lg:col-span-2">
              <CardHeader>
                <CardTitle>课程</CardTitle>
                <CardDescription>主线与 Bonus 课程分别统计；分数未定义。</CardDescription>
              </CardHeader>
              <CardContent className="px-0">
                <Table className="min-w-[640px]">
                  <TableHeader>
                    <TableRow className="hover:bg-transparent">
                      <TableHead>课程</TableHead>
                      <TableHead>类型</TableHead>
                      <TableHead className="text-right">NUB / PRO 难度</TableHead>
                      <TableHead className="text-right">完成玩家</TableHead>
                      <TableHead className="text-right">成绩数</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {detail.courses.map((course) => (
                      <TableRow key={course.id}>
                        <TableCell className="font-mono text-xs">{course.name}</TableCell>
                        <TableCell>
                          <Badge variant="outline">{course.stage === 0 ? 'Main' : 'Bonus'}</Badge>
                        </TableCell>
                        <TableCell className="text-right">
                          <span className="inline-flex items-center gap-1.5">
                            <TierBadge tier={course.nub_tier} />
                            <span className="text-muted-foreground">/</span>
                            <TierBadge tier={course.pro_tier} />
                          </span>
                        </TableCell>
                        <TableCell className="text-right tabular-nums">
                          {formatNumber(course.completed_players)}
                        </TableCell>
                        <TableCell className="text-right tabular-nums">{formatNumber(course.records_count)}</TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>成绩分布</CardTitle>
                <CardDescription>按模式与成绩类型统计</CardDescription>
              </CardHeader>
              <CardContent className="space-y-3">
                <div>
                  <p className="mb-1.5 text-[11px] font-medium text-muted-foreground">按模式</p>
                  <ul className="space-y-1.5">
                    {detail.statistics.records_by_mode.map((item) => (
                      <li key={item.mode.code} className="flex items-center justify-between text-xs">
                        <span>{modeLabel(item.mode.code)}</span>
                        <span className="font-mono tabular-nums">{formatNumber(item.records)}</span>
                      </li>
                    ))}
                  </ul>
                </div>
                <div>
                  <p className="mb-1.5 text-[11px] font-medium text-muted-foreground">按成绩类型</p>
                  <ul className="space-y-1.5">
                    {detail.statistics.records_by_time_type.map((item) => (
                      <li key={item.time_type} className="flex items-center justify-between text-xs">
                        <span>{item.time_type.toUpperCase()}</span>
                        <span className="font-mono tabular-nums">{formatNumber(item.records)}</span>
                      </li>
                    ))}
                  </ul>
                </div>
              </CardContent>
            </Card>
          </div>

          <section className="space-y-3">
            <SectionHeader title="地图排行榜" description="按课程、模式与 TP/PRO 切换。成绩为跑图时长。" />
            <div className="flex flex-col gap-2 lg:flex-row lg:items-center lg:justify-between">
              <ModeTabs
                modes={modes.data?.items}
                value={mode}
                onChange={(value) => list.setFilter('mode', value, 'vanilla')}
                isLoading={modes.isLoading}
              />
              <div className="flex flex-wrap items-center gap-2">
                <Tabs value={timeType} onValueChange={(value) => list.setFilter('time_type', value, 'pro')}>
                  <TabsList>
                    <TabsTrigger value="pro">PRO</TabsTrigger>
                    <TabsTrigger value="nub">NUB</TabsTrigger>
                    <TabsTrigger value="all">全部</TabsTrigger>
                  </TabsList>
                </Tabs>
                <Select value={stage} onValueChange={(value) => list.setFilter('stage', value, '0')}>
                  <SelectTrigger className="w-32" aria-label="课程">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {detail.courses.map((course) => (
                      <SelectItem key={course.id} value={String(course.stage)}>
                        {course.stage === 0 ? 'Main' : `Bonus ${course.stage}`}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </div>

            <Card className="overflow-hidden">
              <div className="border-b border-border/70 px-4 py-2.5 text-xs text-muted-foreground">
                {selectedCourse ? `${selectedCourse.name} · ${modeLabel(mode)} · ${timeType.toUpperCase()}` : '—'}
              </div>
              <RecordTable
                records={leaderboard.data?.items}
                isLoading={leaderboard.isLoading}
                errorMessage={leaderboard.isError ? leaderboard.error.message : undefined}
                onRetry={() => void leaderboard.refetch()}
                showMap={false}
                showRank
                emptyTitle="暂无排行榜"
                emptyDescription="该课程 / 模式 / 成绩类型还没有成绩。"
              />
              <div className="border-t border-border/70 px-4 py-3">
                <div className="flex flex-wrap items-center justify-between gap-2">
                  <span className="text-xs text-muted-foreground">仅展示前 100 名</span>
                  <Pagination
                    page={leaderboard.data?.meta.page ?? list.page}
                    pageSize={leaderboard.data?.meta.page_size ?? list.pageSize}
                    total={visibleTotal}
                    onPageChange={list.setPage}
                    onPageSizeChange={list.setPageSize}
                  />
                </div>
              </div>
            </Card>
          </section>
        </>
      ) : null}
    </div>
  );
}
