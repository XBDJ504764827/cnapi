import { Card } from '@/public/components/ui/card';
import { StatusIndicator } from '@/public/components/ui/status-indicator';
import { formatNumber, formatPercent } from '@/public/lib/format';
import { cn } from '@/public/lib/utils';
import type { PlayerModeStatsPayload } from '@cngokz/shared-types';

export function ModeStatsCards({ mode, active }: { mode: PlayerModeStatsPayload; active: boolean }) {
  return (
    <Card className={cn('gap-1 p-4', active && 'border-primary/40 ring-1 ring-primary/15')}>
      <div className="flex items-center justify-between gap-2">
        <p className="text-[13px] font-medium">{mode.mode.display_name}</p>
        {mode.total_records > 0 ? <StatusIndicator tone="up" label={mode.rank ? `#${mode.rank}` : '—'} /> : null}
      </div>
      <div className="mt-2 grid grid-cols-4 gap-2 text-center">
        <div>
          <p className="text-[10px] text-muted-foreground">完成</p>
          <p className="font-mono text-sm tabular-nums">{formatNumber(mode.completed_maps)}</p>
        </div>
        <div>
          <p className="text-[10px] text-muted-foreground">WR</p>
          <p className="font-mono text-sm tabular-nums">{formatNumber(mode.wr_count)}</p>
        </div>
        <div>
          <p className="text-[10px] text-muted-foreground">成绩</p>
          <p className="font-mono text-sm tabular-nums">{formatNumber(mode.total_records)}</p>
        </div>
        <div>
          <p className="text-[10px] text-muted-foreground">积分</p>
          <p className="font-mono text-sm tabular-nums">{formatNumber(mode.points ?? 0)}</p>
        </div>
      </div>
      <div className="mt-2 h-1.5 overflow-hidden rounded-full bg-muted">
        <div
          className="h-full rounded-full bg-primary/70"
          style={{ width: `${Math.min(100, mode.completion_rate * 100)}%` }}
        />
      </div>
      <p className="mt-1 text-[10px] text-muted-foreground">
        {formatPercent(mode.completion_rate)} · {formatNumber(mode.completed_maps)}/{formatNumber(mode.total_maps)} 张
      </p>
    </Card>
  );
}
