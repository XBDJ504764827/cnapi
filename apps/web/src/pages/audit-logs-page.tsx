import { FilterX, ScrollText } from 'lucide-react';
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
import { auditActions, pageSizeOptions } from '@/lib/kz-constants';

interface AuditLogRow {
  id: number;
  createdAt: string;
  actor: string;
  action: string;
  target: string;
  result: 'success' | 'failed';
  ip: string;
}

const columns: DataTableColumn<AuditLogRow>[] = [
  {
    id: 'time',
    header: '时间',
    cell: (row) => <span className="font-mono text-xs text-muted-foreground">{row.createdAt}</span>,
  },
  { id: 'actor', header: '操作者', cell: (row) => <span className="text-[13px] font-medium">{row.actor}</span> },
  { id: 'action', header: '动作', cell: (row) => <Badge variant="outline">{row.action}</Badge> },
  { id: 'target', header: '目标', cell: (row) => <span className="font-mono text-xs">{row.target}</span> },
  {
    id: 'result',
    header: '结果',
    cell: (row) => (
      <Badge variant={row.result === 'success' ? 'success' : 'danger'}>
        {row.result === 'success' ? '成功' : '失败'}
      </Badge>
    ),
  },
  {
    id: 'ip',
    header: '来源 IP',
    cell: (row) => <span className="font-mono text-xs text-muted-foreground">{row.ip}</span>,
  },
];

export function AuditLogsPage() {
  const table = useTableState();
  const [action, setAction] = useState('all');
  const hasFilters = table.search.trim().length > 0 || action !== 'all';

  return (
    <div className="space-y-5">
      <PageHeader title="操作日志" description="管理员与系统操作审计：审核、封禁、密钥与配置变更。" />

      <Callout
        variant="info"
        icon={ScrollText}
        title="审计日志接口尚未实现"
        description="第二阶段已有 record_history 等模块级审计表；跨模块的操作日志查询接口尚未开发，本页不展示模拟操作。"
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
                  setAction('all');
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
              placeholder="搜索操作者或目标…"
              className="w-full sm:w-64"
              aria-label="搜索操作日志"
            />
            <Select value={action} onValueChange={setAction}>
              <SelectTrigger className="w-40" aria-label="操作类型">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">全部动作</SelectItem>
                {auditActions.map((option) => (
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
          minWidthClass="min-w-[880px]"
          empty={
            <EmptyState
              icon={ScrollText}
              title={hasFilters ? '没有匹配的操作' : '暂无操作日志'}
              description={
                hasFilters
                  ? '当前筛选条件没有结果，试试重置筛选。'
                  : '审计日志查询接口尚未实现。接入后将记录真实管理员操作与结果。'
              }
            />
          }
        />
      </ListPanel>
    </div>
  );
}
