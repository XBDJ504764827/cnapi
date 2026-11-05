import { EmptyState } from '@/public/components/common/empty-state';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/public/components/ui/card';
import { formatNumber, formatPercent, tierLabel } from '@/public/lib/format';
import type { TierProgressPayload } from '@cngokz/shared-types';

export function TierProgress({ tiers }: { tiers: TierProgressPayload[] }) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>难度完成进度</CardTitle>
        <CardDescription>按地图主线课程的 NUB/PRO 难度等级统计；积分未定义，显示为 “—”。</CardDescription>
      </CardHeader>
      <CardContent>
        {tiers.length === 0 ? (
          <EmptyState compact title="暂无难度数据" description="该模式还没有 course_tiers 记录，或玩家还没有成绩。" />
        ) : (
          <div className="grid gap-3 sm:grid-cols-2">
            {tiers.map((tier) => (
              <div
                key={tier.tier ?? 'unrated'}
                className="space-y-1.5 rounded-md border border-border/70 bg-surface-elevated/30 p-3"
              >
                <div className="flex items-center justify-between gap-2 text-xs">
                  <span className="font-medium">{tierLabel(tier.tier)}</span>
                  <span className="text-muted-foreground tabular-nums">
                    {formatNumber(tier.completed_count)}/{formatNumber(tier.map_count)} ·{' '}
                    {formatPercent(tier.completion_rate)}
                  </span>
                </div>
                <div className="h-1.5 overflow-hidden rounded-full bg-muted">
                  <div
                    className="h-full rounded-full bg-primary/70"
                    style={{ width: `${Math.min(100, tier.completion_rate * 100)}%` }}
                  />
                </div>
                <div className="flex items-center justify-between text-[10px] text-muted-foreground">
                  <span>未完成 {formatNumber(tier.uncompleted_count)}</span>
                  <span>积分 — · 平均 —</span>
                </div>
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
