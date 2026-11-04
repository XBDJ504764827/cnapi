import { Card } from '@/components/ui/card';
import { formatNumber } from '@/lib/format';
import type { PlayerPointsPayload } from '@cngokz/shared-types';

/** Three independent mode totals; never sums across modes. */
export function ModePointsCards({ points }: { points: PlayerPointsPayload }) {
  return (
    <div className="grid gap-3 sm:grid-cols-3">
      {points.modes.map((mode) => (
        <Card key={mode.code} className="gap-2 p-4">
          <div className="flex items-center justify-between gap-2">
            <p className="text-sm font-semibold">{mode.code}</p>
            <span className="text-[11px] text-muted-foreground">{mode.display_name}</span>
          </div>
          <p className="text-2xl font-semibold tabular-nums">{formatNumber(mode.total_points)}</p>
          <div className="grid grid-cols-2 gap-2 text-[11px] text-muted-foreground">
            <span>PRO {formatNumber(mode.pro_points)}</span>
            <span>NUB {formatNumber(mode.nub_points)}</span>
          </div>
          <p className="text-[10px] text-muted-foreground">{formatNumber(mode.filter_count)} 个 Filter</p>
        </Card>
      ))}
    </div>
  );
}
