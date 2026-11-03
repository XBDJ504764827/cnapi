import type { LucideIcon } from 'lucide-react';
import type { ReactNode } from 'react';

import { cn } from '@/lib/utils';

interface EmptyStateProps {
  icon?: LucideIcon;
  title: string;
  description?: string;
  action?: ReactNode;
  hint?: string;
  className?: string;
  compact?: boolean;
}

export function EmptyState({ icon: Icon, title, description, action, hint, className, compact }: EmptyStateProps) {
  return (
    <div
      className={cn(
        'flex flex-col items-center justify-center rounded-lg border border-border/70 bg-surface-elevated/30 text-center',
        compact ? 'gap-2 px-4 py-8' : 'gap-3 px-6 py-14',
        className,
      )}
    >
      {Icon ? (
        <span className="flex size-10 items-center justify-center rounded-lg border border-border bg-surface text-muted-foreground">
          <Icon className="size-4" aria-hidden="true" />
        </span>
      ) : null}
      <div className="space-y-1">
        <p className="text-sm font-medium">{title}</p>
        {description ? (
          <p className="mx-auto max-w-md text-xs leading-relaxed text-muted-foreground">{description}</p>
        ) : null}
      </div>
      {action}
      {hint ? <p className="text-[11px] text-muted-foreground/80">{hint}</p> : null}
    </div>
  );
}
