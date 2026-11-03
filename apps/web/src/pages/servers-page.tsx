import { FilterX, LayoutGrid, LayoutList, Server as ServerIcon } from 'lucide-react';
import { useState } from 'react';

import { EmptyState } from '@/components/common/empty-state';
import { ServerCard } from '@/components/servers/server-card';
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
import { pageSizeOptions } from '@/lib/kz-constants';

interface ServerRow {
  id: number;
  name: string;
  status: string;
  address: string;
  currentMap: string | null;
  players: number | null;
  pluginVersion: string | null;
  lastHeartbeat: string | null;
}

const columns: DataTableColumn<ServerRow>[] = [
  {
    id: 'server',
    header: '服务器',
    cell: (row) => (
      <div className="flex items-center gap-2.5">
        <span className="flex size-8 shrink-0 items-center justify-center rounded-md border border-border bg-surface-elevated text-muted-foreground">
          <ServerIcon className="size-4" aria-hidden="true" />
        </span>
        <span className="truncate text-[13px] font-medium">{row.name}</span>
      </div>
    ),
  },
  {
    id: 'status',
    header: '状态',
    cell: (row) => (
      <Badge variant={row.status === 'online' ? 'success' : 'outline'}>
        {row.status === 'online' ? '在线' : '离线'}
      </Badge>
    ),
  },
  { id: 'address', header: '地址', cell: (row) => <span className="font-mono text-xs">{row.address}</span> },
  { id: 'map', header: '当前地图', cell: (row) => <span className="font-mono text-xs">{row.currentMap ?? '—'}</span> },
  { id: 'players', header: '玩家', cell: (row) => <span className="tabular-nums">{row.players ?? '—'}</span> },
  {
    id: 'plugin',
    header: '插件版本',
    cell: (row) => (
      <Badge variant="outline" className="font-mono">
        {row.pluginVersion ?? '—'}
      </Badge>
    ),
  },
  {
    id: 'heartbeat',
    header: '最近心跳',
    cell: (row) => <span className="text-xs text-muted-foreground">{row.lastHeartbeat ?? '—'}</span>,
  },
  {
    id: 'actions',
    header: '',
    headerClassName: 'text-right',
    className: 'text-right',
    cell: () => (
      <Button variant="ghost" size="xs" disabled title="服务器详情接口尚未实现">
        详情
      </Button>
    ),
  },
];

export function ServersPage() {
  const table = useTableState();
  const [status, setStatus] = useState('all');
  const [view, setView] = useState<'list' | 'cards'>('list');

  const rows: ServerRow[] = [];
  const hasFilters = table.search.trim().length > 0 || status !== 'all';

  return (
    <div className="space-y-5">
      <PageHeader title="服务器列表" description="服务器注册、在线状态与插件心跳。监控接口尚未接入。" />

      <Callout
        variant="info"
        title="服务器监控接口尚未实现"
        description="第二阶段已建立 servers 表（API Key 哈希、插件版本、当前地图、心跳），读取接口尚未开发，因此不展示虚构的 CPU、内存或延迟。"
      />

      <ListPanel
        toolbar={
          <FilterBar
            actions={
              <>
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
                <div
                  className="flex items-center rounded-md border border-border p-0.5"
                  role="group"
                  aria-label="视图切换"
                >
                  <Button
                    variant={view === 'list' ? 'secondary' : 'ghost'}
                    size="icon-xs"
                    onClick={() => setView('list')}
                    aria-label="列表视图"
                    aria-pressed={view === 'list'}
                  >
                    <LayoutList />
                  </Button>
                  <Button
                    variant={view === 'cards' ? 'secondary' : 'ghost'}
                    size="icon-xs"
                    onClick={() => setView('cards')}
                    aria-label="卡片视图"
                    aria-pressed={view === 'cards'}
                  >
                    <LayoutGrid />
                  </Button>
                </div>
              </>
            }
          >
            <SearchInput
              value={table.search}
              onChange={table.setSearch}
              placeholder="搜索服务器名称或地址…"
              className="w-full sm:w-60"
              aria-label="搜索服务器"
            />
            <Select value={status} onValueChange={setStatus}>
              <SelectTrigger className="w-28" aria-label="在线状态">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">全部状态</SelectItem>
                <SelectItem value="online">在线</SelectItem>
                <SelectItem value="offline">离线</SelectItem>
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
        {view === 'list' ? (
          <DataTable
            columns={columns}
            rows={rows}
            getRowKey={(row) => row.id}
            minWidthClass="min-w-[1000px]"
            empty={
              <EmptyState
                icon={ServerIcon}
                title={hasFilters ? '没有匹配的服务器' : '暂无服务器数据'}
                description={
                  hasFilters
                    ? '当前筛选条件没有结果，试试重置筛选。'
                    : '服务器列表接口尚未实现，接入后将展示真实心跳与在线状态。'
                }
              />
            }
          />
        ) : rows.length > 0 ? (
          <div className="grid gap-3 p-4 md:grid-cols-2 xl:grid-cols-3">
            {rows.map((row) => (
              <ServerCard
                key={row.id}
                server={{
                  id: row.id,
                  name: row.name,
                  address: row.address,
                  tone: row.status === 'online' ? 'up' : 'unknown',
                  currentMap: row.currentMap,
                  players: row.players,
                  pluginVersion: row.pluginVersion,
                  lastHeartbeat: row.lastHeartbeat,
                }}
              />
            ))}
          </div>
        ) : (
          <div className="p-4">
            <EmptyState
              icon={ServerIcon}
              title={hasFilters ? '没有匹配的服务器' : '暂无服务器数据'}
              description="卡片视图已就绪，服务器接口接入后会展示在线状态、当前地图与插件版本。"
            />
          </div>
        )}
      </ListPanel>
    </div>
  );
}
