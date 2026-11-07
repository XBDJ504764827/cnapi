import { useQuery } from '@tanstack/react-query';
import { FilterX, Trophy } from 'lucide-react';
import { useState } from 'react';
import { Link } from 'react-router';

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
import { Switch } from '@/admin/components/ui/switch';
import { useDebouncedValue } from '@/admin/hooks/use-debounced-value';
import { useTableState } from '@/admin/hooks/use-table-state';
import { formatDateTime, formatNumber } from '@/admin/lib/format';
import { movementModes, pageSizeOptions, timeTypes } from '@/admin/lib/kz-constants';
import { directoryQueries } from '@/admin/services/directory-service';
import type { components } from '@cngokz/shared-types';

type RecordRow = components['schemas']['RecordSummary'];

const sortOptions = [
  { value: 'uploaded', label: '上传时间' },
  { value: 'time', label: '用时' },
  { value: 'rank', label: '排名' },
];

const columns: DataTableColumn<RecordRow>[] = [
  {
    id: 'player',
    header: '玩家',
    cell: (row) => (
      <PlayerCell
        steamId64={row.player.steam_id64}
        displayName={row.player.display_name}
        avatarUrl={row.player.avatar_url}
      />
    ),
  },
  {
    id: 'map',
    header: '地图 / 课程',
    cell: (row) => (
      <div className="min-w-0">
        <Link
          to={`/maps/${row.map.slug}`}
          className="block truncate text-[13px] font-medium hover:text-primary hover:underline"
        >
          {row.map.name}
        </Link>
        <p className="truncate text-[11px] text-muted-foreground">
          {row.course.stage === 0 ? '主线' : `Bonus ${row.course.stage}`} · {row.course.name}
        </p>
      </div>
    ),
  },
  {
    id: 'mode',
    header: '模式',
    cell: (row) => (
      <div className="flex flex-wrap items-center gap-1">
        <Badge variant="outline">{row.mode.display_name}</Badge>
        <Badge variant={row.time_type === 'pro' ? 'success' : 'warning'}>
          {row.time_type === 'pro' ? 'PRO' : 'NUB'}
        </Badge>
        {row.teleports > 0 ? <Badge variant="outline">TP {row.teleports}</Badge> : null}
      </div>
    ),
  },
  {
    id: 'time',
    header: '用时',
    cell: (row) => <span className="font-mono text-xs tabular-nums">{Number(row.time_seconds).toFixed(3)}s</span>,
  },
  {
    id: 'rank',
    header: '排名',
    cell: (row) => (
      <span className="font-mono text-xs tabular-nums">
        #{row.rank}
        {row.is_world_record ? (
          <Badge variant="success" className="ml-1.5">
            WR
          </Badge>
        ) : null}
      </span>
    ),
  },
  {
    id: 'tier',
    header: '等级',
    cell: (row) => (row.tier === null ? '—' : <Badge variant="outline">T{row.tier}</Badge>),
  },
  {
    id: 'points',
    header: '积分',
    cell: (row) =>
      row.points && row.points > 0 ? (
        <span className="font-medium tabular-nums">{formatNumber(row.points)}</span>
      ) : (
        <span className="text-xs text-muted-foreground">—</span>
      ),
  },
  {
    id: 'created',
    header: '上传时间',
    cell: (row) => <span className="text-xs text-muted-foreground">{formatDateTime(row.created_at)}</span>,
  },
];

/** 成绩查询：读取 `/api/v1/public/records`。 */
export function RecordsPage() {
  const table = useTableState();
  const [mode, setMode] = useState('all');
  const [timeType, setTimeType] = useState('all');
  const [sort, setSort] = useState('uploaded');
  const [worldRecordOnly, setWorldRecordOnly] = useState(false);
  const search = useDebouncedValue(table.search);

  const query = useQuery(
    directoryQueries.records({
      map: search || undefined,
      mode: mode === 'all' ? undefined : mode,
      time_type: timeType === 'all' ? undefined : timeType,
      world_record: worldRecordOnly || undefined,
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
        title="成绩"
        description="全部已通过的成绩，可按地图、模式和类型筛选。这里只做只读查询，审核与删除属于后续的写入接口。"
        meta={query.data ? <span>共 {formatNumber(total)} 条成绩</span> : null}
      />

      <ListPanel
        toolbar={
          <FilterBar
            actions={
              <>
                <label className="flex items-center gap-2 text-xs text-muted-foreground">
                  <Switch checked={worldRecordOnly} onCheckedChange={setWorldRecordOnly} aria-label="只看世界纪录" />
                  只看 WR
                </label>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => {
                    table.resetFilters();
                    setMode('all');
                    setTimeType('all');
                    setSort('uploaded');
                    setWorldRecordOnly(false);
                  }}
                >
                  <FilterX />
                  重置
                </Button>
              </>
            }
          >
            <SearchInput
              value={table.search}
              onChange={table.setSearch}
              placeholder="按地图名筛选"
              className="w-full sm:w-52"
            />

            <Select value={mode} onValueChange={setMode}>
              <SelectTrigger className="w-32" aria-label="模式">
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
                  title: '成绩列表读取失败',
                  description: (query.error as Error).message,
                  onRetry: () => void query.refetch(),
                }
              : null
          }
          empty={<EmptyState icon={Trophy} title="没有匹配的成绩" description="调整筛选条件后再试。" />}
        />
      </ListPanel>
    </div>
  );
}
