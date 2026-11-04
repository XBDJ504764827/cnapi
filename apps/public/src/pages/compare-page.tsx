import { useQuery } from '@tanstack/react-query';
import { ArrowLeftRight, FilterX, Scale } from 'lucide-react';

import { EmptyState } from '@/components/common/empty-state';
import { ErrorState } from '@/components/common/error-state';
import { PageHeader } from '@/components/common/page-header';
import { PlayerAvatar } from '@/components/common/player-avatar';
import { PlayerPicker } from '@/components/players/player-picker';
import { TierBadge } from '@/components/common/tier-badge';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Callout } from '@/components/ui/callout';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Pagination } from '@/components/ui/pagination';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Skeleton } from '@/components/ui/skeleton';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { useListParams } from '@/hooks/use-list-params';
import { formatNumber, formatPercent, formatSeconds } from '@/lib/format';
import { publicQueries } from '@/services/public-service';
import { usePageMeta } from '@/seo/use-page-meta';

export function ComparePage() {
  usePageMeta({
    title: '玩家比较',
    description: '比较两位 KZ 玩家的完成地图、世界纪录、各模式/难度进度与共同地图成绩。',
    path: '/compare',
  });

  const list = useListParams();
  const left = list.read('left');
  const right = list.read('right');
  const mode = list.read('mode', 'vanilla');
  const timeType = list.read('time_type', 'all');
  const filter = list.read('filter', 'all');

  const modes = useQuery(publicQueries.modes());
  const topPlayers = useQuery({
    ...publicQueries.players({ sort: 'completion', page_size: 12 }),
    enabled: !left || !right,
  });
  const compare = useQuery({
    ...publicQueries.compare({
      left: left || '',
      right: right || '',
      mode,
      time_type: timeType,
      filter,
      page: list.page,
      page_size: list.pageSize,
    }),
    enabled: left.length > 0 && right.length > 0 && left !== right,
  });

  const setSide = (side: 'left' | 'right', value: string | null) => list.setFilters({ [side]: value ?? undefined });
  const swap = () => list.setFilters({ left: right, right: left });

  const summaryRows = compare.data
    ? [
        {
          label: '全球完成排名',
          a: compare.data.left.overall.global_rank ? `#${compare.data.left.overall.global_rank}` : '—',
          b: compare.data.right.overall.global_rank ? `#${compare.data.right.overall.global_rank}` : '—',
        },
        {
          label: '完成地图',
          a: formatNumber(compare.data.left.overall.completed_maps),
          b: formatNumber(compare.data.right.overall.completed_maps),
        },
        {
          label: '地图完成率',
          a: formatPercent(compare.data.left.overall.completion_rate),
          b: formatPercent(compare.data.right.overall.completion_rate),
        },
        {
          label: '世界纪录',
          a: formatNumber(compare.data.left.overall.wr_count),
          b: formatNumber(compare.data.right.overall.wr_count),
        },
        {
          label: 'PRO 成绩',
          a: formatNumber(compare.data.left.overall.pro_count),
          b: formatNumber(compare.data.right.overall.pro_count),
        },
        {
          label: 'TP (NUB) 成绩',
          a: formatNumber(compare.data.left.overall.tp_count),
          b: formatNumber(compare.data.right.overall.tp_count),
        },
        {
          label: '总成绩数',
          a: formatNumber(compare.data.left.overall.total_records),
          b: formatNumber(compare.data.right.overall.total_records),
        },
        { label: '积分 / Rating', a: '—', b: '—' },
      ]
    : [];

  return (
    <div className="mx-auto w-full max-w-7xl space-y-5 px-4 py-8 sm:px-6 lg:px-8">
      <PageHeader
        title="玩家比较"
        description="选择两位玩家，对比完成度、各模式成绩与同一地图上的最佳成绩。积分比较将在后续版本接入独立积分接口。"
      />

      <div className="grid gap-3 md:grid-cols-[1fr_auto_1fr] md:items-center">
        <PlayerPicker
          label="玩家 A"
          value={left}
          onChange={(value) => setSide('left', value)}
          displayName={compare.data?.left.profile.display_name}
        />
        <Button variant="outline" size="icon-sm" onClick={swap} aria-label="交换玩家" className="mx-auto">
          <ArrowLeftRight />
        </Button>
        <PlayerPicker
          label="玩家 B"
          value={right}
          onChange={(value) => setSide('right', value)}
          displayName={compare.data?.right.profile.display_name}
        />
      </div>

      {!left || !right ? (
        <Card>
          <CardHeader>
            <CardTitle className="text-sm">从排行榜快速选择</CardTitle>
          </CardHeader>
          <CardContent>
            {topPlayers.isLoading ? (
              <Skeleton className="h-20" />
            ) : topPlayers.data && topPlayers.data.items.length > 0 ? (
              <div className="flex flex-wrap gap-2">
                {topPlayers.data.items.map((player) => (
                  <Button
                    key={player.account_id}
                    variant="outline"
                    size="sm"
                    onClick={() => setSide(left ? 'right' : 'left', player.steam_id64)}
                  >
                    <PlayerAvatar
                      steamId64={player.steam_id64}
                      displayName={player.display_name}
                      size="sm"
                      linked={false}
                    />
                    {player.display_name}
                  </Button>
                ))}
              </div>
            ) : (
              <EmptyState compact icon={Scale} title="暂无玩家可比较" description="数据库中还没有公开玩家。" />
            )}
          </CardContent>
        </Card>
      ) : left === right ? (
        <Callout
          variant="warning"
          title="请选择两位不同的玩家"
          description="玩家 A 与玩家 B 不能是同一个 SteamID64。"
        />
      ) : compare.isError ? (
        <ErrorState title="无法比较玩家" description={compare.error.message} onRetry={() => void compare.refetch()} />
      ) : compare.isLoading ? (
        <div className="space-y-3">
          <Skeleton className="h-40 rounded-lg" />
          <Skeleton className="h-64 rounded-lg" />
        </div>
      ) : compare.data ? (
        <>
          <Card>
            <CardHeader>
              <CardTitle className="text-sm">综合数据</CardTitle>
            </CardHeader>
            <CardContent className="px-0">
              <Table className="min-w-[560px]">
                <TableHeader>
                  <TableRow className="hover:bg-transparent">
                    <TableHead>项目</TableHead>
                    <TableHead className="text-right">{compare.data.left.profile.display_name}</TableHead>
                    <TableHead className="text-right">{compare.data.right.profile.display_name}</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {summaryRows.map((row) => (
                    <TableRow key={row.label}>
                      <TableCell className="text-xs text-muted-foreground">{row.label}</TableCell>
                      <TableCell className="text-right font-mono text-xs tabular-nums">{row.a}</TableCell>
                      <TableCell className="text-right font-mono text-xs tabular-nums">{row.b}</TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </CardContent>
          </Card>

          <div className="grid gap-3 lg:grid-cols-2">
            <Card>
              <CardHeader>
                <CardTitle className="text-sm">各模式完成情况</CardTitle>
              </CardHeader>
              <CardContent className="space-y-2">
                {compare.data.left.modes.map((leftMode) => {
                  const rightMode = compare.data?.right.modes.find((item) => item.mode.code === leftMode.mode.code);
                  return (
                    <div
                      key={leftMode.mode.code}
                      className="grid grid-cols-[1fr_auto_auto] items-center gap-3 rounded-md border border-border/70 px-3 py-2 text-xs"
                    >
                      <span>{leftMode.mode.display_name}</span>
                      <span className="font-mono tabular-nums">
                        {leftMode.completed_maps} <span className="text-muted-foreground">/</span>{' '}
                        {rightMode?.completed_maps ?? 0}
                      </span>
                      <span className="text-muted-foreground">
                        {leftMode.total_records} / {rightMode?.total_records ?? 0} 成绩
                      </span>
                    </div>
                  );
                })}
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle className="text-sm">各难度完成数量</CardTitle>
              </CardHeader>
              <CardContent className="space-y-2">
                {compare.data.tiers.length === 0 ? (
                  <EmptyState compact title="暂无难度数据" description="该模式缺少 course_tiers 记录。" />
                ) : (
                  compare.data.tiers.map((tier) => (
                    <div key={tier.tier ?? 'unrated'} className="grid grid-cols-[auto_1fr] items-center gap-3 text-xs">
                      <TierBadge tier={tier.tier} />
                      <div className="flex items-center justify-between gap-3">
                        <span className="font-mono tabular-nums">
                          {tier.left_completed} / {tier.total_maps}
                        </span>
                        <span className="font-mono text-muted-foreground tabular-nums">
                          {tier.right_completed} / {tier.total_maps}
                        </span>
                      </div>
                    </div>
                  ))
                )}
              </CardContent>
            </Card>
          </div>

          <Card className="overflow-hidden">
            <div className="flex flex-col gap-2 border-b border-border/70 px-4 py-3 lg:flex-row lg:items-center lg:justify-between">
              <div className="flex flex-wrap items-center gap-2">
                <Select value={mode} onValueChange={(value) => list.setFilter('mode', value, 'vanilla')}>
                  <SelectTrigger className="w-32" aria-label="模式">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {(modes.data?.items ?? []).map((item) => (
                      <SelectItem key={item.code} value={item.code}>
                        {item.display_name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                <Tabs value={timeType} onValueChange={(value) => list.setFilter('time_type', value, 'all')}>
                  <TabsList>
                    <TabsTrigger value="all">全部</TabsTrigger>
                    <TabsTrigger value="pro">PRO</TabsTrigger>
                    <TabsTrigger value="nub">NUB</TabsTrigger>
                  </TabsList>
                </Tabs>
                <Select value={filter} onValueChange={(value) => list.setFilter('filter', value, 'all')}>
                  <SelectTrigger className="w-40" aria-label="地图筛选">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">全部地图</SelectItem>
                    <SelectItem value="both">双方都完成</SelectItem>
                    <SelectItem value="left_faster">A 更快</SelectItem>
                    <SelectItem value="right_faster">B 更快</SelectItem>
                    <SelectItem value="left_only">仅 A 完成</SelectItem>
                    <SelectItem value="right_only">仅 B 完成</SelectItem>
                    <SelectItem value="neither">双方未完成</SelectItem>
                  </SelectContent>
                </Select>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => list.setFilters({ filter: undefined })}
                  disabled={filter === 'all'}
                >
                  <FilterX />
                  重置筛选
                </Button>
              </div>
              <p className="text-[11px] text-muted-foreground">
                地图列表按难度与名称排序；成绩为跑图时长，差值为 A − B。
              </p>
            </div>

            <Table className="min-w-[900px]">
              <TableHeader>
                <TableRow className="hover:bg-transparent">
                  <TableHead>地图</TableHead>
                  <TableHead>难度</TableHead>
                  <TableHead className="text-right">A 成绩</TableHead>
                  <TableHead className="text-right">B 成绩</TableHead>
                  <TableHead className="text-right">时间差</TableHead>
                  <TableHead>状态</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {compare.data.maps.length === 0 ? (
                  <TableRow className="hover:bg-transparent">
                    <TableCell colSpan={6} className="p-4">
                      <EmptyState compact icon={Scale} title="没有匹配的地图" description="调整筛选条件后再试。" />
                    </TableCell>
                  </TableRow>
                ) : (
                  compare.data.maps.map((row) => (
                    <TableRow key={row.map_id}>
                      <TableCell className="font-mono text-xs">{row.name}</TableCell>
                      <TableCell>
                        <TierBadge tier={row.tier} />
                      </TableCell>
                      <TableCell className="text-right font-mono text-xs tabular-nums">
                        {row.left.time_seconds ? `${formatSeconds(row.left.time_seconds)}s` : '—'}
                        <span className="ml-1 text-[10px] text-muted-foreground">
                          {row.left.rank ? `#${row.left.rank}` : ''}
                        </span>
                      </TableCell>
                      <TableCell className="text-right font-mono text-xs tabular-nums">
                        {row.right.time_seconds ? `${formatSeconds(row.right.time_seconds)}s` : '—'}
                        <span className="ml-1 text-[10px] text-muted-foreground">
                          {row.right.rank ? `#${row.right.rank}` : ''}
                        </span>
                      </TableCell>
                      <TableCell className="text-right font-mono text-xs tabular-nums">
                        {row.time_diff_ns == null
                          ? '—'
                          : `${row.time_diff_ns > 0 ? '+' : ''}${(row.time_diff_ns / 1_000_000_000).toFixed(3)}s`}
                      </TableCell>
                      <TableCell>
                        <Badge
                          variant={row.status === 'both' ? 'success' : row.status === 'neither' ? 'outline' : 'warning'}
                        >
                          {row.status === 'both'
                            ? '双方完成'
                            : row.status === 'left_only'
                              ? '仅 A'
                              : row.status === 'right_only'
                                ? '仅 B'
                                : '双方未完成'}
                        </Badge>
                      </TableCell>
                    </TableRow>
                  ))
                )}
              </TableBody>
            </Table>

            <div className="border-t border-border/70 px-4 py-3">
              <Pagination
                page={compare.data.meta.page}
                pageSize={compare.data.meta.page_size}
                total={compare.data.meta.total}
                onPageChange={list.setPage}
                onPageSizeChange={list.setPageSize}
              />
            </div>
          </Card>
        </>
      ) : null}
    </div>
  );
}
