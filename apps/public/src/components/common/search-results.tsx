import { Map as MapIcon, Search, UserRound } from 'lucide-react';
import { Link } from 'react-router';

import { EmptyState } from '@/components/common/empty-state';
import { PlayerAvatar } from '@/components/common/player-avatar';
import { Badge } from '@/components/ui/badge';
import { Skeleton } from '@/components/ui/skeleton';
import { useQuery } from '@tanstack/react-query';
import { publicQueries } from '@/services/public-service';

/** Shared player/map search results, used by the header dialog and home hero. */
export function SearchResults({ query, onNavigate }: { query: string; onNavigate: () => void }) {
  const trimmed = query.trim();
  const searchQuery = useQuery(publicQueries.search(trimmed));

  if (trimmed.length < 2) {
    return (
      <p className="px-1 py-3 text-xs text-muted-foreground">输入至少 2 个字符，搜索玩家昵称、SteamID64 或地图名称。</p>
    );
  }

  if (searchQuery.isLoading) {
    return (
      <div className="space-y-2 py-2">
        {Array.from({ length: 3 }).map((_, index) => (
          <Skeleton key={index} className="h-11 rounded-md" />
        ))}
      </div>
    );
  }

  if (searchQuery.isError) {
    return <p className="px-1 py-3 text-xs text-danger">搜索失败：{searchQuery.error.message}</p>;
  }

  const players = searchQuery.data?.players ?? [];
  const maps = searchQuery.data?.maps ?? [];

  if (players.length === 0 && maps.length === 0) {
    return (
      <EmptyState
        compact
        icon={Search}
        title="没有匹配结果"
        description="换一个昵称、SteamID64 或地图名称试试。"
        className="border-0 bg-transparent"
      />
    );
  }

  return (
    <div className="space-y-3">
      {players.length > 0 ? (
        <div>
          <p className="px-1 pb-1 text-[10px] font-medium tracking-wider text-muted-foreground uppercase">玩家</p>
          <ul>
            {players.map((player) => (
              <li key={player.account_id}>
                <Link
                  to={`/players/${player.steam_id64}`}
                  onClick={onNavigate}
                  className="flex items-center gap-2.5 rounded-md px-2 py-2 hover:bg-surface-hover"
                >
                  <PlayerAvatar
                    steamId64={player.steam_id64}
                    displayName={player.display_name}
                    linked={false}
                    size="sm"
                  />
                  <span className="min-w-0 flex-1">
                    <span className="block truncate text-[13px] font-medium">{player.display_name}</span>
                    <span className="block truncate font-mono text-[11px] text-muted-foreground">
                      {player.steam_id64}
                    </span>
                  </span>
                  <UserRound className="size-3.5 text-muted-foreground" aria-hidden="true" />
                </Link>
              </li>
            ))}
          </ul>
        </div>
      ) : null}

      {maps.length > 0 ? (
        <div>
          <p className="px-1 pb-1 text-[10px] font-medium tracking-wider text-muted-foreground uppercase">地图</p>
          <ul>
            {maps.map((map) => (
              <li key={map.id}>
                <Link
                  to={`/maps/${map.slug}`}
                  onClick={onNavigate}
                  className="flex items-center gap-2.5 rounded-md px-2 py-2 hover:bg-surface-hover"
                >
                  <span className="flex size-6 items-center justify-center rounded border border-border bg-surface-elevated text-muted-foreground">
                    <MapIcon className="size-3.5" aria-hidden="true" />
                  </span>
                  <span className="min-w-0 flex-1 truncate font-mono text-[12px]">{map.name}</span>
                  <Badge variant="outline">地图</Badge>
                </Link>
              </li>
            ))}
          </ul>
        </div>
      ) : null}
      <p className="px-1 text-[10px] text-muted-foreground">
        搜索 {players.length} 位玩家、{maps.length} 张地图 · 结果来自公开数据库
      </p>
    </div>
  );
}
