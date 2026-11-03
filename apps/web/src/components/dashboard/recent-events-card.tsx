import type { SystemEventsPayload } from '@cngokz/shared-types';
import { History, RefreshCw } from 'lucide-react';

import { EmptyState } from '@/components/common/empty-state';
import { ErrorState } from '@/components/common/error-state';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardAction, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Skeleton } from '@/components/ui/skeleton';
import { formatDateTime, formatRelativeTime } from '@/lib/format';

interface RecentEventsCardProps {
  events?: SystemEventsPayload;
  isLoading: boolean;
  errorMessage?: string;
  onRetry: () => void;
  className?: string;
}

const severityVariant: Record<string, 'info' | 'warning' | 'danger'> = {
  info: 'info',
  warning: 'warning',
  error: 'danger',
};

/** Recent system events; a fresh installation legitimately has none. */
export function RecentEventsCard({ events, isLoading, errorMessage, onRetry, className }: RecentEventsCardProps) {
  return (
    <Card className={className}>
      <CardHeader>
        <CardTitle>最近活动</CardTitle>
        <CardDescription>系统事件（system_events）</CardDescription>
        <CardAction>
          <Button variant="ghost" size="icon-xs" onClick={onRetry} disabled={isLoading} aria-label="刷新系统事件">
            <RefreshCw className={isLoading ? 'animate-spin' : undefined} />
          </Button>
        </CardAction>
      </CardHeader>
      <CardContent>
        {errorMessage ? (
          <ErrorState title="无法读取系统事件" description={errorMessage} onRetry={onRetry} />
        ) : isLoading ? (
          <div className="space-y-2">
            {Array.from({ length: 4 }).map((_, index) => (
              <Skeleton key={index} className="h-11 rounded-md" />
            ))}
          </div>
        ) : events && events.items.length > 0 ? (
          <ul className="divide-y divide-border/70">
            {events.items.map((event) => (
              <li key={event.id} className="flex items-start justify-between gap-4 py-2.5 first:pt-0 last:pb-0">
                <div className="min-w-0 space-y-1">
                  <div className="flex flex-wrap items-center gap-1.5">
                    <Badge variant={severityVariant[event.severity] ?? 'outline'}>{event.severity}</Badge>
                    <span className="truncate font-mono text-[11px] text-muted-foreground">{event.event_type}</span>
                  </div>
                  <p className="line-clamp-2 text-[13px]">{event.message}</p>
                </div>
                <time
                  className="shrink-0 text-[11px] text-muted-foreground"
                  dateTime={event.created_at}
                  title={formatDateTime(event.created_at)}
                >
                  {formatRelativeTime(event.created_at)}
                </time>
              </li>
            ))}
          </ul>
        ) : (
          <EmptyState
            compact
            icon={History}
            title="暂无系统事件"
            description="部署、依赖故障等事件会自动记录在这里。"
          />
        )}
      </CardContent>
    </Card>
  );
}
