import type { ReactNode } from 'react';

import { cn } from '@/admin/lib/utils';

interface PageHeaderProps {
  title: string;
  description?: string;
  actions?: ReactNode;
  meta?: ReactNode;
  className?: string;
}

/** Consistent page heading used by every page. */
export function PageHeader({ title, description, actions, meta, className }: PageHeaderProps) {
  return (
    <header className={cn('flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between', className)}>
      <div className="min-w-0 space-y-1">
        <h1 className="text-[22px] leading-tight font-semibold tracking-tight">{title}</h1>
        {description ? (
          <p className="max-w-3xl text-[13px] leading-relaxed text-muted-foreground">{description}</p>
        ) : null}
        {meta ? (
          <div className="flex flex-wrap items-center gap-x-2.5 gap-y-1 pt-0.5 text-xs text-muted-foreground">
            {meta}
          </div>
        ) : null}
      </div>
      {actions ? <div className="flex shrink-0 flex-wrap items-center gap-2">{actions}</div> : null}
    </header>
  );
}
