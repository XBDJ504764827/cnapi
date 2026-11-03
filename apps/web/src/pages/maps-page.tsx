import { ArrowUpDown, FilterX, Map as MapIcon, MapPinned } from 'lucide-react';
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
import { mapStatuses, pageSizeOptions } from '@/lib/kz-constants';

interface MapRow {
  id: number;
  name: string;
  workshopId: string | null;
  status: string;
  nubTier: number | null;
  proTier: number | null;
  records: number;
}

const statusMeta: Record<string, { label: string; variant: 'success' | 'warning' | 'outline' }> = {
  approved: { label: '已上线', variant: 'success' },
  in_testing: { label: '测试中', variant: 'warning' },
  disabled: { label: '已停用', variant: 'outline' },
};

export function MapsPage() {
  const table = useTableState({ initialSort: { id: 'updated_at', direction: 'desc' } });
  const [status, setStatus] = useState('all');
  const [sortBy, setSortBy] = useState('updated_at');

  const columns: DataTableColumn<MapRow>[] = [
    {
      id: 'map',
      header: '地图',
      cell: (row) => (
        <div className="flex items-center gap-2.5">
          <span className="flex size-8 shrink-0 items-center justify-center rounded-md border border-border bg-surface-elevated text-muted-foreground">
            <MapIcon className="size-4" aria-hidden="true" />
          </span>
          <div className="min-w-0">
            <p className="truncate text-[13px] font-medium">{row.name}</p>
            <p className="truncate font-mono text-[11px] text-muted-foreground">{row.workshopId ?? 'CS:GO 地图'}</p>
          </div>
        </div>
      ),
    },
    {
      id: 'workshop',
      header: 'Workshop ID',
      cell: (row) => <span className="font-mono text-xs text-muted-foreground">{row.workshopId ?? '—'}</span>,
    },
    {
      id: 'tier',
      header: '难度 (NUB / PRO)',
      cell: (row) =>
        row.nubTier || row.proTier ? (
          <span className="tabular-nums">
            {row.nubTier ?? '—'} / {row.proTier ?? '—'}
          </span>
        ) : (
          <span className="text-muted-foreground">未评级</span>
        ),
    },
    {
      id: 'status',
      header: '状态',
      cell: (row) => (
        <Badge variant={statusMeta[row.status]?.variant ?? 'outline'}>
          {statusMeta[row.status]?.label ?? row.status}
        </Badge>
      ),
    },
    {
      id: 'records',
      header: '记录数',
      cell: (row) => <span className="tabular-nums">{row.records.toLocaleString('zh-CN')}</span>,
    },
    {
      id: 'actions',
      header: '',
      headerClassName: 'text-right',
      className: 'text-right',
      cell: () => (
        <Button variant="ghost" size="xs" disabled title="地图详情接口尚未实现">
          详情
        </Button>
      ),
    },
  ];

  const hasFilters = table.search.trim().length > 0 || status !== 'all' || sortBy !== 'updated_at';

  return (
    <div className="space-y-5">
      <PageHeader
        title="地图管理"
        description="地图基础信息、Workshop ID、内容版本与 NUB/PRO 难度。列表接口尚未接入。"
      />

      <Callout
        variant="info"
        title="地图查询接口尚未实现"
        description="地图、关卡与难度数据已在第二阶段建模，但读取接口尚未开发；页面暂不展示模拟地图。"
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
                  setStatus('all');
                  setSortBy('updated_at');
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
              placeholder="搜索地图名称或 Workshop ID…"
              className="w-full sm:w-64"
              aria-label="搜索地图"
            />
            <Select value={status} onValueChange={setStatus}>
              <SelectTrigger className="w-32" aria-label="地图状态">
                <MapPinned className="size-3.5 text-muted-foreground" />
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">全部状态</SelectItem>
                {mapStatuses.map((option) => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Select value={sortBy} onValueChange={setSortBy}>
              <SelectTrigger className="w-36" aria-label="排序方式">
                <ArrowUpDown className="size-3.5 text-muted-foreground" />
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="updated_at">最近更新</SelectItem>
                <SelectItem value="name">地图名称</SelectItem>
                <SelectItem value="tier">难度</SelectItem>
                <SelectItem value="records">记录数</SelectItem>
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
          minWidthClass="min-w-[900px]"
          empty={
            <EmptyState
              icon={MapIcon}
              title={hasFilters ? '没有匹配的地图' : '暂无地图数据'}
              description={
                hasFilters
                  ? '当前筛选条件没有结果，试试重置筛选。'
                  : '地图查询接口尚未实现，接入后将展示 Workshop、难度与记录数量。'
              }
            />
          }
        />
      </ListPanel>
    </div>
  );
}
