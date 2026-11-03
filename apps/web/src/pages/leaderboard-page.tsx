import { Trophy } from 'lucide-react';
import { useState } from 'react';

import { EmptyState } from '@/components/common/empty-state';
import { FilterBar, ListPanel } from '@/components/common/list-panel';
import { PageHeader } from '@/components/common/page-header';
import { Button } from '@/components/ui/button';
import { Callout } from '@/components/ui/callout';
import { DataTable, type DataTableColumn } from '@/components/ui/data-table';
import { Pagination } from '@/components/ui/pagination';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { useTableState } from '@/hooks/use-table-state';
import { movementModes, movementStyles, pageSizeOptions } from '@/lib/kz-constants';

interface GlobalRankRow {
  rank: number;
  player: string;
  points: number;
  finishes: number;
}

interface MapRecordRow {
  rank: number;
  mapName: string;
  stage: number;
  player: string;
  timeSeconds: number;
}

const globalColumns: DataTableColumn<GlobalRankRow>[] = [
  {
    id: 'rank',
    header: '#',
    cell: (row) => <span className="font-mono text-xs text-muted-foreground tabular-nums">{row.rank}</span>,
  },
  { id: 'player', header: '玩家', cell: (row) => <span className="text-[13px] font-medium">{row.player}</span> },
  {
    id: 'points',
    header: '积分',
    cell: (row) => <span className="font-medium tabular-nums">{row.points.toLocaleString('zh-CN')}</span>,
  },
  { id: 'finishes', header: '完成数', cell: (row) => <span className="tabular-nums">{row.finishes}</span> },
];

const mapColumns: DataTableColumn<MapRecordRow>[] = [
  {
    id: 'rank',
    header: '#',
    cell: (row) => <span className="font-mono text-xs text-muted-foreground tabular-nums">{row.rank}</span>,
  },
  { id: 'map', header: '地图', cell: (row) => <span className="text-[13px]">{row.mapName}</span> },
  {
    id: 'stage',
    header: '关卡',
    cell: (row) => <span className="font-mono text-xs">{row.stage === 0 ? '主线' : `Bonus ${row.stage}`}</span>,
  },
  { id: 'player', header: '玩家', cell: (row) => <span className="text-[13px] font-medium">{row.player}</span> },
  {
    id: 'time',
    header: '成绩',
    cell: (row) => <span className="font-mono text-xs tabular-nums">{row.timeSeconds.toFixed(3)} s</span>,
  },
];

export function LeaderboardPage() {
  const [mode, setMode] = useState('all');
  const [style, setStyle] = useState('all');
  const [timeType, setTimeType] = useState('all');
  const globalTable = useTableState({ initialPageSize: 20 });
  const mapTable = useTableState({ initialPageSize: 20 });

  return (
    <div className="space-y-5">
      <PageHeader title="全球排行榜" description="综合积分榜与地图纪录榜。排行榜计算与查询接口尚未实现。" />

      <Callout
        variant="info"
        title="排行榜接口尚未实现"
        description="排行榜依赖积分计算任务（player_mode_stats）与 WR 指针（course_world_records），这些尚未接入读取接口。当前不展示模拟排名。"
      />

      <Tabs defaultValue="global">
        <TabsList>
          <TabsTrigger value="global">总积分榜</TabsTrigger>
          <TabsTrigger value="maps">地图纪录</TabsTrigger>
        </TabsList>

        <TabsContent value="global" className="space-y-4">
          <ListPanel
            toolbar={
              <FilterBar>
                <Select value={mode} onValueChange={setMode}>
                  <SelectTrigger className="w-36" aria-label="移动模式">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">全部模式</SelectItem>
                    {movementModes.map((option) => (
                      <SelectItem key={option.value} value={option.value}>
                        {option.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                <Select value={timeType} onValueChange={setTimeType}>
                  <SelectTrigger className="w-28" aria-label="TP / PRO">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">TP / PRO</SelectItem>
                    <SelectItem value="pro">PRO</SelectItem>
                    <SelectItem value="nub">NUB</SelectItem>
                  </SelectContent>
                </Select>
              </FilterBar>
            }
            footer={
              <Pagination
                page={globalTable.page}
                pageSize={globalTable.pageSize}
                total={0}
                onPageChange={globalTable.setPage}
                onPageSizeChange={globalTable.setPageSize}
                pageSizeOptions={pageSizeOptions}
              />
            }
          >
            <DataTable
              columns={globalColumns}
              rows={[]}
              getRowKey={(row) => row.rank}
              minWidthClass="min-w-[680px]"
              empty={
                <EmptyState
                  icon={Trophy}
                  title="暂无排行榜数据"
                  description="积分榜查询接口尚未实现。接入后会按模式、TP/PRO 展示真实排名。"
                />
              }
            />
          </ListPanel>
        </TabsContent>

        <TabsContent value="maps" className="space-y-4">
          <ListPanel
            toolbar={
              <FilterBar
                actions={
                  <Button variant="ghost" size="sm" onClick={() => setStyle('all')} disabled={style === 'all'}>
                    重置 Style
                  </Button>
                }
              >
                <Select value={mode} onValueChange={setMode}>
                  <SelectTrigger className="w-36" aria-label="移动模式">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">全部模式</SelectItem>
                    {movementModes.map((option) => (
                      <SelectItem key={option.value} value={option.value}>
                        {option.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                <Select value={style} onValueChange={setStyle}>
                  <SelectTrigger className="w-32" aria-label="Style">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">全部 Style</SelectItem>
                    {movementStyles.map((option) => (
                      <SelectItem key={option.value} value={option.value}>
                        {option.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </FilterBar>
            }
            footer={
              <Pagination
                page={mapTable.page}
                pageSize={mapTable.pageSize}
                total={0}
                onPageChange={mapTable.setPage}
                onPageSizeChange={mapTable.setPageSize}
                pageSizeOptions={pageSizeOptions}
              />
            }
          >
            <DataTable
              columns={mapColumns}
              rows={[]}
              getRowKey={(row) => `${row.mapName}-${row.rank}`}
              minWidthClass="min-w-[760px]"
              empty={
                <EmptyState
                  icon={Trophy}
                  title="暂无地图纪录"
                  description="WR 查询接口尚未实现。接入后会按地图、关卡、模式与 TP/PRO 展示真实纪录。"
                />
              }
            />
          </ListPanel>
        </TabsContent>
      </Tabs>
    </div>
  );
}
