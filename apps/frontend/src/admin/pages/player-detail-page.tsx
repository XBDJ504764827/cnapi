import { useQuery } from '@tanstack/react-query';
import { ArrowLeft, ExternalLink, Trophy } from 'lucide-react';

import { Link, useParams } from 'react-router';

import { EmptyState } from '@/admin/components/common/empty-state';
import { ErrorState } from '@/admin/components/common/error-state';
import { PageHeader } from '@/admin/components/common/page-header';
import { PlayerCell } from '@/admin/components/common/player-cell';
import { Badge } from '@/admin/components/ui/badge';
import { Button } from '@/admin/components/ui/button';
import { Card } from '@/admin/components/ui/card';
import { DataTable, type DataTableColumn } from '@/admin/components/ui/data-table';
import { Pagination } from '@/admin/components/ui/pagination';
import { Skeleton } from '@/admin/components/ui/skeleton';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/admin/components/ui/tabs';
import { useTableState } from '@/admin/hooks/use-table-state';
import { formatDateTime, formatNumber } from '@/admin/lib/format';
import { pageSizeOptions } from '@/admin/lib/kz-constants';
import { directoryQueries } from '@/admin/services/directory-service';
import type { components } from '@cngokz/shared-types';

type RecordRow = components['schemas']['RecordSummary'];
type ModeRow = components['schemas']['PlayerModeStatsDto'];

const recordColumns: DataTableColumn<RecordRow>[] = [
  {
    id: 'map',
    header: '地图 / 课程',
    cell: (row) => (
      <div className="min-w-0">
        <p className="truncate text-[13px] font-medium">{row.map.name}</p>
        <p className="truncate text-[11px] text-muted-foreground">
          {row.course.stage === 0 ? '主线' : `Bonus ${row.course.stage}`} · {row.course.name}
        </p>
      </div>
    ),
  },
  {
    id: 'mode',
    header: '模式',
    cell: (row) => (
      <div className="flex flex-wrap items-center gap-1">
        <Badge variant="outline">{row.mode.display_name}</Badge>
        <Badge variant={row.time_type === 'pro' ? 'success' : 'warning'}>
          {row.time_type === 'pro' ? 'PRO' : 'NUB'}
        </Badge>
        {row.is_world_record ? <Badge variant="success">WR</Badge> : null}
      </div>
    ),
  },
  {
    id: 'time',
    header: '用时',
    cell: (row) => <span className="font-mono text-xs tabular-nums">{Number(row.time_seconds).toFixed(3)}s</span>,
  },
  {
    id: 'tier',
    header: '等级',
    cell: (row) => (row.tier === null ? '—' : <span className="tabular-nums">{row.tier}</span>),
  },
  {
    id: 'rank',
    header: '排名',
    cell: (row) => <span className="font-mono text-xs tabular-nums">#{row.rank}</span>,
  },
  {
    id: 'created',
    header: '上传时间',
    cell: (row) => <span className="text-xs text-muted-foreground">{formatDateTime(row.created_at)}</span>,
  },
];

const modeColumns: DataTableColumn<ModeRow>[] = [
  { id: 'mode', header: '模式', cell: (row) => <Badge variant="outline">{row.mode.display_name}</Badge> },
  {
    id: 'rank',
    header: '排名',
    cell: (row) => (row.rank === null ? '—' : <span className="font-mono text-xs tabular-nums">#{row.rank}</span>),
  },
  {
    id: 'completed',
    header: '完成 / 总数',
    cell: (row) => (
      <span className="tabular-nums">
        {formatNumber(row.completed_maps)} / {formatNumber(row.total_maps)}
      </span>
    ),
  },
  {
    id: 'rate',
    header: '完成率',
    cell: (row) => <span className="tabular-nums">{(row.completion_rate * 100).toFixed(1)}%</span>,
  },
  { id: 'wr', header: 'WR', cell: (row) => <span className="tabular-nums">{formatNumber(row.wr_count)}</span> },
  {
    id: 'records',
    header: '成绩',
    cell: (row) => <span className="tabular-nums">{formatNumber(row.total_records)}</span>,
  },
];

/** 玩家详情：资料 + 分模式统计 + 最近成绩。 */
export function PlayerDetailPage() {
  const { playerId = '' } = useParams<{ playerId: string }>();
  const table = useTableState({ initialPageSize: 20 });

  const detail = useQuery(directoryQueries.player(playerId, {}));
  const records = useQuery(
    directoryQueries.playerRecords(playerId, {
      page: table.page,
      page_size: table.pageSize,
    }),
  );

  if (detail.isError) {
    return (
      <div className="space-y-5">
        <Button variant="ghost" size="sm" asChild>
          <Link to="/admin/players">
            <ArrowLeft />
            返回玩家列表
          </Link>
        </Button>
        <ErrorState
          title="玩家资料读取失败"
          description={(detail.error as Error).message}
          onRetry={() => void detail.refetch()}
        />
      </div>
    );
  }

  if (detail.isLoading || !detail.data) {
    return (
      <div className="space-y-4">
        <Skeleton className="h-20 rounded-lg" />
        <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
          {Array.from({ length: 4 }).map((_, index) => (
            <Skeleton key={index} className="h-24 rounded-lg" />
          ))}
        </div>
        <Skeleton className="h-64 rounded-lg" />
      </div>
    );
  }

  const { profile, overall, modes } = detail.data;
  const recordItems = records.data?.items ?? [];
  const recordTotal = records.data?.meta.total ?? 0;

  return (
    <div className="space-y-5">
      <Button variant="ghost" size="sm" asChild>
        <Link to="/admin/players">
          <ArrowLeft />
          返回玩家列表
        </Link>
      </Button>

      <PageHeader
        title={profile.display_name}
        description={`SteamID64 ${profile.steam_id64} · 首次成绩 ${formatDateTime(profile.first_record_at)}`}
        meta={
          <span>
            统计口径：{detail.data.ranking_metric === 'completed_maps' ? '完成地图数' : detail.data.ranking_metric}
          </span>
        }
        actions={
          <>
            <Button variant="outline" size="sm" asChild>
              <a href={profile.steam_profile_url} target="_blank" rel="noreferrer">
                <ExternalLink />
                Steam 资料
              </a>
            </Button>
            <Button variant="outline" size="sm" asChild>
              <Link to={`/players/${profile.steam_id64}`}>
                <ExternalLink />
                门户页面
              </Link>
            </Button>
          </>
        }
      />

      <Card className="flex flex-row items-center gap-4 p-4">
        <PlayerCell
          steamId64={profile.steam_id64}
          displayName={profile.display_name}
          avatarUrl={profile.avatar_url}
          linked={false}
        />
        <div className="ml-auto flex flex-wrap gap-2">
          <Badge variant="outline">地区 {profile.country ?? '—'}</Badge>
          <Badge variant="outline">最后成绩 {formatDateTime(profile.last_record_at)}</Badge>
        </div>
      </Card>

      <section aria-label="关键指标" className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
        {[
          {
            label: '完成地图',
            value: `${formatNumber(overall.completed_maps)} / ${formatNumber(overall.total_maps)}`,
            hint: `完成率 ${(overall.completion_rate * 100).toFixed(1)}%`,
          },
          {
            label: '世界纪录',
            value: formatNumber(overall.wr_count),
            hint: `PRO ${overall.pro_count} · NUB ${overall.tp_count}`,
          },
          { label: '成绩总数', value: formatNumber(overall.total_records), hint: '全部模式' },
          {
            label: '综合排名',
            value: overall.global_rank === null ? '—' : `#${overall.global_rank}`,
            hint: '按完成地图数',
          },
        ].map((item) => (
          <Card key={item.label} className="gap-0 p-4">
            <p className="text-xs text-muted-foreground">{item.label}</p>
            <p className="mt-2 text-[22px] leading-none font-semibold tracking-tight tabular-nums">{item.value}</p>
            <p className="mt-2 text-[11px] text-muted-foreground">{item.hint}</p>
          </Card>
        ))}
      </section>

      <Tabs defaultValue="records">
        <TabsList>
          <TabsTrigger value="records">最近成绩</TabsTrigger>
          <TabsTrigger value="modes">分模式统计</TabsTrigger>
        </TabsList>

        <TabsContent value="records" className="mt-3">
          <Card className="overflow-hidden p-0">
            <DataTable
              columns={recordColumns}
              rows={recordItems}
              getRowKey={(row) => row.id}
              isLoading={records.isLoading}
              error={
                records.isError
                  ? {
                      title: '成绩读取失败',
                      description: (records.error as Error).message,
                      onRetry: () => void records.refetch(),
                    }
                  : null
              }
              empty={<EmptyState icon={Trophy} title="暂无成绩" description="该玩家还没有已通过的记录。" />}
            />
            <div className="border-t border-border/70 px-4 py-3">
              <Pagination
                page={table.page}
                pageSize={table.pageSize}
                total={recordTotal}
                onPageChange={table.setPage}
                onPageSizeChange={table.setPageSize}
                pageSizeOptions={pageSizeOptions}
              />
            </div>
          </Card>
        </TabsContent>

        <TabsContent value="modes" className="mt-3">
          <Card className="overflow-hidden p-0">
            <DataTable
              columns={modeColumns}
              rows={modes}
              getRowKey={(row) => row.mode.code}
              empty={<EmptyState icon={Trophy} title="暂无模式数据" description="该玩家还没有任何模式的成绩。" />}
            />
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}
