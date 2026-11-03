import { useQuery } from '@tanstack/react-query';
import { Search, UserRound, X } from 'lucide-react';
import { useState } from 'react';

import { PlayerAvatar } from '@/components/common/player-avatar';
import { Button } from '@/components/ui/button';
import { Card } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Skeleton } from '@/components/ui/skeleton';
import { useDebouncedValue } from '@/hooks/use-debounced-value';
import { publicQueries } from '@/services/public-service';
import { cn } from '@/lib/utils';

export function PlayerPicker({
  label,
  value,
  onChange,
  displayName,
  className,
}: {
  label: string;
  value: string;
  onChange: (steamid64: string | null) => void;
  displayName?: string;
  className?: string;
}) {
  const [query, setQuery] = useState('');
  const debounced = useDebouncedValue(query.trim(), 250);
  const results = useQuery(publicQueries.search(debounced));

  return (
    <Card className={cn('gap-3 p-4', className)}>
      <div className="flex items-center justify-between gap-2">
        <p className="text-[11px] font-medium tracking-wider text-muted-foreground uppercase">{label}</p>
        {value ? (
          <Button variant="ghost" size="icon-xs" onClick={() => onChange(null)} aria-label={`清除${label}`}>
            <X />
          </Button>
        ) : null}
      </div>

      {value ? (
        <div className="flex items-center gap-3 rounded-md border border-border bg-surface-elevated/40 px-3 py-2.5">
          <PlayerAvatar steamId64={value} displayName={displayName ?? value} linked={false} />
          <div className="min-w-0">
            <p className="truncate text-[13px] font-medium">{displayName ?? '已选择玩家'}</p>
            <p className="truncate font-mono text-[10px] text-muted-foreground">{value}</p>
          </div>
        </div>
      ) : (
        <>
          <div className="relative">
            <Search className="pointer-events-none absolute top-1/2 left-2.5 size-3.5 -translate-y-1/2 text-muted-foreground" />
            <Input
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              placeholder="搜索昵称或 SteamID64…"
              className="pl-8"
              aria-label={`${label} 搜索`}
            />
          </div>
          {debounced.length >= 2 ? (
            <div className="max-h-44 overflow-y-auto rounded-md border border-border">
              {results.isLoading ? (
                <div className="space-y-1 p-2">
                  {Array.from({ length: 2 }).map((_, index) => (
                    <Skeleton key={index} className="h-9 rounded" />
                  ))}
                </div>
              ) : results.data && results.data.players.length > 0 ? (
                <ul className="divide-y divide-border/60">
                  {results.data.players.map((player) => (
                    <li key={player.account_id}>
                      <button
                        type="button"
                        onClick={() => {
                          onChange(player.steam_id64);
                          setQuery('');
                        }}
                        className="flex w-full items-center gap-2 px-2.5 py-2 text-left hover:bg-surface-hover"
                      >
                        <PlayerAvatar
                          steamId64={player.steam_id64}
                          displayName={player.display_name}
                          size="sm"
                          linked={false}
                        />
                        <span className="min-w-0 flex-1">
                          <span className="block truncate text-[13px]">{player.display_name}</span>
                          <span className="block truncate font-mono text-[10px] text-muted-foreground">
                            {player.steam_id64}
                          </span>
                        </span>
                      </button>
                    </li>
                  ))}
                </ul>
              ) : (
                <p className="px-3 py-3 text-xs text-muted-foreground">没有匹配的玩家。</p>
              )}
            </div>
          ) : (
            <p className="flex items-center gap-1.5 text-[11px] text-muted-foreground">
              <UserRound className="size-3" />
              输入至少 2 个字符搜索玩家。
            </p>
          )}
        </>
      )}
    </Card>
  );
}
