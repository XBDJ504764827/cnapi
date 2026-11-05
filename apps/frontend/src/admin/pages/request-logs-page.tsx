import { FilterX, ListFilter } from 'lucide-react';
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
import { httpMethods, pageSizeOptions } from '@/admin/lib/kz-constants';

interface RequestLogRow {
  id: number;
  createdAt: string;
  method: string;
  path: string;
  status: number;
  latencyMs: number;
  requestId: string;
  ip: string;
}

const statusVariant = (status: number) => (status >= 500 ? 'danger' : status >= 400 ? 'warning' : 'success');

const columns: DataTableColumn<RequestLogRow>[] = [
  {
    id: 'time',
    header: '时间',
    cell: (row) => <span className="font-mono text-xs text-muted-foreground">{row.createdAt}</span>,
  },
  {
    id: 'method',
    header: '方法',
    cell: (row) => (
      <Badge variant={row.method === 'GET' ? 'info' : 'primary'} className="font-mono">
        {row.method}
      </Badge>
    ),
  },
  { id: 'path', header: '路径', cell: (row) => <span className="font-mono text-xs">{row.path}</span> },
  { id: 'status', header: '状态', cell: (row) => <Badge variant={statusVariant(row.status)}>{row.status}</Badge> },
  {
    id: 'latency',
    header: '延迟',
    cell: (row) => <span className="font-mono text-xs tabular-nums">{row.latencyMs} ms</span>,
  },
  {
    id: 'requestId',
    header: '请求 ID',
    cell: (row) => <span className="font-mono text-[11px] text-muted-foreground">{row.requestId}</span>,
  },
  {
    id: 'ip',
    header: '来源 IP',
    cell: (row) => <span className="font-mono text-xs text-muted-foreground">{row.ip}</span>,
  },
];

export function RequestLogsPage() {
  const table = useTableState();
  const [method, setMethod] = useState('all');
  const [statusClass, setStatusClass] = useState('all');
  const hasFilters = table.search.trim().length > 0 || method !== 'all' || statusClass !== 'all';

  return (
    <div className="space-y-5">
      <PageHeader
        title="API 请求日志"
        description="接口调用、状态码、延迟与请求 ID。用于排查客户端问题与观察错误率。"
      />

      <Callout
        variant="info"
        icon={ListFilter}
        title="请求日志接口尚未实现"
        description="后端目前只写入 tracing 日志，没有查询接口或持久化表。本页展示查询结构，不展示模拟请求。"
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
                  setMethod('all');
                  setStatusClass('all');
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
              placeholder="搜索路径或请求 ID…"
              className="w-full sm:w-64"
              aria-label="搜索请求日志"
            />
            <Select value={method} onValueChange={setMethod}>
              <SelectTrigger className="w-28" aria-label="HTTP 方法">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">全部方法</SelectItem>
                {httpMethods.map((option) => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Select value={statusClass} onValueChange={setStatusClass}>
              <SelectTrigger className="w-28" aria-label="状态码范围">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">全部状态</SelectItem>
                <SelectItem value="2xx">2xx</SelectItem>
                <SelectItem value="4xx">4xx</SelectItem>
                <SelectItem value="5xx">5xx</SelectItem>
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
          minWidthClass="min-w-[1000px]"
          empty={
            <EmptyState
              icon={ListFilter}
              title={hasFilters ? '没有匹配的请求' : '暂无请求日志'}
              description={
                hasFilters
                  ? '当前筛选条件没有结果，试试重置筛选。'
                  : '请求日志查询接口尚未实现。接入后可排查错误率、延迟与具体请求。'
              }
            />
          }
        />
      </ListPanel>
    </div>
  );
}
