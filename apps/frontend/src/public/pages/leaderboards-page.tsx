import { useQuery } from '@tanstack/react-query';
import { CircleHelp, FilterX, Trophy } from 'lucide-react';

import { EmptyState } from '@/public/components/common/empty-state';
import { ErrorState } from '@/public/components/common/error-state';
import { PageHeader } from '@/public/components/common/page-header';
import { ModeSegment } from '@/public/components/leaderboards/mode-segment';
import { RankRow, RankRowSkeleton } from '@/public/components/leaderboards/rank-row';
import { SummaryStrip } from '@/public/components/leaderboards/summary-strip';
import { Badge } from '@/public/components/ui/badge';
import { Button } from '@/public/components/ui/button';
import { Card } from '@/public/components/ui/card';
import { Pagination } from '@/public/components/ui/pagination';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/public/components/ui/select';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/public/components/ui/tooltip';
import { useListParams } from '@/public/hooks/use-list-params';
import { formatNumber } from '@/public/lib/format';
import { cn } from '@/public/lib/utils';
import { publicQueries } from '@/public/services/public-service';
import { usePageMeta } from '@/public/seo/use-page-meta';

const timeTypeOptions = [
  { value: 'all', label: '全部类型' },
  { value: 'pro', label: 'PRO' },
  { value: 'nub', label: 'NUB' },
] as const;

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

  const activeMode = modes.data?.items.find((item) => item.code === mode);
  const entries = query.data?.items ?? [];
  const hasFilters = mode !== 'vanilla' || timeType !== 'all' || list.page > 1;

  return (
    <div className="mx-auto w-full max-w-7xl space-y-5 px-4 py-8 sm:px-6 lg:px-8">
      <PageHeader
        title="全球排行榜"
        description="按模式与成绩类型查看完成地图排行榜；积分榜请切换到独立的 KZT / SKZ / VNL 页面。"
        meta={
          <>
            <span className="font-mono">{activeMode?.display_name ?? mode}</span>
            <span className="text-border-strong">·</span>
            <span>{timeType === 'all' ? '全部成绩类型' : timeType.toUpperCase()}</span>
            <span className="text-border-strong">·</span>
            <span>共 {formatNumber(query.data?.meta.total)} 位有成绩的玩家</span>
          </>
        }
      />

      {/* Filter console: segmented mode switch + compact selectors, no giant callout. */}
      <Card className="gap-3 p-3 sm:p-4">
        <div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
          <div className="flex flex-wrap items-center gap-2">
            <span className="text-[11px] font-medium tracking-wider text-muted-foreground uppercase">模式</span>
            <ModeSegment
              modes={modes.data?.items}
              value={mode}
              onChange={(value) => list.setFilter('mode', value, 'vanilla')}
              isLoading={modes.isLoading}
            />
          </div>

          <div className="flex flex-wrap items-center gap-2">
            <div className="inline-flex items-center gap-1 rounded-md border border-border bg-surface-elevated/40 p-0.5">
              {timeTypeOptions.map((option) => (
                <button
                  key={option.value}
                  type="button"
                  aria-pressed={timeType === option.value}
                  onClick={() => list.setFilter('time_type', option.value, 'all')}
                  className={cn(
                    'h-8 cursor-pointer rounded-sm px-3 text-xs font-medium transition-colors outline-none select-none',
                    timeType === option.value
                      ? 'bg-primary/15 text-primary shadow-xs'
                      : 'text-muted-foreground hover:bg-surface-hover hover:text-foreground',
                  )}
                >
                  {option.label}
                </button>
              ))}
            </div>

            <Select value={String(list.pageSize)} onValueChange={(value) => list.setPageSize(Number(value))}>
              <SelectTrigger className="w-24" aria-label="每页显示">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {[10, 20, 50, 100].map((size) => (
                  <SelectItem key={size} value={String(size)}>
                    {size} / 页
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>

            <Button variant="ghost" size="sm" onClick={list.reset} disabled={!hasFilters} title="重置筛选与分页">
              <FilterX />
              重置
            </Button>
          </div>
        </div>

        <div className="flex flex-wrap items-center gap-2 text-[11px] text-muted-foreground">
          <Badge variant="outline">完成主线地图数</Badge>
          <span>并列时依次比较 WR 数、成绩数；此榜不以积分排序。</span>
          <Tooltip>
            <TooltipTrigger asChild>
              <button
                type="button"
                className="inline-flex items-center gap-1 text-muted-foreground hover:text-foreground"
              >
                <CircleHelp className="size-3" />
                排名说明
              </button>
            </TooltipTrigger>
            <TooltipContent className="max-w-64">
              完成地图 = 主线课程（stage 0）的已通过成绩；Bonus 成绩不计入。同一模式与成绩类型下分别统计。
            </TooltipContent>
          </Tooltip>
        </div>
      </Card>

      <SummaryStrip
        mode={mode}
        timeType={timeType}
        total={query.data?.meta.total}
        entries={entries}
        isLoading={query.isLoading}
      />

      <Card className="overflow-hidden">
        <div className="flex items-center justify-between gap-3 border-b border-border px-3 py-2.5 sm:px-4">
          <div className="flex items-center gap-2">
            <Trophy className="size-4 text-primary" aria-hidden="true" />
            <h2 className="text-sm font-semibold">{activeMode?.display_name ?? mode} 排名</h2>
          </div>
          <span className="font-mono text-[11px] text-muted-foreground tabular-nums">
            {query.data ? `#${(query.data.meta.page - 1) * query.data.meta.page_size + 1} 起` : '—'}
          </span>
        </div>

        {query.isError ? (
          <div className="p-4">
            <ErrorState title="无法加载排行榜" description={query.error.message} onRetry={() => void query.refetch()} />
          </div>
        ) : query.isLoading ? (
          <div className="divide-y divide-border/60">
            {Array.from({ length: 8 }).map((_, index) => (
              <RankRowSkeleton key={index} />
            ))}
          </div>
        ) : entries.length === 0 ? (
          <div className="p-4">
            <EmptyState
              icon={Trophy}
              title="暂无排行榜数据"
              description="当前模式与成绩类型下还没有已通过的成绩。接入成绩后排行榜会自动生成。"
              hint="排名不使用模拟数据"
            />
          </div>
        ) : (
          <div className="divide-y divide-border/60">
            {entries.map((entry) => (
              <RankRow key={entry.account_id} entry={entry} rank={entry.rank} />
            ))}
          </div>
        )}

        <div className="border-t border-border/70 px-3 py-3 sm:px-4">
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
