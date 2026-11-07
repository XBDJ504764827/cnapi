import { Film, Trophy } from 'lucide-react';
import type { ReactNode } from 'react';
import { Link } from 'react-router';

import { EmptyState } from '@/public/components/common/empty-state';
import { ErrorState } from '@/public/components/common/error-state';
import { ModeBadge } from '@/public/components/common/mode-badge';
import { PlayerAvatar, PlayerNameLink } from '@/public/components/common/player-avatar';
import { TierBadge } from '@/public/components/common/tier-badge';
import { TimeTypeBadge } from '@/public/components/common/time-type-badge';
import { Badge } from '@/public/components/ui/badge';
import { Button } from '@/public/components/ui/button';
import { Skeleton } from '@/public/components/ui/skeleton';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/public/components/ui/table';
import { formatDateTime, formatRelativeTime, formatSeconds } from '@/public/lib/format';
import type { RecordSummaryPayload } from '@cngokz/shared-types';

interface RecordTableProps {
  records: RecordSummaryPayload[] | undefined;
  isLoading?: boolean;
  errorMessage?: string;
  onRetry?: () => void;
  showPlayer?: boolean;
  showMap?: boolean;
  showRank?: boolean;
  showUploaded?: boolean;
  emptyTitle?: string;
  emptyDescription?: string;
  onSelect?: (record: RecordSummaryPayload) => void;
  footer?: ReactNode;
}

/** Shared public record table. Times are durations; uploaded is a wall-clock time. */
export function RecordTable({
  records,
  isLoading = false,
  errorMessage,
  onRetry,
  showPlayer = true,
  showMap = true,
  showRank = false,
  showUploaded = true,
  emptyTitle = '暂无成绩',
  emptyDescription = '当前条件下没有可展示的成绩。',
  onSelect,
  footer,
}: RecordTableProps) {
  // 难度 / 模式 / 类型 / 成绩 / 积分 / WR 是固定列
  const columnCount =
    6 + (showRank ? 1 : 0) + (showPlayer ? 1 : 0) + (showMap ? 1 : 0) + (showUploaded ? 1 : 0) + (onSelect ? 1 : 0);

  let content: ReactNode;

  if (errorMessage) {
    content = (
      <div className="p-4">
        <ErrorState title="无法加载成绩" description={errorMessage} onRetry={onRetry} />
      </div>
    );
  } else if (!isLoading && (!records || records.length === 0)) {
    // Rendered outside the scroll container so the empty state is never clipped
    // on small screens.
    content = (
      <div className="p-4">
        <EmptyState icon={Trophy} title={emptyTitle} description={emptyDescription} compact />
      </div>
    );
  } else {
    content = (
      <Table className="min-w-[820px]">
        <TableHeader>
          <TableRow className="hover:bg-transparent">
            {showRank ? <TableHead className="w-12">#</TableHead> : null}
            {showPlayer ? <TableHead>玩家</TableHead> : null}
            {showMap ? <TableHead>地图</TableHead> : null}
            <TableHead className="w-16">难度</TableHead>
            <TableHead>模式</TableHead>
            <TableHead className="w-20">类型</TableHead>
            <TableHead className="text-right">成绩</TableHead>
            <TableHead className="w-16 text-right">积分</TableHead>
            {showUploaded ? <TableHead className="w-32">上传时间</TableHead> : null}
            <TableHead className="w-16">WR</TableHead>
            {onSelect ? <TableHead className="w-16" /> : null}
          </TableRow>
        </TableHeader>
        <TableBody>
          {isLoading
            ? Array.from({ length: 6 }).map((_, index) => (
                <TableRow key={index} className="hover:bg-transparent">
                  {Array.from({ length: columnCount }).map((__, cellIndex) => (
                    <TableCell key={cellIndex}>
                      <Skeleton className="h-4 w-20" />
                    </TableCell>
                  ))}
                </TableRow>
              ))
            : (records ?? []).map((record) => (
                <TableRow key={record.id}>
                  {showRank ? (
                    <TableCell className="font-mono text-xs text-muted-foreground tabular-nums">
                      #{record.rank}
                    </TableCell>
                  ) : null}
                  {showPlayer ? (
                    <TableCell>
                      <div className="flex items-center gap-2.5">
                        <PlayerAvatar
                          steamId64={record.player.steam_id64}
                          displayName={record.player.display_name}
                          avatarUrl={record.player.avatar_url}
                          size="sm"
                        />
                        <div className="min-w-0">
                          <PlayerNameLink
                            steamId64={record.player.steam_id64}
                            displayName={record.player.display_name}
                          />
                          <p className="truncate font-mono text-[10px] text-muted-foreground">
                            {record.player.steam_id64}
                          </p>
                        </div>
                      </div>
                    </TableCell>
                  ) : null}
                  {showMap ? (
                    <TableCell>
                      <Link
                        to={`/maps/${record.map.slug}`}
                        className="font-mono text-[12px] hover:text-primary hover:underline"
                      >
                        {record.map.name}
                      </Link>
                      <p className="text-[10px] text-muted-foreground">
                        {record.course.stage === 0 ? 'Main' : `Bonus ${record.course.stage}`}
                      </p>
                    </TableCell>
                  ) : null}
                  <TableCell>
                    <TierBadge tier={record.tier} />
                  </TableCell>
                  <TableCell>
                    <ModeBadge code={record.mode.code} />
                  </TableCell>
                  <TableCell>
                    <TimeTypeBadge value={record.time_type} />
                  </TableCell>
                  <TableCell className="text-right">
                    <span className="font-mono text-[13px] font-medium tabular-nums">
                      {formatSeconds(record.time_seconds)}
                    </span>
                    <span className="ml-0.5 text-[10px] text-muted-foreground">s</span>
                  </TableCell>
                  <TableCell className="text-right">
                    {record.points && record.points > 0 ? (
                      <span className="font-medium tabular-nums">{record.points}</span>
                    ) : (
                      <span className="text-xs text-muted-foreground">—</span>
                    )}
                  </TableCell>
                  {showUploaded ? (
                    <TableCell>
                      <span className="text-[11px] text-muted-foreground" title={formatDateTime(record.created_at)}>
                        {formatRelativeTime(record.created_at)}
                      </span>
                    </TableCell>
                  ) : null}
                  <TableCell>
                    <div className="flex items-center gap-1">
                      {record.is_world_record ? (
                        <Badge variant="primary" title="当前世界纪录">
                          <Trophy className="size-3" />
                          WR
                        </Badge>
                      ) : (
                        <span className="text-xs text-muted-foreground">—</span>
                      )}
                      {record.replay_available ? (
                        <Film className="size-3.5 text-muted-foreground" aria-label="有 Replay" />
                      ) : null}
                    </div>
                  </TableCell>
                  {onSelect ? (
                    <TableCell className="text-right">
                      <Button variant="ghost" size="xs" onClick={() => onSelect(record)}>
                        详情
                      </Button>
                    </TableCell>
                  ) : null}
                </TableRow>
              ))}
        </TableBody>
      </Table>
    );
  }

  return (
    <div>
      {content}
      {footer ? <div className="border-t border-border/70 px-4 py-3">{footer}</div> : null}
    </div>
  );
}
