import { useQuery } from '@tanstack/react-query';
import { FilterX, UserRound } from 'lucide-react';
import { useState } from 'react';

import { EmptyState } from '@/admin/components/common/empty-state';
import { FilterBar, ListPanel } from '@/admin/components/common/list-panel';
import { PageHeader } from '@/admin/components/common/page-header';
import { PlayerCell } from '@/admin/components/common/player-cell';
import { Badge } from '@/admin/components/ui/badge';
import { Button } from '@/admin/components/ui/button';
import { DataTable, type DataTableColumn } from '@/admin/components/ui/data-table';
import { Pagination } from '@/admin/components/ui/pagination';
import { SearchInput } from '@/admin/components/ui/search-input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/admin/components/ui/select';
import { useDebouncedValue } from '@/admin/hooks/use-debounced-value';
import { useTableState } from '@/admin/hooks/use-table-state';
import { formatDateTime, formatNumber } from '@/admin/lib/format';
import { pageSizeOptions } from '@/admin/lib/kz-constants';
import { directoryQueries } from '@/admin/services/directory-service';
import type { components } from '@cngokz/shared-types';

type PlayerRow = components['schemas']['PlayerSummary'];

const sortOptions = [
  { value: 'completion', label: '完成地图' },
  { value: 'wr', label: '世界纪录' },
  { value: 'records', label: '成绩数量' },
  { value: 'recent', label: '最近活跃' },
  { value: 'name', label: '昵称' },
];

const columns: DataTableColumn<PlayerRow>[] = [
  {
    id: 'player',
    header: '玩家',
    cell: (row) => <PlayerCell steamId64={row.steam_id64} displayName={row.display_name} avatarUrl={row.avatar_url} />,
  },
  {
    id: 'rank',
    header: '#',
    cell: (row) =>
      row.rank === null ? (
        <span className="text-xs text-muted-foreground">—</span>
      ) : (
        <span className="font-mono text-xs tabular-nums">{row.rank}</span>
      ),
  },
  {
    id: 'completed',
    header: '完成地图',
    cell: (row) => <span className="tabular-nums">{formatNumber(row.completed_maps)}</span>,
  },
  {
    id: 'wr',
    header: 'WR',
    cell: (row) =>
      row.wr_count > 0 ? (
        <Badge variant="success">{formatNumber(row.wr_count)}</Badge>
      ) : (
        <span className="text-xs text-muted-foreground">0</span>
      ),
  },
  {
    id: 'records',
    header: '成绩',
    cell: (row) => <span className="tabular-nums">{formatNumber(row.total_records)}</span>,
  },
  {
    id: 'country',
    header: '地区',
    cell: (row) => <span className="font-mono text-xs text-muted-foreground">{row.country ?? '—'}</span>,
  },
  {
    id: 'last',
    header: '最后活跃',
    cell: (row) => (
      <span className="text-xs text-muted-foreground">
        {row.last_activity_at ? formatDateTime(row.last_activity_at) : '—'}
      </span>
    ),
  },
];

/** 玩家目录：读取公开只读接口 `/api/v1/public/players`。 */
export function PlayersPage() {
  const table = useTableState();
  const [sort, setSort] = useState('completion');
  const search = useDebouncedValue(table.search);
  const query = useQuery(
    directoryQueries.players({
      search: search || undefined,
      sort,
      page: table.page,
      page_size: table.pageSize,
    }),
  );

  const items = query.data?.items ?? [];
  const total = query.data?.meta.total ?? 0;

  return (
    <div className="space-y-5">
      <PageHeader
        title="玩家"
        description="已登记的 Steam 玩家及其完成度、世界纪录与成绩总量。数据来自公开只读接口，与门户完全一致。"
        meta={
          query.data ? (
            <span>
              共 {formatNumber(total)} 名玩家 · 第 {query.data.meta.page} / {query.data.meta.total_pages} 页
            </span>
          ) : null
        }
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
                  setSort('completion');
                }}
              >
                <FilterX />
                重置
              </Button>
            }
          >
            <SearchInput
              value={table.search}
              onChange={table.setSearch}
              placeholder="搜索昵称或 SteamID64"
              className="w-full sm:w-64"
            />
            <Select value={sort} onValueChange={(value) => setSort(value)}>
              <SelectTrigger className="w-36" aria-label="排序">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {sortOptions.map((option) => (
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
            total={total}
            onPageChange={table.setPage}
            onPageSizeChange={table.setPageSize}
            pageSizeOptions={pageSizeOptions}
          />
        }
      >
        <DataTable
          columns={columns}
          rows={items}
          getRowKey={(row) => row.account_id}
          isLoading={query.isLoading}
          error={
            query.isError
              ? {
                  title: '玩家列表读取失败',
                  description: (query.error as Error).message,
                  onRetry: () => void query.refetch(),
                }
              : null
          }
          empty={
            <EmptyState
              icon={UserRound}
              title="没有匹配的玩家"
              description="换个搜索词，或等待游戏服务器上报新的玩家。"
            />
          }
        />
      </ListPanel>
    </div>
  );
}
