import { Link } from 'react-router';

import { EmptyState } from '@/public/components/common/empty-state';
import { ErrorState } from '@/public/components/common/error-state';
import { TierBadge } from '@/public/components/common/tier-badge';
import { Badge } from '@/public/components/ui/badge';
import { Skeleton } from '@/public/components/ui/skeleton';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/public/components/ui/table';
import { formatNumber, formatSecondsPrecise } from '@/public/lib/format';
import type { PlayerFilterPointPayload } from '@cngokz/shared-types';

interface FilterPointsTableProps {
  items: PlayerFilterPointPayload[] | undefined;
  isLoading?: boolean;
  errorMessage?: string;
  onRetry?: () => void;
}

export function FilterPointsTable({ items, isLoading, errorMessage, onRetry }: FilterPointsTableProps) {
  if (errorMessage) return <ErrorState title="无法加载积分明细" description={errorMessage} onRetry={onRetry} />;
  if (!isLoading && (!items || items.length === 0)) {
    return <EmptyState compact title="暂无积分明细" description="该玩家还没有参与任何已完成重算的 Filter。" />;
  }

  return (
    <Table className="min-w-[860px]">
      <TableHeader>
        <TableRow className="hover:bg-transparent">
          <TableHead>地图</TableHead>
          <TableHead>Course</TableHead>
          <TableHead>Tier</TableHead>
          <TableHead>模式</TableHead>
          <TableHead>类型</TableHead>
          <TableHead className="text-right">PB</TableHead>
          <TableHead className="text-right">当前 WR</TableHead>
          <TableHead className="text-right">排名</TableHead>
          <TableHead className="text-right">积分</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {isLoading
          ? Array.from({ length: 5 }).map((_, index) => (
              <TableRow key={index} className="hover:bg-transparent">
                {Array.from({ length: 9 }).map((__, cell) => (
                  <TableCell key={cell}>
                    <Skeleton className="h-4 w-16" />
                  </TableCell>
                ))}
              </TableRow>
            ))
          : (items ?? []).map((item) => (
              <TableRow key={item.filter_id}>
                <TableCell>
                  <Link to={`/maps/${item.map_slug}`} className="font-mono text-xs hover:text-primary hover:underline">
                    {item.map_name}
                  </Link>
                </TableCell>
                <TableCell className="text-xs">{item.course_name}</TableCell>
                <TableCell>
                  <span className="flex items-center gap-1.5">
                    <TierBadge tier={item.tier} />
                    {item.t9_label ? <Badge variant="warning">T9 首通</Badge> : null}
                  </span>
                </TableCell>
                <TableCell className="text-xs">{item.mode.code}</TableCell>
                <TableCell>
                  <Badge variant={item.record_type === 'pro' ? 'success' : 'warning'}>
                    {item.record_type.toUpperCase()}
                  </Badge>
                </TableCell>
                <TableCell className="text-right font-mono text-xs tabular-nums">
                  {formatSecondsPrecise(item.pb_time_seconds)}s
                </TableCell>
                <TableCell className="text-right font-mono text-xs text-muted-foreground tabular-nums">
                  {formatSecondsPrecise(item.wr_time_seconds)}s
                </TableCell>
                <TableCell className="text-right font-mono text-xs tabular-nums">#{item.rank}</TableCell>
                <TableCell className="text-right font-medium tabular-nums">{formatNumber(item.points)}</TableCell>
              </TableRow>
            ))}
      </TableBody>
    </Table>
  );
}
