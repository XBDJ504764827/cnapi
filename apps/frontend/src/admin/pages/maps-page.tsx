import { useQuery } from '@tanstack/react-query';
import { FilterX, Map as MapIcon } from 'lucide-react';
import { useState } from 'react';

import { EmptyState } from '@/admin/components/common/empty-state';
import { FilterBar, ListPanel } from '@/admin/components/common/list-panel';
import { PageHeader } from '@/admin/components/common/page-header';
import { Badge } from '@/admin/components/ui/badge';
import { Button } from '@/admin/components/ui/button';
import { DataTable, type DataTableColumn } from '@/admin/components/ui/data-table';
import { Pagination } from '@/admin/components/ui/pagination';
import { SearchInput } from '@/admin/components/ui/search-input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/admin/components/ui/select';
import { useDebouncedValue } from '@/admin/hooks/use-debounced-value';
import { useTableState } from '@/admin/hooks/use-table-state';
import { formatDateTime, formatNumber } from '@/admin/lib/format';
import { pageSizeOptions, timeTypes } from '@/admin/lib/kz-constants';
import { directoryQueries } from '@/admin/services/directory-service';
import type { components } from '@cngokz/shared-types';

type MapRow = components['schemas']['MapSummary'];

const sortOptions = [
  { value: 'completions', label: '完成人数' },
  { value: 'records', label: '成绩数量' },
  { value: 'recent', label: '最近提交' },
  { value: 'name', label: '地图名' },
];

const tierOptions = ['1', '2', '3', '4', '5', '6', '7', '8'];

const columns: DataTableColumn<MapRow>[] = [
  {
    id: 'map',
    header: '地图',
    cell: (row) => (
      <div className="min-w-0">
        <p className="truncate text-[13px] font-medium">{row.name}</p>
        <p className="truncate font-mono text-[11px] text-muted-foreground">{row.slug}</p>
      </div>
    ),
  },
  {
    id: 'tier',
    header: '等级',
    cell: (row) =>
      row.tier === null ? (
        <span className="text-xs text-muted-foreground">未评级</span>
      ) : (
        <Badge variant="outline">T{row.tier}</Badge>
      ),
  },
  {
    id: 'courses',
    header: '课程',
    cell: (row) => <span className="tabular-nums">{formatNumber(row.courses)}</span>,
  },
  {
    id: 'players',
    header: '完成人数',
    cell: (row) => <span className="tabular-nums">{formatNumber(row.completed_players)}</span>,
  },
  {
    id: 'records',
    header: '成绩',
    cell: (row) => <span className="tabular-nums">{formatNumber(row.records_count)}</span>,
  },
  {
    id: 'wr',
    header: '世界纪录',
    cell: (row) =>
      row.wr_time_seconds ? (
        <div className="min-w-0">
          <p className="font-mono text-xs tabular-nums">{Number(row.wr_time_seconds).toFixed(3)}s</p>
          <p className="truncate text-[11px] text-muted-foreground">{row.wr_player_name ?? '—'}</p>
        </div>
      ) : (
        <span className="text-xs text-muted-foreground">—</span>
      ),
  },
  {
    id: 'created',
    header: '入库时间',
    cell: (row) => <span className="text-xs text-muted-foreground">{formatDateTime(row.created_at)}</span>,
  },
];

/** 地图列表：读取 `/api/v1/public/maps`。 */
export function MapsPage() {
  const table = useTableState();
  const [sort, setSort] = useState('completions');
  const [returnedTier, setTier] = useState('all');
  const [timeType, setTimeType] = useState('all');
  const search = useDebouncedValue(table.search);

  const query = useQuery(
    directoryQueries.maps({
      search: search || undefined,
      sort,
      tier: returnedTier === 'all' ? undefined : Number(returnedTier),
      time_type: timeType === 'all' ? undefined : timeType,
      page: table.page,
      page_size: table.pageSize,
    }),
  );

  const items = query.data?.items ?? [];
  const total = query.data?.meta.total ?? 0;

  return (
    <div className="space-y-5">
      <PageHeader
        title="地图"
        description="地图库、课程数量、完成人数与世界纪录。等级按所选时间类型维度展示。"
        meta={query.data ? <span>共 {formatNumber(total)} 张地图</span> : null}
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
                  setSort('completions');
                  setTimeType('all');
                  setTier('all');
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
              placeholder="搜索地图名"
              className="w-full sm:w-56"
            />

            <Select value={timeType} onValueChange={setTimeType}>
              <SelectTrigger className="w-28" aria-label="时间类型">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">全部类型</SelectItem>
                {timeTypes.map((option) => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>

            <Select value={returnedTier} onValueChange={setTier}>
              <SelectTrigger className="w-24" aria-label="等级">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">全部等级</SelectItem>
                {tierOptions.map((tier) => (
                  <SelectItem key={tier} value={tier}>
                    T{tier}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>

            <Select value={sort} onValueChange={setSort}>
              <SelectTrigger className="w-32" aria-label="排序">
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
          getRowKey={(row) => row.id}
          isLoading={query.isLoading}
          error={
            query.isError
              ? {
                  title: '地图列表读取失败',
                  description: (query.error as Error).message,
                  onRetry: () => void query.refetch(),
                }
              : null
          }
          empty={<EmptyState icon={MapIcon} title="没有匹配的地图" description="调整筛选条件或等待地图同步。" />}
        />
      </ListPanel>
    </div>
  );
}
