import { useQuery } from '@tanstack/react-query';
import { UserRound } from 'lucide-react';
import { Link, useParams } from 'react-router';

import { EmptyState } from '@/components/common/empty-state';
import { ErrorState } from '@/components/common/error-state';
import { ModeTabs } from '@/components/common/mode-tabs';
import { ModeStatsCards } from '@/components/players/mode-stats';
import { PlayerMapProgress } from '@/components/players/map-progress';
import { PlayerRecordHistory } from '@/components/players/record-history';
import { ProfileHeader } from '@/components/players/profile-header';
import { OverallStats } from '@/components/players/overall-stats';
import { ModePointsCards } from '@/components/points/mode-points-cards';
import { TierProgress } from '@/components/players/tier-progress';
import { Callout } from '@/components/ui/callout';
import { Skeleton } from '@/components/ui/skeleton';
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { useListParams } from '@/hooks/use-list-params';
import { pointsQueries } from '@/services/points-service';
import { publicQueries } from '@/services/public-service';
import { usePageMeta } from '@/seo/use-page-meta';
import type { ApiClientError } from '@cngokz/api-client';

export function PlayerDetailPage() {
  const { steamid = '' } = useParams<{ steamid: string }>();
  const list = useListParams({ pageKey: 'page', pageSizeKey: 'page_size' });
  const mode = list.read('mode', 'vanilla');
  const timeType = list.read('time_type', 'all');

  const modes = useQuery(publicQueries.modes());
  const player = useQuery(publicQueries.player(steamid, { mode, time_type: timeType }));
  const pointQuery = useQuery(pointsQueries.player(steamid));

  usePageMeta({
    title: player.data ? `${player.data.profile.display_name} · 玩家资料` : '玩家资料',
    description: player.data
      ? `${player.data.profile.display_name} 的 KZ 数据：完成地图 ${player.data.overall.completed_maps} 张、世界纪录 ${player.data.overall.wr_count} 个、成绩 ${player.data.overall.total_records} 条。`
      : '查看 KZ 玩家资料、地图进度、各模式统计与成绩历史。',
    path: `/players/${steamid}`,
    type: 'profile',
  });

  const error = player.error as ApiClientError | null;
  const notFound = error?.status === 404;

  if (notFound) {
    return (
      <div className="mx-auto w-full max-w-3xl px-4 py-16 sm:px-6">
        <EmptyState
          icon={UserRound}
          title="未找到该玩家"
          description="该 SteamID64 不在公开数据库中。请确认 SteamID64 是否正确，或从玩家目录搜索。"
          action={
            <Link to="/players" className="text-sm text-primary hover:underline">
              返回玩家目录
            </Link>
          }
        />
      </div>
    );
  }

  return (
    <div className="mx-auto w-full max-w-7xl space-y-5 px-4 py-8 sm:px-6 lg:px-8">
      {player.isError && error ? (
        <ErrorState title="无法加载玩家资料" description={error.message} onRetry={() => void player.refetch()} />
      ) : null}

      {player.isLoading ? (
        <div className="space-y-3">
          <Skeleton className="h-28 rounded-xl" />
          <div className="grid grid-cols-2 gap-3 lg:grid-cols-4">
            {Array.from({ length: 8 }).map((_, index) => (
              <Skeleton key={index} className="h-24 rounded-lg" />
            ))}
          </div>
          <Skeleton className="h-40 rounded-lg" />
        </div>
      ) : player.data ? (
        <>
          <ProfileHeader player={player.data} />
          <OverallStats player={player.data} />
          {pointQuery.data ? <ModePointsCards points={pointQuery.data} /> : null}

          <Callout
            variant="neutral"
            title="统计口径"
            description={
              <span>
                上方综合统计为全部模式/类型/风格汇总；下方“当前范围”与难度/地图进度统一使用{' '}
                {player.data.scope.mode.display_name} · {player.data.scope.style.display_name} ·{' '}
                {player.data.scope.time_type.toUpperCase()}。{player.data.scope.completion_definition}；积分区域按 KZT /
                SKZ / VNL 独立展示。
              </span>
            }
          />

          <section className="space-y-3">
            <div className="flex flex-col gap-2 lg:flex-row lg:items-center lg:justify-between">
              <ModeTabs
                modes={modes.data?.items}
                value={mode}
                onChange={(value) => list.setFilter('mode', value, 'vanilla')}
                isLoading={modes.isLoading}
              />
              <Tabs value={timeType} onValueChange={(value) => list.setFilter('time_type', value, 'all')}>
                <TabsList>
                  <TabsTrigger value="all">全部类型</TabsTrigger>
                  <TabsTrigger value="pro">PRO</TabsTrigger>
                  <TabsTrigger value="nub">NUB</TabsTrigger>
                </TabsList>
              </Tabs>
            </div>
            <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
              {player.data.modes.map((item) => (
                <ModeStatsCards key={item.mode.code} mode={item} active={item.mode.code === mode} />
              ))}
            </div>
          </section>

          <TierProgress tiers={player.data.tiers} />

          <section className="space-y-3">
            <div>
              <h2 className="text-base font-semibold tracking-tight">地图进度</h2>
              <p className="text-xs text-muted-foreground">
                {mode} · {timeType === 'all' ? '全部成绩类型' : timeType.toUpperCase()} · 包含全部已上线地图（含未完成）
              </p>
            </div>
            <PlayerMapProgress steamid={steamid} mode={mode} timeType={timeType} />
          </section>

          <section className="space-y-3">
            <div>
              <h2 className="text-base font-semibold tracking-tight">成绩历史</h2>
              <p className="text-xs text-muted-foreground">按上传时间倒序；完成时间指跑图时长。</p>
            </div>
            <PlayerRecordHistory steamid={steamid} mode={mode} timeType={timeType} />
          </section>
        </>
      ) : null}
    </div>
  );
}
