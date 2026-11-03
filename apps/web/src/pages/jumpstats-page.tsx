import { FilterX, Gauge, Ruler } from 'lucide-react';
import { useState } from 'react';

import { EmptyState } from '@/components/common/empty-state';
import { FilterBar, ListPanel } from '@/components/common/list-panel';
import { PageHeader } from '@/components/common/page-header';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Callout } from '@/components/ui/callout';
import { DataTable, type DataTableColumn } from '@/components/ui/data-table';
import { Pagination } from '@/components/ui/pagination';
import { SearchInput } from '@/components/ui/search-input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { useTableState } from '@/hooks/use-table-state';
import { jumpTypes, movementModes, pageSizeOptions } from '@/lib/kz-constants';

interface JumpstatRow {
  id: number;
  player: string;
  mapName: string;
  jumpType: string;
  distance: number;
  sync: number | null;
  strafes: number | null;
  tickrate: number;
  createdAt: string;
}

const columns: DataTableColumn<JumpstatRow>[] = [
  { id: 'player', header: '玩家', cell: (row) => <span className="text-[13px] font-medium">{row.player}</span> },
  { id: 'map', header: '地图', cell: (row) => <span className="font-mono text-xs">{row.mapName}</span> },
  { id: 'type', header: '类型', cell: (row) => <Badge variant="outline">{row.jumpType}</Badge> },
  {
    id: 'distance',
    header: '距离 (units)',
    cell: (row) => <span className="font-mono font-medium tabular-nums">{row.distance.toFixed(2)}</span>,
  },
  {
    id: 'sync',
    header: 'Sync',
    cell: (row) => <span className="tabular-nums">{row.sync != null ? `${row.sync.toFixed(1)}%` : '—'}</span>,
  },
  { id: 'strafes', header: 'Strafes', cell: (row) => <span className="tabular-nums">{row.strafes ?? '—'}</span> },
  { id: 'tickrate', header: 'Tickrate', cell: (row) => <span className="font-mono text-xs">{row.tickrate}</span> },
  {
    id: 'created',
    header: '提交时间',
    cell: (row) => <span className="text-xs text-muted-foreground">{row.createdAt}</span>,
  },
];

export function JumpstatsPage() {
  const table = useTableState();
  const [jumpType, setJumpType] = useState('all');
  const [mode, setMode] = useState('all');

  const hasFilters = table.search.trim().length > 0 || jumpType !== 'all' || mode !== 'all';

  return (
    <div className="space-y-5">
      <PageHeader title="Jumpstats" description="跳跃类型、距离与同步率数据。查询接口尚未接入。" />

      <Callout
        variant="info"
        title="Jumpstats 接口尚未实现"
        description="jumpstats 表与 jump_types 引用数据已在第二阶段建立；读取接口尚未开发。页面不展示模拟距离或排名。"
      />

      <ListPanel
        toolbar={
          <FilterBar
            actions={
              <Button
                variant="ghost"
                size="sm"
                onClick={() => {
                  table.resetFilters();
                  setJumpType('all');
                  setMode('all');
                }}
                disabled={!hasFilters}
              >
                <FilterX />
                重置
              </Button>
            }
          >
            <SearchInput
              value={table.search}
              onChange={table.setSearch}
              placeholder="搜索玩家或地图…"
              className="w-full sm:w-56"
              aria-label="搜索 Jumpstats"
            />
            <Select value={jumpType} onValueChange={setJumpType}>
              <SelectTrigger className="w-44" aria-label="跳跃类型">
                <Ruler className="size-3.5 text-muted-foreground" />
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">全部类型</SelectItem>
                {jumpTypes.map((option) => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Select value={mode} onValueChange={setMode}>
              <SelectTrigger className="w-32" aria-label="移动模式">
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
          </FilterBar>
        }
        footer={
          <Pagination
            page={table.page}
            pageSize={table.pageSize}
            total={0}
            onPageChange={table.setPage}
            onPageSizeChange={table.setPageSize}
            pageSizeOptions={pageSizeOptions}
          />
        }
      >
        <DataTable
          columns={columns}
          rows={[]}
          getRowKey={(row) => row.id}
          minWidthClass="min-w-[980px]"
          empty={
            <EmptyState
              icon={Gauge}
              title={hasFilters ? '没有匹配的 Jumpstats' : '暂无 Jumpstats 数据'}
              description={
                hasFilters
                  ? '当前筛选条件没有结果，试着重置筛选。'
                  : 'Jumpstats 查询接口尚未实现，接入后会展示真实距离、Sync 与 Strafe 数据。'
              }
            />
          }
        />
      </ListPanel>
    </div>
  );
}
