import { Link } from 'react-router';

import { Avatar, AvatarFallback, AvatarImage } from '@/admin/components/ui/avatar';
import { cn } from '@/admin/lib/utils';

interface PlayerCellProps {
  steamId64: string;
  displayName: string;
  avatarUrl?: string | null;
  /** 关掉链接用于详情页自身。 */
  linked?: boolean;
  className?: string;
}

function initials(name: string): string {
  const trimmed = name.trim();
  if (!trimmed) return '?';
  // 中文名按字取前两个，英文按单词首字母
  const characters = Array.from(trimmed);
  if (characters.length <= 2) return characters.join('').toUpperCase();
  return characters.slice(0, 2).join('').toUpperCase();
}

/** 玩家头像 + 昵称 + SteamID64，管理端列表统一用它。 */
export function PlayerCell({ steamId64, displayName, avatarUrl, linked = true, className }: PlayerCellProps) {
  const content = (
    <div className={cn('flex min-w-0 items-center gap-2.5', className)}>
      <Avatar className="size-7 border-border">
        {avatarUrl ? <AvatarImage src={avatarUrl} alt={displayName} loading="lazy" /> : null}
        <AvatarFallback className="bg-primary/15 text-[10px] font-semibold text-primary">
          {initials(displayName)}
        </AvatarFallback>
      </Avatar>
      <div className="min-w-0">
        <p className="truncate text-[13px] font-medium">{displayName}</p>
        <p className="truncate font-mono text-[11px] text-muted-foreground">{steamId64}</p>
      </div>
    </div>
  );

  if (!linked) return content;
  return (
    <Link
      to={`/admin/players/${steamId64}`}
      className="block min-w-0 rounded-md outline-none hover:opacity-90 focus-visible:ring-2 focus-visible:ring-ring/40"
    >
      {content}
    </Link>
  );
}
