import { ArrowRight, Database, Map as MapIcon, Search, Server, Trophy, Users } from 'lucide-react';
import { useQuery } from '@tanstack/react-query';
import { useState } from 'react';
import { Link } from 'react-router';

import { EmptyState } from '@/components/common/empty-state';
import { ErrorState } from '@/components/common/error-state';
import { PlayerAvatar, PlayerNameLink } from '@/components/common/player-avatar';
import { SearchResults } from '@/components/common/search-results';
import { SectionHeader } from '@/components/common/section-header';
import { StatCard } from '@/components/common/stat-card';
import { RecordTable } from '@/components/records/record-table';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Callout } from '@/components/ui/callout';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Skeleton } from '@/components/ui/skeleton';
import { StatusIndicator } from '@/components/ui/status-indicator';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { formatDateTime, formatSeconds } from '@/lib/format';
import { publicQueries } from '@/services/public-service';
import { usePageMeta } from '@/seo/use-page-meta';

export function HomePage() {
  usePageMeta({
    title: '全球 KZ 排行榜与玩家数据',
    description: 'CNGOKZ 公开数据平台：全球 KZ 排行榜、玩家资料与地图进度、最近成绩、世界纪录与服务器状态。',
    path: '/',
  });

  const statistics = useQuery(publicQueries.statistics());
  const modes = useQuery(publicQueries.modes());
  const [mode, setMode] = useState('vanilla');
  const leaderboard = useQuery(publicQueries.leaderboard({ mode, page_size: 10 }));
  const recent = useQuery(publicQueries.recentRecords({ limit: 20 }));
  const worldRecords = useQuery(publicQueries.worldRecords({ limit: 8 }));
  const activePlayers = useQuery(publicQueries.activePlayers({ days: 7, limit: 5 }));
  const servers = useQuery(publicQueries.servers({ page_size: 6, online: true }));
  const [heroQuery, setHeroQuery] = useState('');

  const stats = statistics.data;
  const availableModes = modes.data?.items ?? [];

  return (
    <div>
      <div className="mx-auto w-full max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
        <div className="space-y-8">
          <section className="grid gap-5 lg:grid-cols-[minmax(0,1fr)_420px] lg:items-end">
            <div className="space-y-3">
              <div className="space-y-1.5">
                <h1 className="text-2xl leading-tight font-semibold tracking-tight sm:text-[28px]">数据概览</h1>
                <p className="max-w-2xl text-[13px] leading-relaxed text-muted-foreground">
                  全球 KZ 排行榜、玩家进度、地图完成情况与最近上传成绩。数据来自公开数据库。
                </p>
              </div>
              <div className="flex flex-wrap items-center gap-2">
                <Button size="sm" asChild>
                  <Link to="/leaderboards">
                    排行榜
                    <ArrowRight />
                  </Link>
                </Button>
                <Button size="sm" variant="outline" asChild>
                  <Link to="/players">玩家目录</Link>
                </Button>
                <Button size="sm" variant="ghost" asChild>
                  <Link to="/maps">地图列表</Link>
                </Button>
              </div>
            </div>

            <div className="relative w-full">
              <div className="relative">
                <Search className="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-muted-foreground" />
                <Input
                  value={heroQuery}
                  onChange={(event) => setHeroQuery(event.target.value)}
                  placeholder="搜索玩家、SteamID64 或地图…"
                  className="h-10 border-border bg-surface pl-9 text-[13px] shadow-xs"
                  aria-label="搜索玩家或地图"
                />
              </div>
              {heroQuery.trim().length >= 2 ? (
                <div className="absolute top-full right-0 left-0 z-30 mt-2 rounded-lg border border-border bg-popover p-2 shadow-xl">
                  <SearchResults query={heroQuery} onNavigate={() => setHeroQuery('')} />
                </div>
              ) : (
                <p className="mt-1.5 text-[11px] text-muted-foreground">输入至少 2 个字符，或使用 ⌘K 全局搜索。</p>
              )}
            </div>
          </section>

          <section aria-label="全站统计" className="space-y-3">
            <div className="grid grid-cols-2 gap-3 lg:grid-cols-4">
              <StatCard label="玩家总数" value={stats?.players} icon={Users} isLoading={statistics.isLoading} />
              <StatCard label="地图总数" value={stats?.maps} icon={MapIcon} isLoading={statistics.isLoading} />
              <StatCard label="成绩总数" value={stats?.records} icon={Trophy} isLoading={statistics.isLoading} />
              <StatCard
                label="在线服务器"
                value={stats?.servers_online}
                icon={Server}
                isLoading={statistics.isLoading}
                hint="心跳 2 分钟内"
              />
            </div>
            {statistics.isError ? (
              <ErrorState
                title="无法读取全站统计"
                description={statistics.error.message}
                onRetry={() => void statistics.refetch()}
              />
            ) : null}
          </section>
          <section aria-label="模式排行榜">
            <SectionHeader
              title="模式排行榜"
              description="按已完成主线地图数排序；积分算法尚未定义。"
              to="/leaderboards"
            />
            {modes.isError ? (
              <ErrorState
                title="无法读取模式列表"
                description={modes.error.message}
                onRetry={() => void modes.refetch()}
              />
            ) : (
              <Tabs value={mode} onValueChange={setMode}>
                <TabsList className="mb-3">
                  {availableModes.length === 0 ? <Skeleton className="mb-2 h-5 w-40" /> : null}
                  {availableModes.map((item) => (
                    <TabsTrigger key={item.code} value={item.code}>
                      {item.display_name}
                    </TabsTrigger>
                  ))}
                </TabsList>

                <Card className="overflow-hidden">
                  <CardHeader className="flex-row items-center justify-between gap-3">
                    <div>
                      <CardTitle>Top 10 · {leaderboard.data?.mode.display_name ?? '—'}</CardTitle>
                      <CardDescription>
                        {leaderboard.data?.time_type === 'all'
                          ? '全部成绩类型'
                          : leaderboard.data?.time_type?.toUpperCase()}
                      </CardDescription>
                    </div>
                    <Badge variant="outline">按完成地图排序</Badge>
                  </CardHeader>
                  <CardContent className="px-0 pb-0">
                    {leaderboard.isError ? (
                      <div className="p-4">
                        <ErrorState
                          title="无法加载排行榜"
                          description={leaderboard.error.message}
                          onRetry={() => void leaderboard.refetch()}
                        />
                      </div>
                    ) : leaderboard.isLoading ? (
                      <div className="space-y-2 p-4">
                        {Array.from({ length: 6 }).map((_, index) => (
                          <Skeleton key={index} className="h-10 rounded-md" />
                        ))}
                      </div>
                    ) : leaderboard.data && leaderboard.data.items.length > 0 ? (
                      <Table className="min-w-[720px]">
                        <TableHeader>
                          <TableRow className="hover:bg-transparent">
                            <TableHead className="w-12">#</TableHead>
                            <TableHead>玩家</TableHead>
                            <TableHead className="text-right">完成地图</TableHead>
                            <TableHead className="text-right">WR</TableHead>
                            <TableHead className="text-right">成绩数</TableHead>
                            <TableHead className="text-right">积分</TableHead>
                          </TableRow>
                        </TableHeader>
                        <TableBody>
                          {leaderboard.data.items.map((entry) => (
                            <TableRow key={entry.account_id}>
                              <TableCell className="font-mono text-xs text-muted-foreground tabular-nums">
                                #{entry.rank}
                              </TableCell>
                              <TableCell>
                                <div className="flex items-center gap-2.5">
                                  <PlayerAvatar
                                    steamId64={entry.steam_id64}
                                    displayName={entry.display_name}
                                    size="sm"
                                  />
                                  <div className="min-w-0">
                                    <PlayerNameLink steamId64={entry.steam_id64} displayName={entry.display_name} />
                                    <p className="truncate font-mono text-[10px] text-muted-foreground">
                                      {entry.steam_id64}
                                    </p>
                                  </div>
                                </div>
                              </TableCell>
                              <TableCell className="text-right font-medium tabular-nums">
                                {entry.completed_maps}
                              </TableCell>
                              <TableCell className="text-right tabular-nums">{entry.wr_count}</TableCell>
                              <TableCell className="text-right text-muted-foreground tabular-nums">
                                {entry.total_records}
                              </TableCell>
                              <TableCell className="text-right text-muted-foreground">—</TableCell>
                            </TableRow>
                          ))}
                        </TableBody>
                      </Table>
                    ) : (
                      <div className="p-4">
                        <EmptyState
                          icon={Trophy}
                          title="暂无排行榜数据"
                          description="数据库中还没有该模式的已通过成绩。接入成绩后排行榜会自动出现。"
                        />
                      </div>
                    )}
                  </CardContent>
                </Card>
              </Tabs>
            )}
          </section>

          <section aria-label="最近成绩">
            <SectionHeader
              title="最近上传的成绩"
              description="按上传时间倒序，展示最近 20 条公开成绩。"
              to="/records"
            />
            <Card className="overflow-hidden">
              <RecordTable
                records={recent.data?.items}
                isLoading={recent.isLoading}
                errorMessage={recent.isError ? recent.error.message : undefined}
                onRetry={() => void recent.refetch()}
                showRank
                emptyTitle="暂无成绩"
                emptyDescription="数据库中还没有已通过的公开成绩。"
              />
            </Card>
          </section>

          <div className="grid gap-4 lg:grid-cols-2">
            <section aria-label="最近世界纪录">
              <SectionHeader title="最近世界纪录" description="当前 WR 指针按达成时间排序；历史提升值尚未存储。" />
              <Card>
                <CardContent className="pt-4">
                  {worldRecords.isError ? (
                    <ErrorState
                      title="无法加载世界纪录"
                      description={worldRecords.error.message}
                      onRetry={() => void worldRecords.refetch()}
                    />
                  ) : worldRecords.isLoading ? (
                    <div className="space-y-2">
                      {Array.from({ length: 4 }).map((_, index) => (
                        <Skeleton key={index} className="h-11 rounded-md" />
                      ))}
                    </div>
                  ) : worldRecords.data && worldRecords.data.length > 0 ? (
                    <ul className="divide-y divide-border/70">
                      {worldRecords.data.map((entry) => (
                        <li
                          key={entry.record.id}
                          className="flex items-center justify-between gap-3 py-2.5 first:pt-0 last:pb-0"
                        >
                          <div className="min-w-0">
                            <div className="flex items-center gap-1.5">
                              <Trophy className="size-3 text-primary" />
                              <PlayerNameLink
                                steamId64={entry.record.player.steam_id64}
                                displayName={entry.record.player.display_name}
                              />
                            </div>
                            <Link
                              to={`/maps/${entry.record.map.slug}`}
                              className="block truncate font-mono text-[11px] text-muted-foreground hover:text-primary"
                            >
                              {entry.record.map.name}
                            </Link>
                          </div>
                          <div className="shrink-0 text-right">
                            <p className="font-mono text-[13px] font-medium tabular-nums">
                              {formatSeconds(entry.record.time_seconds)}s
                            </p>
                            <p className="text-[10px] text-muted-foreground">{formatDateTime(entry.achieved_at)}</p>
                          </div>
                        </li>
                      ))}
                    </ul>
                  ) : (
                    <EmptyState compact icon={Trophy} title="暂无世界纪录" description="还没有生成 WR 指针。" />
                  )}
                </CardContent>
              </Card>
            </section>

            <section aria-label="本周活跃玩家">
              <SectionHeader title="本周活跃玩家" description="最近 7 天提交成绩最多的玩家。" />
              <Card>
                <CardContent className="pt-4">
                  {activePlayers.isError ? (
                    <ErrorState
                      title="无法加载活跃玩家"
                      description={activePlayers.error.message}
                      onRetry={() => void activePlayers.refetch()}
                    />
                  ) : activePlayers.isLoading ? (
                    <div className="space-y-2">
                      {Array.from({ length: 4 }).map((_, index) => (
                        <Skeleton key={index} className="h-11 rounded-md" />
                      ))}
                    </div>
                  ) : activePlayers.data && activePlayers.data.items.length > 0 ? (
                    <ol className="divide-y divide-border/70">
                      {activePlayers.data.items.map((entry, index) => (
                        <li
                          key={entry.player.account_id}
                          className="flex items-center gap-3 py-2.5 first:pt-0 last:pb-0"
                        >
                          <span className="w-5 font-mono text-xs text-muted-foreground tabular-nums">{index + 1}</span>
                          <PlayerAvatar
                            steamId64={entry.player.steam_id64}
                            displayName={entry.player.display_name}
                            size="sm"
                          />
                          <PlayerNameLink
                            steamId64={entry.player.steam_id64}
                            displayName={entry.player.display_name}
                            className="flex-1"
                          />
                          <span className="text-xs tabular-nums">{entry.records} 条</span>
                        </li>
                      ))}
                    </ol>
                  ) : (
                    <EmptyState compact icon={Users} title="本周暂无成绩" description="最近 7 天没有新的公开成绩。" />
                  )}
                </CardContent>
              </Card>
            </section>
          </div>

          <section aria-label="在线服务器">
            <SectionHeader title="在线服务器" description="心跳在 2 分钟内的服务器；玩家数暂未接入。" to="/servers" />
            {servers.isError ? (
              <ErrorState
                title="无法加载服务器"
                description={servers.error.message}
                onRetry={() => void servers.refetch()}
              />
            ) : servers.isLoading ? (
              <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
                {Array.from({ length: 3 }).map((_, index) => (
                  <Skeleton key={index} className="h-28 rounded-lg" />
                ))}
              </div>
            ) : servers.data && servers.data.items.length > 0 ? (
              <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
                {servers.data.items.map((server) => (
                  <Card key={server.id} className="gap-2 p-4">
                    <div className="flex items-center justify-between gap-2">
                      <p className="truncate text-[13px] font-medium">{server.name}</p>
                      <StatusIndicator
                        tone={server.online ? 'up' : 'unknown'}
                        label={server.online ? '在线' : '离线'}
                      />
                    </div>
                    <p className="truncate font-mono text-[11px] text-muted-foreground">
                      {server.ip}:{server.port}
                    </p>
                    <div className="flex items-center justify-between gap-2 text-[11px] text-muted-foreground">
                      <span className="truncate">{server.current_map_name ?? '未知地图'}</span>
                      <span>玩家 —</span>
                    </div>
                    <div className="flex items-center justify-between text-[10px] text-muted-foreground">
                      <span className="truncate">插件 {server.plugin_version ?? '—'}</span>
                      <span>{server.last_heartbeat_at ? formatDateTime(server.last_heartbeat_at) : '—'}</span>
                    </div>
                  </Card>
                ))}
              </div>
            ) : (
              <EmptyState
                icon={Server}
                title="暂无在线服务器"
                description="服务器尚未上报心跳，或数据库中没有服务器记录。"
              />
            )}
          </section>

          <Callout
            variant="neutral"
            icon={Database}
            title="数据口径说明"
            description="玩家排名按“已完成主线地图数”排序；积分显示为 “—” 是因为积分算法尚未定义。完成率 = 已完成地图 / 支持该模式的已上线地图。"
          />
        </div>
      </div>
    </div>
  );
}
