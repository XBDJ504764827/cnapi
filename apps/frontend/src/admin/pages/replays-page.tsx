import { Film, HardDrive, FilterX } from 'lucide-react';
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
import { pageSizeOptions } from '@/admin/lib/kz-constants';

interface ReplayRow {
  id: number;
  recordId: number;
  player: string;
  mapName: string;
  sizeBytes: number;
  status: string;
  uploadedAt: string | null;
  expiresAt: string | null;
}

const statusMeta: Record<string, { label: string; variant: 'success' | 'warning' | 'danger' | 'outline' }> = {
  available: { label: '可用', variant: 'success' },
  pending: { label: '上传中', variant: 'warning' },
  failed: { label: '失败', variant: 'danger' },
  deleted: { label: '已删除', variant: 'outline' },
};

const columns: DataTableColumn<ReplayRow>[] = [
  {
    id: 'replay',
    header: 'Replay',
    cell: (row) => (
      <div className="flex items-center gap-2.5">
        <span className="flex size-8 items-center justify-center rounded-md border border-border bg-surface-elevated text-muted-foreground">
          <Film className="size-4" aria-hidden="true" />
        </span>
        <div className="min-w-0">
          <p className="truncate text-[13px] font-medium">#{row.id}</p>
          <p className="truncate font-mono text-[11px] text-muted-foreground">record #{row.recordId}</p>
        </div>
      </div>
    ),
  },
  { id: 'player', header: '玩家', cell: (row) => <span className="text-[13px]">{row.player}</span> },
  { id: 'map', header: '地图', cell: (row) => <span className="font-mono text-xs">{row.mapName}</span> },
  {
    id: 'size',
    header: '大小',
    cell: (row) => (
      <span className="font-mono text-xs tabular-nums">{(row.sizeBytes / 1024 / 1024).toFixed(2)} MB</span>
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
    id: 'uploaded',
    header: '上传时间',
    cell: (row) => <span className="text-xs text-muted-foreground">{row.uploadedAt ?? '—'}</span>,
  },
  {
    id: 'expires',
    header: '保留至',
    cell: (row) => <span className="text-xs text-muted-foreground">{row.expiresAt ?? '永久'}</span>,
  },
];

export function ReplaysPage() {
  const table = useTableState();
  const [status, setStatus] = useState('all');
  const hasFilters = table.search.trim().length > 0 || status !== 'all';

  return (
    <div className="space-y-5">
      <PageHeader title="Replay" description="Replay 对象元数据：R2 Object Key、大小、校验值与保留期。" />

      <Callout
        variant="info"
        icon={HardDrive}
        title="Replay 接口尚未实现"
        description="对象存储（R2）与元数据表已在第二阶段设计；上传/下载接口尚未开发，因此不会展示模拟文件。"
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
              placeholder="搜索 Replay、成绩或玩家…"
              className="w-full sm:w-64"
              aria-label="搜索 Replay"
            />
            <Select value={status} onValueChange={setStatus}>
              <SelectTrigger className="w-28" aria-label="Replay 状态">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">全部状态</SelectItem>
                <SelectItem value="available">可用</SelectItem>
                <SelectItem value="pending">上传中</SelectItem>
                <SelectItem value="failed">失败</SelectItem>
                <SelectItem value="deleted">已删除</SelectItem>
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
              icon={Film}
              title={hasFilters ? '没有匹配的 Replay' : '暂无 Replay'}
              description={
                hasFilters
                  ? '当前筛选条件没有结果，试试重置筛选。'
                  : 'Replay 接口尚未实现。接入后将展示对象存储位置、大小、SHA-256 与保留期。'
              }
            />
          }
        />
      </ListPanel>
    </div>
  );
}
