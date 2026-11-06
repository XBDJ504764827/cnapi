import { ArrowDown, ArrowUp, ArrowUpDown, FilterX, Timer } from 'lucide-react';
import { useState } from 'react';

import { EmptyState } from '@/admin/components/common/empty-state';
import { FilterBar, ListPanel } from '@/admin/components/common/list-panel';
import { PageHeader } from '@/admin/components/common/page-header';
import { Badge } from '@/admin/components/ui/badge';
import { Button } from '@/admin/components/ui/button';
import { Callout } from '@/admin/components/ui/callout';
import { DataTable, type DataTableColumn } from '@/admin/components/ui/data-table';
import { Pagination } from '@/admin/components/ui/pagination';
import { SearchInput } from '@/admin/components/ui/search-input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/admin/components/ui/select';
import { useTableState } from '@/admin/hooks/use-table-state';
import { movementModes, movementStyles, pageSizeOptions, recordStatuses, timeTypes } from '@/admin/lib/kz-constants';

interface RecordRow {
  id: number;
  player: string;
  mapName: string;
  mode: string;
  style: string;
  timeType: 'pro' | 'nub';
  teleports: number;
  timeNs: number;
  status: string;
  worldRecord: boolean;
  submittedAt: string;
}

const statusMeta: Record<string, { label: string; variant: 'success' | 'warning' | 'danger' }> = {
  approved: { label: '已通过', variant: 'success' },
  pending: { label: '待审核', variant: 'warning' },
  rejected: { label: '已拒绝', variant: 'danger' },
};

export function RecordsPage() {
  const table = useTableState({ initialSort: { id: 'time', direction: 'asc' } });
  const [mode, setMode] = useState('all');
  const [timeType, setTimeType] = useState('all');
  const [status, setStatus] = useState('all');
  const [style, setStyle] = useState('all');
  const [sortBy, setSortBy] = useState('time_asc');

  const sortIcon = (id: string) => {
    if (table.sort?.id !== id) return <ArrowUpDown className="size-3 opacity-50" />;
    return table.sort.direction === 'asc' ? <ArrowUp className="size-3" /> : <ArrowDown className="size-3" />;
  };

  const columns: DataTableColumn<RecordRow>[] = [
    {
      id: 'player',
      header: '玩家',
      cell: (row) => <span className="truncate text-[13px] font-medium">{row.player}</span>,
    },
    { id: 'map', header: '地图', cell: (row) => <span className="truncate text-[13px]">{row.mapName}</span> },
    { id: 'mode', header: '模式', cell: (row) => <Badge variant="outline">{row.mode}</Badge> },
    { id: 'style', header: 'Style', cell: (row) => <span className="text-muted-foreground">{row.style}</span> },
    {
      id: 'timeType',
      header: 'TP / PRO',
      cell: (row) => (
        <div className="flex items-center gap-1.5">
          <Badge variant={row.timeType === 'pro' ? 'success' : 'warning'}>{row.timeType.toUpperCase()}</Badge>
          <span className="text-[11px] text-muted-foreground tabular-nums">{row.teleports} TP</span>
        </div>
      ),
    },
    {
      id: 'time',
      header: (
        <button
          type="button"
          className="inline-flex items-center gap-1 text-[11px] font-medium tracking-[0.045em] uppercase hover:text-foreground"
          onClick={() => table.toggleSort('time', 'asc')}
          aria-label={
            table.sort?.id === 'time' && table.sort.direction === 'desc' ? '按成绩时间降序排序' : '按成绩时间升序排序'
          }
          data-sort={table.sort?.id === 'time' ? table.sort.direction : 'none'}
        >
          成绩
          {sortIcon('time')}
        </button>
      ),
      cell: (row) => (
        <span className="font-mono text-xs tabular-nums">
          {(row.timeNs / 1_000_000_000).toFixed(3)}
          <span className="ml-1 text-muted-foreground">s</span>
        </span>
      ),
    },
    {
      id: 'wr',
      header: 'WR',
      cell: (row) =>
        row.worldRecord ? <Badge variant="primary">WR</Badge> : <span className="text-muted-foreground">—</span>,
    },
    {
      id: 'status',
      header: '审核状态',
      cell: (row) => (
        <Badge variant={statusMeta[row.status]?.variant ?? 'outline'}>
          {statusMeta[row.status]?.label ?? row.status}
        </Badge>
      ),
    },
    {
      id: 'submittedAt',
      header: '提交时间',
      cell: (row) => <span className="text-xs text-muted-foreground">{row.submittedAt}</span>,
    },
  ];

  const hasFilters =
    table.search.trim().length > 0 ||
    mode !== 'all' ||
    timeType !== 'all' ||
    status !== 'all' ||
    style !== 'all' ||
    sortBy !== 'time_asc';

  return (
    <div className="space-y-5">
      <PageHeader title="成绩管理" description="成绩审核、TP/PRO 维度、模式与来源服务器。列表接口尚未接入。" />

      <Callout
        variant="info"
        title="成绩查询接口尚未实现"
        description="成绩表、审核状态与 WR 指针已在第二阶段建模。当前筛选与排序可交互，但不会展示模拟成绩。"
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
                  setMode('all');
                  setTimeType('all');
                  setStatus('all');
                  setStyle('all');
                  setSortBy('time_asc');
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
              aria-label="搜索成绩"
            />
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
            <Select value={style} onValueChange={setStyle}>
              <SelectTrigger className="w-28" aria-label="Style">
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
            <Select value={timeType} onValueChange={setTimeType}>
              <SelectTrigger className="w-28" aria-label="TP / PRO">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">TP / PRO</SelectItem>
                {timeTypes.map((option) => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Select
              value={sortBy}
              onValueChange={(value) => {
                setSortBy(value);
                table.toggleSort('time', value === 'time_desc' ? 'desc' : 'asc');
              }}
            >
              <SelectTrigger className="w-36" aria-label="排序">
                <ArrowUpDown className="size-3.5 text-muted-foreground" />
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="time_asc">成绩升序</SelectItem>
                <SelectItem value="time_desc">成绩降序</SelectItem>
                <SelectItem value="submitted">最近提交</SelectItem>
              </SelectContent>
            </Select>
            <Select value={status} onValueChange={setStatus}>
              <SelectTrigger className="w-28" aria-label="审核状态">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">全部状态</SelectItem>
                {recordStatuses.map((option) => (
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
          minWidthClass="min-w-[1040px]"
          empty={
            <EmptyState
              icon={Timer}
              title={hasFilters ? '没有匹配的成绩' : '暂无成绩数据'}
              description={
                hasFilters
                  ? '当前筛选条件没有结果，可以重置后再试。'
                  : '成绩查询接口尚未实现，接入后将展示真实成绩、审核状态与 WR 标记。'
              }
            />
          }
        />
      </ListPanel>
    </div>
  );
}
