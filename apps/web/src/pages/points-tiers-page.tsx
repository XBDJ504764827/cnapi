import { useQuery } from '@tanstack/react-query';
import { useState } from 'react';

import { ErrorState } from '@/components/common/error-state';
import { PageHeader } from '@/components/common/page-header';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Pagination } from '@/components/ui/pagination';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Skeleton } from '@/components/ui/skeleton';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { formatDateTime, formatNumber } from '@/lib/format';
import { pointsQueries } from '@/services/points-service';

const token = () => import.meta.env.VITE_MAINTENANCE_TOKEN ?? '';

export function PointsTiersPage() {
  const [mode, setMode] = useState('all');
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);
  const [editing, setEditing] = useState<{ mapId: number; courseId: number; modeId: number; tier: number } | null>(
    null,
  );
  const [tier, setTier] = useState('8');
  const query = useQuery(pointsQueries.tiers(mode === 'all' ? undefined : mode, page, pageSize));

  const saveTier = async () => {
    if (!editing) return;
    const headers: Record<string, string> = { 'Content-Type': 'application/json' };
    if (token()) headers['X-Maintenance-Token'] = token();
    await fetch(`/api/v1/admin/points/tiers/${editing.mapId}/${editing.courseId}/${editing.modeId}`, {
      method: 'PATCH',
      headers,
      body: JSON.stringify({ tier: Number(tier) }),
    });
    setEditing(null);
    await query.refetch();
  };

  return (
    <div className="space-y-5">
      <PageHeader
        title="地图等级管理"
        description="等级按 map × course × mode 独立配置。修改等级会自动触发该模式下相关 Filter 重算。"
        meta={<span>仅查看接口，修改需配置 VITE_MAINTENANCE_TOKEN</span>}
      />

      <Card className="overflow-hidden">
        <div className="flex flex-wrap items-center gap-2 border-b border-border/70 px-4 py-3">
          <Select
            value={mode}
            onValueChange={(value) => {
              setMode(value);
              setPage(1);
            }}
          >
            <SelectTrigger className="w-28" aria-label="模式">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">全部模式</SelectItem>
              <SelectItem value="KZT">KZT</SelectItem>
              <SelectItem value="SKZ">SKZ</SelectItem>
              <SelectItem value="VNL">VNL</SelectItem>
            </SelectContent>
          </Select>
          {editing ? (
            <div className="flex items-center gap-2">
              <Input
                value={tier}
                onChange={(event) => setTier(event.target.value)}
                className="w-16"
                aria-label="新等级"
              />
              <Button size="sm" onClick={() => void saveTier()}>
                保存
              </Button>
              <Button size="sm" variant="ghost" onClick={() => setEditing(null)}>
                取消
              </Button>
            </div>
          ) : null}
        </div>

        {query.isError ? (
          <div className="p-4">
            <ErrorState title="无法读取等级" description={query.error.message} onRetry={() => void query.refetch()} />
          </div>
        ) : query.isLoading ? (
          <div className="p-4">
            <Skeleton className="h-40" />
          </div>
        ) : (
          <Table className="min-w-[900px]">
            <TableHeader>
              <TableRow className="hover:bg-transparent">
                <TableHead>地图</TableHead>
                <TableHead>Course</TableHead>
                <TableHead>模式</TableHead>
                <TableHead>Tier</TableHead>
                <TableHead>首通</TableHead>
                <TableHead>首通玩家</TableHead>
                <TableHead>首通成绩</TableHead>
                <TableHead>更新时间</TableHead>
                <TableHead />
              </TableRow>
            </TableHeader>
            <TableBody>
              {query.data?.items.length ? (
                query.data.items.map((row) => (
                  <TableRow key={`${row.map_id}-${row.course_id}-${row.mode_id}`}>
                    <TableCell className="font-mono text-xs">{row.map_name}</TableCell>
                    <TableCell className="text-xs">{row.course_name}</TableCell>
                    <TableCell>
                      <Badge variant="outline">{row.mode_code}</Badge>
                    </TableCell>
                    <TableCell className="font-mono tabular-nums">T{row.tier}</TableCell>
                    <TableCell>
                      <Badge variant={row.completed ? 'success' : 'outline'}>
                        {row.completed ? '已完成' : '未完成'}
                      </Badge>
                    </TableCell>
                    <TableCell className="text-xs">{row.first_completion_player || '—'}</TableCell>
                    <TableCell className="font-mono text-xs">
                      {row.first_completion_record_id ? `#${row.first_completion_record_id}` : '—'}
                    </TableCell>
                    <TableCell className="text-xs text-muted-foreground">
                      {formatDateTime(row.first_completion_at)}
                    </TableCell>
                    <TableCell className="text-right">
                      <Button
                        size="xs"
                        variant="ghost"
                        onClick={() => {
                          setEditing({
                            mapId: row.map_id,
                            courseId: row.course_id,
                            modeId: row.mode_id,
                            tier: row.tier,
                          });
                          setTier(String(row.tier));
                        }}
                      >
                        编辑 Tier
                      </Button>
                    </TableCell>
                  </TableRow>
                ))
              ) : (
                <TableRow className="hover:bg-transparent">
                  <TableCell colSpan={9} className="p-4 text-center text-xs text-muted-foreground">
                    暂无等级数据。
                  </TableCell>
                </TableRow>
              )}
            </TableBody>
          </Table>
        )}
        <div className="border-t border-border/70 px-4 py-3">
          <Pagination
            page={query.data?.meta.page ?? page}
            pageSize={query.data?.meta.page_size ?? pageSize}
            total={query.data?.meta.total ?? 0}
            onPageChange={setPage}
            onPageSizeChange={(size) => {
              setPageSize(size);
              setPage(1);
            }}
          />
        </div>
      </Card>
      <p className="text-[11px] text-muted-foreground">当前共 {formatNumber(query.data?.meta.total)} 条配置。</p>
    </div>
  );
}
