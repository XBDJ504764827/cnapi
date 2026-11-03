import type { ReactNode } from 'react';
import { Link } from 'react-router';

import { cn } from '@/lib/utils';

export function SectionHeader({
  title,
  description,
  action,
  to,
  actionLabel = '查看全部',
  className,
}: {
  title: string;
  description?: string;
  action?: ReactNode;
  to?: string;
  actionLabel?: string;
  className?: string;
}) {
  return (
    <div className={cn('mb-3 flex items-end justify-between gap-3', className)}>
      <div className="space-y-0.5">
        <h2 className="text-base font-semibold tracking-tight">{title}</h2>
        {description ? <p className="text-xs text-muted-foreground">{description}</p> : null}
      </div>
      {action}
      {!action && to ? (
        <Link to={to} className="text-xs text-primary underline-offset-4 hover:underline">
          {actionLabel}
        </Link>
      ) : null}
    </div>
  );
}
