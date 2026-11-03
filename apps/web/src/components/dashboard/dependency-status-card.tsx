import type { LucideIcon } from 'lucide-react';

import { StatusBadge } from '@/components/common/status-badge';
import { Card, CardContent, CardHeader } from '@/components/ui/card';
import { Skeleton } from '@/components/ui/skeleton';
import { formatLatency } from '@/lib/format';
import { toneFromStatus } from '@/lib/status';
import { cn } from '@/lib/utils';
import type { DependencyStatusPayload } from '@cngokz/shared-types';

interface DependencyStatusCardProps {
  title: string;
  description: string;
  icon: LucideIcon;
  check?: DependencyStatusPayload;
  isLoading: boolean;
  className?: string;
}

/** Shows a real dependency probe result coming from `/api/v1/system/status`. */
export function DependencyStatusCard({
  title,
  description,
  icon: Icon,
  check,
  isLoading,
  className,
}: DependencyStatusCardProps) {
  return (
    <Card className={cn('gap-3 py-4', className)}>
      <CardHeader className="flex-row items-start justify-between gap-2 space-y-0 px-4">
        <div className="flex min-w-0 items-center gap-2.5">
          <span className="flex size-8 shrink-0 items-center justify-center rounded-md bg-muted text-muted-foreground">
            <Icon className="size-4" aria-hidden="true" />
          </span>
          <div className="min-w-0">
            <p className="truncate text-sm font-medium">{title}</p>
            <p className="truncate text-[11px] text-muted-foreground">{description}</p>
          </div>
        </div>
        {isLoading ? (
          <Skeleton className="h-5 w-14 rounded-md" />
        ) : (
          <StatusBadge tone={toneFromStatus(check?.status)} />
        )}
      </CardHeader>
      <CardContent className="px-4">
        <dl className="grid grid-cols-2 gap-2 text-xs">
          <div className="space-y-0.5">
            <dt className="text-muted-foreground">响应延迟</dt>
            <dd className="font-medium tabular-nums">
              {isLoading ? <Skeleton className="h-4 w-12" /> : formatLatency(check?.latency_ms)}
            </dd>
          </div>
          <div className="space-y-0.5">
            <dt className="text-muted-foreground">探测详情</dt>
            <dd className="truncate font-medium">
              {isLoading ? <Skeleton className="h-4 w-16" /> : (check?.detail ?? '—')}
            </dd>
          </div>
        </dl>
      </CardContent>
    </Card>
  );
}
