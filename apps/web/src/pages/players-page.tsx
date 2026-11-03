import { ArrowUpDown, FilterX, UserRound, Users } from 'lucide-react';
import { useState } from 'react';
import { Link } from 'react-router';

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
import { Switch } from '@/components/ui/switch';
import { Avatar, AvatarFallback } from '@/components/ui/avatar';
import { useTableState } from '@/hooks/use-table-state';
import { pageSizeOptions } from '@/lib/kz-constants';

interface PlayerRow {
  id: string;
  displayName: string;
  steamId64: string;
  worldRecords: number;
  points: number;
  lastSeen: string;
}

/**
 * Players list. The read endpoint does not exist yet; filters, sorting and
 * pagination are interactive but the table shows an explicit empty state.
 */
export function PlayersPage() {
  const table = useTableState({ initialSort: { id: 'last_seen', direction: 'desc' } });
  const [sortBy, setSortBy] = useState('last_seen');
  const [onlyWithRecords, setOnlyWithRecords] = useState(false);

  const columns: DataTableColumn<PlayerRow>[] = [
    {
      id: 'player',
      header: '玩家',
      cell: (row) => (
        <div className="flex items-center gap-2.5">
          <Avatar className="size-7">
            <AvatarFallback>
              <UserRound className="size-3.5" />
            </AvatarFallback>
          </Avatar>
          <div className="min-w-0">
            <p className="truncate text-[13px] font-medium">{row.displayName}</p>
            <p className="truncate font-mono text-[11px] text-muted-foreground">{row.steamId64}</p>
          </div>
        </div>
      ),
    },
    { id: 'wr', header: 'WR', cell: (row) => <span className="tabular-nums">{row.worldRecords}</span> },
    {
      id: 'points',
      header: '总积分',
      cell: (row) => <span className="font-medium tabular-nums">{row.points.toLocaleString('zh-CN')}</span>,
    },
    {
      id: 'lastSeen',
      header: '最近活动',
      cell: (row) => <span className="text-muted-foreground">{row.lastSeen}</span>,
    },
    {
      id: 'actions',
      header: '',
      headerClassName: 'text-right',
      className: 'text-right',
      cell: (row) => (
        <Button variant="ghost" size="xs" asChild>
          <Link to={`/players/${row.steamId64}`}>详情</Link>
        </Button>
      ),
    },
  ];

  const hasFilters = table.search.trim().length > 0 || onlyWithRecords || sortBy !== 'last_seen';

  return (
    <div className="space-y-5">
      <PageHeader
        title="玩家列表"
        description="检索 Steam 玩家档案、WR 数量与总积分。当前查询接口尚未接入，页面仅展示结构与空状态。"
      />

      <Callout
        variant="info"
        title="玩家查询接口尚未实现"
        description="筛选、排序与分页状态已可交互；在玩家查询接口接入前，不会展示任何模拟玩家。"
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
                  setSortBy('last_seen');
                  setOnlyWithRecords(false);
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
              placeholder="搜索昵称或 SteamID64…"
              className="w-full sm:w-64"
              aria-label="搜索玩家"
            />
            <Select value={sortBy} onValueChange={setSortBy}>
              <SelectTrigger className="w-40" aria-label="排序方式">
                <ArrowUpDown className="size-3.5 text-muted-foreground" />
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="last_seen">最近活动</SelectItem>
                <SelectItem value="points">总积分</SelectItem>
                <SelectItem value="wr">WR 数量</SelectItem>
                <SelectItem value="steam_id">SteamID64</SelectItem>
              </SelectContent>
            </Select>
            <label className="flex items-center gap-2 text-xs text-muted-foreground">
              <Switch checked={onlyWithRecords} onCheckedChange={setOnlyWithRecords} aria-label="仅显示有 WR 的玩家" />
              仅有 WR
            </label>
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
          minWidthClass="min-w-[860px]"
          empty={
            <EmptyState
              icon={Users}
              title={hasFilters ? '没有匹配的玩家' : '暂无玩家数据'}
              description={
                hasFilters
                  ? '当前搜索或筛选条件没有结果。可以重置筛选后再试。'
                  : '玩家查询接口尚未实现，接入后会展示真实玩家、积分与最近活动。'
              }
              hint={hasFilters ? undefined : '玩家查询接口将在阶段 4 接入'}
            />
          }
        />
      </ListPanel>

      <p className="text-[11px] text-muted-foreground">
        提示：<Badge variant="outline">阶段 4</Badge> 将提供玩家分页查询、详情与成绩历史接口。
      </p>
    </div>
  );
}
