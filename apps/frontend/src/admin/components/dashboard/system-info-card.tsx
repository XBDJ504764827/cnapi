import type { SystemInfoPayload } from '@cngokz/shared-types';

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/admin/components/ui/card';
import { Skeleton } from '@/admin/components/ui/skeleton';
import { formatDateTime, formatUptime } from '@/admin/lib/format';

interface SystemInfoCardProps {
  info?: SystemInfoPayload;
  isLoading: boolean;
  className?: string;
}

/** Build/runtime information from `/api/v1/system/info`. */
export function SystemInfoCard({ info, isLoading, className }: SystemInfoCardProps) {
  const rows: Array<{ label: string; value: string; mono?: boolean }> = [
    { label: '服务版本', value: info?.version ?? '—', mono: true },
    { label: '运行环境', value: info?.environment ?? '—', mono: true },
    { label: 'API 版本', value: info ? `/${info.api_version}` : '—', mono: true },
    { label: '启动时间', value: formatDateTime(info?.started_at) },
    { label: '运行时长', value: info ? formatUptime(info.uptime_seconds) : '—' },
  ];

  return (
    <Card className={className}>
      <CardHeader>
        <CardTitle>系统信息</CardTitle>
        <CardDescription>来自 /api/v1/system/info 的实时数据</CardDescription>
      </CardHeader>
      <CardContent>
        <dl className="divide-y divide-border/70">
          {rows.map((row) => (
            <div
              key={row.label}
              className="flex items-center justify-between gap-4 py-2 text-[13px] first:pt-0 last:pb-0"
            >
              <dt className="text-muted-foreground">{row.label}</dt>
              <dd className="truncate text-right font-medium">
                {isLoading ? (
                  <Skeleton className="ml-auto h-4 w-24" />
                ) : (
                  <span className={row.mono ? 'font-mono text-xs' : undefined}>{row.value}</span>
                )}
              </dd>
            </div>
          ))}
        </dl>
      </CardContent>
    </Card>
  );
}
