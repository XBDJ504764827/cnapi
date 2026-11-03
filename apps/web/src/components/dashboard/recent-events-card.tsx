import type { SystemEventsPayload } from '@cngokz/shared-types';
import { History } from 'lucide-react';

import { EmptyState } from '@/components/common/empty-state';
import { ErrorState } from '@/components/common/error-state';
import { Badge } from '@/components/ui/badge';
import { Card, CardAction, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Skeleton } from '@/components/ui/skeleton';
import { formatDateTime, formatRelativeTime } from '@/lib/format';

interface RecentEventsCardProps {
  events?: SystemEventsPayload;
  isLoading: boolean;
  errorMessage?: string;
  onRetry: () => void;
}

const severityVariant: Record<string, 'secondary' | 'warning' | 'destructive'> = {
  info: 'secondary',
  warning: 'warning',
  error: 'destructive',
};

/** Recent system events; a fresh installation legitimately has none. */
export function RecentEventsCard({ events, isLoading, errorMessage, onRetry }: RecentEventsCardProps) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-sm">最近系统事件</CardTitle>
        <CardDescription>来自 system_events 表，最多显示 10 条</CardDescription>
        <CardAction>
          <Button variant="ghost" size="sm" onClick={onRetry} disabled={isLoading}>
            刷新
          </Button>
        </CardAction>
      </CardHeader>
      <CardContent>
        {errorMessage ? (
          <ErrorState title="无法读取系统事件" description={errorMessage} onRetry={onRetry} />
        ) : isLoading ? (
          <div className="space-y-2">
            {Array.from({ length: 3 }).map((_, index) => (
              <Skeleton key={index} className="h-12 rounded-md" />
            ))}
          </div>
        ) : events && events.items.length > 0 ? (
          <ul className="divide-y">
            {events.items.map((event) => (
              <li key={event.id} className="flex items-start justify-between gap-3 py-3 first:pt-0 last:pb-0">
                <div className="min-w-0 space-y-1">
                  <div className="flex flex-wrap items-center gap-2">
                    <Badge variant={severityVariant[event.severity] ?? 'outline'}>{event.severity}</Badge>
                    <span className="font-mono text-[11px] text-muted-foreground">{event.event_type}</span>
                  </div>
                  <p className="truncate text-sm">{event.message}</p>
                </div>
                <time
                  className="shrink-0 text-xs text-muted-foreground"
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
