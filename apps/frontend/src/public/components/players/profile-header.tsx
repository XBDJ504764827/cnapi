import { ExternalLink, MapPin } from 'lucide-react';

import { PlayerAvatar } from '@/public/components/common/player-avatar';
import { Badge } from '@/public/components/ui/badge';
import { Button } from '@/public/components/ui/button';
import { formatDateTime } from '@/public/lib/format';
import type { PlayerDetailPayload } from '@cngokz/shared-types';

export function ProfileHeader({ player }: { player: PlayerDetailPayload }) {
  const { profile } = player;

  return (
    <div className="flex flex-col gap-4 rounded-xl border border-border bg-surface p-5 sm:flex-row sm:items-center sm:justify-between">
      <div className="flex items-center gap-4">
        <PlayerAvatar
          steamId64={profile.steam_id64}
          displayName={profile.display_name}
          avatarUrl={profile.avatar_url}
          size="lg"
          linked={false}
        />
        <div className="min-w-0">
          <div className="flex flex-wrap items-center gap-2">
            <h1 className="truncate text-xl font-semibold tracking-tight sm:text-2xl">{profile.display_name}</h1>
            {profile.country ? (
              <Badge variant="outline">
                <MapPin className="size-3" />
                {profile.country}
              </Badge>
            ) : null}
          </div>
          <p className="mt-1 font-mono text-xs text-muted-foreground">{profile.steam_id64}</p>
          <p className="mt-2 text-[11px] text-muted-foreground">
            首次成绩 {formatDateTime(profile.first_record_at)} · 最近成绩 {formatDateTime(profile.last_record_at)}
          </p>
        </div>
      </div>
      <div className="flex shrink-0 flex-wrap items-center gap-2">
        <Button variant="outline" size="sm" asChild>
          <a href={profile.steam_profile_url} target="_blank" rel="noreferrer">
            Steam 主页
            <ExternalLink className="size-3" />
          </a>
        </Button>
        <Button variant="secondary" size="sm" asChild>
          <a href={`/compare?left=${profile.steam_id64}`}>与其他玩家比较</a>
        </Button>
      </div>
    </div>
  );
}
