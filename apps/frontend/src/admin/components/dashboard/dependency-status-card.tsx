import type { LucideIcon } from 'lucide-react';

import { StatusBadge } from '@/admin/components/common/status-badge';
import { Card, CardContent, CardHeader } from '@/admin/components/ui/card';
import { Skeleton } from '@/admin/components/ui/skeleton';
import { formatLatency } from '@/admin/lib/format';
import { toneFromStatus } from '@/admin/lib/status';
import type { DependencyStatusPayload } from '@cngokz/shared-types';

interface DependencyStatusCardProps {
  title: string;
  description: string;
  icon: LucideIcon;
  check?: DependencyStatusPayload;
  /** Explicit values for services that are not part of the dependency checks (e.g. the API itself). */
  status?: string;
  latencyMs?: number | null;
  detail?: string | null;
  isLoading: boolean;
  className?: string;
}

/** Shows a real dependency probe result coming from `/api/v1/system/status`. */
export function DependencyStatusCard({
  title,
  description,
  icon: Icon,
  check,
  status,
  latencyMs,
  detail,
  isLoading,
  className,
}: DependencyStatusCardProps) {
  const effectiveStatus = status ?? check?.status;
  const effectiveLatency = latencyMs ?? check?.latency_ms;
  const effectiveDetail = detail ?? check?.detail;

  return (
    <Card className={className}>
      <CardHeader className="flex-row items-center justify-between gap-2">
        <div className="flex min-w-0 items-center gap-2.5">
          <span className="flex size-8 shrink-0 items-center justify-center rounded-md border border-border bg-surface-elevated text-muted-foreground">
            <Icon className="size-4" aria-hidden="true" />
          </span>
          <div className="min-w-0">
            <p className="truncate text-[13px] font-medium">{title}</p>
            <p className="truncate text-[11px] text-muted-foreground">{description}</p>
          </div>
        </div>
        {isLoading ? (
          <Skeleton className="h-5 w-14 rounded-md" />
        ) : (
          <StatusBadge tone={toneFromStatus(effectiveStatus)} />
        )}
      </CardHeader>
      <CardContent>
        <dl className="grid grid-cols-2 gap-3 rounded-md border border-border/70 bg-surface-elevated/30 px-3 py-2.5 text-xs">
          <div className="space-y-0.5">
            <dt className="text-muted-foreground">响应延迟</dt>
            <dd className="font-mono font-medium tabular-nums">
              {isLoading ? <Skeleton className="h-4 w-12" /> : formatLatency(effectiveLatency)}
            </dd>
          </div>
          <div className="space-y-0.5">
            <dt className="text-muted-foreground">探测详情</dt>
            <dd className="truncate font-mono font-medium">
              {isLoading ? <Skeleton className="h-4 w-16" /> : (effectiveDetail ?? 'ok')}
            </dd>
          </div>
        </dl>
      </CardContent>
    </Card>
  );
}
