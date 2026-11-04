import { ArrowUpRight, Crown, Map as MapIcon, Trophy } from 'lucide-react';
import { Link } from 'react-router';

import { PlayerAvatar, PlayerNameLink } from '@/components/common/player-avatar';
import { Badge } from '@/components/ui/badge';
import { Skeleton } from '@/components/ui/skeleton';
import { cn } from '@/lib/utils';
import { formatDateTime, formatNumber } from '@/lib/format';
import type { LeaderboardEntryPayload } from '@cngokz/shared-types';

const rankStyles: Record<number, string> = {
  1: 'border-warning/45 bg-warning/12 text-warning',
  2: 'border-info/35 bg-info/10 text-info',
  3: 'border-primary/35 bg-primary/10 text-primary',
};

/**
 * A leaderboard row that reads like a ranking card rather than a dense admin
 * table. The podium is subtly highlighted; no fake medals or neon effects.
 */
export function RankRow({ entry, rank }: { entry: LeaderboardEntryPayload; rank: number }) {
  const podium = rank <= 3;

  return (
    <div
      className={cn(
        'group grid grid-cols-[2.5rem_minmax(0,1fr)_auto] items-center gap-3 px-3 py-3 transition-colors sm:grid-cols-[3rem_minmax(0,1fr)_7rem_7rem_8rem_2rem] sm:gap-4 sm:px-4',
        podium ? 'bg-surface-elevated/30' : 'hover:bg-surface-hover/40',
      )}
    >
      <div
        className={cn(
          'flex h-9 w-9 items-center justify-center rounded-md border font-mono text-sm font-semibold tabular-nums sm:h-10 sm:w-10',
          rankStyles[rank] ?? 'border-border bg-surface-elevated/40 text-muted-foreground',
        )}
        aria-label={`第 ${rank} 名`}
      >
        {podium ? <Crown className="size-4" aria-hidden="true" /> : rank}
      </div>

      <div className="flex min-w-0 items-center gap-2.5">
        <PlayerAvatar steamId64={entry.steam_id64} displayName={entry.display_name} size="md" />
        <div className="min-w-0">
          <div className="flex items-center gap-1.5">
            <PlayerNameLink steamId64={entry.steam_id64} displayName={entry.display_name} className="text-[13px]" />
            {podium ? (
              <Badge
                variant={rank === 1 ? 'warning' : rank === 2 ? 'info' : 'primary'}
                className="hidden sm:inline-flex"
              >
                TOP {rank}
              </Badge>
            ) : null}
          </div>
          <p className="truncate font-mono text-[10px] text-muted-foreground">{entry.steam_id64}</p>
          <div className="mt-1 flex flex-wrap items-center gap-2 text-[10px] text-muted-foreground sm:hidden">
            <span>{entry.country ?? '—'}</span>
            <span>·</span>
            <span className="inline-flex items-center gap-1">
              <MapIcon className="size-3" />
              {formatNumber(entry.completed_maps)}
            </span>
            <span>·</span>
            <span className="inline-flex items-center gap-1">
              <Trophy className="size-3" />
              {formatNumber(entry.wr_count)}
            </span>
          </div>
        </div>
      </div>

      <div className="hidden text-right sm:block">
        <p className="text-[10px] text-muted-foreground">完成地图</p>
        <p className="mt-0.5 text-sm font-semibold tabular-nums">{formatNumber(entry.completed_maps)}</p>
      </div>

      <div className="hidden text-right sm:block">
        <p className="text-[10px] text-muted-foreground">世界纪录</p>
        <p className="mt-0.5 text-sm font-medium tabular-nums">{formatNumber(entry.wr_count)}</p>
      </div>

      <div className="hidden text-right sm:block">
        <p className="text-[10px] text-muted-foreground">成绩 / 最近活动</p>
        <p className="mt-0.5 text-xs tabular-nums" title={formatDateTime(entry.last_activity_at)}>
          {formatNumber(entry.total_records)} · {formatDateTime(entry.last_activity_at)}
        </p>
      </div>

      <div className="hidden justify-end sm:flex">
        <Link
          to={`/players/${entry.steam_id64}`}
          className="flex size-8 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-surface-hover hover:text-foreground"
          aria-label={`查看 ${entry.display_name} 的玩家资料`}
        >
          <ArrowUpRight className="size-4" />
        </Link>
      </div>
    </div>
  );
}

export function RankRowSkeleton() {
  return (
    <div className="grid grid-cols-[2.5rem_minmax(0,1fr)_auto] items-center gap-3 px-3 py-3 sm:grid-cols-[3rem_minmax(0,1fr)_7rem_7rem_8rem_2rem] sm:gap-4 sm:px-4">
      <Skeleton className="size-9 rounded-md sm:size-10" />
      <div className="flex items-center gap-2.5">
        <Skeleton className="size-8 rounded-full" />
        <div className="space-y-2">
          <Skeleton className="h-3.5 w-32" />
          <Skeleton className="h-3 w-44" />
        </div>
      </div>
      <Skeleton className="hidden h-4 w-14 sm:block" />
      <Skeleton className="hidden h-4 w-10 sm:block" />
      <Skeleton className="hidden h-4 w-28 sm:block" />
      <Skeleton className="hidden size-8 rounded-md sm:block" />
    </div>
  );
}
