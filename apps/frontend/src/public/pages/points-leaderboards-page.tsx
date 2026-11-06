import { useQuery } from '@tanstack/react-query';
import { useState } from 'react';
import { Link } from 'react-router';

import { PageHeader } from '@/public/components/common/page-header';
import { PointsLeaderboardTable } from '@/public/components/points/points-leaderboard-table';
import { Badge } from '@/public/components/ui/badge';
import { Card } from '@/public/components/ui/card';
import { Pagination } from '@/public/components/ui/pagination';
import { pointsQueries } from '@/public/services/points-service';
import { usePageMeta } from '@/public/seo/use-page-meta';

const modes = ['KZT', 'SKZ', 'VNL'] as const;

export function PointsLeaderboardsPage() {
  usePageMeta({
    title: '模式积分排行榜',
    description: '分别查看 KZT、SKZ、VNL 三种模式的玩家积分排名，三种模式完全独立，不混合排名。',
    path: '/points',
  });

  const [mode, setMode] = useState<(typeof modes)[number]>('KZT');
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);
  const query = useQuery(pointsQueries.mode(mode, page, pageSize));

  return (
    <div className="mx-auto w-full max-w-7xl space-y-5 px-4 py-8 sm:px-6 lg:px-8">
      <PageHeader
        title="模式积分排行榜"
        description="KZT、SKZ、VNL 分别计算总分、分别排名；切换模式不会把其他模式的分数合并。"
        actions={
          <Link to="/leaderboards" className="text-xs text-primary hover:underline">
            查看完成地图排行榜
          </Link>
        }
      />

      <Card className="gap-3 p-3">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div className="inline-flex items-center gap-0.5 rounded-md border border-border bg-surface-elevated/40 p-0.5">
            {modes.map((value) => (
              <button
                key={value}
                type="button"
                aria-pressed={mode === value}
                onClick={() => {
                  setMode(value);
                  setPage(1);
                }}
                className={
                  mode === value
                    ? 'h-8 rounded-sm bg-primary/15 px-4 text-xs font-medium text-primary'
                    : 'h-8 rounded-sm px-4 text-xs font-medium text-muted-foreground hover:text-foreground'
                }
              >
                {value}
              </button>
            ))}
          </div>
          <div className="flex items-center gap-2 text-[11px] text-muted-foreground">
            <Badge variant="outline">总分 = PRO + NUB</Badge>
            <span>Filter 满分 1000</span>
          </div>
        </div>
        <PointsLeaderboardTable
          items={query.data?.items}
          isLoading={query.isLoading}
          errorMessage={query.isError ? query.error.message : undefined}
          onRetry={() => void query.refetch()}
        />
        <div className="border-t border-border/70 px-1 pt-3">
          <Pagination
            page={query.data?.meta.page ?? page}
            pageSize={query.data?.meta.page_size ?? pageSize}
            total={query.data?.meta.total ?? 0}
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
