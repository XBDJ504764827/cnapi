import { Link } from 'react-router';

import { Avatar, AvatarFallback } from '@/public/components/ui/avatar';
import { initials } from '@/public/lib/format';
import { cn } from '@/public/lib/utils';

const sizes = {
  sm: 'size-6 text-[10px]',
  md: 'size-8 text-[11px]',
  lg: 'size-12 text-sm',
};

export function PlayerAvatar({
  steamId64,
  displayName,
  size = 'md',
  linked = true,
  className,
}: {
  steamId64: string;
  displayName: string;
  size?: keyof typeof sizes;
  linked?: boolean;
  className?: string;
}) {
  const avatar = (
    <Avatar className={cn(sizes[size], 'border-border', className)}>
      <AvatarFallback className="bg-surface-hover font-medium text-muted-foreground">
        {initials(displayName)}
      </AvatarFallback>
    </Avatar>
  );

  if (!linked) return avatar;
  return (
    <Link
      to={`/players/${steamId64}`}
      className="shrink-0 rounded-full outline-none focus-visible:ring-2 focus-visible:ring-ring/40"
    >
      {avatar}
    </Link>
  );
}

export function PlayerNameLink({
  steamId64,
  displayName,
  className,
}: {
  steamId64: string;
  displayName: string;
  className?: string;
}) {
  return (
    <Link
      to={`/players/${steamId64}`}
      className={cn('truncate text-[13px] font-medium hover:text-primary hover:underline', className)}
    >
      {displayName}
    </Link>
  );
}
