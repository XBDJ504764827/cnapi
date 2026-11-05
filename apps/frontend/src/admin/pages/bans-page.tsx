import { Ban, FilterX, ShieldAlert } from 'lucide-react';
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
import { banSources, banStatuses, pageSizeOptions } from '@/admin/lib/kz-constants';

interface BanRow {
  id: number;
  player: string;
  steamId64: string;
  banType: string;
  reason: string;
  issuedAt: string;
  expiresAt: string | null;
  status: string;
  source: string;
}

const sourceLabel: Record<string, string> = {
  admin: '管理员',
  server: '服务器',
  globalapi_import: 'GlobalAPI 导入',
};

const columns: DataTableColumn<BanRow>[] = [
  {
    id: 'player',
    header: '玩家',
    cell: (row) => (
      <div className="min-w-0">
        <p className="truncate text-[13px] font-medium">{row.player}</p>
        <p className="truncate font-mono text-[11px] text-muted-foreground">{row.steamId64}</p>
      </div>
    ),
  },
  { id: 'type', header: '类型', cell: (row) => <Badge variant="outline">{row.banType}</Badge> },
  { id: 'reason', header: '原因', cell: (row) => <span className="line-clamp-1 text-[13px]">{row.reason}</span> },
  {
    id: 'issuedAt',
    header: '开始时间',
    cell: (row) => <span className="text-xs text-muted-foreground">{row.issuedAt}</span>,
  },
  {
    id: 'expiresAt',
    header: '结束时间',
    cell: (row) => <span className="text-xs text-muted-foreground">{row.expiresAt ?? '永久'}</span>,
  },
  {
    id: 'status',
    header: '状态',
    cell: (row) => (
      <Badge variant={row.status === 'active' ? 'danger' : 'outline'}>
        {row.status === 'active' ? '生效中' : '已撤销'}
      </Badge>
    ),
  },
  {
    id: 'source',
    header: '来源',
    cell: (row) => <span className="text-xs text-muted-foreground">{sourceLabel[row.source] ?? row.source}</span>,
  },
];

export function BansPage() {
  const table = useTableState();
  const [status, setStatus] = useState('all');
  const [source, setSource] = useState('all');
  const hasFilters = table.search.trim().length > 0 || status !== 'all' || source !== 'all';

  return (
    <div className="space-y-5">
      <PageHeader title="封禁记录" description="封禁、解封与审计记录。封禁为追加式数据，不会物理删除。" />

      <Callout
        variant="info"
        icon={ShieldAlert}
        title="封禁接口尚未实现"
        description="bans 与 ban_revocations 表已在第二阶段建立（过期时间推导、不可软删除）。读取接口尚未开发，页面不展示模拟封禁。"
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
                  setSource('all');
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
              placeholder="搜索玩家、SteamID64 或原因…"
              className="w-full sm:w-64"
              aria-label="搜索封禁"
            />
            <Select value={status} onValueChange={setStatus}>
              <SelectTrigger className="w-28" aria-label="封禁状态">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">全部状态</SelectItem>
                {banStatuses.map((option) => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Select value={source} onValueChange={setSource}>
              <SelectTrigger className="w-36" aria-label="封禁来源">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">全部来源</SelectItem>
                {banSources.map((option) => (
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
              icon={Ban}
              title={hasFilters ? '没有匹配的封禁记录' : '暂无封禁记录'}
              description={
                hasFilters
                  ? '当前筛选条件没有结果，试试重置筛选。'
                  : '封禁接口尚未实现。接入后将展示真实封禁、执行者与解封记录。'
              }
            />
          }
        />
      </ListPanel>
    </div>
  );
}
