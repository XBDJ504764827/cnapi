import { Activity, BarChart3, Map as MapIcon, Trophy, Users } from 'lucide-react';

import { Skeleton } from '@/components/ui/skeleton';
import { formatDateTime, formatNumber, modeLabel } from '@/lib/format';

interface SummaryStripProps {
  mode: string;
  timeType: string;
  total: number | undefined;
  entries: Array<{
    completed_maps: number;
    wr_count: number;
    total_records: number;
    last_activity_at?: string | null;
  }>;
  isLoading: boolean;
}

/** Compact facts about the current leaderboard. All values come from the API. */
export function SummaryStrip({ mode, timeType, total, entries, isLoading }: SummaryStripProps) {
  const fastestCompletion = entries.reduce((max, entry) => Math.max(max, entry.completed_maps), 0);
  const worldRecords = entries.reduce((sum, entry) => sum + entry.wr_count, 0);
  const records = entries.reduce((sum, entry) => sum + entry.total_records, 0);
  const lastActivity = entries.reduce<string | null>((latest, entry) => {
    if (!entry.last_activity_at) return latest;
    if (!latest || entry.last_activity_at > latest) return entry.last_activity_at;
    return latest;
  }, null);

  const facts = [
    {
      label: '模式 / 类型',
      value: `${modeLabel(mode)} · ${timeType === 'all' ? '全部' : timeType.toUpperCase()}`,
      icon: BarChart3,
    },
    { label: '上榜玩家', value: formatNumber(total), icon: Users },
    { label: '最高完成', value: `${formatNumber(fastestCompletion)} 张`, icon: MapIcon },
    { label: '页面 WR', value: formatNumber(worldRecords), icon: Trophy },
    { label: '页面成绩', value: formatNumber(records), icon: Activity },
    { label: '最近活动', value: formatDateTime(lastActivity), icon: Activity },
  ];

  return (
    <dl className="grid grid-cols-2 gap-px overflow-hidden rounded-lg border border-border bg-border/70 sm:grid-cols-3 lg:grid-cols-6">
      {facts.map((fact) => {
        const Icon = fact.icon;
        return (
          <div key={fact.label} className="min-w-0 bg-surface px-3 py-2.5">
            <div className="flex items-center gap-1.5 text-muted-foreground">
              <Icon className="size-3 shrink-0" aria-hidden="true" />
              <dt className="truncate text-[10px]">{fact.label}</dt>
            </div>
            {isLoading ? (
              <Skeleton className="mt-1.5 h-4 w-16" />
            ) : (
              <dd className="mt-1 truncate text-xs font-medium tabular-nums" title={fact.value}>
                {fact.value}
              </dd>
            )}
          </div>
        );
      })}
    </dl>
  );
}
