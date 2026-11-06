import { EmptyState } from '@/public/components/common/empty-state';
import { ErrorState } from '@/public/components/common/error-state';
import { PlayerAvatar, PlayerNameLink } from '@/public/components/common/player-avatar';
import { Skeleton } from '@/public/components/ui/skeleton';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/public/components/ui/table';
import { formatNumber } from '@/public/lib/format';
import type { PointsLeaderboardEntryPayload } from '@cngokz/shared-types';

interface PointsLeaderboardTableProps {
  items: PointsLeaderboardEntryPayload[] | undefined;
  isLoading?: boolean;
  errorMessage?: string;
  onRetry?: () => void;
}

export function PointsLeaderboardTable({ items, isLoading, errorMessage, onRetry }: PointsLeaderboardTableProps) {
  if (errorMessage) return <ErrorState title="无法加载积分榜" description={errorMessage} onRetry={onRetry} />;
  if (!isLoading && (!items || items.length === 0)) {
    return <EmptyState compact title="暂无积分榜数据" description="该模式尚无已重算的玩家积分。" />;
  }
  return (
    <Table className="min-w-[720px]">
      <TableHeader>
        <TableRow className="hover:bg-transparent">
          <TableHead className="w-12">#</TableHead>
          <TableHead>玩家</TableHead>
          <TableHead className="text-right">PRO 积分</TableHead>
          <TableHead className="text-right">NUB 积分</TableHead>
          <TableHead className="text-right">模式总分</TableHead>
          <TableHead className="text-right">Filter 数</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {isLoading
          ? Array.from({ length: 8 }).map((_, index) => (
              <TableRow key={index} className="hover:bg-transparent">
                {Array.from({ length: 6 }).map((__, cell) => (
                  <TableCell key={cell}>
                    <Skeleton className="h-4 w-16" />
                  </TableCell>
                ))}
              </TableRow>
            ))
          : (items ?? []).map((entry) => (
              <TableRow key={entry.account_id}>
                <TableCell className="font-mono text-xs text-muted-foreground tabular-nums">#{entry.rank}</TableCell>
                <TableCell>
                  <div className="flex items-center gap-2.5">
                    <PlayerAvatar steamId64={entry.steam_id64} displayName={entry.display_name} size="sm" />
                    <div className="min-w-0">
                      <PlayerNameLink steamId64={entry.steam_id64} displayName={entry.display_name} />
                      <p className="truncate font-mono text-[10px] text-muted-foreground">{entry.steam_id64}</p>
                    </div>
                  </div>
                </TableCell>
                <TableCell className="text-right tabular-nums">{formatNumber(entry.pro_points)}</TableCell>
                <TableCell className="text-right tabular-nums">{formatNumber(entry.nub_points)}</TableCell>
                <TableCell className="text-right font-semibold tabular-nums">
                  {formatNumber(entry.total_points)}
                </TableCell>
                <TableCell className="text-right text-xs text-muted-foreground tabular-nums">
                  {formatNumber(entry.filter_count)}
                </TableCell>
              </TableRow>
            ))}
      </TableBody>
    </Table>
  );
}
