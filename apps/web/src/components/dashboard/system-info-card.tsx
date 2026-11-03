import type { SystemInfoPayload } from '@cngokz/shared-types';

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Separator } from '@/components/ui/separator';
import { Skeleton } from '@/components/ui/skeleton';
import { formatDateTime, formatUptime } from '@/lib/format';

interface SystemInfoCardProps {
  info?: SystemInfoPayload;
  isLoading: boolean;
}

/** Build/runtime information from `/api/v1/system/info`. */
export function SystemInfoCard({ info, isLoading }: SystemInfoCardProps) {
  const rows: Array<{ label: string; value: string; mono?: boolean }> = [
    { label: '服务版本', value: info?.version ?? '—', mono: true },
    { label: '运行环境', value: info?.environment ?? '—', mono: true },
    { label: 'API 版本', value: info ? `/${info.api_version}` : '—', mono: true },
    { label: '启动时间', value: formatDateTime(info?.started_at) },
    { label: '运行时长', value: info ? formatUptime(info.uptime_seconds) : '—' },
  ];

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-sm">系统信息</CardTitle>
        <CardDescription>来自后端 /api/v1/system/info 的实时数据</CardDescription>
      </CardHeader>
      <CardContent className="space-y-3">
        <dl className="grid gap-3">
          {rows.map((row, index) => (
            <div key={row.label} className="space-y-3">
              {index > 0 ? <Separator /> : null}
              <div className="flex items-center justify-between gap-4 text-sm">
                <dt className="text-muted-foreground">{row.label}</dt>
                <dd className="truncate text-right font-medium">
                  {isLoading ? (
                    <Skeleton className="ml-auto h-4 w-24" />
                  ) : (
                    <span className={row.mono ? 'font-mono text-xs' : undefined}>{row.value}</span>
                  )}
                </dd>
              </div>
            </div>
          ))}
        </dl>
      </CardContent>
    </Card>
  );
}
