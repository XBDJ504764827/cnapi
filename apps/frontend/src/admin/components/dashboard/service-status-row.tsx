import type { LucideIcon } from 'lucide-react';

import { Skeleton } from '@/admin/components/ui/skeleton';
import { StatusIndicator } from '@/admin/components/ui/status-indicator';
import { cn } from '@/admin/lib/utils';
import { toneFromStatus } from '@/admin/lib/status';

interface ServiceStatusRowProps {
  name: string;
  description: string;
  icon: LucideIcon;
  status?: string;
  latencyMs?: number | null;
  detail?: string | null;
  isLoading?: boolean;
  fallbackDown?: boolean;
  className?: string;
}

/** One line of the system service list on the dashboard. */
export function ServiceStatusRow({
  name,
  description,
  icon: Icon,
  status,
  latencyMs,
  detail,
  isLoading = false,
  fallbackDown = false,
  className,
}: ServiceStatusRowProps) {
  return (
    <div className={cn('flex items-center gap-3 py-2.5', className)}>
      <span className="flex size-8 shrink-0 items-center justify-center rounded-md border border-border bg-surface-elevated text-muted-foreground">
        <Icon className="size-4" aria-hidden="true" />
      </span>
      <div className="min-w-0 flex-1">
        <p className="truncate text-[13px] font-medium">{name}</p>
        <p className="truncate text-[11px] text-muted-foreground">{description}</p>
      </div>
      <div className="flex shrink-0 flex-col items-end gap-0.5">
        {isLoading ? (
          <Skeleton className="h-4 w-16" />
        ) : (
          <StatusIndicator tone={fallbackDown ? 'down' : toneFromStatus(status)} />
        )}
        <span className="font-mono text-[10px] text-muted-foreground tabular-nums">
          {isLoading ? '—' : latencyMs != null ? `${latencyMs} ms` : (detail ?? (status ? 'ok' : '未知'))}
        </span>
      </div>
    </div>
  );
}
